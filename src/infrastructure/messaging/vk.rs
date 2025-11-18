use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

use crate::{
    application::services::messenger::{MessengerClient, PaginatedChats, PaginationParams},
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
        let url = format!("{}/method/messages.send", self.base_url);
        
        let peer_id: i64 = recipient.parse().map_err(|_| {
            anyhow::anyhow!("invalid vk peer_id format: expected integer, got '{}'", recipient)
        })?;

        let peer_id_str = peer_id.to_string();
        let random_id_str = chrono::Utc::now().timestamp_millis().to_string();
        
        let response = self
            .http
            .get(&url)
            .query(&[
                ("access_token", token.access_token.as_str()),
                ("v", self.api_version.as_str()),
                ("peer_id", &peer_id_str),
                ("message", &content.body),
                ("random_id", &random_id_str),
            ])
            .send()
            .await?;

        let payload: VkEnvelope<i64> = response.json().await?;

        if let Some(error) = payload.error {
            anyhow::bail!(
                "vk api error {}: {}",
                error.error_code,
                error.error_msg.unwrap_or_else(|| "unknown".to_string())
            );
        }

        // If response is present, message was sent successfully
        // The response value is the message_id, but we don't need it
        Ok(())
    }

    async fn list_chats(
        &self,
        token: &MessengerToken,
        pagination: PaginationParams,
    ) -> anyhow::Result<PaginatedChats> {
        let url = format!("{}/method/messages.getConversations", self.base_url);
        
        let count = pagination.limit.unwrap_or(50).min(200) as i32;
        let offset = pagination.offset.unwrap_or(0) as i32;

        let count_str = count.to_string();
        let offset_str = offset.to_string();
        
        let mut query_params: Vec<(&str, &str)> = vec![
            ("access_token", token.access_token.as_str()),
            ("v", self.api_version.as_str()),
            ("count", &count_str),
            ("extended", "1"), // Get extended info including user profiles
        ];
        
        if offset > 0 {
            query_params.push(("offset", &offset_str));
        }

        let response = self
            .http
            .get(url)
            .query(&query_params)
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
        
        let users_map: HashMap<i64, &VkUser> = data
            .profiles
            .iter()
            .map(|u| (u.id, u))
            .collect();
        
        for item in data.items {
            let peer = item.conversation.peer;
            let chat_type = Self::chat_type(peer.peer_type.as_str());
            
            let title = match chat_type {
                MessengerChatType::Direct => {
                    users_map
                        .get(&peer.id)
                        .map(|user| {
                            format!(
                                "{} {}",
                                user.first_name.as_deref().unwrap_or(""),
                                user.last_name.as_deref().unwrap_or("")
                            )
                            .trim()
                            .to_string()
                        })
                        .filter(|s| !s.is_empty())
                        .unwrap_or_else(|| format!("User {}", peer.id))
                }
                _ => {
                    item.conversation
                        .chat_settings
                        .as_ref()
                        .and_then(|settings| settings.title.clone())
                        .unwrap_or_else(|| format!("Chat {}", peer.id))
                }
            };

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

        let total_count = data.count.unwrap_or(0);
        let current_offset = offset as u32;
        let has_more = (current_offset + chats.len() as u32) < total_count as u32;
        let next_offset = if has_more {
            Some(current_offset + chats.len() as u32)
        } else {
            None
        };

        Ok(PaginatedChats {
            chats,
            has_more,
            next_offset,
        })
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
    count: Option<i32>,
    items: Vec<VkConversationItem>,
    #[serde(default)]
    profiles: Vec<VkUser>,
}

#[derive(Debug, Deserialize)]
struct VkUser {
    id: i64,
    #[serde(rename = "first_name")]
    first_name: Option<String>,
    #[serde(rename = "last_name")]
    last_name: Option<String>,
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

