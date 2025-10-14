use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter, Result};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MessengerType {
    Telegram,
    VK,
    MAX,
}

impl Display for MessengerType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            MessengerType::Telegram => write!(f, "telegram"),
            MessengerType::VK => write!(f, "vk"),
            MessengerType::MAX => write!(f, "max"),
        }
    }
}
