use lapin::{
    options::*, types::FieldTable, Connection, ConnectionProperties,
};
use futures_util::stream::StreamExt;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Notification {
    user_id: String,
    message: String,
}

#[tokio::main]
async fn main() {
    env_logger::init();
    println!("🔧 Worker listening...");

    let conn = Connection::connect("amqp://guest:guest@localhost:5672/%2f", ConnectionProperties::default())
        .await
        .expect("❌ Cannot connect to RabbitMQ");

    let channel = conn.create_channel().await.expect("❌ Cannot create channel");

    channel.queue_declare(
        "main_queue",
        QueueDeclareOptions::default(),
        FieldTable::default(),
    )
    .await
    .expect("❌ Cannot declare queue");

    let mut consumer = channel
        .basic_consume(
            "main_queue",
            "push_worker",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await
        .expect("❌ Cannot consume");

    println!("✅ Worker ready to receive messages...");

    while let Some(delivery) = consumer.next().await {
        if let Ok(delivery) = delivery {
            let payload = delivery.data.clone();
            match serde_json::from_slice::<Notification>(&payload) {
                Ok(notification) => {
                    log::info!("📩 Received: {:?}", notification);
                    log::info!("📲 Push sent to {}: {}", notification.user_id, notification.message);
                    // Si el procesamiento es exitoso, hacemos ack
                    delivery.ack(BasicAckOptions::default()).await.unwrap();
                }
                Err(e) => {
                    log::error!("❌ Error deserializing message: {}", e);
                    // Si falla la deserialización, mandamos a la DLQ (no requeue)
                    delivery.nack(BasicNackOptions { requeue: false, ..Default::default() }).await.unwrap();
                }
            }
        }
    }
}
