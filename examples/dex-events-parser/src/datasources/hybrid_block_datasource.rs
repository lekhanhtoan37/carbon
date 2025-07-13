use {
    async_trait::async_trait,
    carbon_core::{
        datasource::{Datasource, DatasourceId, TransactionUpdate, Update, UpdateType},
        error::CarbonResult,
        metrics::MetricsCollection,
        transformers::transaction_metadata_from_original_meta,
    },
    futures::StreamExt,
    solana_client::{
        nonblocking::{pubsub_client::PubsubClient, rpc_client::RpcClient},
        rpc_client::SerializableTransaction,
        rpc_config::{RpcBlockConfig, RpcBlockSubscribeConfig, RpcBlockSubscribeFilter},
    },
    solana_commitment_config::CommitmentConfig,
    solana_hash::Hash,
    solana_transaction_status::{TransactionDetails, UiTransactionEncoding},
    std::{str::FromStr, sync::Arc, time::{Duration, Instant}},
    tokio::sync::mpsc::{self, Receiver, Sender},
    tokio_util::sync::CancellationToken,
};

const MAX_RECONNECTION_ATTEMPTS: u32 = 10;
const RECONNECTION_DELAY_MS: u64 = 3000;
const BLOCK_FETCH_CHANNEL_SIZE: usize = 1000;
const MAX_CONCURRENT_BLOCK_REQUESTS: usize = 5;

#[derive(Debug, Clone)]
pub struct HybridFilters {
    pub block_filter: RpcBlockSubscribeFilter,
    pub block_subscribe_config: Option<RpcBlockSubscribeConfig>,
    pub block_fetch_config: RpcBlockConfig,
}

impl HybridFilters {
    pub fn new(
        block_filter: RpcBlockSubscribeFilter,
        commitment: Option<CommitmentConfig>,
    ) -> Self {
        // Configure WebSocket subscription for block notifications only (no transactions)
        let block_subscribe_config = Some(RpcBlockSubscribeConfig {
            commitment: commitment.clone(),
            encoding: Some(UiTransactionEncoding::Base64),
            transaction_details: Some(TransactionDetails::None), // Key: No transactions via WebSocket
            show_rewards: Some(false),
            max_supported_transaction_version: Some(0),
        });

        // Configure HTTP RPC for full block data with transactions
        let block_fetch_config = RpcBlockConfig {
            encoding: Some(UiTransactionEncoding::Base64),
            transaction_details: Some(TransactionDetails::Full), // Key: Full transactions via HTTP
            rewards: Some(false),
            commitment,
            max_supported_transaction_version: Some(0),
        };

        Self {
            block_filter,
            block_subscribe_config,
            block_fetch_config,
        }
    }
}

pub struct HybridBlockDatasource {
    pub rpc_ws_url: String,
    pub rpc_http_url: String,
    pub filters: HybridFilters,
}

impl HybridBlockDatasource {
    pub fn new(rpc_ws_url: String, rpc_http_url: String, filters: HybridFilters) -> Self {
        Self {
            rpc_ws_url,
            rpc_http_url,
            filters,
        }
    }
}

#[async_trait]
impl Datasource for HybridBlockDatasource {
    async fn consume(
        &self,
        id: DatasourceId,
        sender: Sender<(Update, DatasourceId)>,
        cancellation_token: CancellationToken,
        metrics: Arc<MetricsCollection>,
    ) -> CarbonResult<()> {
        log::info!("Starting Hybrid Block Datasource...");
        log::info!("WebSocket URL: {}", self.rpc_ws_url);
        log::info!("HTTP RPC URL: {}", self.rpc_http_url);

        // Create HTTP RPC client for block fetching
        let http_client = Arc::new(RpcClient::new_with_commitment(
            self.rpc_http_url.clone(),
            self.filters
                .block_fetch_config
                .commitment
                .unwrap_or(CommitmentConfig::confirmed()),
        ));

        // Create channel for slot notifications
        let (slot_sender, slot_receiver) = mpsc::channel(BLOCK_FETCH_CHANNEL_SIZE);

        // Start block notification subscriber (WebSocket)
        let notification_task = self.start_block_notification_subscriber(
            slot_sender,
            cancellation_token.clone(),
            metrics.clone(),
        );

        // Start block data fetcher (HTTP RPC)
        let fetcher_task = self.start_block_data_fetcher(
            http_client,
            slot_receiver,
            sender,
            id,
            cancellation_token.clone(),
            metrics.clone(),
        );

        // Wait for tasks to complete
        tokio::select! {
            _ = notification_task => {
                log::info!("Block notification subscriber completed");
            }
            _ = fetcher_task => {
                log::info!("Block data fetcher completed");
            }
            _ = cancellation_token.cancelled() => {
                log::info!("Hybrid Block Datasource cancelled");
            }
        }

        Ok(())
    }

    fn update_types(&self) -> Vec<UpdateType> {
        vec![UpdateType::Transaction]
    }
}

impl HybridBlockDatasource {
    async fn start_block_notification_subscriber(
        &self,
        slot_sender: Sender<u64>,
        cancellation_token: CancellationToken,
        metrics: Arc<MetricsCollection>,
    ) -> tokio::task::JoinHandle<()> {
        let rpc_ws_url = self.rpc_ws_url.clone();
        let filters = self.filters.clone();
        
        tokio::spawn(async move {
            let mut reconnection_attempts = 0;

            loop {
                if cancellation_token.is_cancelled() {
                    log::info!("Block notification subscriber cancelled");
                    break;
                }

                let client = match PubsubClient::new(&rpc_ws_url).await {
                    Ok(client) => client,
                    Err(err) => {
                        log::error!("Failed to create WebSocket client: {}", err);
                        reconnection_attempts += 1;
                        if reconnection_attempts >= MAX_RECONNECTION_ATTEMPTS {
                            log::error!("Max reconnection attempts reached for WebSocket");
                            break;
                        }
                        tokio::time::sleep(Duration::from_millis(RECONNECTION_DELAY_MS)).await;
                        continue;
                    }
                };

                let (mut block_stream, _unsub) = match client
                    .block_subscribe(filters.block_filter.clone(), filters.block_subscribe_config.clone())
                    .await
                {
                    Ok(subscription) => subscription,
                    Err(err) => {
                        log::error!("Failed to subscribe to blocks: {:?}", err);
                        reconnection_attempts += 1;
                        if reconnection_attempts > MAX_RECONNECTION_ATTEMPTS {
                            log::error!("Max subscription attempts reached");
                            break;
                        }
                        tokio::time::sleep(Duration::from_millis(RECONNECTION_DELAY_MS)).await;
                        continue;
                    }
                };

                reconnection_attempts = 0;
                log::info!("Successfully subscribed to block notifications");

                loop {
                    tokio::select! {
                        _ = cancellation_token.cancelled() => {
                            log::info!("Block notification subscription cancelled");
                            return;
                        }
                        block_event = block_stream.next() => {
                            match block_event {
                                Some(event) => {
                                    let slot = event.context.slot;
                                    log::debug!("Received block notification for slot: {}", slot);
                                    
                                    // Send slot to fetcher
                                    if let Err(err) = slot_sender.send(slot).await {
                                        log::error!("Failed to send slot to fetcher: {}", err);
                                        break;
                                    }

                                    metrics
                                        .increment_counter("hybrid_block_notifications_received", 1)
                                        .await
                                        .unwrap_or_else(|e| log::error!("Error recording metric: {}", e));
                                }
                                None => {
                                    log::warn!("Block notification stream closed, reconnecting...");
                                    break;
                                }
                            }
                        }
                    }
                }

                tokio::time::sleep(Duration::from_millis(RECONNECTION_DELAY_MS)).await;
            }
        })
    }

    async fn start_block_data_fetcher(
        &self,
        http_client: Arc<RpcClient>,
        mut slot_receiver: Receiver<u64>,
        sender: Sender<(Update, DatasourceId)>,
        id: DatasourceId,
        cancellation_token: CancellationToken,
        metrics: Arc<MetricsCollection>,
    ) -> tokio::task::JoinHandle<()> {
        let block_config = self.filters.block_fetch_config.clone();
        
        tokio::spawn(async move {
            log::info!("Block data fetcher started");

            while let Some(slot) = slot_receiver.recv().await {
                if cancellation_token.is_cancelled() {
                    log::info!("Block data fetcher cancelled");
                    break;
                }

                log::debug!("Fetching full block data for slot: {}", slot);
                let start_time = Instant::now();

                match http_client.get_block_with_config(slot, block_config.clone()).await {
                    Ok(block) => {
                        let fetch_time = start_time.elapsed();
                        log::debug!("Fetched block {} in {:?}", slot, fetch_time);

                        // Record metrics
                        metrics
                            .record_histogram(
                                "hybrid_block_fetch_time_milliseconds",
                                fetch_time.as_millis() as f64,
                            )
                            .await
                            .unwrap_or_else(|e| log::error!("Error recording metric: {}", e));

                        metrics
                            .increment_counter("hybrid_blocks_fetched", 1)
                            .await
                            .unwrap_or_else(|e| log::error!("Error recording metric: {}", e));

                        // Process transactions from the block
                        if let Some(transactions) = block.transactions {
                            let block_hash = Hash::from_str(&block.blockhash).ok();
                            
                            for encoded_transaction_with_status_meta in transactions {
                                let tx_start_time = Instant::now();

                                let meta_original = if let Some(meta) = encoded_transaction_with_status_meta.meta.clone() {
                                    meta
                                } else {
                                    continue;
                                };

                                if meta_original.status.is_err() {
                                    continue;
                                }

                                let Some(decoded_transaction) = encoded_transaction_with_status_meta.transaction.decode() else {
                                    log::error!("Failed to decode transaction");
                                    continue;
                                };

                                let Ok(meta_needed) = transaction_metadata_from_original_meta(meta_original) else {
                                    log::error!("Error processing transaction metadata");
                                    continue;
                                };

                                let update = Update::Transaction(Box::new(TransactionUpdate {
                                    signature: *decoded_transaction.get_signature(),
                                    transaction: decoded_transaction,
                                    meta: meta_needed,
                                    is_vote: false,
                                    slot,
                                    block_time: block.block_time,
                                    block_hash,
                                }));

                                // Send transaction update
                                if let Err(err) = sender.send((update, id.clone())).await {
                                    log::error!("Failed to send transaction update: {}", err);
                                    break;
                                }

                                metrics
                                    .record_histogram(
                                        "hybrid_transaction_process_time_nanoseconds",
                                        tx_start_time.elapsed().as_nanos() as f64,
                                    )
                                    .await
                                    .unwrap_or_else(|e| log::error!("Error recording metric: {}", e));

                                metrics
                                    .increment_counter("hybrid_transactions_processed", 1)
                                    .await
                                    .unwrap_or_else(|e| log::error!("Error recording metric: {}", e));
                            }
                        }
                    }
                    Err(err) => {
                        // Handle skipped slots gracefully
                        if err.to_string().contains("-32009")
                            || err.to_string().contains("-32004")
                            || err.to_string().contains("-32007")
                        {
                            log::debug!("Slot {} was skipped or missing: {}", slot, err);
                            metrics
                                .increment_counter("hybrid_blocks_skipped", 1)
                                .await
                                .unwrap_or_else(|e| log::error!("Error recording metric: {}", e));
                        } else {
                            log::error!("Error fetching block {}: {}", slot, err);
                            metrics
                                .increment_counter("hybrid_block_fetch_errors", 1)
                                .await
                                .unwrap_or_else(|e| log::error!("Error recording metric: {}", e));
                        }
                    }
                }
            }

            log::info!("Block data fetcher completed");
        })
    }
} 