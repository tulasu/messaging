use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    application::services::messenger::MessengerClient,
    domain::models::{MessageContent, MessengerToken, MessengerType},
};

#[derive(Default)]
pub struct VkClient;

impl VkClient {
    pub fn new() -> Arc<dyn MessengerClient> {
        Arc::new(Self) as Arc<dyn MessengerClient>
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
}
