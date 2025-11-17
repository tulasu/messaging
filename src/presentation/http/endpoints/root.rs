use std::sync::Arc;

use poem_openapi::Tags;

use crate::application::usecases::{
    authenticate_user::AuthenticateUserUseCase, list_messages::ListMessagesUseCase,
    list_tokens::ListTokensUseCase, register_token::RegisterTokenUseCase,
    retry_message::RetryMessageUseCase, schedule_message::ScheduleMessageUseCase,
};
use crate::application::services::jwt::JwtServiceConfig;

/// Root of messaging HTTP API.
///
/// Used with `poem` HTTP server.
#[derive(Clone)]
pub struct Endpoints {
    pub state: Arc<ApiState>,
}

impl Endpoints {
    pub fn new(state: Arc<ApiState>) -> Self {
        Self { state }
    }
}

#[derive(Clone)]
pub struct ApiState {
    pub auth_usecase: Arc<AuthenticateUserUseCase>,
    pub register_token_usecase: Arc<RegisterTokenUseCase>,
    pub list_tokens_usecase: Arc<ListTokensUseCase>,
    pub schedule_message_usecase: Arc<ScheduleMessageUseCase>,
    pub list_messages_usecase: Arc<ListMessagesUseCase>,
    pub retry_message_usecase: Arc<RetryMessageUseCase>,
    pub jwt_config: JwtServiceConfig,
}

/// Enum of API sections (tags)
#[derive(Tags)]
pub enum EndpointsTags {
    Health,
    Auth,
    Tokens,
    Messages,
}
