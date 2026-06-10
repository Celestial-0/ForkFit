use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::common::id::{BackgroundJobId, UserId, NotificationLogId};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct BackgroundJob {
    pub id: BackgroundJobId,
    pub queue_name: String,
    pub job_type: String,
    pub payload: serde_json::Value,
    pub status: String, // 'queued', 'processing', 'completed', 'failed'
    pub attempts: i32,
    pub max_attempts: i32,
    pub run_at: DateTime<Utc>,
    pub locked_at: Option<DateTime<Utc>>,
    pub locked_by: Option<Uuid>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error_log: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct NotificationLog {
    pub id: NotificationLogId,
    pub user_id: Option<UserId>,
    pub channel: String, // 'email', 'sms'
    pub recipient: String,
    pub status: String, // 'queued', 'sent', 'failed'
    pub provider: Option<String>,
    pub provider_message_id: Option<String>,
    pub error_message: Option<String>,
    pub sent_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}
