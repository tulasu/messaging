use std::sync::Arc;

use poem_openapi::{OpenApi, payload::Json};

use crate::{
    application::usecases::register_token::RegisterTokenRequest,
    presentation::http::{
        endpoints::root::{ApiState, EndpointsTags},
        mappers::map_token,
        requests::RegisterTokenRequestDto,
        responses::MessengerTokenDto,
        security::JwtAuth,
    },
};

#[derive(Clone)]
pub struct TokensEndpoints {
    state: Arc<ApiState>,
}

impl TokensEndpoints {
    pub fn new(state: Arc<ApiState>) -> Self {
        Self { state }
    }
}

#[OpenApi]
impl TokensEndpoints {
    #[oai(
        path = "/messengers/tokens",
        method = "post",
        tag = EndpointsTags::Tokens,
    )]
    pub async fn register_token(
        &self,
        auth: JwtAuth,
        request: Json<RegisterTokenRequestDto>,
    ) -> poem::Result<Json<MessengerTokenDto>> {
        let user = auth.into_user(&self.state.jwt_config)?;
        let payload = RegisterTokenRequest {
            user_id: user.user_id,
            messenger: request.messenger.into(),
            access_token: request.access_token.clone(),
            refresh_token: request.refresh_token.clone(),
        };

        let token = self
            .state
            .register_token_usecase
            .execute(payload)
            .await
            .map_err(internal_error)?;

        Ok(Json(map_token(&token)))
    }

    #[oai(
        path = "/messengers/tokens",
        method = "get",
        tag = EndpointsTags::Tokens,
    )]
    pub async fn list_tokens(&self, auth: JwtAuth) -> poem::Result<Json<Vec<MessengerTokenDto>>> {
        let user = auth.into_user(&self.state.jwt_config)?;

        let tokens = self
            .state
            .list_tokens_usecase
            .execute(user.user_id)
            .await
            .map_err(internal_error)?;

        Ok(Json(tokens.iter().map(map_token).collect()))
    }
}

fn internal_error(err: anyhow::Error) -> poem::Error {
    poem::Error::from_string(
        err.to_string(),
        poem::http::StatusCode::INTERNAL_SERVER_ERROR,
    )
}
