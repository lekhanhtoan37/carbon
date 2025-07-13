use {
    async_trait::async_trait,
    carbon_core::{
        error::CarbonResult,
        instruction::{DecodedInstruction, InstructionMetadata, NestedInstructions},
        metrics::MetricsCollection,
        processor::Processor,
    },
    carbon_raydium_amm_v4_decoder::instructions::RaydiumAmmV4Instruction,
    std::{sync::Arc, time::SystemTime},
    serde_json::json,
};

use crate::{DexEvent, publishers::{DexEventData, UnifiedPublisher, Publisher}};

pub struct RaydiumAmmV4Processor {
    publisher: UnifiedPublisher,
}

impl RaydiumAmmV4Processor {
    pub fn new(publisher: UnifiedPublisher) -> Self {
        Self { publisher }
    }
}

#[async_trait]
impl Processor for RaydiumAmmV4Processor {
    type InputType = (
        InstructionMetadata,
        DecodedInstruction<RaydiumAmmV4Instruction>,
        NestedInstructions,
        solana_instruction::Instruction,
    );

    async fn process(
        &mut self,
        (metadata, instruction, _, _): Self::InputType,
        _metrics: Arc<MetricsCollection>,
    ) -> CarbonResult<()> {
        let signature = metadata.transaction_metadata.signature.to_string();
        let platform = "Raydium AMM V4".to_string();
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let (event_type, details) = match instruction.data {
            RaydiumAmmV4Instruction::SwapBaseIn(swap) => {
                ("swap", json!({
                    "type": "SwapBaseIn",
                    "amount_in": swap.amount_in,
                    "minimum_amount_out": swap.minimum_amount_out
                }))
            }
            RaydiumAmmV4Instruction::SwapBaseOut(swap) => {
                ("swap", json!({
                    "type": "SwapBaseOut",
                    "max_amount_in": swap.max_amount_in,
                    "amount_out": swap.amount_out
                }))
            }
            RaydiumAmmV4Instruction::Deposit(deposit) => {
                ("liquidity", json!({
                    "type": "add",
                    "action": "Deposit",
                    "max_coin_amount": deposit.max_coin_amount,
                    "max_pc_amount": deposit.max_pc_amount,
                    "base_side": deposit.base_side
                }))
            }
            RaydiumAmmV4Instruction::Withdraw(withdraw) => {
                ("liquidity", json!({
                    "type": "remove",
                    "action": "Withdraw",
                    "amount": withdraw.amount
                }))
            }
            RaydiumAmmV4Instruction::Initialize(init) => {
                ("new_pool", json!({
                    "type": "Initialize",
                    "nonce": init.nonce
                }))
            }
            RaydiumAmmV4Instruction::Initialize2(init) => {
                ("new_pool", json!({
                    "type": "Initialize2",
                    "nonce": init.nonce,
                    "open_time": init.open_time
                }))
            }
            RaydiumAmmV4Instruction::PreInitialize(pre_init) => {
                ("new_pool", json!({
                    "type": "PreInitialize",
                    "nonce": pre_init.nonce
                }))
            }
            _ => return Ok(()),
        };

        // Create DexEvent for logging
        let event = match event_type {
            "swap" => DexEvent::Swap {
                platform: platform.clone(),
                signature: signature.clone(),
                details: details.to_string(),
            },
            "liquidity" => {
                if details["type"] == "add" {
                    DexEvent::AddLiquidity {
                        platform: platform.clone(),
                        signature: signature.clone(),
                        details: details.to_string(),
                    }
                } else {
                    DexEvent::RemoveLiquidity {
                        platform: platform.clone(),
                        signature: signature.clone(),
                        details: details.to_string(),
                    }
                }
            }
            "new_pool" => DexEvent::AddPair {
                platform: platform.clone(),
                signature: signature.clone(),
                details: details.to_string(),
            },
            _ => return Ok(()),
        };

        // Log the event
        event.log();

        // Create ZeroMQ event data
        let zmq_data = DexEventData {
            event_type: event_type.to_string(),
            platform,
            signature,
            timestamp,
            details,
        };

        // Publish to ZeroMQ
        if let Err(e) = self.publisher.publish("dex_events", &zmq_data).await {
            log::error!("Failed to publish to ZeroMQ: {}", e);
        }

        Ok(())
    }
} 