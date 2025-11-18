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

pub struct TelegramClient {
    http: Client,
    base_url: String,
}

impl TelegramClient {
    pub fn new() -> Arc<dyn MessengerClient> {
        Arc::new(Self {
            http: Client::builder()
                .user_agent("messaging-service/telegram")
                .build()
                .expect("failed to build telegram client"),
            base_url: "https://api.telegram.org".to_string(),
        }) as Arc<dyn MessengerClient>
    }

    fn build_url(&self, token: &MessengerToken, method: &str) -> String {
        format!("{}/bot{}/{}", self.base_url, token.access_token, method)
    }

    fn map_chat(chat: TelegramChat) -> MessengerChat {
        let chat_type = match chat.chat_type.as_str() {
            "private" => MessengerChatType::Direct,
            "group" | "supergroup" => MessengerChatType::Group,
            "channel" => MessengerChatType::Channel,
            _ => MessengerChatType::Unknown,
        };

        let mut title_candidates = vec![];
        if let Some(title) = chat.title {
            title_candidates.push(title);
        }
        if let Some(username) = chat.username {
            title_candidates.push(format!("@{}", username));
        }
        let full_name = match (chat.first_name, chat.last_name) {
            (Some(first), Some(last)) => Some(format!("{first} {last}")),
            (Some(first), None) => Some(first),
            (None, Some(last)) => Some(last),
            _ => None,
        };
        if let Some(name) = full_name {
            title_candidates.push(name);
        }
        let title = title_candidates
            .into_iter()
            .find(|s| !s.trim().is_empty())
            .unwrap_or_else(|| "Unnamed chat".to_string());

        let can_send_messages = matches!(
            chat_type,
            MessengerChatType::Direct | MessengerChatType::Group | MessengerChatType::Channel
        );

        MessengerChat {
            messenger: MessengerType::Telegram,
            chat_id: chat.id.to_string(),
            title,
            chat_type,
            can_send_messages,
        }
    }
}

#[async_trait]
impl MessengerClient for TelegramClient {
    fn messenger(&self) -> MessengerType {
        MessengerType::Telegram
    }

    async fn send(
        &self,
        token: &MessengerToken,
        recipient: &str,
        content: &MessageContent,
    ) -> anyhow::Result<()> {
        let url = self.build_url(token, "sendMessage");
        
        let chat_id: i64 = recipient.parse().map_err(|_| {
            anyhow::anyhow!("invalid telegram chat_id format: expected integer, got '{}'", recipient)
        })?;

        let request_body = serde_json::json!({
            "chat_id": chat_id,
            "text": content.body,
        });

        let response = self
            .http
            .post(&url)
            .json(&request_body)
            .send()
            .await?;

        let payload: TelegramApiResponse<TelegramMessageResponse> = response.json().await?;
        
        if !payload.ok {
            anyhow::bail!(
                "telegram api error: {}",
                payload
                    .description
                    .unwrap_or_else(|| "unknown error".to_string())
            );
        }

        Ok(())
    }

    async fn list_chats(
        &self,
        token: &MessengerToken,
        pagination: PaginationParams,
    ) -> anyhow::Result<PaginatedChats> {
        let url = self.build_url(token, "getUpdates");
        
        let limit = pagination.limit.unwrap_or(100).min(100) as i32;
        let offset = pagination.offset.unwrap_or(0) as i32;

        let limit_str = limit.to_string();
        let offset_str = offset.to_string();
        
        let mut query_params: Vec<(&str, &str)> = vec![
            ("allowed_updates", r#"["message","channel_post","chat_member"]"#),
            ("limit", &limit_str),
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

        let payload: TelegramUpdatesResponse = response.json().await?;
        if !payload.ok {
            anyhow::bail!(
                "telegram api returned error: {}",
                payload
                    .description
                    .unwrap_or_else(|| "unknown error".to_string())
            );
        }

        let mut chats: HashMap<i64, MessengerChat> = HashMap::new();
        for update in payload.result {
            if let Some(message) = update.message {
                chats
                    .entry(message.chat.id)
                    .or_insert_with(|| Self::map_chat(message.chat));
            }
            if let Some(post) = update.channel_post {
                chats
                    .entry(post.chat.id)
                    .or_insert_with(|| Self::map_chat(post.chat));
            }
            if let Some(member) = update.my_chat_member {
                chats
                    .entry(member.chat.id)
                    .or_insert_with(|| Self::map_chat(member.chat));
            }
        }

        let chats_vec: Vec<MessengerChat> = chats.into_values().collect();
        let has_more = chats_vec.len() >= limit as usize;
        let next_offset = if has_more {
            Some(offset as u32 + chats_vec.len() as u32)
        } else {
            None
        };

        Ok(PaginatedChats {
            chats: chats_vec,
            has_more,
            next_offset,
        })
    }
}

#[derive(Debug, Deserialize)]
struct TelegramApiResponse<T> {
    ok: bool,
    description: Option<String>,
    #[serde(default)]
    result: Option<T>,
}

#[derive(Debug, Deserialize)]
struct TelegramUpdatesResponse {
    ok: bool,
    description: Option<String>,
    #[serde(default)]
    result: Vec<TelegramUpdate>,
}

#[derive(Debug, Default, Deserialize)]
struct TelegramMessageResponse {
    message_id: i64,
}

#[derive(Debug, Deserialize)]
struct TelegramUpdate {
    message: Option<TelegramMessage>,
    #[serde(rename = "channel_post")]
    channel_post: Option<TelegramMessage>,
    #[serde(rename = "my_chat_member")]
    my_chat_member: Option<TelegramChatMember>,
}

#[derive(Debug, Deserialize)]
struct TelegramMessage {
    chat: TelegramChat,
}

#[derive(Debug, Deserialize)]
struct TelegramChatMember {
    chat: TelegramChat,
}

#[derive(Debug, Deserialize)]
struct TelegramChat {
    id: i64,
    #[serde(rename = "type")]
    chat_type: String,
    title: Option<String>,
    username: Option<String>,
    #[serde(rename = "first_name")]
    first_name: Option<String>,
    #[serde(rename = "last_name")]
    last_name: Option<String>,
}
