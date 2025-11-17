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

    pub async fn execute(
        &self,
        request: RegisterTokenRequest,
    ) -> anyhow::Result<MessengerToken> {
        let token = MessengerToken {
            id: Uuid::new_v4(),
            user_id: request.user_id,
            messenger: request.messenger,
            access_token: request.access_token,
            refresh_token: request.refresh_token,
            status: MessengerTokenStatus::Active,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        self.repo.upsert(token.clone()).await
    }
}

