use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use uuid::Uuid;
use messaging::domain::{DeliveryStatus, MessengerType};
use crate::AppState;
use crate::dtos::{SendMessageRequest, MessageResponse, MessageStatusResponse, MessengerTypeDto, DestinationStatusDto};
use crate::error::ApiError;

pub struct MessageHandler;

impl MessageHandler {
    pub async fn send_message(
        State(state): State<AppState>,
        Json(request): Json<SendMessageRequest>,
    ) -> Result<Json<MessageResponse>, ApiError> {
        let destinations = request.destinations
            .into_iter()
            .map(|d| (MessengerType::from(d.messenger_type), d.chat_id))
            .collect();

        let send_request = messaging::application::usecases::SendMessageRequest {
            content: request.content,
            format: request.format,
            destinations,
        };

        let message_id = state.send_message_use_case
            .execute(send_request)
            .await
            .map_err(ApiError::from)?;

        Ok(Json(MessageResponse {
            id: message_id,
            status: "queued".to_string(),
            created_at: chrono::Utc::now(),
        }))
    }

    pub async fn get_message_status(
        State(state): State<AppState>,
        Path(message_id): Path<Uuid>,
    ) -> Result<Json<MessageStatusResponse>, ApiError> {
        let message = state.message_service
            .get_message_details(message_id)
            .await
            .map_err(ApiError::from)?;

        let destinations = message.destinations
            .into_iter()
            .map(|d| DestinationStatusDto {
                destination_id: d.id,
                messenger_type: match d.messenger_type {
                    MessengerType::Telegram => MessengerTypeDto::Telegram,
                    MessengerType::VK => MessengerTypeDto::Vk,
                    MessengerType::MAX => MessengerTypeDto::Max,
                },
                chat_id: d.chat_id.to_string(),
                status: DeliveryStatus::into(d.status.clone()),
                retry_count: d.retry_count,
                last_attempt: d.last_attempt,
                sent_at: d.sent_at,
                error_message: d.error_message,
            })
            .collect();

        Ok(Json(MessageStatusResponse {
            message_id: message.id,
            status: "processing".to_string(),
            destinations,
            created_at: message.created_at,
        }))
    }

    pub async fn retry_message(
        State(state): State<AppState>,
        Path(destination_id): Path<Uuid>,
    ) -> Result<StatusCode, ApiError> {
        state.message_service
            .retry_failed_message(destination_id)
            .await
            .map_err(ApiError::from)?;

        Ok(StatusCode::ACCEPTED)
    }
}