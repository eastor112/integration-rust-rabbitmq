use actix_web::{post, web, HttpResponse, Result as ActixResult};
use crate::models::{Notification, ScheduledNotification, ScheduleNotificationRequest, ScheduleAtRequest, SCHEDULED_NOTIFICATIONS};
use crate::connection::get_rabbitmq_pool;
use lapin::{options::*, types::{FieldTable, AMQPValue}, BasicProperties, Channel};
use serde_json::{to_vec, json};
use uuid::Uuid;
use chrono::{Utc};
use tracing::{info, error};

// Reusable helper function to publish notifications
async fn publish_notification(
    channel: &Channel,
    notification: &Notification,
    delay_ms: i32
) -> Result<(), String> {
    let body = to_vec(notification).map_err(|e| format!("Serialization error: {}", e))?;

    let properties = if delay_ms > 0 {
        BasicProperties::default().with_headers({
            let mut table = FieldTable::default();
            table.insert("x-delay".into(), AMQPValue::LongInt(delay_ms));
            table
        })
    } else {
        BasicProperties::default()
    };

    channel
        .basic_publish(
            "delayed_exchange",
            "main",
            BasicPublishOptions::default(),
            &body,
            properties,
        )
        .await
        .map_err(|e| format!("Failed to publish message: {}", e))?
        .await
        .map_err(|e| format!("Failed to confirm message: {}", e))?;

    Ok(())
}

#[post("/notify")]
pub async fn send_notification(payload: web::Json<Notification>) -> ActixResult<HttpResponse> {
    let pool = get_rabbitmq_pool().map_err(|e| {
        error!("RabbitMQ pool error: {}", e);
        actix_web::error::ErrorInternalServerError(e)
    })?;

    let channel = pool.get_channel().await.map_err(|e| {
        error!("Failed to get channel: {}", e);
        actix_web::error::ErrorInternalServerError(format!("Failed to get channel: {}", e))
    })?;

    let mut notification = payload.into_inner();
    notification.notification_type = "immediate".to_string();

    info!("📨 Sending immediate notification to user: {}", notification.user_id);

    if let Err(e) = publish_notification(&channel, &notification, 0).await {
        error!("Failed to publish immediate notification: {}", e);
        return Ok(HttpResponse::InternalServerError().json(json!({
            "error": "Failed to send notification",
            "details": e.to_string()
        })));
    }

    info!("✅ Immediate notification sent successfully");
    Ok(HttpResponse::Ok().json(json!({
        "status": "sent",
        "type": "immediate",
        "user_id": notification.user_id
    })))
}

#[post("/notify-delayed")]
pub async fn send_notification_delayed(payload: web::Json<Notification>) -> ActixResult<HttpResponse> {
    let pool = get_rabbitmq_pool().map_err(|e| {
        error!("RabbitMQ pool error: {}", e);
        actix_web::error::ErrorInternalServerError(e)
    })?;

    let channel = pool.get_channel().await.map_err(|e| {
        error!("Failed to get channel: {}", e);
        actix_web::error::ErrorInternalServerError(format!("Failed to get channel: {}", e))
    })?;

    let mut notification = payload.into_inner();
    notification.notification_type = "delayed".to_string();
    let delay_ms = (notification.delay_secs * 1000) as i32;

    info!("🕐 Sending delayed notification to user: {} with {}s delay",
          notification.user_id, notification.delay_secs);

    if let Err(e) = publish_notification(&channel, &notification, delay_ms).await {
        error!("Failed to publish delayed notification: {}", e);
        return Ok(HttpResponse::InternalServerError().json(json!({
            "error": "Failed to send delayed notification",
            "details": e.to_string()
        })));
    }

    info!("✅ Delayed notification scheduled successfully");
    Ok(HttpResponse::Ok().json(json!({
        "status": "scheduled",
        "type": "delayed",
        "user_id": notification.user_id,
        "delay_seconds": notification.delay_secs
    })))
}

#[post("/notify-at")]
pub async fn send_notification_at(payload: web::Json<ScheduleAtRequest>) -> ActixResult<HttpResponse> {
    let pool = get_rabbitmq_pool().map_err(|e| {
        error!("RabbitMQ pool error: {}", e);
        actix_web::error::ErrorInternalServerError(e)
    })?;

    let channel = pool.get_channel().await.map_err(|e| {
        error!("Failed to get channel: {}", e);
        actix_web::error::ErrorInternalServerError(format!("Failed to get channel: {}", e))
    })?;

    let now = Utc::now();
    let scheduled_at = payload.scheduled_at;
    let max_delay_ms = 7 * 24 * 60 * 60 * 1000; // 7 days in ms
    let delay_ms = (scheduled_at.timestamp_millis() - now.timestamp_millis()).max(0) as i64;

    // If the delay is greater than the maximum, only schedule 7 days and save the real date
    let (final_delay_ms, real_scheduled_at) = if delay_ms > max_delay_ms {
        (max_delay_ms, Some(scheduled_at))
    } else {
        (delay_ms, None)
    };

    // Notification with real scheduled_at if applicable
    let notification = Notification {
        user_id: payload.user_id.clone(),
        message: payload.message.clone(),
        delay_secs: 0,
        notification_type: "scheduled".to_string(),
    };

    // Pack the payload with the real date if applicable
    let mut json_payload = serde_json::to_value(&notification).unwrap();
    if let Some(real_at) = real_scheduled_at {
        json_payload["scheduled_at"] = serde_json::json!(real_at);
    } else {
        json_payload["scheduled_at"] = serde_json::json!(scheduled_at);
    }

    info!("🕐 Scheduling notification for user: {} at {} (delay {} ms, real_scheduled_at: {:?})",
          notification.user_id, scheduled_at, final_delay_ms, real_scheduled_at);

    // Use publish_notification but serializing the payload manually
    let body = serde_json::to_vec(&json_payload).map_err(|e| {
        error!("Serialization error: {}", e);
        actix_web::error::ErrorInternalServerError("Serialization error")
    })?;

    let properties = if final_delay_ms > 0 {
        BasicProperties::default().with_headers({
            let mut table = FieldTable::default();
            table.insert("x-delay".into(), AMQPValue::LongInt(final_delay_ms as i32));
            table
        })
    } else {
        BasicProperties::default()
    };

    channel
        .basic_publish(
            "delayed_exchange",
            "main",
            BasicPublishOptions::default(),
            &body,
            properties,
        )
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Failed to publish message: {}", e)))?
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Failed to confirm message: {}", e)))?;

    Ok(HttpResponse::Ok().json(json!({
        "status": "scheduled",
        "type": "scheduled",
        "user_id": notification.user_id,
        "scheduled_at": scheduled_at
    })))
}

#[post("/schedule-notification")]
pub async fn schedule_notification(payload: web::Json<ScheduleNotificationRequest>) -> ActixResult<HttpResponse> {
    let mut db = SCHEDULED_NOTIFICATIONS.lock().map_err(|e| {
        error!("Failed to lock scheduled notifications: {}", e);
        actix_web::error::ErrorInternalServerError("Database lock error")
    })?;

    let id = Uuid::new_v4();
    let notification = ScheduledNotification {
        id,
        user_id: payload.user_id.clone(),
        scheduled_at: payload.scheduled_at,
        payload: payload.payload.clone(),
        status: "pending".to_string(),
    };

    info!("📅 Scheduling notification {} for user: {} at {}",
          id, payload.user_id, payload.scheduled_at);

    db.insert(id, notification.clone());

    info!("✅ Notification scheduled successfully with ID: {}", id);
    Ok(HttpResponse::Ok().json(json!({
        "id": id,
        "status": "scheduled",
        "scheduled_at": payload.scheduled_at,
        "user_id": payload.user_id
    })))
}

pub async fn notification_scheduler_task() {
    info!("🕐 Starting notification scheduler task");

    loop {
        if let Err(e) = run_scheduler_cycle().await {
            error!("Scheduler cycle failed: {}", e);
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            continue;
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}

async fn run_scheduler_cycle() -> Result<(), String> {
    let pool = get_rabbitmq_pool().map_err(|e| format!("Failed to get pool: {}", e))?;
    let channel = pool.get_channel().await.map_err(|e| format!("Failed to get channel: {}", e))?;

    let mut notifications_to_send = Vec::new();

    // Collect pending notifications
    {
        let mut db = SCHEDULED_NOTIFICATIONS.lock().map_err(|e| {
            format!("Failed to lock scheduled notifications: {}", e)
        })?;

        let now = Utc::now();

        for (id, notification) in db.iter_mut() {
            if notification.status == "pending" && notification.scheduled_at <= now {
                notification.status = "processing".to_string();
                notifications_to_send.push((*id, notification.clone()));
            }
        }
    }

    // Process notifications
    for (id, scheduled_notification) in notifications_to_send {
        match process_scheduled_notification(&channel, &scheduled_notification).await {
            Ok(_) => {
                // Mark as sent
                if let Ok(mut db) = SCHEDULED_NOTIFICATIONS.lock() {
                    if let Some(notification) = db.get_mut(&id) {
                        notification.status = "sent".to_string();
                    }
                }
                info!("✅ Scheduled notification {} sent successfully", id);
            }
            Err(e) => {
                error!("Failed to send scheduled notification {}: {}", id, e);
                // Mark as failed
                if let Ok(mut db) = SCHEDULED_NOTIFICATIONS.lock() {
                    if let Some(notification) = db.get_mut(&id) {
                        notification.status = "failed".to_string();
                    }
                }
            }
        }
    }

    Ok(())
}

async fn process_scheduled_notification(
    channel: &Channel,
    scheduled_notification: &ScheduledNotification
) -> Result<(), String> {
    // Convert payload to Notification
    let mut notification: Notification = serde_json::from_value(scheduled_notification.payload.clone())
        .map_err(|e| format!("Failed to deserialize notification: {}", e))?;
    notification.notification_type = "scheduled".to_string();

    info!("📨 Processing scheduled notification for user: {}", notification.user_id);

    publish_notification(channel, &notification, 0).await?;

    Ok(())
}
