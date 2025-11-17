use std::sync::Arc;

use chrono::Utc;
use uuid::Uuid;

use crate::{
    application::services::jwt::{JwtService, JwtServiceConfig},
    domain::models::User,
    domain::repositories::UserRepository,
};

pub struct AuthenticateUserUseCase {
    user_repo: Arc<dyn UserRepository>,
    jwt: JwtService,
}

pub struct AuthRequest {
    pub email: String,
    pub display_name: Option<String>,
}

pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
}

impl AuthenticateUserUseCase {
    pub fn new(user_repo: Arc<dyn UserRepository>, jwt_config: JwtServiceConfig) -> Self {
        let jwt = JwtService::new(jwt_config);
        Self { user_repo, jwt }
    }

    pub async fn execute(&self, request: AuthRequest) -> anyhow::Result<AuthResponse> {
        let mut user = if let Some(existing) = self.user_repo.find_by_email(&request.email).await? {
            existing
        } else {
            User {
                id: Uuid::new_v4(),
                email: request.email.clone(),
                display_name: request.display_name.clone(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            }
        };

        user.display_name = user.display_name.or(request.display_name.clone());
        user.updated_at = Utc::now();
        self.user_repo.upsert(&user).await?;

        let access_token = self.jwt.issue(&user)?;
        let refresh_token = self.jwt.issue_refresh(&user)?;
        Ok(AuthResponse {
            access_token,
            refresh_token,
        })
    }

    pub async fn refresh(&self, user_id: Uuid) -> anyhow::Result<AuthResponse> {
        let user = self
            .user_repo
            .get(&user_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("user not found"))?;

        let access_token = self.jwt.issue(&user)?;
        let refresh_token = self.jwt.issue_refresh(&user)?;
        Ok(AuthResponse {
            access_token,
            refresh_token,
        })
    }
}
