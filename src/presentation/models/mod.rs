use poem_openapi::Enum;

use crate::domain::models::{MessageStatus, MessengerChatType, MessengerType, RequestedBy};

#[derive(Enum, Copy, Clone, Debug, Eq, PartialEq)]
pub enum MessengerKind {
    #[oai(rename = "telegram")]
    Telegram,
    #[oai(rename = "vk")]
    Vk,
}

impl From<MessengerKind> for MessengerType {
    fn from(value: MessengerKind) -> Self {
        match value {
            MessengerKind::Telegram => MessengerType::Telegram,
            MessengerKind::Vk => MessengerType::Vk,
        }
    }
}

impl From<MessengerType> for MessengerKind {
    fn from(value: MessengerType) -> Self {
        match value {
            MessengerType::Telegram => MessengerKind::Telegram,
            MessengerType::Vk => MessengerKind::Vk,
        }
    }
}

#[derive(Enum, Copy, Clone, Debug, Eq, PartialEq)]
pub enum RequestedByKind {
    #[oai(rename = "system")]
    System,
    #[oai(rename = "user")]
    User,
}

impl Default for RequestedByKind {
    fn default() -> Self {
        RequestedByKind::User
    }
}

impl From<RequestedByKind> for RequestedBy {
    fn from(value: RequestedByKind) -> Self {
        match value {
            RequestedByKind::System => RequestedBy::System,
            RequestedByKind::User => RequestedBy::User,
        }
    }
}

impl From<RequestedBy> for RequestedByKind {
    fn from(value: RequestedBy) -> Self {
        match value {
            RequestedBy::System => RequestedByKind::System,
            RequestedBy::User => RequestedByKind::User,
        }
    }
}

#[derive(Enum, Copy, Clone, Debug, Eq, PartialEq)]
pub enum MessageStatusDto {
    Pending,
    Scheduled,
    InFlight,
    Sent,
    Retrying,
    Failed,
    Cancelled,
}

impl From<&MessageStatus> for MessageStatusDto {
    fn from(value: &MessageStatus) -> Self {
        match value {
            MessageStatus::Pending => MessageStatusDto::Pending,
            MessageStatus::Scheduled => MessageStatusDto::Scheduled,
            MessageStatus::InFlight => MessageStatusDto::InFlight,
            MessageStatus::Sent => MessageStatusDto::Sent,
            MessageStatus::Retrying { .. } => MessageStatusDto::Retrying,
            MessageStatus::Failed { .. } => MessageStatusDto::Failed,
            MessageStatus::Cancelled => MessageStatusDto::Cancelled,
        }
    }
}

#[derive(Enum, Copy, Clone, Debug, Eq, PartialEq)]
pub enum ChatTypeKind {
    #[oai(rename = "direct")]
    Direct,
    #[oai(rename = "group")]
    Group,
    #[oai(rename = "channel")]
    Channel,
    #[oai(rename = "bot")]
    Bot,
    #[oai(rename = "unknown")]
    Unknown,
}

impl From<MessengerChatType> for ChatTypeKind {
    fn from(value: MessengerChatType) -> Self {
        match value {
            MessengerChatType::Direct => ChatTypeKind::Direct,
            MessengerChatType::Group => ChatTypeKind::Group,
            MessengerChatType::Channel => ChatTypeKind::Channel,
            MessengerChatType::Bot => ChatTypeKind::Bot,
            MessengerChatType::Unknown => ChatTypeKind::Unknown,
        }
    }
}
