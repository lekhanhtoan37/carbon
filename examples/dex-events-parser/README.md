# DEX Events Parser

Service này sử dụng RPC Block Subscribe để parse và phân tích tất cả các event từ các sàn DEX trên Solana, bao gồm:

## Các loại event được parse:

### 1. Swap Events
- **Raydium AMM V4**: SwapBaseIn, SwapBaseOut
- **Raydium CLMM**: Swap, SwapV2, TwoHopSwap, TwoHopSwapV2
- **Raydium CPMM**: Swap events
- **Jupiter**: Route events, SwapEvent
- **Orca Whirlpool**: Swap, SwapV2, TwoHopSwap, TwoHopSwapV2
- **Meteora DLMM**: Swap, Swap2, SwapExactOut, SwapExactOut2
- **Meteora Pools**: Swap events
- **Pumpfun**: Buy, Sell, TradeEvent
- **Pump Swap**: Swap events
- **OpenBook V2**: Swap events
- **Phoenix V1**: Swap events
- **Fluxbeam**: Swap events
- **Lifinity AMM V2**: Swap events
- **Moonshot**: Swap events
- **OKX DEX**: Swap events
- **Gavel**: Swap events
- **Virtuals**: Swap events
- **Virtual Curve**: Swap events
- **Zeta**: Swap events

### 2. Add Liquidity Events
- **Raydium AMM V4**: Deposit
- **Raydium CLMM**: IncreaseLiquidity, IncreaseLiquidityV2
- **Raydium CPMM**: Deposit events
- **Orca Whirlpool**: IncreaseLiquidity, IncreaseLiquidityV2
- **Meteora DLMM**: AddLiquidity, AddLiquidity2, AddLiquidityByStrategy, AddLiquidityByWeight
- **Meteora Pools**: AddLiquidity events
- **Kamino Farms**: Deposit events
- **Kamino Vault**: Deposit events
- **MarginFi V2**: Deposit events
- **Drift V2**: Deposit events
- **Marinade Finance**: Deposit events

### 3. Remove Liquidity Events
- **Raydium AMM V4**: Withdraw
- **Raydium CLMM**: DecreaseLiquidity, DecreaseLiquidityV2
- **Raydium CPMM**: Withdraw events
- **Orca Whirlpool**: DecreaseLiquidity, DecreaseLiquidityV2
- **Meteora DLMM**: RemoveLiquidity, RemoveLiquidity2, RemoveAllLiquidity
- **Meteora Pools**: RemoveLiquidity events
- **Kamino Farms**: Withdraw events
- **Kamino Vault**: Withdraw events
- **MarginFi V2**: Withdraw events
- **Drift V2**: Withdraw events
- **Marinade Finance**: Withdraw events

### 4. Add Pair/Pool Events
- **Raydium AMM V4**: Initialize, Initialize2, PreInitialize
- **Raydium CLMM**: InitializePool, InitializePoolV2
- **Raydium CPMM**: Initialize events
- **Orca Whirlpool**: InitializePool, InitializePoolV2
- **Meteora DLMM**: InitializeLbPair, InitializeLbPair2, InitializeCustomizablePermissionlessLbPair
- **Meteora Pools**: InitializePair events
- **Pumpfun**: CreateEvent
- **Openbook V2**: CreateMarket events
- **Phoenix V1**: CreateMarket events

## Các sàn DEX được hỗ trợ:

1. **Raydium** (AMM V4, CLMM, CPMM, Stable Swap, Launchpad, Liquidity Locking)
2. **Jupiter** (Swap, DCA, Limit Order, Perpetuals)
3. **Orca** (Whirlpool)
4. **Meteora** (DLMM, Pools, DAMM V2)
5. **Pumpfun**
6. **Pump Swap**
7. **OpenBook V2**
8. **Phoenix V1**
9. **Fluxbeam**
10. **Lifinity AMM V2**
11. **Stabble** (Stable Swap, Weighted Swap)
12. **Moonshot**
13. **OKX DEX**
14. **Gavel**
15. **Virtuals**
16. **Virtual Curve**
17. **Zeta**
18. **Kamino** (Farms, Lending, Vault, Limit Order)
19. **MarginFi V2**
20. **Drift V2**
21. **Marinade Finance**
22. **Boop**
23. **Sharky**
24. **Solayer Restaking Program**

## Cách sử dụng:

1. Cấu hình các biến môi trường:

### Sử dụng ZeroMQ (mặc định):
```bash
export RPC_WS_URL="wss://api.mainnet-beta.solana.com"
export PUBLISHER_TYPE="zmq"
export ZMQ_ENDPOINT="tcp://*:5555"
export RUST_LOG=info
```

### Sử dụng Kafka:
```bash
export RPC_WS_URL="wss://api.mainnet-beta.solana.com"
export PUBLISHER_TYPE="kafka"
export KAFKA_BROKERS="localhost:9092"
export KAFKA_TIMEOUT_MS="5000"
export RUST_LOG=info
```

2. Chạy service:
```bash
cargo run --bin carbon-dex-events-parser
```

## Cấu hình Publishers:

### ZeroMQ Publisher:
- `ZMQ_ENDPOINT`: Endpoint cho ZeroMQ (mặc định: `tcp://*:5555`)
- Topic: `dex_events`
- Format: Multipart message với topic và JSON data

### Kafka Publisher:
- `KAFKA_BROKERS`: Địa chỉ Kafka brokers (mặc định: `localhost:9092`)
- `KAFKA_TIMEOUT_MS`: Timeout cho producer (mặc định: `5000`)
- Topic: Có thể cấu hình trong code
- Key: `platform:signature`
- Value: JSON data

## Data Format:

Tất cả event đều được publish theo format JSON:
```json
{
  "event_type": "swap|mint_burn|liquidity|new_pool",
  "platform": "Raydium AMM V4",
  "signature": "transaction_signature",
  "timestamp": 1640995200,
  "details": {
    // Chi tiết event cụ thể
  }
}
```

## Cấu trúc Output:

Service sẽ log tất cả các event được parse theo format:
```
[EVENT_TYPE] [PLATFORM] [SIGNATURE] [DETAILS]
```

Ví dụ:
```
[SWAP] [Raydium AMM V4] [signature] SwapBaseIn: amount_in=1000000, amount_out=950000
[ADD_LIQUIDITY] [Orca Whirlpool] [signature] IncreaseLiquidity: liquidity_amount=5000000
[REMOVE_LIQUIDITY] [Meteora DLMM] [signature] RemoveLiquidity: amount=2000000
[ADD_PAIR] [Pumpfun] [signature] CreateEvent: mint=token_address, name=token_name
```

## Lưu ý:

- Service này sử dụng RPC Block Subscribe để nhận real-time data
- Tất cả các event đều được decode từ các instruction trong transaction
- Service có thể xử lý hàng nghìn transaction mỗi giây
- Cần cấu hình RPC endpoint có khả năng xử lý websocket connection 