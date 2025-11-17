use std::sync::Arc;
use std::time::Duration;

use poem::{
    Error as PoemError, Result as PoemResult,
    http::StatusCode,
    web::cookie::{Cookie, CookieJar, SameSite},
};
use poem_openapi::{OpenApi, payload::Json};

use crate::{
    application::usecases::authenticate_user::AuthRequest,
    presentation::http::{
        endpoints::root::{ApiState, EndpointsTags},
        requests::AuthRequestDto,
        responses::AuthResponseDto,
    },
};

#[derive(Clone)]
pub struct AuthEndpoints {
    state: Arc<ApiState>,
}

impl AuthEndpoints {
    pub fn new(state: Arc<ApiState>) -> Self {
        Self { state }
    }
}

#[OpenApi]
impl AuthEndpoints {
    #[oai(path = "/auth/login", method = "post", tag = EndpointsTags::Auth)]
    pub async fn login(
        &self,
        cookie_jar: &CookieJar,
        request: Json<AuthRequestDto>,
    ) -> PoemResult<Json<AuthResponseDto>> {
        let payload = AuthRequest {
            email: request.email.clone(),
            display_name: request.display_name.clone(),
        };

        let response = self
            .state
            .auth_usecase
            .execute(payload)
            .await
            .map_err(internal_error)?;

        let mut access_token_cookie = Cookie::new_with_str("access_token", response.access_token);
        access_token_cookie.set_http_only(true);
        access_token_cookie.set_secure(true);
        access_token_cookie.set_same_site(Some(SameSite::Strict));
        access_token_cookie.set_path("/");
        access_token_cookie.set_max_age(Duration::from_secs(
            self.state.jwt_config.expiration.as_secs(),
        ));

        let mut refresh_token_cookie =
            Cookie::new_with_str("refresh_token", response.refresh_token);
        refresh_token_cookie.set_http_only(true);
        refresh_token_cookie.set_secure(true);
        refresh_token_cookie.set_same_site(Some(SameSite::Strict));
        refresh_token_cookie.set_path("/");
        refresh_token_cookie.set_max_age(Duration::from_secs(
            self.state.jwt_config.refresh_expiration.as_secs(),
        ));

        cookie_jar.add(access_token_cookie);
        cookie_jar.add(refresh_token_cookie);

        Ok(Json(AuthResponseDto { success: true }))
    }

    #[oai(path = "/auth/refresh", method = "post", tag = EndpointsTags::Auth)]
    pub async fn refresh(&self, cookie_jar: &CookieJar) -> PoemResult<Json<AuthResponseDto>> {
        let refresh_token = cookie_jar
            .get("refresh_token")
            .map(|c| c.value_str().to_string())
            .ok_or_else(|| {
                PoemError::from_string("refresh token not found", StatusCode::UNAUTHORIZED)
            })?;

        let jwt_service =
            crate::application::services::jwt::JwtService::new(self.state.jwt_config.clone());
        let claims = jwt_service.verify(&refresh_token).map_err(|_| {
            PoemError::from_string("invalid or expired refresh token", StatusCode::UNAUTHORIZED)
        })?;

        let response = self
            .state
            .auth_usecase
            .refresh(claims.sub)
            .await
            .map_err(internal_error)?;

        let mut access_token_cookie = Cookie::new_with_str("access_token", response.access_token);
        access_token_cookie.set_http_only(true);
        access_token_cookie.set_secure(true);
        access_token_cookie.set_same_site(Some(SameSite::Strict));
        access_token_cookie.set_path("/");
        access_token_cookie.set_max_age(Duration::from_secs(
            self.state.jwt_config.expiration.as_secs(),
        ));

        let mut refresh_token_cookie =
            Cookie::new_with_str("refresh_token", response.refresh_token);
        refresh_token_cookie.set_http_only(true);
        refresh_token_cookie.set_secure(true);
        refresh_token_cookie.set_same_site(Some(SameSite::Strict));
        refresh_token_cookie.set_path("/");
        refresh_token_cookie.set_max_age(Duration::from_secs(
            self.state.jwt_config.refresh_expiration.as_secs(),
        ));

        cookie_jar.add(access_token_cookie);
        cookie_jar.add(refresh_token_cookie);

        Ok(Json(AuthResponseDto { success: true }))
    }

    #[oai(path = "/auth/logout", method = "post", tag = EndpointsTags::Auth)]
    pub async fn logout(&self, cookie_jar: &CookieJar) -> PoemResult<Json<AuthResponseDto>> {
        let mut access_token_cookie = Cookie::named("access_token");
        access_token_cookie.set_http_only(true);
        access_token_cookie.set_secure(true);
        access_token_cookie.set_same_site(Some(SameSite::Strict));
        access_token_cookie.set_path("/");
        access_token_cookie.make_removal();

        let mut refresh_token_cookie = Cookie::named("refresh_token");
        refresh_token_cookie.set_http_only(true);
        refresh_token_cookie.set_secure(true);
        refresh_token_cookie.set_same_site(Some(SameSite::Strict));
        refresh_token_cookie.set_path("/");
        refresh_token_cookie.make_removal();

        cookie_jar.add(access_token_cookie);
        cookie_jar.add(refresh_token_cookie);

        Ok(Json(AuthResponseDto { success: true }))
    }
}

fn internal_error(err: anyhow::Error) -> PoemError {
    PoemError::from_string(err.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
}
