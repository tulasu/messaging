use crate::domain::MessengerType;
use crate::infrastructure::messengers::{
    MaxAdapter, MessengerAdapter, MessengerError, TelegramAdapter, VKAdapter,
};
use std::sync::Arc;

pub struct MessengerAdapterFactory {
    telegram_bot_token: Option<String>,
    vk_access_token: Option<String>,
    max_access_token: Option<String>,
}

impl MessengerAdapterFactory {
    pub fn new() -> Self {
        Self {
            telegram_bot_token: None,
            vk_access_token: None,
            max_access_token: None,
        }
    }

    pub fn with_telegram_token(mut self, token: String) -> Self {
        self.telegram_bot_token = Some(token);
        self
    }

    pub fn with_vk_token(mut self, token: String) -> Self {
        self.vk_access_token = Some(token);
        self
    }

    pub fn with_max_token(mut self, token: String) -> Self {
        self.max_access_token = Some(token);
        self
    }

    pub fn create_adapter(
        &self,
        messenger_type: &MessengerType,
    ) -> Result<Arc<dyn MessengerAdapter>, MessengerError> {
        match messenger_type {
            MessengerType::Telegram => {
                let token = self
                    .telegram_bot_token
                    .as_ref()
                    .ok_or_else(|| MessengerError::AuthenticationError)?;
                Ok(Arc::new(TelegramAdapter::new(token.clone())))
            }
            MessengerType::VK => {
                let token = self
                    .vk_access_token
                    .as_ref()
                    .ok_or_else(|| MessengerError::AuthenticationError)?;
                Ok(Arc::new(VKAdapter::new(token.clone())))
            }
            MessengerType::MAX => {
                let token = self
                    .max_access_token
                    .as_ref()
                    .ok_or_else(|| MessengerError::AuthenticationError)?;
                Ok(Arc::new(MaxAdapter::new(token.clone())))
            }
        }
    }
}

impl Default for MessengerAdapterFactory {
    fn default() -> Self {
        Self::new()
            .with_telegram_token(
                std::env::var("TELEGRAM_BOT_TOKEN")
                    .unwrap_or_else(|_| "default_telegram_token".to_string()),
            )
            .with_vk_token(
                std::env::var("VK_ACCESS_TOKEN").unwrap_or_else(|_| "default_vk_token".to_string()),
            )
            .with_max_token(
                std::env::var("MAX_ACCESS_TOKEN")
                    .unwrap_or_else(|_| "default_max_token".to_string()),
            )
    }
}
