use poem::error::Unauthorized;
use poem_openapi::auth::Bearer;
use poem_openapi::SecurityScheme;
use uuid::Uuid;

use crate::application::services::jwt::{JwtService, JwtServiceConfig};

#[derive(SecurityScheme)]
#[oai(
    ty = "bearer",
    bearer_format = "JWT",
    in = "header",
    key_name = "Authorization"
)]
pub struct JwtAuth(pub Bearer);

pub struct AuthenticatedUser {
    pub user_id: Uuid,
    pub email: String,
}

impl JwtAuth {
    pub fn into_user(self, config: &JwtServiceConfig) -> Result<AuthenticatedUser, Unauthorized> {
        let token = self
            .0
            .token
            .ok_or_else(|| Unauthorized("Missing bearer token"))?;

        let service = JwtService::new(config.clone());
        match service.verify(&token) {
            Ok(claims) => Ok(AuthenticatedUser {
                user_id: claims.sub,
                email: claims.email,
            }),
            Err(_) => Err(Unauthorized("Invalid or expired token")),
        }
    }
}

