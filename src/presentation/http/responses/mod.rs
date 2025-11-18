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
