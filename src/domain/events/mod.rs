use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::models::{MessageContent, MessageType, MessengerType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutboundMessageEvent {
    pub event_id: Uuid,
    pub message_id: Uuid,
    pub user_id: Uuid,
    pub messenger: MessengerType,
    pub recipient: String,
    pub message_type: MessageType,
    pub content: MessageContent,
    pub attempt: u32,
    pub max_attempts: u32,
    pub scheduled_at: DateTime<Utc>,
}
