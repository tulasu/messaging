use std::sync::Arc;

use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

use crate::{
    application::services::messenger::MessengerClient,
    domain::models::{
        MessageContent, MessengerChat, MessengerChatType, MessengerToken, MessengerType,
    },
};

pub struct VkClient {
    http: Client,
    base_url: String,
    api_version: String,
}

impl VkClient {
    pub fn new() -> Arc<dyn MessengerClient> {
        Arc::new(Self {
            http: Client::builder()
                .user_agent("messaging-service/vk")
                .build()
                .expect("failed to build vk client"),
            base_url: "https://api.vk.com".to_string(),
            api_version: "5.199".to_string(),
        }) as Arc<dyn MessengerClient>
    }

    fn chat_type(peer_type: &str) -> MessengerChatType {
        match peer_type {
            "user" => MessengerChatType::Direct,
            "chat" => MessengerChatType::Group,
            "group" => MessengerChatType::Channel,
            "email" => MessengerChatType::Bot,
            _ => MessengerChatType::Unknown,
        }
    }
}

#[async_trait]
impl MessengerClient for VkClient {
    fn messenger(&self) -> MessengerType {
        MessengerType::Vk
    }

    async fn send(
        &self,
        token: &MessengerToken,
        recipient: &str,
        content: &MessageContent,
    ) -> anyhow::Result<()> {
        println!(
            "[vk] sending '{}' to {} using token {}",
            content.body, recipient, token.id
        );
        Ok(())
    }

    async fn list_chats(&self, token: &MessengerToken) -> anyhow::Result<Vec<MessengerChat>> {
        let url = format!("{}/method/messages.getConversations", self.base_url);
        let response = self
            .http
            .get(url)
            .query(&[
                ("access_token", token.access_token.as_str()),
                ("v", self.api_version.as_str()),
                ("count", "50"),
            ])
            .send()
            .await?;

        let payload: VkEnvelope<VkConversationsResponse> = response.json().await?;

        if let Some(error) = payload.error {
            anyhow::bail!(
                "vk api error {}: {}",
                error.error_code,
                error.error_msg.unwrap_or_else(|| "unknown".to_string())
            );
        }

        let data = payload
            .response
            .ok_or_else(|| anyhow::anyhow!("vk: empty response body"))?;

        let mut chats = Vec::with_capacity(data.items.len());
        for item in data.items {
            let peer = item.conversation.peer;
            let chat_type = Self::chat_type(peer.peer_type.as_str());
            let title = item
                .conversation
                .chat_settings
                .as_ref()
                .and_then(|settings| settings.title.clone())
                .unwrap_or_else(|| format!("peer {}", peer.id));

            let can_send = item
                .conversation
                .can_write
                .map(|c| c.allowed)
                .unwrap_or(true);

            chats.push(MessengerChat {
                messenger: MessengerType::Vk,
                chat_id: peer.id.to_string(),
                title,
                chat_type,
                can_send_messages: can_send,
            });
        }

        Ok(chats)
    }
}

#[derive(Debug, Deserialize)]
struct VkEnvelope<T> {
    response: Option<T>,
    error: Option<VkError>,
}

#[derive(Debug, Deserialize)]
struct VkError {
    error_code: i32,
    error_msg: Option<String>,
}

#[derive(Debug, Deserialize)]
struct VkConversationsResponse {
    items: Vec<VkConversationItem>,
}

#[derive(Debug, Deserialize)]
struct VkConversationItem {
    conversation: VkConversation,
}

#[derive(Debug, Deserialize)]
struct VkConversation {
    peer: VkPeer,
    #[serde(default)]
    can_write: Option<VkCanWrite>,
    #[serde(default)]
    chat_settings: Option<VkChatSettings>,
}

#[derive(Debug, Deserialize)]
struct VkPeer {
    id: i64,
    #[serde(rename = "type")]
    peer_type: String,
}

#[derive(Debug, Deserialize)]
struct VkCanWrite {
    allowed: bool,
}

#[derive(Debug, Deserialize)]
struct VkChatSettings {
    title: Option<String>,
}
