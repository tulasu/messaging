use crate::domain::{ChatId, MessageContent};
use crate::infrastructure::messengers::{MessengerAdapter, MessengerError, SentMessage};
use async_trait::async_trait;
use reqwest::Client;
use serde_json::{Value, json};

pub struct TelegramAdapter {
    client: Client,
    bot_token: String,
}

impl TelegramAdapter {
    pub fn new(bot_token: String) -> Self {
        Self {
            client: Client::new(),
            bot_token,
        }
    }

    fn build_api_url(&self, method: &str) -> String {
        format!("https://api.telegram.org/bot{}/{}", self.bot_token, method)
    }
}

#[async_trait]
impl MessengerAdapter for TelegramAdapter {
    async fn send_message(
        &self,
        chat_id: &ChatId,
        content: &MessageContent,
    ) -> Result<SentMessage, MessengerError> {
        let url = self.build_api_url("sendMessage");

        let mut payload = json!({
            "chat_id": chat_id.as_str(),
            "text": content.text
        });

        // Add parse mode if specified
        if let Some(format) = &content.format {
            match format {
                crate::domain::TextFormat::Markdown => {
                    payload["parse_mode"] = Value::String("Markdown".to_string());
                }
                crate::domain::TextFormat::Html => {
                    payload["parse_mode"] = Value::String("HTML".to_string());
                }
                crate::domain::TextFormat::Plain => {
                    // No parse mode needed for plain text
                }
            }
        }

        let response = self
            .client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| MessengerError::NetworkError(e.to_string()))?;

        let status = response.status();
        let response_text = response
            .text()
            .await
            .map_err(|e| MessengerError::NetworkError(e.to_string()))?;

        if !status.is_success() {
            return Err(MessengerError::ApiError(format!(
                "Telegram API error: {} - {}",
                status, response_text
            )));
        }

        let response_json: Value = serde_json::from_str(&response_text)
            .map_err(|e| MessengerError::ApiError(format!("Failed to parse response: {}", e)))?;

        if !response_json["ok"].as_bool().unwrap_or(false) {
            return Err(MessengerError::ApiError(format!(
                "Telegram API returned error: {}",
                response_json["description"]
                    .as_str()
                    .unwrap_or("Unknown error")
            )));
        }

        let result = &response_json["result"];
        let platform_message_id = result["message_id"]
            .as_u64()
            .map(|id| id.to_string())
            .unwrap_or_else(|| "unknown".to_string());

        Ok(SentMessage {
            platform_message_id,
            timestamp: chrono::Utc::now(),
        })
    }

    async fn validate_chat_id(&self, chat_id: &str) -> Result<bool, MessengerError> {
        // For Telegram, chat IDs can be:
        // - Numeric user IDs (positive integers)
        // - Numeric chat IDs (negative integers for groups/channels)
        // - Usernames starting with @ for public channels/groups

        if chat_id.is_empty() {
            return Ok(false);
        }

        // Check if it's a username
        if chat_id.starts_with('@') {
            return Ok(chat_id.len() > 1
                && chat_id[1..]
                    .chars()
                    .all(|c| c.is_alphanumeric() || c == '_'));
        }

        // Check if it's a numeric ID
        chat_id
            .parse::<i64>()
            .map(|_| true)
            .map_err(|_| MessengerError::InvalidChatId(chat_id.to_string()))
    }
}
