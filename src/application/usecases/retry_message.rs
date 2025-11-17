use std::sync::Arc;

use uuid::Uuid;

use crate::{
    domain::models::RequestedBy,
    domain::repositories::MessageHistoryRepository,
};

use super::schedule_message::{ScheduleMessageRequest, ScheduleMessageUseCase};

pub struct RetryMessageUseCase {
    history_repo: Arc<dyn MessageHistoryRepository>,
    scheduler: Arc<ScheduleMessageUseCase>,
}

pub struct RetryMessageRequest {
    pub user_id: Uuid,
    pub message_id: Uuid,
}

impl RetryMessageUseCase {
    pub fn new(
        history_repo: Arc<dyn MessageHistoryRepository>,
        scheduler: Arc<ScheduleMessageUseCase>,
    ) -> Self {
        Self {
            history_repo,
            scheduler,
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

        self.scheduler
            .execute(ScheduleMessageRequest {
                user_id: message.user_id,
                messenger: message.messenger,
                recipient: message.recipient.clone(),
                text: message.content.body.clone(),
                requested_by: RequestedBy::User,
            })
            .await?;

        Ok(())
    }
}

