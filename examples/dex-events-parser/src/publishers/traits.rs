use async_trait::async_trait;
use super::common::DexEventData;

#[async_trait]
pub trait Publisher: Send + Sync {
    type Error: std::error::Error + Send + Sync + 'static;
    
    async fn publish(&self, topic: &str, data: &DexEventData) -> Result<(), Self::Error>;
    
    async fn close(&self) -> Result<(), Self::Error>;
} 