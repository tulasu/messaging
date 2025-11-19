use std::sync::Arc;

use uuid::Uuid;

use crate::domain::{models::MessageAttempt, repositories::MessageHistoryRepository};

pub struct GetMessageAttemptsUseCase {
    repo: Arc<dyn MessageHistoryRepository>,
}

impl GetMessageAttemptsUseCase {
    pub fn new(repo: Arc<dyn MessageHistoryRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(
        &self,
        message_id: Uuid,
        user_id: Uuid,
    ) -> anyhow::Result<Vec<MessageAttempt>> {
        // Verify ownership
        let message = self
            .repo
            .get(message_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("message not found"))?;

        if message.user_id != user_id {
            anyhow::bail!("forbidden: message does not belong to user");
        }

        self.repo.get_attempts(message_id).await
    }
}
