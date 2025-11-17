use poem::error::InternalServerError;
use poem_openapi::{payload::Json, OpenApi};

use crate::{
    application::usecases::register_token::RegisterTokenRequest,
    presentation::{
        http::{
            endpoints::root::{Endpoints, EndpointsTags},
            mappers::map_token,
            requests::RegisterTokenRequestDto,
            responses::MessengerTokenDto,
            security::JwtAuth,
        },
    },
};

#[OpenApi]
impl Endpoints {
    #[oai(
        path = "/messengers/tokens",
        method = "post",
        tag = EndpointsTags::Tokens,
        security(("jwt" = [JwtAuth]))
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
            .map_err(|err| InternalServerError(err.to_string()))?;

        Ok(Json(map_token(&token)))
    }

    #[oai(
        path = "/messengers/tokens",
        method = "get",
        tag = EndpointsTags::Tokens,
        security(("jwt" = [JwtAuth]))
    )]
    pub async fn list_tokens(
        &self,
        auth: JwtAuth,
    ) -> poem::Result<Json<Vec<MessengerTokenDto>>> {
        let user = auth.into_user(&self.state.jwt_config)?;

        let tokens = self
            .state
            .list_tokens_usecase
            .execute(user.user_id)
            .await
            .map_err(|err| InternalServerError(err.to_string()))?;

        Ok(Json(tokens.iter().map(map_token).collect()))
    }
}

