use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct SendMessageRequest {
    pub payload: PayloadDto,
    pub destinations: Vec<DestinationDto>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum PayloadDto {
    Plain { content: String },
    Formatted {
        content: String,
        format: TextFormatDto
    },
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TextFormatDto {
    Plain,
    Markdown,
    Html,
}

#[derive(Debug, Deserialize)]
pub struct DestinationDto {
    pub messenger_type: MessengerTypeDto,
    pub chat_id: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MessengerTypeDto {
    Telegram,
    Vk,
    Max,
}

impl From<MessengerTypeDto> for crate::domain::models::MessengerType {
    fn from(dto: MessengerTypeDto) -> Self {
        match dto {
            MessengerTypeDto::Telegram => Self::Telegram,
            MessengerTypeDto::Vk => Self::VK,
            MessengerTypeDto::Max => Self::MAX,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct SendMessageResponse {
    pub message_id: Uuid,
    pub status: String,
    pub queued_destinations: Vec<QueuedDestinationDto>,
}

#[derive(Debug, Serialize)]
pub struct QueuedDestinationDto {
    pub destination_id: Uuid,
    pub messenger_type: MessengerTypeDto,
    pub chat_id: String,
}

#[derive(Debug, Serialize)]
pub struct MessageStatusResponse {
    pub message_id: Uuid,
    pub status: String,
    pub destinations: Vec<DestinationStatusDto>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct DestinationStatusDto {
    pub destination_id: Uuid,
    pub messenger_type: MessengerTypeDto,
    pub chat_id: String,
    pub status: DeliveryStatusDto,
    pub retry_count: u32,
    pub last_attempt: Option<chrono::DateTime<chrono::Utc>>,
    pub sent_at: Option<chrono::DateTime<chrono::Utc>>,
    pub error_message: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DeliveryStatusDto {
    Pending,
    Queued,
    Sent,
    Failed,
    RetryScheduled,
}

impl From<crate::domain::models::DeliveryStatus> for DeliveryStatusDto {
    fn from(status: crate::domain::models::DeliveryStatus) -> Self {
        match status {
            crate::domain::models::DeliveryStatus::Pending => Self::Pending,
            crate::domain::models::DeliveryStatus::Queued => Self::Queued,
            crate::domain::models::DeliveryStatus::Sent => Self::Sent,
            crate::domain::models::DeliveryStatus::Failed => Self::Failed,
            crate::domain::models::DeliveryStatus::RetryScheduled => Self::RetryScheduled,
        }
    }
}