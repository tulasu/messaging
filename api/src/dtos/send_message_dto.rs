use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct SendMessageRequest {
    pub content: String,
    pub format: Option<String>,
    pub destinations: Vec<DestinationDto>,
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

impl From<MessengerTypeDto> for messaging::domain::MessengerType {
    fn from(dto: MessengerTypeDto) -> Self {
        match dto {
            MessengerTypeDto::Telegram => Self::Telegram,
            MessengerTypeDto::Vk => Self::VK,
            MessengerTypeDto::Max => Self::MAX,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct MessageResponse {
    pub id: Uuid,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
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

impl From<messaging::domain::DeliveryStatus> for DeliveryStatusDto {
    fn from(status: messaging::domain::DeliveryStatus) -> Self {
        match status {
            messaging::domain::DeliveryStatus::Pending => Self::Pending,
            messaging::domain::DeliveryStatus::Queued => Self::Queued,
            messaging::domain::DeliveryStatus::Sent => Self::Sent,
            messaging::domain::DeliveryStatus::Failed => Self::Failed,
            messaging::domain::DeliveryStatus::RetryScheduled => Self::RetryScheduled,
            _ => Self::Pending,
        }
    }
}