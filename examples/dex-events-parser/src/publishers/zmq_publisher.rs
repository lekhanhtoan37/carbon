use std::sync::Arc;
use tokio::sync::Mutex;
use async_trait::async_trait;
use super::{common::DexEventData, traits::Publisher};

#[derive(Debug)]
pub struct ZmqPublisherError(pub String);

impl std::fmt::Display for ZmqPublisherError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "ZMQ Publisher Error: {}", self.0)
    }
}

impl std::error::Error for ZmqPublisherError {}

pub struct ZmqPublisher {
    context: Arc<Mutex<zmq::Context>>,
    socket: Arc<Mutex<zmq::Socket>>,
}

impl ZmqPublisher {
    pub fn new(endpoint: &str) -> Result<Self, ZmqPublisherError> {
        let context = zmq::Context::new();
        let socket = context.socket(zmq::PUB)
            .map_err(|e| ZmqPublisherError(format!("Failed to create socket: {}", e)))?;
        socket.bind(endpoint)
            .map_err(|e| ZmqPublisherError(format!("Failed to bind to {}: {}", endpoint, e)))?;
        
        Ok(Self {
            context: Arc::new(Mutex::new(context)),
            socket: Arc::new(Mutex::new(socket)),
        })
    }
}

#[async_trait]
impl Publisher for ZmqPublisher {
    type Error = ZmqPublisherError;

    async fn publish(&self, topic: &str, data: &DexEventData) -> Result<(), Self::Error> {
        let socket = self.socket.lock().await;
        let json_data = serde_json::to_string(data)
            .map_err(|e| ZmqPublisherError(format!("Failed to serialize data: {}", e)))?;
        
        socket.send_multipart([topic.as_bytes(), json_data.as_bytes()], 0)
            .map_err(|e| ZmqPublisherError(format!("Failed to send message: {}", e)))?;
        
        Ok(())
    }

    async fn close(&self) -> Result<(), Self::Error> {
        // ZMQ socket will be closed when dropped
        Ok(())
    }
}

impl Clone for ZmqPublisher {
    fn clone(&self) -> Self {
        Self {
            context: Arc::clone(&self.context),
            socket: Arc::clone(&self.socket),
        }
    }
} 