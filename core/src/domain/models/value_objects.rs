use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ChatId(String);

impl ChatId {
    pub fn new(id: String) -> Result<Self, Error> {
        if id.is_empty() {
            return Err(Error::InvalidChatId);
        }
        Ok(ChatId(id))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct MessageContent {
    pub text: String,
    pub format: Option<TextFormat>,
}

impl MessageContent {
    pub fn new(text: String) -> Self {
        Self {
            text,
            format: Some(TextFormat::Plain),
        }
    }

    pub fn with_format(text: String, format: TextFormat) -> Self {
        Self {
            text,
            format: Some(format),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TextFormat {
    Plain,
    Markdown,
    Html,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Invalid chat ID: cannot be empty")]
    InvalidChatId,
}
