use crate::error::MaxApiError;
use crate::models::*;
use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;

#[async_trait]
pub trait MaxApiClientTrait {
    async fn send_message(
        &self,
        request: SendMessageRequest,
    ) -> Result<SendMessageResponse, MaxApiError>;
    async fn get_chat_info(&self, chat_id: &str) -> Result<ChatInfo, MaxApiError>;
    async fn validate_chat_id(&self, chat_id: &str) -> Result<bool, MaxApiError>;
}

pub struct MaxApiClient {
    client: Client,
    base_url: String,
    access_token: String,
}

impl MaxApiClient {
    pub fn new(access_token: String) -> Self {
        Self::with_base_url("https://api.max.ru", access_token)
    }

    pub fn with_base_url(base_url: &str, access_token: String) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.to_string(),
            access_token,
        }
    }

    fn build_url(&self, endpoint: &str) -> String {
        format!("{}/v1/{}", self.base_url, endpoint)
    }

    async fn make_request<T>(
        &self,
        endpoint: &str,
        payload: serde_json::Value,
    ) -> Result<T, MaxApiError>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        let url = self.build_url(endpoint);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| MaxApiError::Network(e.to_string()))?;

        if response.status() == 401 {
            return Err(MaxApiError::Authentication(
                "Invalid access token".to_string(),
            ));
        }

        if response.status() == 429 {
            return Err(MaxApiError::RateLimitExceeded);
        }

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(MaxApiError::Api(format!("{} - {}", status, error_text)));
        }

        let api_response: ApiResponse<T> = response
            .json()
            .await
            .map_err(|e| MaxApiError::InvalidResponse(e.to_string()))?;

        if !api_response.success {
            if let Some(error) = api_response.error {
                return Err(MaxApiError::Api(format!(
                    "{}: {}",
                    error.code, error.message
                )));
            } else {
                return Err(MaxApiError::Api("Unknown API error".to_string()));
            }
        }

        api_response
            .data
            .ok_or_else(|| MaxApiError::InvalidResponse("No data in response".to_string()))
    }
}

#[async_trait]
impl MaxApiClientTrait for MaxApiClient {
    async fn send_message(
        &self,
        request: SendMessageRequest,
    ) -> Result<SendMessageResponse, MaxApiError> {
        let payload = json!({
            "chat_id": request.chat_id,
            "content": request.content,
            "format": request.format
        });

        self.make_request("messages/send", payload).await
    }

    async fn get_chat_info(&self, chat_id: &str) -> Result<ChatInfo, MaxApiError> {
        let payload = json!({
            "chat_id": chat_id
        });

        self.make_request("chats/info", payload).await
    }

    async fn validate_chat_id(&self, chat_id: &str) -> Result<bool, MaxApiError> {
        match self.get_chat_info(chat_id).await {
            Ok(_) => Ok(true),
            Err(MaxApiError::Api(_)) => Ok(false), // Chat not found or invalid
            Err(e) => Err(e),                      // Other errors should be propagated
        }
    }
}
