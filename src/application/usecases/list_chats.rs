use std::sync::Arc;

use uuid::Uuid;

use crate::{
    application::services::messenger::MessengerGateway,
    domain::{
        models::{MessengerChat, MessengerType},
        repositories::MessengerTokenRepository,
    },
};

pub struct ListChatsUseCase {
    token_repo: Arc<dyn MessengerTokenRepository>,
    gateway: MessengerGateway,
}

impl ListChatsUseCase {
    pub fn new(token_repo: Arc<dyn MessengerTokenRepository>, gateway: MessengerGateway) -> Self {
        Self {
            token_repo,
            gateway,
        }
    }

    pub async fn execute(
        &self,
        user_id: Uuid,
        messenger: MessengerType,
    ) -> anyhow::Result<Vec<MessengerChat>> {
        let token = self
            .token_repo
            .find_active(&user_id, messenger)
            .await?
            .ok_or_else(|| anyhow::anyhow!("no active token for messenger"))?;

        let client = self
            .gateway
            .get(messenger)
            .ok_or_else(|| anyhow::anyhow!("no client registered for messenger"))?;

        client.list_chats(&token).await
    }
}
