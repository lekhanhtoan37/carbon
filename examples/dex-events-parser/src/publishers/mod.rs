pub mod common;
pub mod traits;
pub mod zmq_publisher;
pub mod kafka_publisher;
pub mod unified_publisher;

// Re-export commonly used types
pub use common::DexEventData;
use rdkafka::ClientConfig;
pub use traits::Publisher;
pub use zmq_publisher::{ZmqPublisher, ZmqPublisherError};
pub use kafka_publisher::{KafkaPublisher, KafkaPublisherError};
pub use unified_publisher::{UnifiedPublisher, MultiPublisher};

// Helper function to create publishers from environment variables
pub fn create_unified_publisher_from_env() -> Result<UnifiedPublisher, Box<dyn std::error::Error + Send + Sync>> {
    match std::env::var("PUBLISHER_TYPE").as_deref() {
        Ok("zmq") => {
            let endpoint = std::env::var("ZMQ_ENDPOINT").unwrap_or_else(|_| "tcp://*:5555".to_string());
            let publisher = ZmqPublisher::new(&endpoint)?;
            Ok(UnifiedPublisher::zmq(publisher))
        }
        Ok("kafka") => {
            let brokers = std::env::var("KAFKA_BROKERS").unwrap_or_else(|_| "localhost:9092".to_string());
            let timeout = std::env::var("KAFKA_TIMEOUT_MS")
                .unwrap_or_else(|_| "5000".to_string())
                .parse::<u64>()
                .unwrap_or(5000);
            let publisher_config = ClientConfig::new()
                .set("bootstrap.servers", brokers)
                .set("message.timeout.ms", "5000")
                .clone();

            println!("Kafka publisher config: {:?}", publisher_config);

            let publisher = KafkaPublisher::new_with_config(publisher_config, timeout)?;

            Ok(UnifiedPublisher::kafka(publisher))
        }
        Ok("both") => {
            let zmq_endpoint = std::env::var("ZMQ_ENDPOINT").unwrap_or_else(|_| "tcp://*:5555".to_string());
            let zmq_publisher = ZmqPublisher::new(&zmq_endpoint)?;
            
            let brokers = std::env::var("KAFKA_BROKERS").unwrap_or_else(|_| "localhost:9092".to_string());
            // let kafka_timeout = std::env::var("KAFKA_TIMEOUT_MS")
            //     .unwrap_or_else(|_| "5000".to_string())
            //     .parse::<u64>()
            //     .unwrap_or(5000);
            // let kafka_publisher = KafkaPublisher::new(&kafka_brokers, kafka_timeout)?;
            let publisher_config = ClientConfig::new()
                .set("bootstrap.servers", brokers)
                .set("message.timeout.ms", "5000")
                .clone();

            println!("Kafka publisher config: {:?}", publisher_config);

            let publisher = KafkaPublisher::new_with_config(publisher_config, 5000)?;


            let multi_publisher = MultiPublisher::new()
                .with_zmq(zmq_publisher)
                .with_kafka(publisher);
            
            Ok(UnifiedPublisher::multi(multi_publisher))
        }
        _ => {
            // Default to ZMQ
            let endpoint = std::env::var("ZMQ_ENDPOINT").unwrap_or_else(|_| "tcp://*:5555".to_string());
            let publisher = ZmqPublisher::new(&endpoint)?;
            Ok(UnifiedPublisher::zmq(publisher))
        }
    }
} 