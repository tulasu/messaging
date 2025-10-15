use async_trait::async_trait;
use uuid::Uuid;
use std::sync::Arc;
use chrono::Utc;
use crate::domain::{Message, MessengerType, Payload, MessageDestination, DeliveryStatus, DomainEvent, ChatId, MessageContent, TextFormat, DomainError};
use crate::application::ports::{MessageRepository, MessageQueue, RepositoryError, QueueError};
use crate::domain::services::MessageRoutingService;

#[async_trait]
pub trait SendMessageUseCase: Send + Sync {
    async fn execute(&self, request: SendMessageRequest) -> Result<Uuid, SendMessageError>;
    async fn execute_batch(&self, request: BatchSendMessageRequest) -> Result<Vec<Uuid>, SendMessageError>;
}

pub struct SendMessageRequest {
    pub content: String,
    pub format: Option<String>,
    pub destinations: Vec<(MessengerType, String)>,
}

pub struct BatchSendMessageRequest {
    pub requests: Vec<SendMessageRequest>,
}

#[derive(Debug, thiserror::Error)]
pub enum SendMessageError {
    #[error("Invalid content: {0}")]
    InvalidContent(String),
    #[error("Invalid destination: {0}")]
    InvalidDestination(String),
    #[error("Domain error: {0}")]
    DomainError(#[from] DomainError),
    #[error("Repository error: {0}")]
    RepositoryError(#[from] RepositoryError),
    #[error("Queue error: {0}")]
    QueueError(#[from] QueueError),
}

pub struct SendMessageUseCaseImpl {
    repository: Arc<dyn MessageRepository>,
    queue: Arc<dyn MessageQueue>,
    routing_service: Arc<dyn MessageRoutingService>,
}

impl SendMessageUseCaseImpl {
    pub fn new(
        repository: Arc<dyn MessageRepository>,
        queue: Arc<dyn MessageQueue>,
        routing_service: Arc<dyn MessageRoutingService>,
    ) -> Self {
        Self {
            repository,
            queue,
            routing_service,
        }
    }
}

#[async_trait]
impl SendMessageUseCase for SendMessageUseCaseImpl {
    async fn execute(&self, request: SendMessageRequest) -> Result<Uuid, SendMessageError> {
        if request.content.trim().is_empty() {
            return Err(SendMessageError::InvalidContent("Content cannot be empty".to_string()));
        }

        if request.destinations.is_empty() {
            return Err(SendMessageError::InvalidDestination("At least one destination is required".to_string()));
        }

        let format = match request.format.as_deref() {
            Some("markdown") => Some(TextFormat::Markdown),
            Some("html") => Some(TextFormat::Html),
            _ => Some(TextFormat::Plain),
        };

        let message_content = MessageContent {
            text: request.content,
            format,
        };

        let message_id = Uuid::new_v4();

        let mut message_destinations = Vec::new();
        for (messenger_type, chat_id_str) in request.destinations {
            let chat_id = ChatId::new(chat_id_str)?;

            let destination = MessageDestination {
                id: Uuid::new_v4(),
                message_id,
                messenger_type,
                chat_id,
                status: DeliveryStatus::Pending,
                retry_count: 0,
                last_attempt: None,
                sent_at: None,
                error_message: None,
            };
            message_destinations.push(destination);
        }

        let payload = match message_content.format {
            Some(TextFormat::Plain) => Payload::Plain { content: message_content.text },
            Some(format) => Payload::Formatted { content: message_content.text, format },
            None => Payload::Plain { content: message_content.text },
        };

        let message = Message {
            id: message_id,
            payload,
            destinations: message_destinations,
            created_at: Utc::now(),
        };

        self.repository.save(&message).await?;

        let events = self.routing_service.route_message(&message);

        for event in events {
            if let DomainEvent::MessageQueued(queued) = event {
                self.queue.enqueue(
                    queued.message_id,
                    queued.destination_id,
                    queued.messenger_type,
                ).await?;
            }
        }

        Ok(message_id)
    }

    async fn execute_batch(&self, request: BatchSendMessageRequest) -> Result<Vec<Uuid>, SendMessageError> {
        let mut message_ids = Vec::new();

        for send_request in request.requests {
            let message_id = self.execute(send_request).await?;
            message_ids.push(message_id);
        }

        Ok(message_ids)
    }
}

// Ensure the struct is Send + Sync for async trait safety
unsafe impl Send for SendMessageUseCaseImpl {}
unsafe impl Sync for SendMessageUseCaseImpl {}