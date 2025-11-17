use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    application::services::messenger::MessengerClient,
    domain::models::{MessageContent, MessengerToken, MessengerType},
};

#[derive(Default)]
pub struct TelegramClient;

impl TelegramClient {
    pub fn new() -> Arc<dyn MessengerClient> {
        Arc::new(Self) as Arc<dyn MessengerClient>
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
        println!(
            "[telegram] sending '{}' to {} using token {}",
            content.body, recipient, token.id
        );
        Ok(())
    }
}
