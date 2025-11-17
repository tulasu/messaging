use serde::{Deserialize, Serialize};

use super::messenger::MessengerType;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MessengerChatType {
    Direct,
    Group,
    Channel,
    Bot,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessengerChat {
    pub messenger: MessengerType,
    pub chat_id: String,
    pub title: String,
    pub chat_type: MessengerChatType,
    pub can_send_messages: bool,
}
