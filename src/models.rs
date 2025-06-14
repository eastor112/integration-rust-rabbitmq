use std::collections::HashMap;
use std::sync::Mutex;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Notification {
    pub user_id: String,
    pub message: String,
    #[serde(default)]
    pub delay_secs: u64,
    #[serde(default)]
    pub notification_type: String, // "immediate", "delayed", "scheduled"
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScheduledNotification {
    pub id: Uuid,
    pub user_id: String,
    pub scheduled_at: DateTime<Utc>,
    pub payload: serde_json::Value,
    pub status: String,
}

// In-memory database simulation
lazy_static::lazy_static! {
    pub static ref SCHEDULED_NOTIFICATIONS: Mutex<HashMap<Uuid, ScheduledNotification>> = Mutex::new(HashMap::new());
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScheduleNotificationRequest {
    pub user_id: String,
    pub scheduled_at: DateTime<Utc>,
    pub payload: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScheduleAtRequest {
    pub user_id: String,
    pub message: String,
    pub scheduled_at: DateTime<Utc>,
}
