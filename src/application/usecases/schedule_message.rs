use std::sync::Arc;

use chrono::Utc;
use uuid::Uuid;

use crate::{
    application::services::event_bus::MessageBus,
    domain::{
        events::OutboundMessageEvent,
        models::{
            MessageContent, MessageStatus, MessageType, MessengerType, RequestedBy,
        },
        repositories::{MessageHistoryRepository, MessengerTokenRepository},
    },
};

pub struct ScheduleMessageConfig {
    pub max_attempts: u32,
}

pub struct ScheduleMessageUseCase {
    token_repo: Arc<dyn MessengerTokenRepository>,
    history_repo: Arc<dyn MessageHistoryRepository>,
    bus: Arc<dyn MessageBus>,
    config: ScheduleMessageConfig,
}

pub struct ScheduleMessageRequest {
    pub user_id: Uuid,
    pub messenger: MessengerType,
    pub recipient: String,
    pub text: String,
    pub requested_by: RequestedBy,
}

pub struct ScheduleMessageResponse {
    pub message_id: Uuid,
}

impl ScheduleMessageUseCase {
    pub fn new(
        token_repo: Arc<dyn MessengerTokenRepository>,
        history_repo: Arc<dyn MessageHistoryRepository>,
        bus: Arc<dyn MessageBus>,
        config: ScheduleMessageConfig,
    ) -> Self {
        Self {
            token_repo,
            history_repo,
            bus,
            config,
        }
    }

    pub async fn execute(
        &self,
        request: ScheduleMessageRequest,
    ) -> anyhow::Result<ScheduleMessageResponse> {
        self.ensure_token_exists(&request).await?;

        let content = MessageContent {
            body: request.text.clone(),
            message_type: MessageType::PlainText,
        };

        let history_entry = self
            .history_repo
            .insert(
                request.user_id,
                request.messenger,
                request.recipient.clone(),
                content.clone(),
                request.requested_by,
            )
            .await?;

        self.history_repo
            .update_status(history_entry.id, MessageStatus::Scheduled, 0)
            .await?;

        let event = OutboundMessageEvent {
            event_id: Uuid::new_v4(),
            message_id: history_entry.id,
            user_id: request.user_id,
            messenger: request.messenger,
            recipient: request.recipient,
            message_type: content.message_type,
            content,
            attempt: 1,
            max_attempts: self.config.max_attempts,
            scheduled_at: Utc::now(),
        };

        self.bus.publish(event).await?;

        Ok(ScheduleMessageResponse {
            message_id: history_entry.id,
        })
    }

    async fn ensure_token_exists(
        &self,
        request: &ScheduleMessageRequest,
    ) -> anyhow::Result<()> {
        let token = self
            .token_repo
            .find_active(&request.user_id, request.messenger)
            .await?;
        if token.is_none() {
            anyhow::bail!("no active token for messenger");
        }
        Ok(())
    }
}

