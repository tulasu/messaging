use crate::domain::{ChatId, MessageContent};
use crate::infrastructure::messengers::{MessengerAdapter, MessengerError, SentMessage};
use async_trait::async_trait;
use reqwest::Client;
use serde_json::{Value, json};

pub struct VKAdapter {
    client: Client,
    access_token: String,
    api_version: String,
}

impl VKAdapter {
    pub fn new(access_token: String) -> Self {
        Self {
            client: Client::new(),
            access_token,
            api_version: "5.199".to_string(),
        }
    }

    fn build_api_url(&self, method: &str) -> String {
        format!("https://api.vk.com/method/{}", method)
    }
}

#[async_trait]
impl MessengerAdapter for VKAdapter {
    async fn send_message(
        &self,
        chat_id: &ChatId,
        content: &MessageContent,
    ) -> Result<SentMessage, MessengerError> {
        let url = self.build_api_url("messages.send");

        let mut payload = json!({
            "user_id": chat_id.as_str(),
            "message": content.text,
            "random_id": chrono::Utc::now().timestamp(),
            "access_token": self.access_token,
            "v": self.api_version
        });

        // Add message format if specified
        if let Some(format) = &content.format {
            match format {
                crate::domain::TextFormat::Markdown => {
                    payload["message"] = json!(format_message_for_vk(&content.text, "markdown"));
                }
                crate::domain::TextFormat::Html => {
                    payload["message"] = json!(format_message_for_vk(&content.text, "html"));
                }
                crate::domain::TextFormat::Plain => {
                    // No formatting needed
                }
            }
        }

        let response = self
            .client
            .post(&url)
            .form(&payload)
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
                "VK API error: {} - {}",
                status, response_text
            )));
        }

        let response_json: Value = serde_json::from_str(&response_text)
            .map_err(|e| MessengerError::ApiError(format!("Failed to parse response: {}", e)))?;

        if let Some(error) = response_json.get("error") {
            return Err(MessengerError::ApiError(format!(
                "VK API returned error: {}",
                error["error_msg"].as_str().unwrap_or("Unknown error")
            )));
        }

        let response_data = &response_json["response"];
        let platform_message_id = response_data
            .as_u64()
            .map(|id| id.to_string())
            .unwrap_or_else(|| "unknown".to_string());

        Ok(SentMessage {
            platform_message_id,
            timestamp: chrono::Utc::now(),
        })
    }

    async fn validate_chat_id(&self, chat_id: &str) -> Result<bool, MessengerError> {
        // VK chat IDs should be numeric user IDs or peer IDs
        if chat_id.is_empty() {
            return Ok(false);
        }

        chat_id
            .parse::<i64>()
            .map(|id| id > 0) // User IDs should be positive
            .map_err(|_| MessengerError::InvalidChatId(chat_id.to_string()))
    }
}

fn format_message_for_vk(text: &str, format: &str) -> String {
    match format {
        "markdown" => {
            // Convert basic markdown to plain text (VK doesn't support markdown)
            text.replace("**", "")
                .replace("*", "")
                .replace("`", "")
                .replace("```", "")
        }
        "html" => {
            // Strip HTML tags (VK doesn't support HTML in messages)
            // This is a very basic implementation
            let mut result = text.to_string();
            while let Some(start) = result.find('<') {
                if let Some(end) = result[start..].find('>') {
                    result.replace_range(start..=start + end, "");
                } else {
                    break;
                }
            }
            result
        }
        _ => text.to_string(),
    }
}
