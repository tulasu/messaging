use std::sync::Arc;

use poem::{Error as PoemError, Result as PoemResult, web::cookie::CookieJar};
use poem_openapi::{OpenApi, param::Path, payload::Json};

use crate::presentation::http::{
    endpoints::root::{ApiState, EndpointsTags},
    mappers::map_chat,
    responses::MessengerChatDto,
    security::JwtAuth,
};
use crate::presentation::models::MessengerKind;

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
    ) -> PoemResult<Json<Vec<MessengerChatDto>>> {
        let user = JwtAuth::from_cookies(cookie_jar, &self.state.jwt_config)?;
        let chats = self
            .state
            .list_chats_usecase
            .execute(user.user_id, messenger.0.into())
            .await
            .map_err(bad_request)?;

        Ok(Json(chats.iter().map(map_chat).collect()))
    }
}

fn bad_request(err: anyhow::Error) -> PoemError {
    PoemError::from_string(err.to_string(), poem::http::StatusCode::BAD_REQUEST)
}
