use crate::domain::models::{ChatId, MessengerType};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DomainEvent {
    MessageCreated(MessageCreated),
    MessageQueued(MessageQueued),
    MessageProcessing(MessageProcessing),
    MessageSent(MessageSent),
    MessageFailed(MessageFailed),
    MessageRetryScheduled(MessageRetryScheduled),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageCreated {
    pub message_id: Uuid,
    pub destinations: Vec<(MessengerType, ChatId)>,
    pub occurred_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageQueued {
    pub message_id: Uuid,
    pub destination_id: Uuid,
    pub messenger_type: MessengerType,
    pub occurred_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageProcessing {
    pub message_id: Uuid,
    pub destination_id: Uuid,
    pub messenger_type: MessengerType,
    pub occurred_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageSent {
    pub message_id: Uuid,
    pub destination_id: Uuid,
    pub messenger_type: MessengerType,
    pub chat_id: String,
    pub platform_message_id: Option<String>,
    pub occurred_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageFailed {
    pub message_id: Uuid,
    pub destination_id: Uuid,
    pub messenger_type: MessengerType,
    pub error: String,
    pub retry_count: u32,
    pub occurred_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageRetryScheduled {
    pub message_id: Uuid,
    pub destination_id: Uuid,
    pub messenger_type: MessengerType,
    pub retry_count: u32,
    pub scheduled_at: DateTime<Utc>,
}

impl DomainEvent {
    pub fn message_id(&self) -> Uuid {
        match self {
            DomainEvent::MessageCreated(event) => event.message_id,
            DomainEvent::MessageQueued(event) => event.message_id,
            DomainEvent::MessageProcessing(event) => event.message_id,
            DomainEvent::MessageSent(event) => event.message_id,
            DomainEvent::MessageFailed(event) => event.message_id,
            DomainEvent::MessageRetryScheduled(event) => event.message_id,
        }
    }

    pub fn occurred_at(&self) -> DateTime<Utc> {
        match self {
            DomainEvent::MessageCreated(event) => event.occurred_at,
            DomainEvent::MessageQueued(event) => event.occurred_at,
            DomainEvent::MessageProcessing(event) => event.occurred_at,
            DomainEvent::MessageSent(event) => event.occurred_at,
            DomainEvent::MessageFailed(event) => event.occurred_at,
            DomainEvent::MessageRetryScheduled(event) => event.scheduled_at,
        }
    }
}
