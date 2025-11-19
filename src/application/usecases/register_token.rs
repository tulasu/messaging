use std::sync::Arc;

use chrono::Utc;
use uuid::Uuid;

use crate::domain::{
    models::{MessengerToken, MessengerTokenStatus, MessengerType},
    repositories::MessengerTokenRepository,
};

pub struct RegisterTokenUseCase {
    repo: Arc<dyn MessengerTokenRepository>,
}

pub struct RegisterTokenRequest {
    pub user_id: Uuid,
    pub messenger: MessengerType,
    pub access_token: String,
    pub refresh_token: Option<String>,
}

impl RegisterTokenUseCase {
    pub fn new(repo: Arc<dyn MessengerTokenRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, request: RegisterTokenRequest) -> anyhow::Result<MessengerToken> {
        let existing_tokens = self.repo.list_by_user(&request.user_id).await?;
        let existing_token = existing_tokens
            .into_iter()
            .find(|t| t.messenger == request.messenger);

        let (id, created_at) = if let Some(existing) = existing_token {
            (existing.id, existing.created_at)
        } else {
            (Uuid::new_v4(), Utc::now())
        };

        let token = MessengerToken {
            id,
            user_id: request.user_id,
            messenger: request.messenger,
            access_token: request.access_token,
            refresh_token: request.refresh_token,
            status: MessengerTokenStatus::Active,
            created_at,
            updated_at: Utc::now(),
        };

        self.repo.upsert(token.clone()).await
    }
}
