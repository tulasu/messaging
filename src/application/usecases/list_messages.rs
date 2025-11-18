use std::sync::Arc;

use uuid::Uuid;

use crate::domain::{models::MessageHistoryEntry, repositories::MessageHistoryRepository};

pub struct ListMessagesUseCase {
    repo: Arc<dyn MessageHistoryRepository>,
}

pub struct PaginatedMessages {
    pub messages: Vec<MessageHistoryEntry>,
    pub has_more: bool,
    pub next_offset: Option<u32>,
}

impl ListMessagesUseCase {
    pub fn new(repo: Arc<dyn MessageHistoryRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(
        &self,
        user_id: Uuid,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> anyhow::Result<PaginatedMessages> {
        let (messages, has_more) = self.repo.list_by_user(user_id, limit, offset).await?;
        let current_offset = offset.unwrap_or(0);
        let next_offset = if has_more {
            Some(current_offset + messages.len() as u32)
        } else {
            None
        };

        Ok(PaginatedMessages {
            messages,
            has_more,
            next_offset,
        })
    }
}
