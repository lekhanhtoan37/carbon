use {
    async_trait::async_trait,
    carbon_core::{
        error::CarbonResult,
        instruction::{DecodedInstruction, InstructionMetadata, NestedInstructions},
        metrics::MetricsCollection,
        processor::Processor,
    },
    carbon_pumpfun_decoder::instructions::PumpfunInstruction,
    std::{sync::Arc, time::SystemTime},
    serde_json::json,
};

use crate::{DexEvent, publishers::{DexEventData, UnifiedPublisher, Publisher}};

pub struct PumpfunProcessor {
    publisher: UnifiedPublisher,
}

impl PumpfunProcessor {
    pub fn new(publisher: UnifiedPublisher) -> Self {
        Self { publisher }
    }
}

#[async_trait]
impl Processor for PumpfunProcessor {
    type InputType = (
        InstructionMetadata,
        DecodedInstruction<PumpfunInstruction>,
        NestedInstructions,
        solana_instruction::Instruction,
    );

    async fn process(
        &mut self,
        (metadata, instruction, _, _): Self::InputType,
        _metrics: Arc<MetricsCollection>,
    ) -> CarbonResult<()> {
        let signature = metadata.transaction_metadata.signature.to_string();
        let platform = "Pumpfun".to_string();
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let (event_type, details) = match instruction.data {
            PumpfunInstruction::Buy(buy) => {
                ("swap", json!({
                    "type": "Buy",
                    "amount": buy.amount,
                    "max_sol_cost": buy.max_sol_cost
                }))
            }
            PumpfunInstruction::Sell(sell) => {
                ("swap", json!({
                    "type": "Sell",
                    "amount": sell.amount,
                    "min_sol_output": sell.min_sol_output
                }))
            }
            PumpfunInstruction::TradeEvent(trade) => {
                ("swap", json!({
                    "type": "TradeEvent",
                    "mint": trade.mint.to_string(),
                    "sol_amount": trade.sol_amount,
                    "token_amount": trade.token_amount,
                    "is_buy": trade.is_buy
                }))
            }
            PumpfunInstruction::CreateEvent(create) => {
                ("mint_burn", json!({
                    "type": "mint",
                    "action": "CreateEvent",
                    "mint": create.mint.to_string(),
                    "name": create.name,
                    "symbol": create.symbol
                }))
            }
            PumpfunInstruction::CompleteEvent(complete) => {
                ("new_pool", json!({
                    "type": "CompleteEvent",
                    "mint": complete.mint.to_string(),
                    "bonding_curve": complete.bonding_curve.to_string()
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
            "mint_burn" => DexEvent::Swap { // Use Swap for now since we don't have MintBurn variant
                platform: platform.clone(),
                signature: signature.clone(),
                details: details.to_string(),
            },
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