use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum MessengerType {
    Telegram,
    Vk,
}

impl MessengerType {
    pub fn as_str(&self) -> &'static str {
        match self {
            MessengerType::Telegram => "telegram",
            MessengerType::Vk => "vk",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "telegram" => Some(MessengerType::Telegram),
            "vk" => Some(MessengerType::Vk),
            _ => None,
        }
    }
}
