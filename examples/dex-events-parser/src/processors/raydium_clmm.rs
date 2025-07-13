use {
    async_trait::async_trait,
    carbon_core::{
        error::CarbonResult,
        instruction::{DecodedInstruction, InstructionMetadata, NestedInstructions},
        metrics::MetricsCollection,
        processor::Processor,
    },
    carbon_raydium_clmm_decoder::instructions::RaydiumClmmInstruction,
    std::{sync::Arc, time::SystemTime},
    serde_json::json,
};

use crate::{DexEvent, publishers::{DexEventData, UnifiedPublisher, Publisher}};

pub struct RaydiumClmmProcessor {
    publisher: UnifiedPublisher,
}

impl RaydiumClmmProcessor {
    pub fn new(publisher: UnifiedPublisher) -> Self {
        Self { publisher }
    }
}

#[async_trait]
impl Processor for RaydiumClmmProcessor {
    type InputType = (
        InstructionMetadata,
        DecodedInstruction<RaydiumClmmInstruction>,
        NestedInstructions,
        solana_instruction::Instruction,
    );

    async fn process(
        &mut self,
        (metadata, instruction, _, _): Self::InputType,
        _metrics: Arc<MetricsCollection>,
    ) -> CarbonResult<()> {
        let signature = metadata.transaction_metadata.signature.to_string();
        let platform = "Raydium CLMM".to_string();
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let (event_type, details) = match instruction.data {
            RaydiumClmmInstruction::Swap(swap) => {
                ("swap", json!({
                    "type": "Swap",
                    "amount": swap.amount,
                    "other_amount_threshold": swap.other_amount_threshold,
                    "sqrt_price_limit_x64": swap.sqrt_price_limit_x64
                }))
            }
            RaydiumClmmInstruction::SwapV2(swap) => {
                ("swap", json!({
                    "type": "SwapV2",
                    "amount": swap.amount,
                    "other_amount_threshold": swap.other_amount_threshold,
                    "sqrt_price_limit_x64": swap.sqrt_price_limit_x64
                }))
            }
            RaydiumClmmInstruction::IncreaseLiquidity(increase) => {
                ("liquidity", json!({
                    "type": "add",
                    "action": "IncreaseLiquidity",
                    "liquidity": increase.liquidity,
                    "amount_0_max": increase.amount0_max,
                    "amount_1_max": increase.amount1_max
                }))
            }
            RaydiumClmmInstruction::IncreaseLiquidityV2(increase) => {
                ("liquidity", json!({
                    "type": "add",
                    "action": "IncreaseLiquidityV2",
                    "liquidity": increase.liquidity,
                    "amount_0_max": increase.amount0_max,
                    "amount_1_max": increase.amount1_max
                }))
            }
            RaydiumClmmInstruction::DecreaseLiquidity(decrease) => {
                ("liquidity", json!({
                    "type": "remove",
                    "action": "DecreaseLiquidity",
                    "liquidity": decrease.liquidity,
                    "amount_0_min": decrease.amount0_min,
                    "amount_1_min": decrease.amount1_min
                }))
            }
            RaydiumClmmInstruction::DecreaseLiquidityV2(decrease) => {
                ("liquidity", json!({
                    "type": "remove",
                    "action": "DecreaseLiquidityV2",
                    "liquidity": decrease.liquidity,
                    "amount_0_min": decrease.amount0_min,
                    "amount_1_min": decrease.amount1_min
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