use crate::models;
use chrono::{DateTime, Duration as ChronoDuration, Utc};
use futures_util::stream::StreamExt;
use lapin::message::Delivery;
use lapin::{
    BasicProperties,
    options::*,
    types::{AMQPValue, FieldTable},
};
use serde_json::Value;
use tracing::{error, info, warn};

/// Main worker loop: consumes messages and handles graceful shutdown.
pub async fn run_worker() -> Result<(), Box<dyn std::error::Error>> {
    let pool = crate::connection::get_rabbitmq_pool()
        .map_err(|e| format!("RabbitMQ pool not initialized: {}", e))?;

    let channel = pool.get_channel().await?;
    channel.basic_qos(1, BasicQosOptions::default()).await?;
    channel
        .queue_declare(
            "main_queue",
            QueueDeclareOptions {
                passive: true,
                ..Default::default()
            },
            FieldTable::default(),
        )
        .await?;

    let mut consumer = channel
        .basic_consume(
            "main_queue",
            "push_worker",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await?;

    let shutdown = tokio::signal::ctrl_c();
    tokio::pin!(shutdown);

    loop {
        tokio::select! {
            delivery_result = consumer.next() => match delivery_result {
                Some(Ok(delivery)) => handle_delivery(delivery).await,
                Some(Err(e)) => handle_consume_error(e).await,
                None => {
                    warn!("Consumer stream ended");
                    break;
                }
            },
            _ = &mut shutdown => {
                info!("🛑 Shutdown signal received. Closing gracefully...");
                break;
            }
        }
    }
    Ok(())
}

/// Handles a single delivery, including timeout and error logging.
async fn handle_delivery(delivery: Delivery) {
    if let Err(e) = process_message_with_timeout(delivery).await {
        error!("Failed to process message: {}", e);
    }
}

/// Handles consumer errors (e.g., connection issues).
async fn handle_consume_error(e: lapin::Error) {
    error!("Error receiving message: {}", e);
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

/// Processes a message with a timeout.
async fn process_message_with_timeout(
    delivery: Delivery,
) -> Result<(), Box<dyn std::error::Error>> {
    const PROCESSING_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);
    match tokio::time::timeout(PROCESSING_TIMEOUT, process_message(delivery)).await {
        Ok(result) => result,
        Err(_) => {
            error!(
                "⏰ Message processing timed out after {:?}",
                PROCESSING_TIMEOUT
            );
            Err("Processing timeout".into())
        }
    }
}

/// Processes a message: deserializes, schedules, or delivers.
async fn process_message(delivery: Delivery) -> Result<(), Box<dyn std::error::Error>> {
    let payload = delivery.data.clone();
    info!("📦 Received message of {} bytes", payload.len());
    let json_value: Value = match serde_json::from_slice(&payload) {
        Ok(val) => val,
        Err(e) => {
            error!("❌ Error deserializing message: {}", e);
            delivery
                .nack(BasicNackOptions {
                    requeue: false,
                    ..Default::default()
                })
                .await?;
            warn!("🗑️ Malformed message sent to dead letter queue");
            return Ok(());
        }
    };
    let scheduled_at = json_value
        .get("scheduled_at")
        .and_then(|v| v.as_str())
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc));
    let now = Utc::now();
    let max_delay = ChronoDuration::days(7);
    if let Some(scheduled_at) = scheduled_at {
        let remaining = scheduled_at - now;
        if remaining > max_delay || remaining > chrono::Duration::zero() {
            return reschedule_notification(
                &json_value,
                scheduled_at,
                remaining,
                max_delay,
                &delivery,
            )
            .await;
        }
    }
    handle_final_delivery(json_value, delivery, |notification| {
        let notification = notification.clone();
        Box::pin(process_notification_owned(notification))
    })
    .await
}

/// Converts owned notification to ref for processing.
async fn process_notification_owned(
    notification: models::Notification,
) -> Result<(), Box<dyn std::error::Error>> {
    process_notification(&notification).await
}

/// Simulates sending a push notification.
async fn process_notification(
    notification: &models::Notification,
) -> Result<(), Box<dyn std::error::Error>> {
    info!(
        "📲 Sending push notification to user: {}",
        notification.user_id
    );
    // Simulate realistic processing time
    let processing_time = match notification.notification_type.as_str() {
        "immediate" => std::time::Duration::from_millis(100),
        "delayed" => std::time::Duration::from_millis(200),
        "scheduled" => std::time::Duration::from_millis(150),
        _ => std::time::Duration::from_millis(100),
    };
    tokio::time::sleep(processing_time).await;
    info!(
        "✨ Push notification sent successfully to {}",
        notification.user_id
    );
    Ok(())
}

pub async fn reschedule_notification(
    json_value: &Value,
    scheduled_at: DateTime<Utc>,
    remaining: ChronoDuration,
    max_delay: ChronoDuration,
    delivery: &Delivery,
) -> Result<(), Box<dyn std::error::Error>> {
    // Usa el pool global en vez de crear una nueva conexión
    let pool = crate::connection::get_rabbitmq_pool()
        .map_err(|e| format!("RabbitMQ pool not initialized: {}", e))?;
    let channel = pool.get_channel().await?;
    let body = serde_json::to_vec(&json_value).map_err(|e| {
        error!("Serialization error: {}", e);
        e
    })?;
    let delay_ms = if remaining > max_delay {
        max_delay.num_milliseconds() as i32
    } else {
        remaining.num_milliseconds() as i32
    };
    let properties = BasicProperties::default().with_headers({
        let mut table = FieldTable::default();
        table.insert("x-delay".into(), AMQPValue::LongInt(delay_ms));
        table
    });
    channel
        .basic_publish(
            "delayed_exchange",
            "main",
            BasicPublishOptions::default(),
            &body,
            properties,
        )
        .await?
        .await?;
    if remaining > max_delay {
        info!(
            "🔄 Requeued notification for another 7 days (scheduled_at: {})",
            scheduled_at
        );
    } else {
        info!(
            "⏳ Requeued notification for remaining {} ms (scheduled_at: {})",
            remaining.num_milliseconds(),
            scheduled_at
        );
    }
    delivery.ack(BasicAckOptions::default()).await?;
    Ok(())
}

pub async fn handle_final_delivery(
    json_value: Value,
    delivery: Delivery,
    process_notification: fn(
        &models::Notification,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<(), Box<dyn std::error::Error>>> + Send>,
    >,
) -> Result<(), Box<dyn std::error::Error>> {
    match serde_json::from_value::<models::Notification>(json_value) {
        Ok(notification) => {
            info!(
                "📩 Processing notification: user_id={}, type={}, delay={}s",
                notification.user_id, notification.notification_type, notification.delay_secs
            );
            match process_notification(&notification).await {
                Ok(_) => {
                    delivery.ack(BasicAckOptions::default()).await?;
                    info!("✅ Message acknowledged successfully");
                }
                Err(e) => {
                    error!("❌ Failed to process notification: {}", e);
                    delivery
                        .nack(BasicNackOptions {
                            requeue: true,
                            ..Default::default()
                        })
                        .await?;
                    warn!("🔄 Message requeued for retry");
                }
            }
        }
        Err(e) => {
            error!("❌ Error deserializing notification: {}", e);
            delivery
                .nack(BasicNackOptions {
                    requeue: false,
                    ..Default::default()
                })
                .await?;
            warn!("🗑️ Malformed message sent to dead letter queue");
        }
    }
    Ok(())
}
