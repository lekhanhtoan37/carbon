use {
    async_trait::async_trait, 
    carbon_core::{
        error::CarbonResult,
        instruction::{DecodedInstruction, InstructionMetadata, NestedInstructions},
        metrics::MetricsCollection,
        processor::Processor,
    }, 
    carbon_lifinity_amm_v2_decoder::instructions::LifinityAmmV2Instruction, 
    carbon_moonshot_decoder::instructions::MoonshotInstruction, 
    std::{sync::Arc, time::SystemTime},
    serde_json::json,
};

// Import all decoder instruction types
use carbon_raydium_amm_v4_decoder::instructions::RaydiumAmmV4Instruction;
use carbon_raydium_clmm_decoder::instructions::RaydiumClmmInstruction;
use carbon_raydium_cpmm_decoder::instructions::RaydiumCpmmInstruction;
use carbon_jupiter_swap_decoder::instructions::JupiterSwapInstruction;
use carbon_orca_whirlpool_decoder::instructions::OrcaWhirlpoolInstruction;
use carbon_meteora_dlmm_decoder::instructions::MeteoraDlmmInstruction;
use carbon_pumpfun_decoder::instructions::PumpfunInstruction;
use carbon_openbook_v2_decoder::instructions::OpenbookV2Instruction;
use carbon_phoenix_v1_decoder::instructions::PhoenixInstruction;
use carbon_fluxbeam_decoder::instructions::FluxbeamInstruction;

use crate::DexEvent;

// Raydium AMM V4 Processor
pub struct RaydiumAmmV4Processor;

impl RaydiumAmmV4Processor {
    pub fn new() -> Self {
        Self
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

        let event = match instruction.data {
            RaydiumAmmV4Instruction::SwapBaseIn(swap) => {
                Some(DexEvent::Swap {
                    platform,
                    signature,
                    details: format!("SwapBaseIn: amount_in={}, minimum_amount_out={}", 
                                   swap.amount_in, swap.minimum_amount_out),
                })
            }
            RaydiumAmmV4Instruction::SwapBaseOut(swap) => {
                Some(DexEvent::Swap {
                    platform,
                    signature,
                    details: format!("SwapBaseOut: max_amount_in={}, amount_out={}", 
                                   swap.max_amount_in, swap.amount_out),
                })
            }
            RaydiumAmmV4Instruction::Deposit(deposit) => {
                Some(DexEvent::AddLiquidity {
                    platform,
                    signature,
                    details: format!("Deposit: max_coin_amount={}, max_pc_amount={}, base_side={}", 
                                   deposit.max_coin_amount, deposit.max_pc_amount, deposit.base_side),
                })
            }
            RaydiumAmmV4Instruction::Withdraw(withdraw) => {
                Some(DexEvent::RemoveLiquidity {
                    platform,
                    signature,
                    details: format!("Withdraw: amount={}", withdraw.amount),
                })
            }
            RaydiumAmmV4Instruction::Initialize(init) => {
                Some(DexEvent::AddPair {
                    platform,
                    signature,
                    details: format!("Initialize: nonce={}", init.nonce),
                })
            }
            RaydiumAmmV4Instruction::Initialize2(init) => {
                Some(DexEvent::AddPair {
                    platform,
                    signature,
                    details: format!("Initialize2: nonce={}, open_time={}", init.nonce, init.open_time),
                })
            }
            RaydiumAmmV4Instruction::PreInitialize(pre_init) => {
                Some(DexEvent::AddPair {
                    platform,
                    signature,
                    details: format!("PreInitialize: nonce={}", pre_init.nonce),
                })
            }
            _ => None,
        };

        if let Some(event) = event {
            event.log();
        }

        Ok(())
    }
}

// Raydium CLMM Processor
pub struct RaydiumClmmProcessor;

impl RaydiumClmmProcessor {
    pub fn new() -> Self {
        Self
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

        let event = match instruction.data {
            RaydiumClmmInstruction::Swap(swap) => {
                Some(DexEvent::Swap {
                    platform,
                    signature,
                    details: format!("Swap: amount={}, other_amount_threshold={}, sqrt_price_limit_x64={}", 
                                   swap.amount, swap.other_amount_threshold, swap.sqrt_price_limit_x64),
                })
            }
            RaydiumClmmInstruction::SwapV2(swap) => {
                Some(DexEvent::Swap {
                    platform,
                    signature,
                    details: format!("SwapV2: amount={}, other_amount_threshold={}, sqrt_price_limit_x64={}", 
                                   swap.amount, swap.other_amount_threshold, swap.sqrt_price_limit_x64),
                })
            }
            RaydiumClmmInstruction::IncreaseLiquidity(increase) => {
                Some(DexEvent::AddLiquidity {
                    platform,
                    signature,
                    details: format!("IncreaseLiquidity: liquidity={}, amount_0_max={}, amount_1_max={}", 
                                   increase.liquidity, increase.amount0_max, increase.amount1_max),
                })
            }
            RaydiumClmmInstruction::IncreaseLiquidityV2(increase) => {
                Some(DexEvent::AddLiquidity {
                    platform,
                    signature,
                    details: format!("IncreaseLiquidityV2: liquidity={}, amount_0_max={}, amount_1_max={}", 
                                   increase.liquidity, increase.amount0_max, increase.amount1_max),
                })
            }
            RaydiumClmmInstruction::DecreaseLiquidity(decrease) => {
                Some(DexEvent::RemoveLiquidity {
                    platform,
                    signature,
                    details: format!("DecreaseLiquidity: liquidity={}, amount_0_min={}, amount_1_min={}", 
                                   decrease.liquidity, decrease.amount0_min, decrease.amount1_min),
                })
            }
            RaydiumClmmInstruction::DecreaseLiquidityV2(decrease) => {
                Some(DexEvent::RemoveLiquidity {
                    platform,
                    signature,
                    details: format!("DecreaseLiquidityV2: liquidity={}, amount_0_min={}, amount_1_min={}", 
                                   decrease.liquidity, decrease.amount0_min, decrease.amount1_min),
                })
            }
            // Note: InitializePool variants are not available in this version
            _ => None,
        };

        if let Some(event) = event {
            event.log();
        }

        Ok(())
    }
}

// Raydium CPMM Processor
pub struct RaydiumCpmmProcessor;

impl RaydiumCpmmProcessor {
    pub fn new() -> Self {
        Self
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

        let event = match instruction.data {
            RaydiumCpmmInstruction::SwapBaseInput(swap) => {
                Some(DexEvent::Swap {
                    platform,
                    signature,
                    details: format!("SwapBaseInput: amount_in={}, minimum_amount_out={}", 
                                   swap.amount_in, swap.minimum_amount_out),
                })
            }
            RaydiumCpmmInstruction::SwapBaseOutput(swap) => {
                Some(DexEvent::Swap {
                    platform,
                    signature,
                    details: format!("SwapBaseOutput: max_amount_in={}, amount_out={}", 
                                   swap.max_amount_in, swap.amount_out),
                })
            }
            _ => None,
        };

        if let Some(event) = event {
            event.log();
        }

        Ok(())
    }
}

// Jupiter Swap Processor
pub struct JupiterSwapProcessor;

impl JupiterSwapProcessor {
    pub fn new() -> Self {
        Self
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

        let event = match instruction.data {
            JupiterSwapInstruction::Route(route) => {
                Some(DexEvent::Swap {
                    platform,
                    signature,
                    details: format!("Route: platform_fee_bps={}, in_amount={}, quoted_out_amount={}", 
                                   route.platform_fee_bps, route.in_amount, route.quoted_out_amount),
                })
            }
            JupiterSwapInstruction::ExactOutRoute(exact_out_route) => {
                Some(DexEvent::Swap {
                    platform,
                    signature,
                    details: format!("ExactOutRoute: platform_fee_bps={}, out_amount={}, quoted_in_amount={}", 
                                   exact_out_route.platform_fee_bps, exact_out_route.out_amount, exact_out_route.quoted_in_amount),
                })
            }
            JupiterSwapInstruction::SharedAccountsRoute(shared_route) => {
                Some(DexEvent::Swap {
                    platform,
                    signature,
                    details: format!("SharedAccountsRoute: id={}, in_amount={}, quoted_out_amount={}", 
                                   shared_route.id, shared_route.in_amount, shared_route.quoted_out_amount),
                })
            }
            JupiterSwapInstruction::SharedAccountsExactOutRoute(shared_exact_out) => {
                Some(DexEvent::Swap {
                    platform,
                    signature,
                    details: format!("SharedAccountsExactOutRoute: id={}, out_amount={}, quoted_in_amount={}", 
                                   shared_exact_out.id, shared_exact_out.out_amount, shared_exact_out.quoted_in_amount),
                })
            }
            JupiterSwapInstruction::SwapEvent(swap_event) => {
                Some(DexEvent::Swap {
                    platform,
                    signature,
                    details: format!("SwapEvent: amm={}, input_amount={}, output_amount={}", 
                                   swap_event.amm, swap_event.input_amount, swap_event.output_amount),
                })
            }
            _ => None,
        };

        if let Some(event) = event {
            event.log();
        }

        Ok(())
    }
}

// Orca Whirlpool Processor
pub struct OrcaWhirlpoolProcessor;

impl OrcaWhirlpoolProcessor {
    pub fn new() -> Self {
        Self
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

        let event = match instruction.data {
            OrcaWhirlpoolInstruction::Swap(swap) => {
                Some(DexEvent::Swap {
                    platform,
                    signature,
                    details: format!("Swap: amount={}, other_amount_threshold={}, sqrt_price_limit={}", 
                                   swap.amount, swap.other_amount_threshold, swap.sqrt_price_limit),
                })
            }
            OrcaWhirlpoolInstruction::SwapV2(swap) => {
                Some(DexEvent::Swap {
                    platform,
                    signature,
                    details: format!("SwapV2: amount={}, other_amount_threshold={}, sqrt_price_limit={}", 
                                   swap.amount, swap.other_amount_threshold, swap.sqrt_price_limit),
                })
            }
            OrcaWhirlpoolInstruction::TwoHopSwap(two_hop) => {
                Some(DexEvent::Swap {
                    platform,
                    signature,
                    details: format!("TwoHopSwap: amount={}, other_amount_threshold={}", 
                                   two_hop.amount, two_hop.other_amount_threshold),
                })
            }
            OrcaWhirlpoolInstruction::TwoHopSwapV2(two_hop) => {
                Some(DexEvent::Swap {
                    platform,
                    signature,
                    details: format!("TwoHopSwapV2: amount={}, other_amount_threshold={}", 
                                   two_hop.amount, two_hop.other_amount_threshold),
                })
            }
            OrcaWhirlpoolInstruction::IncreaseLiquidity(increase) => {
                Some(DexEvent::AddLiquidity {
                    platform,
                    signature,
                    details: format!("IncreaseLiquidity: liquidity_amount={}, token_max_a={}, token_max_b={}", 
                                   increase.liquidity_amount, increase.token_max_a, increase.token_max_b),
                })
            }
            OrcaWhirlpoolInstruction::IncreaseLiquidityV2(increase) => {
                Some(DexEvent::AddLiquidity {
                    platform,
                    signature,
                    details: format!("IncreaseLiquidityV2: liquidity_amount={}, token_max_a={}, token_max_b={}", 
                                   increase.liquidity_amount, increase.token_max_a, increase.token_max_b),
                })
            }
            OrcaWhirlpoolInstruction::DecreaseLiquidity(decrease) => {
                Some(DexEvent::RemoveLiquidity {
                    platform,
                    signature,
                    details: format!("DecreaseLiquidity: liquidity_amount={}, token_min_a={}, token_min_b={}", 
                                   decrease.liquidity_amount, decrease.token_min_a, decrease.token_min_b),
                })
            }
            OrcaWhirlpoolInstruction::DecreaseLiquidityV2(decrease) => {
                Some(DexEvent::RemoveLiquidity {
                    platform,
                    signature,
                    details: format!("DecreaseLiquidityV2: liquidity_amount={}, token_min_a={}, token_min_b={}", 
                                   decrease.liquidity_amount, decrease.token_min_a, decrease.token_min_b),
                })
            }
            OrcaWhirlpoolInstruction::InitializePool(init) => {
                Some(DexEvent::AddPair {
                    platform,
                    signature,
                    details: format!("InitializePool: tick_spacing={}, initial_sqrt_price={}", 
                                   init.tick_spacing, init.initial_sqrt_price),
                })
            }
            OrcaWhirlpoolInstruction::InitializePoolV2(init) => {
                Some(DexEvent::AddPair {
                    platform,
                    signature,
                    details: format!("InitializePoolV2: tick_spacing={}, initial_sqrt_price={}", 
                                   init.tick_spacing, init.initial_sqrt_price),
                })
            }
            _ => None,
        };

        if let Some(event) = event {
            event.log();
        }

        Ok(())
    }
}

// Meteora DLMM Processor
pub struct MeteoraDlmmProcessor;

impl MeteoraDlmmProcessor {
    pub fn new() -> Self {
        Self
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

        let event = match instruction.data {
            MeteoraDlmmInstruction::Swap(swap) => {
                Some(DexEvent::Swap {
                    platform,
                    signature,
                    details: format!("Swap: amount={}", swap.amount_in),
                })
            }
            MeteoraDlmmInstruction::Swap2(swap) => {
                Some(DexEvent::Swap {
                    platform,
                    signature,
                    details: format!("Swap2: amount_in={}, min_amount_out={}", 
                                   swap.amount_in, swap.min_amount_out),
                })
            }
            MeteoraDlmmInstruction::SwapExactOut(swap_exact_out) => {
                Some(DexEvent::Swap {
                    platform,
                    signature,
                    details: format!("SwapExactOut: max_amount_in={}, amount_out={}", 
                                   swap_exact_out.max_in_amount, swap_exact_out.out_amount),
                })
            }
            MeteoraDlmmInstruction::SwapExactOut2(swap_exact_out) => {
                Some(DexEvent::Swap {
                    platform,
                    signature,
                    details: format!("SwapExactOut2: max_amount_in={}, amount_out={}", 
                                   swap_exact_out.max_in_amount, swap_exact_out.out_amount),
                })
            }
            MeteoraDlmmInstruction::AddLiquidity(add_liquidity) => {
                Some(DexEvent::AddLiquidity {
                    platform,
                    signature,
                    details: format!("AddLiquidity: liquidity_parameter={:?}", add_liquidity.liquidity_parameter),
                })
            }
            MeteoraDlmmInstruction::AddLiquidity2(add_liquidity) => {
                Some(DexEvent::AddLiquidity {
                    platform,
                    signature,
                    details: format!("AddLiquidity2: liquidity_parameter={:?}", add_liquidity.liquidity_parameter),
                })
            }
            MeteoraDlmmInstruction::RemoveLiquidity(remove_liquidity) => {
                Some(DexEvent::RemoveLiquidity {
                    platform,
                    signature,
                    details: format!("RemoveLiquidity: bin_liquidity_removal={:?}", remove_liquidity.bin_liquidity_removal),
                })
            }
            MeteoraDlmmInstruction::RemoveLiquidity2(remove_liquidity) => {
                Some(DexEvent::RemoveLiquidity {
                    platform,
                    signature,
                    details: format!("RemoveLiquidity2: bin_liquidity_removal={:?}", remove_liquidity.bin_liquidity_removal),
                })
            }
            MeteoraDlmmInstruction::InitializeLbPair(init) => {
                Some(DexEvent::AddPair {
                    platform,
                    signature,
                    details: format!("InitializeLbPair: active_id={}, bin_step={}", 
                                   init.active_id, init.bin_step),
                })
            }
            MeteoraDlmmInstruction::InitializeLbPair2(init) => {
                Some(DexEvent::AddPair {
                    platform,
                    signature,
                    details: format!("InitializeLbPair2: active_id={}", 
                                   init.params.active_id),
                })
            }
            _ => None,
        };

        if let Some(event) = event {
            event.log();
        }

        Ok(())
    }
}

// Pumpfun Processor
pub struct PumpfunProcessor;

impl PumpfunProcessor {
    pub fn new() -> Self {
        Self
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

        let event = match instruction.data {
            PumpfunInstruction::Buy(buy) => {
                Some(DexEvent::Swap {
                    platform,
                    signature,
                    details: format!("Buy: amount={}, max_sol_cost={}", buy.amount, buy.max_sol_cost),
                })
            }
            PumpfunInstruction::Sell(sell) => {
                Some(DexEvent::Swap {
                    platform,
                    signature,
                    details: format!("Sell: amount={}, min_sol_output={}", sell.amount, sell.min_sol_output),
                })
            }
            PumpfunInstruction::TradeEvent(trade) => {
                Some(DexEvent::Swap {
                    platform,
                    signature,
                    details: format!("TradeEvent: mint={}, sol_amount={}, token_amount={}, is_buy={}", 
                                   trade.mint, trade.sol_amount, trade.token_amount, trade.is_buy),
                })
            }
            PumpfunInstruction::CreateEvent(create) => {
                Some(DexEvent::AddPair {
                    platform,
                    signature,
                    details: format!("CreateEvent: mint={}, name={}, symbol={}", 
                                   create.mint, create.name, create.symbol),
                })
            }
            PumpfunInstruction::CompleteEvent(complete) => {
                Some(DexEvent::AddPair {
                    platform,
                    signature,
                    details: format!("CompleteEvent: mint={}, bonding_curve={}", 
                                   complete.mint, complete.bonding_curve),
                })
            }
            _ => None,
        };

        if let Some(event) = event {
            event.log();
        }

        Ok(())
    }
}

// OpenBook V2 Processor
pub struct OpenbookV2Processor;

impl OpenbookV2Processor {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Processor for OpenbookV2Processor {
    type InputType = (
        InstructionMetadata,
        DecodedInstruction<OpenbookV2Instruction>,
        NestedInstructions,
        solana_instruction::Instruction,
    );

    async fn process(
        &mut self,
        (metadata, instruction, _, _): Self::InputType,
        _metrics: Arc<MetricsCollection>,
    ) -> CarbonResult<()> {
        let signature = metadata.transaction_metadata.signature.to_string();
        let platform = "OpenBook V2".to_string();

        let event = match instruction.data {
            OpenbookV2Instruction::PlaceTakeOrder(order) => {
                Some(DexEvent::Swap {
                    platform,
                    signature,
                    details: format!("PlaceTakeOrder: side={:?}, price_lots={}, max_base_lots={}", 
                                   order.args.side, order.args.price_lots, order.args.max_base_lots),
                })
            }
            OpenbookV2Instruction::CreateMarket(_) => {
                Some(DexEvent::AddPair {
                    platform,
                    signature,
                    details: "CreateMarket".to_string(),
                })
            }
            _ => None,
        };

        if let Some(event) = event {
            event.log();
        }

        Ok(())
    }
}

// Phoenix V1 Processor
pub struct PhoenixProcessor;

impl PhoenixProcessor {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Processor for PhoenixProcessor {
    type InputType = (
        InstructionMetadata,
        DecodedInstruction<PhoenixInstruction>,
        NestedInstructions,
        solana_instruction::Instruction,
    );

    async fn process(
        &mut self,
        (metadata, instruction, _, _): Self::InputType,
        _metrics: Arc<MetricsCollection>,
    ) -> CarbonResult<()> {
        let signature = metadata.transaction_metadata.signature.to_string();
        let platform = "Phoenix V1".to_string();

        let event = match instruction.data {
            PhoenixInstruction::PlaceLimitOrder(order) => {
                Some(DexEvent::Swap {
                    platform,
                    signature,
                    details: format!("PlaceLimitOrder: packet={:?}", order.order_packet),
                })
            }
            PhoenixInstruction::InitializeMarket(_) => {
                Some(DexEvent::AddPair {
                    platform,
                    signature,
                    details: "InitializeMarket".to_string(),
                })
            }
            _ => None,
        };

        if let Some(event) = event {
            event.log();
        }

        Ok(())
    }
}

// Fluxbeam Processor
pub struct FluxbeamProcessor;

impl FluxbeamProcessor {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Processor for FluxbeamProcessor {
    type InputType = (
        InstructionMetadata,
        DecodedInstruction<FluxbeamInstruction>,
        NestedInstructions,
        solana_instruction::Instruction,
    );

    async fn process(
        &mut self,
        (metadata, instruction, _, _): Self::InputType,
        _metrics: Arc<MetricsCollection>,
    ) -> CarbonResult<()> {
        let signature = metadata.transaction_metadata.signature.to_string();
        let platform = "Fluxbeam".to_string();

        let event = match instruction.data {
            FluxbeamInstruction::Swap(swap) => {
                Some(DexEvent::Swap {
                    platform,
                    signature,
                    details: format!("Swap: amount_in={}", swap.amount_in),
                })
            }
            _ => None,
        };

        if let Some(event) = event {
            event.log();
        }

        Ok(())
    }
}

// Lifinity AMM V2 Processor
pub struct LifinityAmmV2Processor;

impl LifinityAmmV2Processor {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Processor for LifinityAmmV2Processor {
    type InputType = (
        InstructionMetadata,
        DecodedInstruction<LifinityAmmV2Instruction>,
        NestedInstructions,
        solana_instruction::Instruction,
    );

    async fn process(
        &mut self,
        (metadata, instruction, _, _): Self::InputType,
        _metrics: Arc<MetricsCollection>,
    ) -> CarbonResult<()> {
        let signature = metadata.transaction_metadata.signature.to_string();
        let platform = "Lifinity AMM V2".to_string();

        let event = match instruction.data {
            LifinityAmmV2Instruction::Swap(swap) => {
                Some(DexEvent::Swap {
                    platform,
                    signature,
                    details: format!("Swap: amount_in={}, minimum_amount_out={}", 
                                   swap.amount_in, swap.minimum_amount_out),
                })
            }
            _ => None,
        };

        if let Some(event) = event {
            event.log();
        }

        Ok(())
    }
}

// Moonshot Processor
pub struct MoonshotProcessor;

impl MoonshotProcessor {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Processor for MoonshotProcessor {
    type InputType = (
        InstructionMetadata,
        DecodedInstruction<MoonshotInstruction>,
        NestedInstructions,
        solana_instruction::Instruction,
    );

    async fn process(
        &mut self,
        (metadata, instruction, _, _): Self::InputType,
        _metrics: Arc<MetricsCollection>,
    ) -> CarbonResult<()> {
        let signature = metadata.transaction_metadata.signature.to_string();
        let platform = "Moonshot".to_string();

        let event = match instruction.data {
            MoonshotInstruction::Buy(buy) => {
                Some(DexEvent::Swap {
                    platform,
                    signature,
                    details: format!("Buy: collateral_amount={}, token_amount={}", 
                                   buy.data.collateral_amount, buy.data.token_amount),
                })
            }
            MoonshotInstruction::Sell(sell) => {
                Some(DexEvent::Swap {
                    platform,
                    signature,
                    details: format!("Sell: token_amount={}, collateral_amount={}", 
                                   sell.data.token_amount, sell.data.collateral_amount),
                })
            }

            _ => None,
        };

        if let Some(event) = event {
            event.log();
        }

        Ok(())
    }
} 