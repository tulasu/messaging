use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::Json;
use uuid::Uuid;
use crate::application::usecases::SendMessageUseCase;
use crate::application::ports::MessageRepository;
use crate::domain::models::{Message, Payload, MessageDestination, MessengerType, DeliveryStatus};
use crate::domain::value_objects::{ChatId, MessageContent, TextFormat};
use crate::api::error::ApiError;
use crate::api::dtos::send_message_dto::*;
use std::sync::Arc;

pub struct MessageHandler<R: MessageRepository> {
    send_message_use_case: Arc<SendMessageUseCase<R>>,
}

impl<R: MessageRepository> MessageHandler<R> {
    pub fn new(message_repository: Arc<R>) -> Self {
        Self {
            send_message_use_case: Arc::new(SendMessageUseCase::new(message_repository)),
        }
    }

    pub async fn send_message(
        State(handler): State<Arc<MessageHandler<R>>>,
        Json(request): Json<SendMessageRequest>,
    ) -> Result<Json<SendMessageResponse>, ApiError> {
        // Convert DTOs to domain models
        let payload = match request.payload {
            PayloadDto::Plain { content } => Payload::Plain { content },
            PayloadDto::Formatted { content, format } => Payload::Formatted {
                content,
                format: match format {
                    TextFormatDto::Plain => TextFormat::Plain,
                    TextFormatDto::Markdown => TextFormat::Markdown,
                    TextFormatDto::Html => TextFormat::Html,
                },
            },
        };

        let mut destinations = Vec::new();
        for dest_dto in request.destinations {
            let chat_id = ChatId::new(dest_dto.chat_id)
                .map_err(|e| ApiError::Validation(format!("Invalid chat ID: {}", e)))?;

            destinations.push(MessageDestination {
                id: Uuid::new_v4(),
                message_id: Uuid::new_v4(), // Will be set after message creation
                messenger_type: dest_dto.messenger_type.into(),
                chat_id,
                status: DeliveryStatus::Pending,
                retry_count: 0,
                last_attempt: None,
                sent_at: None,
                error_message: None,
            });
        }

        // Create the message
        let message = Message {
            id: Uuid::new_v4(),
            payload,
            destinations: destinations.clone(),
            created_at: chrono::Utc::now(),
        };

        // Set the message_id for each destination
        for destination in &mut destinations {
            destination.message_id = message.id;
        }

        // Execute use case
        handler.send_message_use_case
            .execute(&message)
            .await
            .map_err(|e| ApiError::Service(format!("Failed to send message: {}", e)))?;

        // Convert to response
        let queued_destinations = destinations
            .into_iter()
            .map(|dest| QueuedDestinationDto {
                destination_id: dest.id,
                messenger_type: match dest.messenger_type {
                    MessengerType::Telegram => MessengerTypeDto::Telegram,
                    MessengerType::VK => MessengerTypeDto::Vk,
                    MessengerType::MAX => MessengerTypeDto::Max,
                },
                chat_id: dest.chat_id.as_str().to_string(),
            })
            .collect();

        Ok(Json(SendMessageResponse {
            message_id: message.id,
            status: "queued".to_string(),
            queued_destinations,
        }))
    }

    pub async fn get_message_status(
        State(handler): State<Arc<MessageHandler<R>>>,
        Path(message_id): Path<Uuid>,
    ) -> Result<Json<MessageStatusResponse>, ApiError> {
        // This would require implementing a GetMessageUseCase
        // For now, we'll return a placeholder response
        Err(ApiError::MessageNotFound)
    }

    pub async fn get_messages(
        State(handler): State<Arc<MessageHandler<R>>>,
    ) -> Result<Json<Vec<MessageStatusResponse>>, ApiError> {
        // This would require implementing a ListMessagesUseCase
        // For now, we'll return an empty list
        Ok(Json(vec![]))
    }

    pub async fn retry_message(
        State(handler): State<Arc<MessageHandler<R>>>,
        Path(message_id): Path<Uuid>,
    ) -> Result<StatusCode, ApiError> {
        // This would require implementing a RetryMessageUseCase
        // For now, we'll return not implemented
        Err(ApiError::Internal("Retry functionality not implemented".to_string()))
    }
}