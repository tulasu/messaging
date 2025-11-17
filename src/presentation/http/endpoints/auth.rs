use std::sync::Arc;

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
        request: Json<AuthRequestDto>,
    ) -> poem::Result<Json<AuthResponseDto>> {
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

        Ok(Json(AuthResponseDto {
            token: response.token,
        }))
    }
}

fn internal_error(err: anyhow::Error) -> poem::Error {
    poem::Error::from_string(
        err.to_string(),
        poem::http::StatusCode::INTERNAL_SERVER_ERROR,
    )
}
