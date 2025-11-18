use std::sync::Arc;

use poem::{Error as PoemError, Result as PoemResult, web::cookie::CookieJar};
use poem_openapi::{OpenApi, param::{Path, Query}, payload::Json};

use crate::{
    application::services::messenger::PaginationParams,
    presentation::http::{
        endpoints::root::{ApiState, EndpointsTags},
        mappers::map_chat,
        responses::{MessengerChatDto, PaginatedChatsDto},
        security::JwtAuth,
    },
    presentation::models::MessengerKind,
};

#[derive(Clone)]
pub struct ChatsEndpoints {
    state: Arc<ApiState>,
}

impl ChatsEndpoints {
    pub fn new(state: Arc<ApiState>) -> Self {
        Self { state }
    }
}

#[OpenApi]
impl ChatsEndpoints {
    #[oai(
        path = "/messengers/:messenger/chats",
        method = "get",
        tag = EndpointsTags::Chats,
    )]
    pub async fn list_chats(
        &self,
        cookie_jar: &CookieJar,
        messenger: Path<MessengerKind>,
        limit: Query<Option<u32>>,
        offset: Query<Option<u32>>,
    ) -> PoemResult<Json<PaginatedChatsDto>> {
        let user = JwtAuth::from_cookies(cookie_jar, &self.state.jwt_config)?;
        
        let pagination = PaginationParams {
            limit: limit.0,
            offset: offset.0,
        };
        
        let result = self
            .state
            .list_chats_usecase
            .execute(user.user_id, messenger.0.into(), pagination)
            .await
            .map_err(bad_request)?;

        Ok(Json(PaginatedChatsDto {
            chats: result.chats.iter().map(map_chat).collect(),
            has_more: result.has_more,
            next_offset: result.next_offset,
        }))
    }
}

fn bad_request(err: anyhow::Error) -> PoemError {
    PoemError::from_string(err.to_string(), poem::http::StatusCode::BAD_REQUEST)
}
