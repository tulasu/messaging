use poem_openapi::Object;
use uuid::Uuid;

use crate::presentation::models::{MessengerKind, RequestedByKind};

#[derive(Object, Debug)]
pub struct AuthRequestDto {
    #[oai(validator(email))]
    pub email: String,
    pub display_name: Option<String>,
}

#[derive(Object, Debug)]
pub struct RegisterTokenRequestDto {
    pub messenger: MessengerKind,
    #[oai(validator(min_length = 1))]
    pub access_token: String,
    pub refresh_token: Option<String>,
}

#[derive(Object, Debug)]
pub struct SendMessageRequestDto {
    pub messenger: MessengerKind,
    #[oai(validator(min_length = 1))]
    pub recipient: String,
    #[oai(validator(min_length = 1, max_length = 4096))]
    pub text: String,
    #[oai(default)]
    pub requested_by: RequestedByKind,
}

#[derive(Object, Debug)]
pub struct RetryMessageRequestDto {
    pub message_id: Uuid,
}
