pub mod max;
pub mod telegram;
pub mod vk;

use crate::domain::{ChatId, MessageContent};
use async_trait::async_trait;

#[async_trait]
pub trait MessengerAdapter: Send + Sync {
    async fn send_message(
        &self,
        chat_id: &ChatId,
        content: &MessageContent,
    ) -> Result<SentMessage, MessengerError>;
    async fn validate_chat_id(&self, chat_id: &str) -> Result<bool, MessengerError>;
}

#[derive(Debug)]
pub struct SentMessage {
    pub platform_message_id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, thiserror::Error)]
pub enum MessengerError {
    #[error("API request failed: {0}")]
    ApiError(String),
    #[error("Authentication failed")]
    AuthenticationError,
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    #[error("Invalid chat ID: {0}")]
    InvalidChatId(String),
    #[error("Message too long")]
    MessageTooLong,
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("Unknown error: {0}")]
    Unknown(String),
}

pub use max::MaxAdapter;
pub use telegram::TelegramAdapter;
pub use vk::VKAdapter;

pub mod factory;
