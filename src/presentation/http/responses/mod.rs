use poem_openapi::{Enum, Object};
use uuid::Uuid;

use crate::presentation::models::{ChatTypeKind, MessageStatusDto, MessengerKind, RequestedByKind};

#[derive(Object)]
pub struct AuthResponseDto {
    pub success: bool,
}

#[derive(Object)]
pub struct MessengerTokenDto {
    pub id: Uuid,
    pub messenger: MessengerKind,
    pub status: MessengerTokenStatusDto,
    pub updated_at: String,
}

#[derive(Enum, Copy, Clone, Debug, Eq, PartialEq)]
pub enum MessengerTokenStatusDto {
    Active,
    Inactive,
}

#[derive(Object)]
pub struct SendMessageResponseDto {
    pub message_id: Uuid,
}

#[derive(Object)]
pub struct MessageHistoryDto {
    pub id: Uuid,
    pub messenger: MessengerKind,
    pub recipient: String,
    pub status: MessageStatusDto,
    pub attempts: u32,
    pub body: String,
    pub last_error: Option<String>,
    pub requested_by: RequestedByKind,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Object)]
pub struct MessengerChatDto {
    pub messenger: MessengerKind,
    pub chat_id: String,
    pub title: String,
    pub chat_type: ChatTypeKind,
    pub can_send_messages: bool,
}

#[derive(Object)]
pub struct PaginatedChatsDto {
    pub chats: Vec<MessengerChatDto>,
    pub has_more: bool,
    pub next_offset: Option<u32>,
}

#[derive(Object)]
pub struct PaginatedMessagesDto {
    pub messages: Vec<MessageHistoryDto>,
    pub has_more: bool,
    pub next_offset: Option<u32>,
}

#[derive(Object)]
pub struct MessageAttemptDto {
    pub id: Uuid,
    pub message_id: Uuid,
    pub attempt_number: u32,
    pub status: MessageStatusDto,
    pub status_reason: Option<String>,
    pub requested_by: RequestedByKind,
    pub created_at: String,
}

#[derive(Object)]
pub struct BatchSendItemResultDto {
    pub index: u32,
    pub success: bool,
    pub message_id: Option<Uuid>,
    pub error: Option<String>,
}

#[derive(Object)]
pub struct BatchSendResponseDto {
    pub results: Vec<BatchSendItemResultDto>,
    pub total: u32,
    pub successful: u32,
    pub failed: u32,
}
