use crate::publishers::{create_unified_publisher_from_env};

use {
    async_trait::async_trait,
    carbon_core::{
        datasource::Update,
        error::CarbonResult,
        metrics::MetricsCollection,
        processor::Processor,
    },
    carbon_log_metrics::LogMetrics,
    carbon_rpc_block_subscribe_datasource::{Filters, RpcBlockSubscribe},
    solana_client::rpc_config::{RpcBlockSubscribeConfig, RpcBlockSubscribeFilter},
    solana_commitment_config::CommitmentConfig,
    solana_transaction_status::{UiTransactionEncoding, TransactionDetails},
    std::{env, sync::Arc},
};


// Import all decoder types
use carbon_raydium_amm_v4_decoder::{
    RaydiumAmmV4Decoder,
    PROGRAM_ID as RAYDIUM_AMM_V4_PROGRAM_ID,
};
use carbon_raydium_clmm_decoder::{
    RaydiumClmmDecoder, PROGRAM_ID as RAYDIUM_CLMM_PROGRAM_ID,
};
use carbon_raydium_cpmm_decoder::{
    RaydiumCpmmDecoder, PROGRAM_ID as RAYDIUM_CPMM_PROGRAM_ID,
};
use carbon_jupiter_swap_decoder::{
    JupiterSwapDecoder, PROGRAM_ID as JUPITER_SWAP_PROGRAM_ID,
};
use carbon_orca_whirlpool_decoder::{
    OrcaWhirlpoolDecoder, PROGRAM_ID as ORCA_WHIRLPOOL_PROGRAM_ID,
};
use carbon_meteora_dlmm_decoder::{
    MeteoraDlmmDecoder, PROGRAM_ID as METEORA_DLMM_PROGRAM_ID,
};
use carbon_pumpfun_decoder::{
    PumpfunDecoder, PROGRAM_ID as PUMPFUN_PROGRAM_ID,
};
use carbon_lifinity_amm_v2_decoder::{
    LifinityAmmV2Decoder, PROGRAM_ID as LIFINITY_AMM_V2_PROGRAM_ID,
};
use carbon_moonshot_decoder::{
    MoonshotDecoder, PROGRAM_ID as MOONSHOT_PROGRAM_ID,
};
use carbon_openbook_v2_decoder::{
    OpenbookV2Decoder, PROGRAM_ID as OPENBOOK_V2_PROGRAM_ID,
};
use carbon_phoenix_v1_decoder::{
    PhoenixDecoder, PROGRAM_ID as PHOENIX_PROGRAM_ID,
};
use carbon_fluxbeam_decoder::{
    FluxbeamDecoder, PROGRAM_ID as FLUXBEAM_PROGRAM_ID,
};

mod processors;
mod publishers;
mod datasources;

use processors::{
    raydium_amm_v4::RaydiumAmmV4Processor,
    raydium_clmm::RaydiumClmmProcessor,
    pumpfun::PumpfunProcessor,
    others::{
        RaydiumCpmmProcessor,
        JupiterSwapProcessor,
        OrcaWhirlpoolProcessor,
        MeteoraDlmmProcessor,
        OpenbookV2Processor,
        PhoenixProcessor,
        FluxbeamProcessor,
        LifinityAmmV2Processor,
        MoonshotProcessor,
    },
};
use datasources::{HybridBlockDatasource, HybridFilters};

#[derive(Debug, Clone)]
pub enum DexEvent {
    // Swap Events
    Swap {
        platform: String,
        signature: String,
        details: String,
    },
    // Add Liquidity Events
    AddLiquidity {
        platform: String,
        signature: String,
        details: String,
    },
    // Remove Liquidity Events
    RemoveLiquidity {
        platform: String,
        signature: String,
        details: String,
    },
    // Add Pair/Pool Events
    AddPair {
        platform: String,
        signature: String,
        details: String,
    },
    NewPair {
        platform: String,
        signature: String,
        details: String,
    },
}

impl DexEvent {
    pub fn log(&self) {
        match self {
            DexEvent::Swap { platform, signature, details } => {
                log::info!("[SWAP] [{}] [{}] {}", platform, signature, details);
            }
            DexEvent::AddLiquidity { platform, signature, details } => {
                log::info!("[ADD_LIQUIDITY] [{}] [{}] {}", platform, signature, details);
            }
            DexEvent::RemoveLiquidity { platform, signature, details } => {
                log::info!("[REMOVE_LIQUIDITY] [{}] [{}] {}", platform, signature, details);
            }
            DexEvent::AddPair { platform, signature, details } => {
                log::info!("[ADD_PAIR] [{}] [{}] {}", platform, signature, details);
            }
            DexEvent::NewPair { platform, signature, details } => {
                log::info!("[NEW_PAIR] [{}] [{}] {}", platform, signature, details);
            }
        }
    }
}

#[tokio::main]
pub async fn main() -> CarbonResult<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    log::info!("Starting DEX Events Parser...");

    let rpc_ws_url = env::var("RPC_WS_URL")
        .unwrap_or_else(|_| "wss://api.mainnet-beta.solana.com".to_string());
    let rpc_http_url = env::var("RPC_HTTP_URL")
        .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string());
    let datasource_type = env::var("DATASOURCE_TYPE")
        .unwrap_or_else(|_| "websocket".to_string());

    log::info!("RPC WebSocket: {}", rpc_ws_url);
    log::info!("RPC HTTP: {}", rpc_http_url);
    log::info!("Datasource type: {}", datasource_type);
    
    // Get publisher type from environment
    let publisher_type = env::var("PUBLISHER_TYPE").unwrap_or_else(|_| "zmq".to_string());
    
    log::info!("Publisher type: {}", publisher_type);
    let publisher = create_unified_publisher_from_env().map_err(|e| carbon_core::error::Error::Custom(format!("Failed to create publisher: {}", e)))?;
    
    // Configure RPC block subscribe with multiple program IDs
    let program_ids = vec![
        RAYDIUM_AMM_V4_PROGRAM_ID.to_string(),
        RAYDIUM_CLMM_PROGRAM_ID.to_string(),
        RAYDIUM_CPMM_PROGRAM_ID.to_string(),
        JUPITER_SWAP_PROGRAM_ID.to_string(),
        ORCA_WHIRLPOOL_PROGRAM_ID.to_string(),
        METEORA_DLMM_PROGRAM_ID.to_string(),
        PUMPFUN_PROGRAM_ID.to_string(),
        OPENBOOK_V2_PROGRAM_ID.to_string(),
        PHOENIX_PROGRAM_ID.to_string(),
        FLUXBEAM_PROGRAM_ID.to_string(),
        LIFINITY_AMM_V2_PROGRAM_ID.to_string(),
        MOONSHOT_PROGRAM_ID.to_string(),
    ];
    
    // Use the first program ID as the main filter
    let block_filter = RpcBlockSubscribeFilter::MentionsAccountOrProgram(
        program_ids[0].clone()
    );

    let block_subscribe_config = RpcBlockSubscribeConfig {
        commitment: Some(CommitmentConfig::confirmed()),
        encoding: Some(UiTransactionEncoding::Base64),
        transaction_details: Some(TransactionDetails::Full),
        show_rewards: Some(false),
        max_supported_transaction_version: Some(0),
    };

    // Create datasource based on type
    match datasource_type.as_str() {
        "hybrid" => {
            log::info!("Using Hybrid Datasource (WebSocket notifications + HTTP RPC data)");
            
            let hybrid_filters = HybridFilters::new(
                block_filter,
                Some(CommitmentConfig::confirmed()),
            );
            
            let hybrid_datasource = HybridBlockDatasource::new(
                rpc_ws_url,
                rpc_http_url,
                hybrid_filters,
            );
            
            // Create processors for all decoders
            carbon_core::pipeline::Pipeline::builder()
                .datasource(hybrid_datasource)
                .metrics(Arc::new(LogMetrics::new()))
                .metrics_flush_interval(5)
                .instruction(RaydiumAmmV4Decoder, RaydiumAmmV4Processor::new(publisher.clone()))
                .instruction(RaydiumClmmDecoder, RaydiumClmmProcessor::new(publisher.clone()))
                .instruction(RaydiumCpmmDecoder, RaydiumCpmmProcessor::new(publisher.clone()))
                .instruction(JupiterSwapDecoder, JupiterSwapProcessor::new(publisher.clone()))
                .instruction(OrcaWhirlpoolDecoder, OrcaWhirlpoolProcessor::new(publisher.clone()))
                .instruction(MeteoraDlmmDecoder, MeteoraDlmmProcessor::new(publisher.clone()))
                .instruction(PumpfunDecoder, PumpfunProcessor::new(publisher.clone()))
                .instruction(OpenbookV2Decoder, OpenbookV2Processor::new(publisher.clone()))
                .instruction(PhoenixDecoder, PhoenixProcessor::new(publisher.clone()))
                .instruction(FluxbeamDecoder, FluxbeamProcessor::new(publisher.clone()))
                .instruction(LifinityAmmV2Decoder, LifinityAmmV2Processor::new(publisher.clone()))
                .instruction(MoonshotDecoder, MoonshotProcessor::new(publisher.clone()))
                .shutdown_strategy(carbon_core::pipeline::ShutdownStrategy::Immediate)
                .build()?
                .run()
                .await?;
        }
        _ => {
            log::info!("Using Traditional WebSocket Datasource (full data over WebSocket)");
            
            let filters = Filters::new(block_filter, Some(block_subscribe_config));
            let datasource = RpcBlockSubscribe::new(rpc_ws_url, filters);
            
            // Create processors for all decoders
            carbon_core::pipeline::Pipeline::builder()
                .datasource(datasource)
                .metrics(Arc::new(LogMetrics::new()))
                .metrics_flush_interval(5)
                .instruction(RaydiumAmmV4Decoder, RaydiumAmmV4Processor::new(publisher.clone()))
                .instruction(RaydiumClmmDecoder, RaydiumClmmProcessor::new(publisher.clone()))
                .instruction(RaydiumCpmmDecoder, RaydiumCpmmProcessor::new(publisher.clone()))
                .instruction(JupiterSwapDecoder, JupiterSwapProcessor::new(publisher.clone()))
                .instruction(OrcaWhirlpoolDecoder, OrcaWhirlpoolProcessor::new(publisher.clone()))
                .instruction(MeteoraDlmmDecoder, MeteoraDlmmProcessor::new(publisher.clone()))
                .instruction(PumpfunDecoder, PumpfunProcessor::new(publisher.clone()))
                .instruction(OpenbookV2Decoder, OpenbookV2Processor::new(publisher.clone()))
                .instruction(PhoenixDecoder, PhoenixProcessor::new(publisher.clone()))
                .instruction(FluxbeamDecoder, FluxbeamProcessor::new(publisher.clone()))
                .instruction(LifinityAmmV2Decoder, LifinityAmmV2Processor::new(publisher.clone()))
                .instruction(MoonshotDecoder, MoonshotProcessor::new(publisher.clone()))
                .shutdown_strategy(carbon_core::pipeline::ShutdownStrategy::Immediate)
                .build()?
                .run()
                .await?;
        }
    }



    Ok(())
}



// Generic Update Processor for block details
pub struct UpdateProcessor;

#[async_trait]
impl Processor for UpdateProcessor {
    type InputType = Update;

    async fn process(
        &mut self,
        data: Self::InputType,
        _metrics: Arc<MetricsCollection>,
    ) -> CarbonResult<()> {
        match data {
            Update::BlockDetails(block_details) => {
                log::debug!("Block processed: slot={}, transactions={}", 
                           block_details.slot, 
                           block_details.block_time.unwrap_or(0));
            }
            _ => {}
        }
        Ok(())
    }
} 