use std::time::{SystemTime, UNIX_EPOCH};
use tokio;
use publishers::{
    DexEventData, Publisher, ZmqPublisher, KafkaPublisher, ZmqPublisherError, KafkaPublisherError
};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Example data
    let sample_data = DexEventData {
        event_type: "swap".to_string(),
        platform: "Raydium AMM V4".to_string(),
        signature: "sample_signature_123".to_string(),
        timestamp: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        details: json!({
            "amount_in": 1000000,
            "amount_out": 950000,
            "token_a": "So11111111111111111111111111111111111111112",
            "token_b": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
        }),
    };

    // ZeroMQ Publisher Example
    println!("Testing ZeroMQ Publisher...");
    let zmq_publisher = ZmqPublisher::new("tcp://*:5555")?;
    zmq_publisher.publish("dex_events", &sample_data).await?;
    println!("Published to ZeroMQ successfully");

    // Kafka Publisher Example
    println!("Testing Kafka Publisher...");
    let kafka_publisher = KafkaPublisher::new("localhost:9092", 5000)?;
    kafka_publisher.publish("dex_events", &sample_data).await?;
    println!("Published to Kafka successfully");

    // Generic Publisher Example
    println!("Testing Generic Publisher...");
    let publisher: Box<dyn Publisher<Error = Box<dyn std::error::Error + Send + Sync>>> = 
        Box::new(zmq_publisher);
    publisher.publish("dex_events", &sample_data).await?;
    println!("Published via generic publisher successfully");

    Ok(())
}

// Example of how to create publishers from environment variables
pub fn create_publisher_from_env() -> Result<Box<dyn Publisher<Error = Box<dyn std::error::Error + Send + Sync>>>, Box<dyn std::error::Error + Send + Sync>> {
    let publisher_type = std::env::var("PUBLISHER_TYPE").unwrap_or_else(|_| "zmq".to_string());
    
    match publisher_type.as_str() {
        "kafka" => {
            let brokers = std::env::var("KAFKA_BROKERS")?;
            let timeout = std::env::var("KAFKA_TIMEOUT_MS")?
                .parse::<u64>()
                .unwrap_or(5000);
            let publisher = KafkaPublisher::new(&brokers, timeout)?;
            Ok(Box::new(publisher))
        }
        "zmq" => {
            let endpoint = std::env::var("ZMQ_ENDPOINT")?;
            let publisher = ZmqPublisher::new(&endpoint)?;
            Ok(Box::new(publisher))
        }
        _ => Err("Unsupported publisher type".into())
    }
}

// Example of publishing different event types
pub async fn publish_events_example() -> Result<(), Box<dyn std::error::Error>> {
    let publisher = ZmqPublisher::new("tcp://*:5555")?;
    
    // Swap event
    let swap_data = DexEventData {
        event_type: "swap".to_string(),
        platform: "Raydium AMM V4".to_string(),
        signature: "swap_signature_123".to_string(),
        timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        details: json!({"amount_in": 1000000, "amount_out": 950000}),
    };
    publisher.publish("dex_events", &swap_data).await?;
    
    // Liquidity event
    let liquidity_data = DexEventData {
        event_type: "liquidity".to_string(),
        platform: "Orca Whirlpool".to_string(),
        signature: "liquidity_signature_456".to_string(),
        timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        details: json!({"liquidity_amount": 5000000, "token_a": "SOL", "token_b": "USDC"}),
    };
    publisher.publish("dex_events", &liquidity_data).await?;
    
    // New pool event
    let new_pool_data = DexEventData {
        event_type: "new_pool".to_string(),
        platform: "Meteora DLMM".to_string(),
        signature: "new_pool_signature_789".to_string(),
        timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        details: json!({"active_id": 8000, "bin_step": 25}),
    };
    publisher.publish("dex_events", &new_pool_data).await?;
    
    Ok(())
} 