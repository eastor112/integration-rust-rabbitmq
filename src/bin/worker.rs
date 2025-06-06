use lapin::{
    options::*, types::FieldTable, Connection, ConnectionProperties,
};
use futures_util::stream::StreamExt;
use serde::Deserialize;
use tokio::time::{sleep, Duration};

#[derive(Debug, Deserialize)]
struct Notification {
    user_id: String,
    message: String,
    delay_secs: u64,
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
        "notification_queue",
        QueueDeclareOptions::default(),
        FieldTable::default(),
    )
    .await
    .expect("❌ Cannot declare queue");

    let mut consumer = channel
        .basic_consume(
            "notification_queue",
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
            if let Ok(notification) = serde_json::from_slice::<Notification>(&payload) {
                println!("📩 Received: {:?}", notification);
                sleep(Duration::from_secs(notification.delay_secs)).await;
                println!("📲 Push sent to {}: {}", notification.user_id, notification.message);
            }

            delivery.ack(BasicAckOptions::default()).await.unwrap();
        }
    }
}
