use std::sync::Arc;

use uuid::Uuid;

use crate::domain::{models::MessengerToken, repositories::MessengerTokenRepository};

pub struct ListTokensUseCase {
    repo: Arc<dyn MessengerTokenRepository>,
}

impl ListTokensUseCase {
    pub fn new(repo: Arc<dyn MessengerTokenRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, user_id: Uuid) -> anyhow::Result<Vec<MessengerToken>> {
        self.repo.list_by_user(&user_id).await
    }
}
