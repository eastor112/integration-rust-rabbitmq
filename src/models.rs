use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Notification {
    pub user_id: String,
    pub message: String,
    pub delay_secs: u64,
}
