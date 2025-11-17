use std::sync::Arc;

use crate::{
    application::services::messenger::MessengerGateway,
    domain::{
        events::OutboundMessageEvent,
        models::{MessageStatus, MessageType},
        repositories::{MessageHistoryRepository, MessengerTokenRepository},
    },
};

pub struct MessageDispatchHandler {
    token_repo: Arc<dyn MessengerTokenRepository>,
    history_repo: Arc<dyn MessageHistoryRepository>,
    gateway: MessengerGateway,
}

impl MessageDispatchHandler {
    pub fn new(
        token_repo: Arc<dyn MessengerTokenRepository>,
        history_repo: Arc<dyn MessageHistoryRepository>,
        gateway: MessengerGateway,
    ) -> Self {
        Self {
            token_repo,
            history_repo,
            gateway,
        }
    }

    pub async fn handle(&self, event: OutboundMessageEvent) -> anyhow::Result<()> {
        if !matches!(event.content.message_type, MessageType::PlainText) {
            self.history_repo
                .update_status(
                    event.message_id,
                    MessageStatus::Failed {
                        reason: "unsupported message type".to_string(),
                        attempts: event.attempt,
                    },
                    event.attempt,
                )
                .await?;
            anyhow::bail!("unsupported message type");
        }

        let token = self
            .token_repo
            .find_active(&event.user_id, event.messenger)
            .await?
            .ok_or_else(|| anyhow::anyhow!("missing active token for messenger"))?;

        let client = self
            .gateway
            .get(event.messenger)
            .ok_or_else(|| anyhow::anyhow!("no client registered for messenger"))?;

        if let Err(err) = client.send(&token, &event.recipient, &event.content).await {
            let reason = err.to_string();
            let status = if event.attempt >= event.max_attempts {
                MessageStatus::Failed {
                    reason,
                    attempts: event.attempt,
                }
            } else {
                MessageStatus::Retrying {
                    reason,
                    attempts: event.attempt,
                }
            };
            self.history_repo
                .update_status(event.message_id, status, event.attempt)
                .await?;
            return Err(err);
        }

        self.history_repo
            .update_status(event.message_id, MessageStatus::Sent, event.attempt)
            .await?;

        Ok(())
    }
}
