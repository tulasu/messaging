use std::sync::Arc;

use poem::{Result as PoemResult, web::cookie::CookieJar};
use poem_openapi::{OpenApi, payload::Json};

use crate::{
    application::usecases::{
        retry_message::RetryMessageRequest, schedule_message::ScheduleMessageRequest,
    },
    presentation::http::{
        endpoints::root::{ApiState, EndpointsTags},
        mappers::map_history,
        requests::{RetryMessageRequestDto, SendMessageRequestDto},
        responses::{MessageHistoryDto, SendMessageResponseDto},
        security::JwtAuth,
    },
};

#[derive(Clone)]
pub struct MessagesEndpoints {
    state: Arc<ApiState>,
}

impl MessagesEndpoints {
    pub fn new(state: Arc<ApiState>) -> Self {
        Self { state }
    }
}

#[OpenApi]
impl MessagesEndpoints {
    #[oai(
        path = "/messages/send",
        method = "post",
        tag = EndpointsTags::Messages,
    )]
    pub async fn send_message(
        &self,
        cookie_jar: &CookieJar,
        request: Json<SendMessageRequestDto>,
    ) -> PoemResult<Json<SendMessageResponseDto>> {
        let user = JwtAuth::from_cookies(cookie_jar, &self.state.jwt_config)?;
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
            .map_err(internal_error)?;

        Ok(Json(SendMessageResponseDto {
            message_id: response.message_id,
        }))
    }

    #[oai(
        path = "/messages/history",
        method = "get",
        tag = EndpointsTags::Messages,
    )]
    pub async fn list_messages(
        &self,
        cookie_jar: &CookieJar,
    ) -> PoemResult<Json<Vec<MessageHistoryDto>>> {
        let user = JwtAuth::from_cookies(cookie_jar, &self.state.jwt_config)?;

        let entries = self
            .state
            .list_messages_usecase
            .execute(user.user_id)
            .await
            .map_err(internal_error)?;

        Ok(Json(entries.iter().map(map_history).collect()))
    }

    #[oai(
        path = "/messages/retry",
        method = "post",
        tag = EndpointsTags::Messages,
    )]
    pub async fn retry_message(
        &self,
        cookie_jar: &CookieJar,
        request: Json<RetryMessageRequestDto>,
    ) -> PoemResult<()> {
        let user = JwtAuth::from_cookies(cookie_jar, &self.state.jwt_config)?;

        self.state
            .retry_message_usecase
            .execute(RetryMessageRequest {
                user_id: user.user_id,
                message_id: request.message_id,
            })
            .await
            .map_err(bad_request)?;

        Ok(())
    }
}

fn internal_error(err: anyhow::Error) -> poem::Error {
    poem::Error::from_string(
        err.to_string(),
        poem::http::StatusCode::INTERNAL_SERVER_ERROR,
    )
}

fn bad_request(err: anyhow::Error) -> poem::Error {
    poem::Error::from_string(err.to_string(), poem::http::StatusCode::BAD_REQUEST)
}
