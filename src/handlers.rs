use actix_web::{post, web, HttpResponse};
use crate::models::Notification;
use lapin::{options::*, types::FieldTable, BasicProperties, Connection, ConnectionProperties};
use serde_json::to_vec;

#[post("/notify")]
pub async fn send_notification(payload: web::Json<Notification>) -> HttpResponse {
    let conn = Connection::connect("amqp://guest:guest@localhost:5672/%2f", ConnectionProperties::default())
        .await
        .expect("❌ RabbitMQ connection failed");

    let channel = conn.create_channel().await.expect("❌ Failed to create channel");

    channel.queue_declare(
        "notification_queue",
        QueueDeclareOptions::default(),
        FieldTable::default(),
    ).await.expect("❌ Failed to declare queue");

    let body = to_vec(&payload.into_inner()).expect("❌ Failed to serialize payload");

    channel
        .basic_publish(
            "",
            "notification_queue",
            BasicPublishOptions::default(),
            &body,
            BasicProperties::default(),
        )
        .await
        .expect("❌ Failed to publish message")
        .await
        .expect("❌ Failed to confirm");

    HttpResponse::Ok().body("📨 Notification enqueued")
}
