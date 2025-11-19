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
        // Get message entry to know who requested it
        let message_entry = self
            .history_repo
            .get(event.message_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("message not found"))?;

        let requested_by = message_entry.requested_by.clone();

        if !matches!(event.content.message_type, MessageType::PlainText) {
            let status = MessageStatus::Failed {
                reason: "unsupported message type".to_string(),
                attempts: event.attempt,
            };
            self.history_repo
                .update_status(event.message_id, status.clone(), event.attempt)
                .await?;
            // Log attempt
            self.history_repo
                .log_attempt(event.message_id, event.attempt, status, requested_by)
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

        // Log attempt start (InFlight status)
        let in_flight_status = MessageStatus::InFlight;
        self.history_repo
            .log_attempt(
                event.message_id,
                event.attempt,
                in_flight_status,
                requested_by.clone(),
            )
            .await?;

        if let Err(err) = client.send(&token, &event.recipient, &event.content).await {
            let reason = err.to_string();
            let status = if event.attempt >= event.max_attempts {
                MessageStatus::Failed {
                    reason: reason.clone(),
                    attempts: event.attempt,
                }
            } else {
                MessageStatus::Retrying {
                    reason: reason.clone(),
                    attempts: event.attempt,
                }
            };
            self.history_repo
                .update_status(event.message_id, status.clone(), event.attempt)
                .await?;
            // Log failed/retrying attempt
            self.history_repo
                .log_attempt(event.message_id, event.attempt, status, requested_by)
                .await?;
            return Err(err);
        }

        let sent_status = MessageStatus::Sent;
        self.history_repo
            .update_status(event.message_id, sent_status.clone(), event.attempt)
            .await?;
        // Log successful attempt
        self.history_repo
            .log_attempt(event.message_id, event.attempt, sent_status, requested_by)
            .await?;

        Ok(())
    }
}
