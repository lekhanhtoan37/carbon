use async_trait::async_trait;
use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::util::Timeout;
use std::sync::Arc;
use super::{common::DexEventData, traits::Publisher};

#[derive(Debug)]
pub struct KafkaPublisherError(pub String);

impl std::fmt::Display for KafkaPublisherError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Kafka Publisher Error: {}", self.0)
    }
}

impl std::error::Error for KafkaPublisherError {}

#[derive(Clone)]
pub struct KafkaPublisher {
    producer: Arc<FutureProducer>,
    timeout: Timeout,
}

impl KafkaPublisher {
    // pub fn new(brokers: &str, timeout_ms: u64) -> Result<Self, KafkaPublisherError> {
    //     let producer: FutureProducer = ClientConfig::new()
    //         .set("bootstrap.servers", brokers)
    //         .set("message.timeout.ms", "5000")
    //         .create()
    //         .map_err(|e| KafkaPublisherError(format!("Failed to create producer: {}", e)))?;

    //     Ok(Self {
    //         producer: Arc::new(producer),
    //         timeout: Timeout::After(std::time::Duration::from_millis(timeout_ms)),
    //     })
    // }

    pub fn new_with_config(config: ClientConfig, timeout_ms: u64) -> Result<Self, KafkaPublisherError> {
        let producer: FutureProducer = config
            .create()
            .map_err(|e| KafkaPublisherError(format!("Failed to create producer: {}", e)))?;

        Ok(Self {
            producer: Arc::new(producer),
            timeout: Timeout::After(std::time::Duration::from_millis(timeout_ms)),
        })
    }
}

#[async_trait]
impl Publisher for KafkaPublisher {
    type Error = KafkaPublisherError;

    async fn publish(&self, topic: &str, data: &DexEventData) -> Result<(), Self::Error> {
        let json_data = serde_json::to_string(data)
            .map_err(|e| KafkaPublisherError(format!("Failed to serialize data: {}", e)))?;
        
        let key = format!("{}:{}", data.platform, data.signature);
        
        let record = FutureRecord::to(topic)
            .key(&key)
            .payload(&json_data);

        self.producer
            .send(record, self.timeout)
            .await
            .map_err(|(e, _)| KafkaPublisherError(format!("Failed to send message: {}", e)))?;

        Ok(())
    }

    async fn close(&self) -> Result<(), Self::Error> {
        // Kafka producer will be closed when dropped
        Ok(())
    }
} 