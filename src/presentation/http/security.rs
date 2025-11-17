use poem::{Error as PoemError, Result as PoemResult, http::StatusCode, web::cookie::CookieJar};
use uuid::Uuid;

use crate::application::services::jwt::{JwtService, JwtServiceConfig};

pub struct JwtAuth;

pub struct AuthenticatedUser {
    pub user_id: Uuid,
    pub email: String,
}

impl JwtAuth {
    pub fn from_cookies(
        cookie_jar: &CookieJar,
        config: &JwtServiceConfig,
    ) -> PoemResult<AuthenticatedUser> {
        let token = cookie_jar
            .get("access_token")
            .map(|c| c.value_str().to_string())
            .ok_or_else(|| {
                PoemError::from_string("access token not found", StatusCode::UNAUTHORIZED)
            })?;

        let service = JwtService::new(config.clone());
        match service.verify(&token) {
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
