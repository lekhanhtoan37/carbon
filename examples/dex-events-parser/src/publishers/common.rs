use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DexEventData {
    pub event_type: String,  // "swap", "mint_burn", "liquidity", "new_pool"
    pub platform: String,
    pub signature: String,
    pub timestamp: u64,
    pub details: serde_json::Value,
} 