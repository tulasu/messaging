use poem::error::{BadRequest, InternalServerError};
use poem_openapi::{OpenApi, payload::Json};

use crate::{
    application::usecases::{
        retry_message::RetryMessageRequest, schedule_message::ScheduleMessageRequest,
    },
    presentation::http::{
        endpoints::root::{Endpoints, EndpointsTags},
        mappers::map_history,
        requests::{RetryMessageRequestDto, SendMessageRequestDto},
        responses::{MessageHistoryDto, SendMessageResponseDto},
        security::JwtAuth,
    },
};

#[OpenApi]
impl Endpoints {
    #[oai(
        path = "/messages/send",
        method = "post",
        tag = EndpointsTags::Messages,
        security(("jwt" = [JwtAuth]))
    )]
    pub async fn send_message(
        &self,
        auth: JwtAuth,
        request: Json<SendMessageRequestDto>,
    ) -> poem::Result<Json<SendMessageResponseDto>> {
        let user = auth.into_user(&self.state.jwt_config)?;
        let payload = ScheduleMessageRequest {
            user_id: user.user_id,
            messenger: request.messenger.into(),
            recipient: request.recipient.clone(),
            text: request.text.clone(),
            requested_by: request.requested_by.into(),
        };

        let response = self
            .state
            .schedule_message_usecase
            .execute(payload)
            .await
            .map_err(|err| InternalServerError(err.to_string()))?;

        Ok(Json(SendMessageResponseDto {
            message_id: response.message_id,
        }))
    }

    #[oai(
        path = "/messages/history",
        method = "get",
        tag = EndpointsTags::Messages,
        security(("jwt" = [JwtAuth]))
    )]
    pub async fn list_messages(&self, auth: JwtAuth) -> poem::Result<Json<Vec<MessageHistoryDto>>> {
        let user = auth.into_user(&self.state.jwt_config)?;

        let entries = self
            .state
            .list_messages_usecase
            .execute(user.user_id)
            .await
            .map_err(|err| InternalServerError(err.to_string()))?;

        Ok(Json(entries.iter().map(map_history).collect()))
    }

    #[oai(
        path = "/messages/retry",
        method = "post",
        tag = EndpointsTags::Messages,
        security(("jwt" = [JwtAuth]))
    )]
    pub async fn retry_message(
        &self,
        auth: JwtAuth,
        request: Json<RetryMessageRequestDto>,
    ) -> poem::Result<()> {
        let user = auth.into_user(&self.state.jwt_config)?;

        self.state
            .retry_message_usecase
            .execute(RetryMessageRequest {
                user_id: user.user_id,
                message_id: request.message_id,
            })
            .await
            .map_err(|err| BadRequest(err.to_string()))?;

        Ok(())
    }
}
