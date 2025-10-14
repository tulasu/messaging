use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Payload {
    Plain { content: String },
    Formatted { content: String, format: TextFormat },
}

impl Payload {
    pub fn content(&self) -> &str {
        match self {
            Payload::Plain { content } => content,
            Payload::Formatted { content, .. } => content,
        }
    }

    pub fn format(&self) -> TextFormat {
        match self {
            Payload::Plain { .. } => TextFormat::Plain,
            Payload::Formatted { format, .. } => format.clone(),
        }
    }
}

use super::value_objects::TextFormat;
