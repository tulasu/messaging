use poem::{Error as PoemError, Result as PoemResult, http::StatusCode};
use poem_openapi::SecurityScheme;
use poem_openapi::auth::Bearer;
use uuid::Uuid;

use crate::application::services::jwt::{JwtService, JwtServiceConfig};

#[derive(SecurityScheme)]
#[oai(ty = "bearer", bearer_format = "JWT")]
pub struct JwtAuth(pub Bearer);

pub struct AuthenticatedUser {
    pub user_id: Uuid,
    pub email: String,
}

impl JwtAuth {
    pub fn into_user(self, config: &JwtServiceConfig) -> PoemResult<AuthenticatedUser> {
        let service = JwtService::new(config.clone());
        match service.verify(&self.0.token) {
            Ok(claims) => Ok(AuthenticatedUser {
                user_id: claims.sub,
                email: claims.email,
            }),
            Err(_) => Err(PoemError::from_string(
                "invalid or expired token",
                StatusCode::UNAUTHORIZED,
            )),
        }
    }
}
