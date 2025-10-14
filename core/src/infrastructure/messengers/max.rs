use crate::domain::{ChatId, MessageContent};
use crate::infrastructure::messengers::{MessengerAdapter, MessengerError, SentMessage};
use async_trait::async_trait;
use max_api_client::client::MaxApiClientTrait;
use max_api_client::{MaxApiClient, SendMessageRequest, TextFormat};

pub struct MaxAdapter {
    client: MaxApiClient,
}

impl MaxAdapter {
    pub fn new(access_token: String) -> Self {
        Self {
            client: MaxApiClient::new(access_token),
        }
    }
}

#[async_trait]
impl MessengerAdapter for MaxAdapter {
    async fn send_message(
        &self,
        chat_id: &ChatId,
        content: &MessageContent,
    ) -> Result<SentMessage, MessengerError> {
        let format = match &content.format {
            Some(crate::domain::TextFormat::Plain) => Some(TextFormat::Plain),
            Some(crate::domain::TextFormat::Markdown) => Some(TextFormat::Markdown),
            Some(crate::domain::TextFormat::Html) => Some(TextFormat::Html),
            None => None,
        };

        let request = SendMessageRequest {
            chat_id: chat_id.as_str().to_string(),
            content: content.text.clone(),
            format,
        };

        let response = self
            .client
            .send_message(request)
            .await
            .map_err(|e| match e {
                max_api_client::MaxApiError::Authentication(_) => {
                    MessengerError::AuthenticationError
                }
                max_api_client::MaxApiError::Api(msg) => MessengerError::ApiError(msg),
                max_api_client::MaxApiError::Network(msg) => MessengerError::NetworkError(msg),
                max_api_client::MaxApiError::InvalidChatId(msg) => {
                    MessengerError::InvalidChatId(msg)
                }
                max_api_client::MaxApiError::RateLimitExceeded => MessengerError::RateLimitExceeded,
                _ => MessengerError::ApiError(e.to_string()),
            })?;

        Ok(SentMessage {
            platform_message_id: response.message_id,
            timestamp: response.timestamp,
        })
    }

    async fn validate_chat_id(&self, chat_id: &str) -> Result<bool, MessengerError> {
        self.client
            .validate_chat_id(chat_id)
            .await
            .map_err(|e| match e {
                max_api_client::MaxApiError::Network(msg) => MessengerError::NetworkError(msg),
                _ => MessengerError::ApiError(e.to_string()),
            })
    }
}
