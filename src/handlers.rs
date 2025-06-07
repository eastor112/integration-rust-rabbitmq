use actix_web::{post, web, HttpResponse};
use crate::models::{Notification, ScheduledNotification, ScheduleNotificationRequest, SCHEDULED_NOTIFICATIONS};
use lapin::{options::*, types::{FieldTable, AMQPValue}, BasicProperties, Connection, ConnectionProperties};
use serde_json::to_vec;
use uuid::Uuid;
use chrono::{Utc};
use serde_json::json;
use std::time::Duration as StdDuration;
use tokio::time::sleep;

#[post("/notify")]
pub async fn send_notification(payload: web::Json<Notification>) -> HttpResponse {
    let conn = Connection::connect("amqp://guest:guest@localhost:5672/%2f", ConnectionProperties::default())
        .await
        .expect("❌ RabbitMQ connection failed");

    let channel = conn.create_channel().await.expect("❌ Failed to create channel");

    let mut notification = payload.into_inner();
    notification.notification_type = "immediate".to_string(); // Marcar tipo

    let body = to_vec(&notification).expect("❌ Failed to serialize payload");

    // Send immediate notification through delayed_exchange with 0 delay
    channel
        .basic_publish(
            "delayed_exchange",  // Use delayed_exchange instead of default exchange
            "main",             // routing key
            BasicPublishOptions::default(),
            &body,
            BasicProperties::default().with_headers({
                let mut table = FieldTable::default();
                table.insert("x-delay".into(), AMQPValue::LongInt(0)); // 0 delay = immediate
                table
            }),
        )
        .await
        .expect("❌ Failed to publish message")
        .await
        .expect("❌ Failed to confirm");

    HttpResponse::Ok().body("📨 Notification enqueued")
}

#[post("/notify-delayed")]
pub async fn send_notification_delayed(payload: web::Json<Notification>) -> HttpResponse {
    let conn = Connection::connect("amqp://guest:guest@localhost:5672/%2f", ConnectionProperties::default())
        .await
        .expect("❌ RabbitMQ connection failed");

    let channel = conn.create_channel().await.expect("❌ Failed to create channel");

    let mut notification = payload.into_inner();
    notification.notification_type = "delayed".to_string(); // Marcar tipo
    let delay_ms = notification.delay_secs * 1000;

    println!("🕐 Sending delayed notification with {} seconds delay ({} ms)", notification.delay_secs, delay_ms);

    let body = to_vec(&notification).expect("❌ Failed to serialize payload");

    channel
        .basic_publish(
            "delayed_exchange",
            "main",
            BasicPublishOptions::default(),
            &body,
            BasicProperties::default().with_headers({
                let mut table = FieldTable::default();
                table.insert("x-delay".into(), AMQPValue::LongInt(delay_ms as i32));
                table
            }),
        )
        .await
        .expect("❌ Failed to publish message")
        .await
        .expect("❌ Failed to confirm");

    HttpResponse::Ok().body(format!("📨 Notification enqueued with {} seconds delay", notification.delay_secs))
}

#[post("/schedule-notification")]
pub async fn schedule_notification(payload: web::Json<ScheduleNotificationRequest>) -> HttpResponse {
    let mut db = SCHEDULED_NOTIFICATIONS.lock().unwrap();
    let id = Uuid::new_v4();
    let notification = ScheduledNotification {
        id,
        user_id: payload.user_id.clone(),
        scheduled_at: payload.scheduled_at,
        payload: payload.payload.clone(),
        status: "pending".to_string(),
    };
    db.insert(id, notification.clone());
    HttpResponse::Ok().json(json!({"id": id}))
}

pub async fn notification_scheduler_task() {
    let amqp_addr = "amqp://guest:guest@localhost:5672/%2f";
    let conn = Connection::connect(amqp_addr, ConnectionProperties::default())
        .await
        .expect("❌ RabbitMQ connection failed");
    let channel = conn.create_channel().await.expect("❌ Failed to create channel");
    loop {
        let now = Utc::now();
        let to_send: Vec<ScheduledNotification> = {
            let mut db = SCHEDULED_NOTIFICATIONS.lock().unwrap();
            let mut to_send = vec![];
            for (_id, notif) in db.iter_mut() {
                if notif.status == "pending" && notif.scheduled_at <= now {
                    notif.status = "sent".to_string();
                    to_send.push(notif.clone());
                }
            }
            to_send
        };
        for notif in to_send {
            // Crear una notificación estándar desde la scheduled notification
            let standard_notification = Notification {
                user_id: notif.user_id.clone(),
                message: notif.payload.get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Scheduled notification")
                    .to_string(),
                delay_secs: notif.payload.get("delay_secs").and_then(|v| v.as_u64()).unwrap_or(0),
                notification_type: "scheduled".to_string(),
            };

            let body = serde_json::to_vec(&standard_notification).expect("❌ Failed to serialize payload");
            let delay_ms = standard_notification.delay_secs * 1000;

            let _ = channel.basic_publish(
                "delayed_exchange",
                "main",
                BasicPublishOptions::default(),
                &body,
                BasicProperties::default().with_headers({
                    let mut table = FieldTable::default();
                    table.insert("x-delay".into(), AMQPValue::LongInt(delay_ms as i32));
                    table
                }),
            ).await;
        }
        sleep(StdDuration::from_secs(1)).await;
    }
}
