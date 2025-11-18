use std::sync::Arc;

use poem::{Result as PoemResult, web::cookie::CookieJar};
use poem_openapi::{OpenApi, param::Query, payload::Json};

use crate::{
    application::usecases::{
        retry_message::RetryMessageRequest,
        schedule_message::ScheduleMessageRequest,
    },
    presentation::http::{
        endpoints::root::{ApiState, EndpointsTags},
        mappers::{map_attempt, map_history},
        requests::{RetryMessageRequestDto, SendMessageRequestDto},
        responses::{MessageAttemptDto, PaginatedMessagesDto, SendMessageResponseDto},
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
        path = "/messages",
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
        path = "/messages",
        method = "get",
        tag = EndpointsTags::Messages,
    )]
    pub async fn list_messages(
        &self,
        cookie_jar: &CookieJar,
        limit: Query<Option<u32>>,
        offset: Query<Option<u32>>,
    ) -> PoemResult<Json<PaginatedMessagesDto>> {
        let user = JwtAuth::from_cookies(cookie_jar, &self.state.jwt_config)?;

        let result = self
            .state
            .list_messages_usecase
            .execute(user.user_id, limit.0, offset.0)
            .await
            .map_err(internal_error)?;

        Ok(Json(PaginatedMessagesDto {
            messages: result.messages.iter().map(map_history).collect(),
            has_more: result.has_more,
            next_offset: result.next_offset,
        }))
    }

    #[oai(
        path = "/messages/:message_id/attempts",
        method = "get",
        tag = EndpointsTags::Messages,
    )]
    pub async fn get_message_attempts(
        &self,
        cookie_jar: &CookieJar,
        message_id: poem_openapi::param::Path<uuid::Uuid>,
    ) -> PoemResult<Json<Vec<MessageAttemptDto>>> {
        let user = JwtAuth::from_cookies(cookie_jar, &self.state.jwt_config)?;

        let attempts = self
            .state
            .get_message_attempts_usecase
            .execute(message_id.0, user.user_id)
            .await
            .map_err(|e| {
                if e.to_string().contains("forbidden") {
                    poem::Error::from_string("forbidden", poem::http::StatusCode::FORBIDDEN)
                } else if e.to_string().contains("not found") {
                    poem::Error::from_string("message not found", poem::http::StatusCode::NOT_FOUND)
                } else {
                    internal_error(e)
                }
            })?;

        Ok(Json(attempts.iter().map(map_attempt).collect()))
    }

    #[oai(
        path = "/messages/actions/retry",
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
