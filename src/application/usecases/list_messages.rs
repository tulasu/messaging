use std::sync::Arc;

use uuid::Uuid;

use crate::domain::{models::MessageHistoryEntry, repositories::MessageHistoryRepository};

pub struct ListMessagesUseCase {
    repo: Arc<dyn MessageHistoryRepository>,
}

impl ListMessagesUseCase {
    pub fn new(repo: Arc<dyn MessageHistoryRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, user_id: Uuid) -> anyhow::Result<Vec<MessageHistoryEntry>> {
        self.repo.list_by_user(user_id).await
    }
}
