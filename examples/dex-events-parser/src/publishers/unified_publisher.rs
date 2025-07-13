use async_trait::async_trait;
use super::{common::DexEventData, traits::Publisher, ZmqPublisher, KafkaPublisher, ZmqPublisherError, KafkaPublisherError};

#[derive(Debug)]
pub enum UnifiedPublisherError {
    Zmq(ZmqPublisherError),
    Kafka(KafkaPublisherError),
    Multi(Vec<String>),
}

impl std::fmt::Display for UnifiedPublisherError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            UnifiedPublisherError::Zmq(e) => write!(f, "ZMQ Error: {}", e),
            UnifiedPublisherError::Kafka(e) => write!(f, "Kafka Error: {}", e),
            UnifiedPublisherError::Multi(errors) => write!(f, "Multiple errors: {}", errors.join(", ")),
        }
    }
}

impl std::error::Error for UnifiedPublisherError {}

#[derive(Clone)]
pub enum UnifiedPublisher {
    Zmq(ZmqPublisher),
    Kafka(KafkaPublisher),
    Multi(MultiPublisher),
}

#[async_trait]
impl Publisher for UnifiedPublisher {
    type Error = UnifiedPublisherError;
    
    async fn publish(&self, topic: &str, data: &DexEventData) -> Result<(), Self::Error> {
        match self {
            UnifiedPublisher::Zmq(publisher) => publisher.publish(topic, data).await.map_err(UnifiedPublisherError::Zmq),
            UnifiedPublisher::Kafka(publisher) => publisher.publish(topic, data).await.map_err(UnifiedPublisherError::Kafka),
            UnifiedPublisher::Multi(publisher) => publisher.publish(topic, data).await.map_err(UnifiedPublisherError::Multi),
        }
    }
    
    async fn close(&self) -> Result<(), Self::Error> {
        match self {
            UnifiedPublisher::Zmq(publisher) => publisher.close().await.map_err(UnifiedPublisherError::Zmq),
            UnifiedPublisher::Kafka(publisher) => publisher.close().await.map_err(UnifiedPublisherError::Kafka),
            UnifiedPublisher::Multi(publisher) => publisher.close().await.map_err(UnifiedPublisherError::Multi),
        }
    }
}

#[derive(Clone)]
pub struct MultiPublisher {
    zmq_publisher: Option<ZmqPublisher>,
    kafka_publisher: Option<KafkaPublisher>,
}

impl MultiPublisher {
    pub fn new() -> Self {
        Self {
            zmq_publisher: None,
            kafka_publisher: None,
        }
    }
    
    pub fn with_zmq(mut self, publisher: ZmqPublisher) -> Self {
        self.zmq_publisher = Some(publisher);
        self
    }
    
    pub fn with_kafka(mut self, publisher: KafkaPublisher) -> Self {
        self.kafka_publisher = Some(publisher);
        self
    }
    
    pub async fn publish(&self, topic: &str, data: &DexEventData) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        
        if let Some(zmq) = &self.zmq_publisher {
            if let Err(e) = zmq.publish(topic, data).await {
                errors.push(format!("ZMQ: {}", e));
            }
        }
        
        if let Some(kafka) = &self.kafka_publisher {
            if let Err(e) = kafka.publish(topic, data).await {
                errors.push(format!("Kafka: {}", e));
            }
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
    
    pub async fn close(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        
        if let Some(zmq) = &self.zmq_publisher {
            if let Err(e) = zmq.close().await {
                errors.push(format!("ZMQ: {}", e));
            }
        }
        
        if let Some(kafka) = &self.kafka_publisher {
            if let Err(e) = kafka.close().await {
                errors.push(format!("Kafka: {}", e));
            }
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl UnifiedPublisher {
    pub fn zmq(publisher: ZmqPublisher) -> Self {
        UnifiedPublisher::Zmq(publisher)
    }
    
    pub fn kafka(publisher: KafkaPublisher) -> Self {
        UnifiedPublisher::Kafka(publisher)
    }
    
    pub fn multi(publisher: MultiPublisher) -> Self {
        UnifiedPublisher::Multi(publisher)
    }
} 