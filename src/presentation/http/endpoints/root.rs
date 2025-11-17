use std::sync::Arc;

use poem_openapi::Tags;

use crate::application::services::jwt::JwtServiceConfig;
use crate::application::usecases::{
    authenticate_user::AuthenticateUserUseCase, list_chats::ListChatsUseCase,
    list_messages::ListMessagesUseCase, list_tokens::ListTokensUseCase,
    register_token::RegisterTokenUseCase, retry_message::RetryMessageUseCase,
    schedule_message::ScheduleMessageUseCase,
};

#[derive(Clone)]
pub struct ApiState {
    pub auth_usecase: Arc<AuthenticateUserUseCase>,
    pub register_token_usecase: Arc<RegisterTokenUseCase>,
    pub list_tokens_usecase: Arc<ListTokensUseCase>,
    pub list_chats_usecase: Arc<ListChatsUseCase>,
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
    Chats,
}
