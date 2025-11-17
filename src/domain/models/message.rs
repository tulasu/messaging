use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::messenger::MessengerType;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    PlainText,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageStatus {
    Pending,
    Scheduled,
    InFlight,
    Sent,
    Retrying { reason: String, attempts: u32 },
    Failed { reason: String, attempts: u32 },
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageContent {
    pub body: String,
    pub message_type: MessageType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageHistoryEntry {
    pub id: Uuid,
    pub user_id: Uuid,
    pub messenger: MessengerType,
    pub recipient: String,
    pub content: MessageContent,
    pub status: MessageStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub attempts: u32,
    pub requested_by: RequestedBy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RequestedBy {
    System,
    User,
}

