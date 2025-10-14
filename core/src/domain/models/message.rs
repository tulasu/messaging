use crate::domain::models::{ChatId, MessengerType, Payload};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Message {
    pub id: Uuid,
    pub payload: Payload,
    pub destinations: Vec<MessageDestination>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct MessageDestination {
    pub id: Uuid,
    pub message_id: Uuid,
    pub messenger_type: MessengerType,
    pub chat_id: ChatId,
    pub status: DeliveryStatus,
    pub retry_count: u32,
    pub last_attempt: Option<DateTime<Utc>>,
    pub sent_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DeliveryStatus {
    Pending,
    Queued,
    Processing,
    Sent,
    Failed,
    RetryScheduled,
}
