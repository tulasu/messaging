use std::sync::Arc;

use chrono::Utc;
use uuid::Uuid;

use crate::{
    application::services::event_bus::MessageBus,
    domain::{
        events::OutboundMessageEvent,
        models::MessageStatus,
        repositories::{MessageHistoryRepository, MessengerTokenRepository},
    },
};

pub struct RetryMessageConfig {
    pub max_attempts: u32,
}

pub struct RetryMessageUseCase {
    history_repo: Arc<dyn MessageHistoryRepository>,
    token_repo: Arc<dyn MessengerTokenRepository>,
    bus: Arc<dyn MessageBus>,
    config: RetryMessageConfig,
}

pub struct RetryMessageRequest {
    pub user_id: Uuid,
    pub message_id: Uuid,
}

impl RetryMessageUseCase {
    pub fn new(
        history_repo: Arc<dyn MessageHistoryRepository>,
        token_repo: Arc<dyn MessengerTokenRepository>,
        bus: Arc<dyn MessageBus>,
        config: RetryMessageConfig,
    ) -> Self {
        Self {
            history_repo,
            token_repo,
            bus,
            config,
        }
    }

    pub async fn execute(&self, request: RetryMessageRequest) -> anyhow::Result<()> {
        let message = self
            .history_repo
            .get(request.message_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("message not found"))?;

        if message.user_id != request.user_id {
            anyhow::bail!("message does not belong to user");
        }

        let token = self
            .token_repo
            .find_active(&message.user_id, message.messenger)
            .await?;
        if token.is_none() {
            anyhow::bail!("no active token for messenger");
        }

        let next_attempt = message.attempts + 1;

        self.history_repo
            .update_status(request.message_id, MessageStatus::Scheduled, next_attempt)
            .await?;

        let event = OutboundMessageEvent {
            event_id: Uuid::new_v4(),
            message_id: request.message_id,
            user_id: message.user_id,
            messenger: message.messenger,
            recipient: message.recipient.clone(),
            message_type: message.content.message_type.clone(),
            content: message.content.clone(),
            attempt: next_attempt,
            max_attempts: self.config.max_attempts,
            scheduled_at: Utc::now(),
        };

        self.bus.publish(event).await?;

        Ok(())
    }
}
