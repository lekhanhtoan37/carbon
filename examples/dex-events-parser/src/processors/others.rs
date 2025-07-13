use {
    async_trait::async_trait,
    carbon_core::{
        error::CarbonResult,
        instruction::{DecodedInstruction, InstructionMetadata, NestedInstructions},
        metrics::MetricsCollection,
        processor::Processor,
    },
    std::{sync::Arc, time::SystemTime},
    serde_json::json,
};

use carbon_raydium_cpmm_decoder::instructions::RaydiumCpmmInstruction;
use carbon_jupiter_swap_decoder::instructions::JupiterSwapInstruction;
use carbon_orca_whirlpool_decoder::instructions::OrcaWhirlpoolInstruction;
use carbon_meteora_dlmm_decoder::instructions::MeteoraDlmmInstruction;
use carbon_openbook_v2_decoder::instructions::OpenbookV2Instruction;
use carbon_phoenix_v1_decoder::instructions::PhoenixInstruction;
use carbon_fluxbeam_decoder::instructions::FluxbeamInstruction;
use carbon_lifinity_amm_v2_decoder::instructions::LifinityAmmV2Instruction;
use carbon_moonshot_decoder::instructions::MoonshotInstruction;

use crate::{DexEvent, publishers::{DexEventData, UnifiedPublisher, Publisher}};

// Raydium CPMM Processor
pub struct RaydiumCpmmProcessor {
    publisher: UnifiedPublisher,
}

impl RaydiumCpmmProcessor {
    pub fn new(publisher: UnifiedPublisher) -> Self {
        Self { publisher }
    }
}

#[async_trait]
impl Processor for RaydiumCpmmProcessor {
    type InputType = (
        InstructionMetadata,
        DecodedInstruction<RaydiumCpmmInstruction>,
        NestedInstructions,
        solana_instruction::Instruction,
    );

    async fn process(
        &mut self,
        (metadata, instruction, _, _): Self::InputType,
        _metrics: Arc<MetricsCollection>,
    ) -> CarbonResult<()> {
        let signature = metadata.transaction_metadata.signature.to_string();
        let platform = "Raydium CPMM".to_string();
        let timestamp = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();

        let (event_type, details) = match instruction.data {
            RaydiumCpmmInstruction::SwapBaseInput(swap) => {
                ("swap", json!({
                    "type": "SwapBaseInput",
                    "amount_in": swap.amount_in,
                    "minimum_amount_out": swap.minimum_amount_out
                }))
            }
            RaydiumCpmmInstruction::SwapBaseOutput(swap) => {
                ("swap", json!({
                    "type": "SwapBaseOutput",
                    "max_amount_in": swap.max_amount_in,
                    "amount_out": swap.amount_out
                }))
            }
            _ => return Ok(()),
        };

        self.process_event(event_type, platform, signature, timestamp, details).await
    }
}

// Jupiter Swap Processor
pub struct JupiterSwapProcessor {
    publisher: UnifiedPublisher,
}

impl JupiterSwapProcessor {
    pub fn new(publisher: UnifiedPublisher) -> Self {
        Self { publisher }
    }
}

#[async_trait]
impl Processor for JupiterSwapProcessor {
    type InputType = (
        InstructionMetadata,
        DecodedInstruction<JupiterSwapInstruction>,
        NestedInstructions,
        solana_instruction::Instruction,
    );

    async fn process(
        &mut self,
        (metadata, instruction, _, _): Self::InputType,
        _metrics: Arc<MetricsCollection>,
    ) -> CarbonResult<()> {
        let signature = metadata.transaction_metadata.signature.to_string();
        let platform = "Jupiter Swap".to_string();
        let timestamp = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();

        let (event_type, details) = match instruction.data {
            JupiterSwapInstruction::Route(route) => {
                ("swap", json!({
                    "type": "Route",
                    "platform_fee_bps": route.platform_fee_bps,
                    "in_amount": route.in_amount,
                    "quoted_out_amount": route.quoted_out_amount
                }))
            }
            JupiterSwapInstruction::ExactOutRoute(exact_out_route) => {
                ("swap", json!({
                    "type": "ExactOutRoute",
                    "platform_fee_bps": exact_out_route.platform_fee_bps,
                    "out_amount": exact_out_route.out_amount,
                    "quoted_in_amount": exact_out_route.quoted_in_amount
                }))
            }
            _ => return Ok(()),
        };

        self.process_event(event_type, platform, signature, timestamp, details).await
    }
}

// Orca Whirlpool Processor
pub struct OrcaWhirlpoolProcessor {
    publisher: UnifiedPublisher,
}

impl OrcaWhirlpoolProcessor {
    pub fn new(publisher: UnifiedPublisher) -> Self {
        Self { publisher }
    }
}

#[async_trait]
impl Processor for OrcaWhirlpoolProcessor {
    type InputType = (
        InstructionMetadata,
        DecodedInstruction<OrcaWhirlpoolInstruction>,
        NestedInstructions,
        solana_instruction::Instruction,
    );

    async fn process(
        &mut self,
        (metadata, instruction, _, _): Self::InputType,
        _metrics: Arc<MetricsCollection>,
    ) -> CarbonResult<()> {
        let signature = metadata.transaction_metadata.signature.to_string();
        let platform = "Orca Whirlpool".to_string();
        let timestamp = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();

        let (event_type, details) = match instruction.data {
            OrcaWhirlpoolInstruction::Swap(swap) => {
                ("swap", json!({
                    "type": "Swap",
                    "amount": swap.amount,
                    "other_amount_threshold": swap.other_amount_threshold,
                    "sqrt_price_limit": swap.sqrt_price_limit
                }))
            }
            OrcaWhirlpoolInstruction::IncreaseLiquidity(increase) => {
                ("liquidity", json!({
                    "type": "add",
                    "action": "IncreaseLiquidity",
                    "liquidity_amount": increase.liquidity_amount,
                    "token_max_a": increase.token_max_a,
                    "token_max_b": increase.token_max_b
                }))
            }
            OrcaWhirlpoolInstruction::DecreaseLiquidity(decrease) => {
                ("liquidity", json!({
                    "type": "remove",
                    "action": "DecreaseLiquidity",
                    "liquidity_amount": decrease.liquidity_amount,
                    "token_min_a": decrease.token_min_a,
                    "token_min_b": decrease.token_min_b
                }))
            }
            OrcaWhirlpoolInstruction::InitializePool(init) => {
                ("new_pool", json!({
                    "type": "InitializePool",
                    "tick_spacing": init.tick_spacing,
                    "initial_sqrt_price": init.initial_sqrt_price
                }))
            }
            _ => return Ok(()),
        };

        self.process_event(event_type, platform, signature, timestamp, details).await
    }
}

// Meteora DLMM Processor
pub struct MeteoraDlmmProcessor {
    publisher: UnifiedPublisher,
}

impl MeteoraDlmmProcessor {
    pub fn new(publisher: UnifiedPublisher) -> Self {
        Self { publisher }
    }
}

#[async_trait]
impl Processor for MeteoraDlmmProcessor {
    type InputType = (
        InstructionMetadata,
        DecodedInstruction<MeteoraDlmmInstruction>,
        NestedInstructions,
        solana_instruction::Instruction,
    );

    async fn process(
        &mut self,
        (metadata, instruction, _, _): Self::InputType,
        _metrics: Arc<MetricsCollection>,
    ) -> CarbonResult<()> {
        let signature = metadata.transaction_metadata.signature.to_string();
        let platform = "Meteora DLMM".to_string();
        let timestamp = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();

        let (event_type, details) = match instruction.data {
            MeteoraDlmmInstruction::Swap(swap) => {
                ("swap", json!({
                    "type": "Swap",
                    "amount_in": swap.amount_in
                }))
            }
            MeteoraDlmmInstruction::AddLiquidity(add_liquidity) => {
                ("liquidity", json!({
                    "type": "add",
                    "action": "AddLiquidity",
                    "liquidity_parameter": format!("{:?}", add_liquidity.liquidity_parameter)
                }))
            }
            MeteoraDlmmInstruction::RemoveLiquidity(remove_liquidity) => {
                ("liquidity", json!({
                    "type": "remove",
                    "action": "RemoveLiquidity",
                    "bin_liquidity_removal": format!("{:?}", remove_liquidity.bin_liquidity_removal)
                }))
            }
            MeteoraDlmmInstruction::InitializeLbPair(init) => {
                ("new_pool", json!({
                    "type": "InitializeLbPair",
                    "active_id": init.active_id,
                    "bin_step": init.bin_step
                }))
            }
            _ => return Ok(()),
        };

        self.process_event(event_type, platform, signature, timestamp, details).await
    }
}

// Các processors khác tương tự...
macro_rules! simple_processor {
    ($name:ident, $instruction_type:ty, $platform_name:expr) => {
        pub struct $name {
            publisher: UnifiedPublisher,
        }

        impl $name {
            pub fn new(publisher: UnifiedPublisher) -> Self {
                Self { publisher }
            }
        }

        #[async_trait]
        impl Processor for $name {
            type InputType = (
                InstructionMetadata,
                DecodedInstruction<$instruction_type>,
                NestedInstructions,
                solana_instruction::Instruction,
            );

            async fn process(
                &mut self,
                (metadata, instruction, _, _): Self::InputType,
                _metrics: Arc<MetricsCollection>,
            ) -> CarbonResult<()> {
                let signature = metadata.transaction_metadata.signature.to_string();
                let platform = $platform_name.to_string();
                let timestamp = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
                
                let details = json!({
                    "instruction": format!("{:?}", instruction.data)
                });

                self.process_event("swap", platform, signature, timestamp, details).await
            }
        }
    };
}

simple_processor!(OpenbookV2Processor, OpenbookV2Instruction, "OpenBook V2");
simple_processor!(PhoenixProcessor, PhoenixInstruction, "Phoenix V1");
simple_processor!(FluxbeamProcessor, FluxbeamInstruction, "Fluxbeam");
simple_processor!(LifinityAmmV2Processor, LifinityAmmV2Instruction, "Lifinity AMM V2");
simple_processor!(MoonshotProcessor, MoonshotInstruction, "Moonshot");

// Shared helper implementation for all processors
impl RaydiumCpmmProcessor {
    async fn process_event(&self, event_type: &str, platform: String, signature: String, timestamp: u64, details: serde_json::Value) -> CarbonResult<()> {
        self.common_process_event(event_type, platform, signature, timestamp, details).await
    }
}

impl JupiterSwapProcessor {
    async fn process_event(&self, event_type: &str, platform: String, signature: String, timestamp: u64, details: serde_json::Value) -> CarbonResult<()> {
        self.common_process_event(event_type, platform, signature, timestamp, details).await
    }
}

impl OrcaWhirlpoolProcessor {
    async fn process_event(&self, event_type: &str, platform: String, signature: String, timestamp: u64, details: serde_json::Value) -> CarbonResult<()> {
        self.common_process_event(event_type, platform, signature, timestamp, details).await
    }
}

impl MeteoraDlmmProcessor {
    async fn process_event(&self, event_type: &str, platform: String, signature: String, timestamp: u64, details: serde_json::Value) -> CarbonResult<()> {
        self.common_process_event(event_type, platform, signature, timestamp, details).await
    }
}

impl OpenbookV2Processor {
    async fn process_event(&self, event_type: &str, platform: String, signature: String, timestamp: u64, details: serde_json::Value) -> CarbonResult<()> {
        self.common_process_event(event_type, platform, signature, timestamp, details).await
    }
}

impl PhoenixProcessor {
    async fn process_event(&self, event_type: &str, platform: String, signature: String, timestamp: u64, details: serde_json::Value) -> CarbonResult<()> {
        self.common_process_event(event_type, platform, signature, timestamp, details).await
    }
}

impl FluxbeamProcessor {
    async fn process_event(&self, event_type: &str, platform: String, signature: String, timestamp: u64, details: serde_json::Value) -> CarbonResult<()> {
        self.common_process_event(event_type, platform, signature, timestamp, details).await
    }
}

impl LifinityAmmV2Processor {
    async fn process_event(&self, event_type: &str, platform: String, signature: String, timestamp: u64, details: serde_json::Value) -> CarbonResult<()> {
        self.common_process_event(event_type, platform, signature, timestamp, details).await
    }
}

impl MoonshotProcessor {
    async fn process_event(&self, event_type: &str, platform: String, signature: String, timestamp: u64, details: serde_json::Value) -> CarbonResult<()> {
        self.common_process_event(event_type, platform, signature, timestamp, details).await
    }
}

// Trait for common event processing
trait CommonProcessor {
    fn get_publisher(&self) -> &UnifiedPublisher;
    
    async fn common_process_event(&self, event_type: &str, platform: String, signature: String, timestamp: u64, details: serde_json::Value) -> CarbonResult<()> {
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
        if let Err(e) = self.get_publisher().publish("dex_events", &zmq_data).await {
            log::error!("Failed to publish to ZeroMQ: {}", e);
        }

        Ok(())
    }
}

// Implement the trait for all processors
impl CommonProcessor for RaydiumCpmmProcessor {
    fn get_publisher(&self) -> &UnifiedPublisher { &self.publisher }
}

impl CommonProcessor for JupiterSwapProcessor {
    fn get_publisher(&self) -> &UnifiedPublisher { &self.publisher }
}

impl CommonProcessor for OrcaWhirlpoolProcessor {
    fn get_publisher(&self) -> &UnifiedPublisher { &self.publisher }
}

impl CommonProcessor for MeteoraDlmmProcessor {
    fn get_publisher(&self) -> &UnifiedPublisher { &self.publisher }
}

impl CommonProcessor for OpenbookV2Processor {
    fn get_publisher(&self) -> &UnifiedPublisher { &self.publisher }
}

impl CommonProcessor for PhoenixProcessor {
    fn get_publisher(&self) -> &UnifiedPublisher { &self.publisher }
}

impl CommonProcessor for FluxbeamProcessor {
    fn get_publisher(&self) -> &UnifiedPublisher { &self.publisher }
}

impl CommonProcessor for LifinityAmmV2Processor {
    fn get_publisher(&self) -> &UnifiedPublisher { &self.publisher }
}

impl CommonProcessor for MoonshotProcessor {
    fn get_publisher(&self) -> &UnifiedPublisher { &self.publisher }
} 