use crate::application::ports::{EventDispatcher, MessageQueue, MessageRepository};
use crate::domain::{
    ChatId, DeliveryStatus, DomainEvent, MessageDestination, MessageProcessing,
    MessageRetryScheduled, MessageSent, MessengerType,
};
use chrono::Utc;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum ProcessingError {
    #[error("Message not found: {0}")]
    MessageNotFound(Uuid),
    #[error("Destination not found: {0}")]
    DestinationNotFound(Uuid),
    #[error("Repository error: {0}")]
    RepositoryError(String),
    #[error("Queue error: {0}")]
    QueueError(String),
    #[error("Event dispatch error: {0}")]
    EventError(String),
    #[error("Invalid status transition")]
    InvalidStatusTransition,
}

pub struct MessageService {
    repository: Box<dyn MessageRepository>,
    event_dispatcher: Box<dyn EventDispatcher>,
    queue: Box<dyn MessageQueue>,
}

impl MessageService {
    pub fn new(
        repository: Box<dyn MessageRepository>,
        event_dispatcher: Box<dyn EventDispatcher>,
        queue: Box<dyn MessageQueue>,
    ) -> Self {
        Self {
            repository,
            event_dispatcher,
            queue,
        }
    }

    async fn update_destination_status(
        &self,
        destination_id: Uuid,
        status: DeliveryStatus,
        error_message: Option<String>,
    ) -> Result<(), ProcessingError> {
        // In a real implementation, we'd fetch the destination first
        // For now, we'll create a placeholder
        let destination = MessageDestination {
            id: destination_id,
            message_id: Uuid::new_v4(),              // Placeholder
            messenger_type: MessengerType::Telegram, // Placeholder
            chat_id: ChatId::new("placeholder".to_string()).unwrap(),
            status: status.clone(),
            retry_count: 0,
            last_attempt: Some(Utc::now()),
            sent_at: if status == DeliveryStatus::Sent {
                Some(Utc::now())
            } else {
                None
            },
            error_message,
        };

        self.repository
            .update_destination(&destination)
            .await
            .map_err(|e| ProcessingError::RepositoryError(e.to_string()))?;

        Ok(())
    }
}

impl MessageService {
    pub async fn process_pending_message(
        &self,
        messenger_type: MessengerType,
    ) -> Result<(), ProcessingError> {
        while let Some((message_id, destination_id)) = self
            .queue
            .dequeue(messenger_type.clone())
            .await
            .map_err(|e| ProcessingError::QueueError(e.to_string()))?
        {
            self.process_single_message(message_id, destination_id, messenger_type.clone())
                .await?;
        }
        Ok(())
    }

    async fn process_single_message(
        &self,
        message_id: Uuid,
        destination_id: Uuid,
        messenger_type: MessengerType,
    ) -> Result<(), ProcessingError> {
        // Update status to Processing
        self.update_destination_status(destination_id, DeliveryStatus::Processing, None)
            .await?;

        // Dispatch MessageProcessing event
        let event = DomainEvent::MessageProcessing(MessageProcessing {
            message_id,
            destination_id,
            messenger_type: messenger_type.clone(),
            occurred_at: Utc::now(),
        });

        self.event_dispatcher
            .dispatch(event)
            .await
            .map_err(|e| ProcessingError::EventError(e.to_string()))?;

        // In a real implementation, this would send the actual message
        // For now, we'll simulate success
        self.handle_message_success(message_id, destination_id, messenger_type)
            .await?;

        Ok(())
    }

    async fn handle_message_success(
        &self,
        message_id: Uuid,
        destination_id: Uuid,
        messenger_type: MessengerType,
    ) -> Result<(), ProcessingError> {
        // Update status to Sent
        self.update_destination_status(destination_id, DeliveryStatus::Sent, None)
            .await?;

        // Dispatch MessageSent event
        let event = DomainEvent::MessageSent(MessageSent {
            message_id,
            destination_id,
            messenger_type,
            chat_id: "placeholder".to_string(), // Would be actual chat ID
            platform_message_id: Some("platform_id".to_string()),
            occurred_at: Utc::now(),
        });

        self.event_dispatcher
            .dispatch(event)
            .await
            .map_err(|e| ProcessingError::EventError(e.to_string()))?;

        Ok(())
    }

    pub async fn retry_failed_message(&self, destination_id: Uuid) -> Result<(), ProcessingError> {
        // In a real implementation, we'd fetch the destination to check retry count
        // For now, we'll just enqueue it again

        let event = DomainEvent::MessageRetryScheduled(MessageRetryScheduled {
            message_id: Uuid::new_v4(), // Placeholder
            destination_id,
            messenger_type: MessengerType::Telegram, // Placeholder
            retry_count: 1,                          // Placeholder
            scheduled_at: Utc::now(),
        });

        self.event_dispatcher
            .dispatch(event)
            .await
            .map_err(|e| ProcessingError::EventError(e.to_string()))?;

        Ok(())
    }

    pub async fn get_message_status(
        &self,
        message_id: Uuid,
    ) -> Result<Vec<MessageDestination>, ProcessingError> {
        self.repository
            .find_destinations_by_message_id(message_id)
            .await
            .map_err(|e| ProcessingError::RepositoryError(e.to_string()))
    }
}
