use futures_util::StreamExt;
use lapin::{Connection, ConnectionProperties, options::*, types::FieldTable};
use push_rabbit_demo::Notification;
use tokio::time::{Duration, sleep};

#[tokio::main]
async fn main() {
    let conn = Connection::connect(
        "amqp://guest:guest@localhost:5672/%2f",
        ConnectionProperties::default(),
    )
    .await
    .expect("Failed to connect to RabbitMQ");

    let channel = conn.create_channel().await.expect("Create channel");

    channel
        .exchange_declare(
            "notifications",
            lapin::ExchangeKind::Fanout,
            ExchangeDeclareOptions::default(),
            FieldTable::default(),
        )
        .await
        .unwrap();

    let queue = channel
        .queue_declare(
            "",
            QueueDeclareOptions {
                exclusive: true,
                auto_delete: true,
                ..Default::default()
            },
            FieldTable::default(),
        )
        .await
        .unwrap();

    channel
        .queue_bind(
            queue.name().as_str(),
            "notifications",
            "",
            QueueBindOptions::default(),
            FieldTable::default(),
        )
        .await
        .unwrap();

    let mut consumer = channel
        .basic_consume(
            queue.name().as_str(),
            "notifier",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await
        .unwrap();

    println!("🔔 Notification worker ready...");

    while let Some(delivery) = consumer.next().await {
        if let Ok((_, delivery)) = delivery {
            let notification: Notification =
                serde_json::from_slice(&delivery.data).expect("Invalid message");

            let delay = notification.delay_secs.unwrap_or(0);
            println!(
                "📬 Received notification for user {}: '{}' (delayed {}s)",
                notification.user_id, notification.message, delay
            );

            let cloned = notification.clone();
            tokio::spawn(async move {
                sleep(Duration::from_secs(delay)).await;
                println!(
                    "📲 Push sent to user {}: {}",
                    cloned.user_id, cloned.message
                );
            });

            delivery
                .ack(lapin::options::BasicAckOptions::default())
                .await
                .unwrap();
        }
    }
}
