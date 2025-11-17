use poem::error::InternalServerError;
use poem_openapi::{OpenApi, payload::Json};

use crate::{
    application::usecases::authenticate_user::AuthRequest,
    presentation::http::{
        endpoints::root::{Endpoints, EndpointsTags},
        requests::AuthRequestDto,
        responses::AuthResponseDto,
    },
};

#[OpenApi]
impl Endpoints {
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
            .map_err(|err| InternalServerError(err.to_string()))?;

        Ok(Json(AuthResponseDto {
            token: response.token,
        }))
    }
}
