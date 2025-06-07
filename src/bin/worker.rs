use lapin::{
    options::*, types::FieldTable, Connection, ConnectionProperties,
};
use futures_util::stream::StreamExt;
use serde::{Deserialize, Serialize};

// Definimos la estructura Notification directamente aquí para el worker
#[derive(Debug, Serialize, Deserialize)]
struct Notification {
    user_id: String,
    message: String,
    #[serde(default)]
    delay_secs: u64,
    #[serde(default)]
    notification_type: String,
}

#[tokio::main]
async fn main() {
    env_logger::init();
    println!("🔧 Worker listening...");

    let conn = Connection::connect("amqp://guest:guest@localhost:5672/%2f", ConnectionProperties::default())
        .await
        .expect("❌ Cannot connect to RabbitMQ");

    let channel = conn.create_channel().await.expect("❌ Cannot create channel");

    // No need to declare queue - it already exists from docker-compose setup
    // Just verify it exists by doing a passive declaration
    channel.queue_declare(
        "main_queue",
        QueueDeclareOptions {
            passive: true,  // Only check if queue exists, don't create/modify
            ..Default::default()
        },
        FieldTable::default(),
    )
    .await
    .expect("❌ main_queue does not exist - make sure docker-compose setup ran");

    let mut consumer = channel
        .basic_consume(
            "main_queue",
            "push_worker",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await
        .expect("❌ Cannot consume");

    println!("✅ Worker ready to receive messages from main_queue...");
    println!("🔍 Waiting for messages. To exit press CTRL+C");

    while let Some(delivery) = consumer.next().await {
        match delivery {
            Ok(delivery) => {
                let payload = delivery.data.clone();
                println!("📦 Received message of {} bytes", payload.len());

                match serde_json::from_slice::<Notification>(&payload) {
                    Ok(notification) => {
                        println!("📩 Processing notification: {:?}", notification);
                        println!("📲 Push sent to {}: {}", notification.user_id, notification.message);

                        // Successful processing - acknowledge the message
                        if let Err(e) = delivery.ack(BasicAckOptions::default()).await {
                            println!("❌ Error acknowledging message: {}", e);
                        } else {
                            println!("✅ Message acknowledged successfully");
                        }
                    }
                    Err(e) => {
                        println!("❌ Error deserializing message: {}", e);
                        println!("📄 Raw payload: {:?}", String::from_utf8_lossy(&payload));

                        // Failed deserialization - send to DLQ (no requeue)
                        if let Err(e) = delivery.nack(BasicNackOptions { requeue: false, ..Default::default() }).await {
                            println!("❌ Error nacking message: {}", e);
                        } else {
                            println!("🗑️ Message sent to dead letter queue");
                        }
                    }
                }
            }
            Err(e) => {
                println!("❌ Error receiving message: {}", e);
                // Small delay to prevent tight loop on persistent errors
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
        }
    }
}
