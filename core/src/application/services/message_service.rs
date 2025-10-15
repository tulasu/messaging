use std::sync::Arc;
use anyhow::Error;
use thiserror::Error;
use uuid::Uuid;
use crate::application::ports::{MessageRepository, MessageQueue, RepositoryError};
use crate::domain::{Message, DeliveryStatus, MessengerType, MessageContent};
use crate::infrastructure::messengers::factory::MessengerAdapterFactory;

#[derive(Debug, Error)]
pub enum MessageServiceError {
    #[error("Message not found: {0}")]
    MessageNotFound(Uuid),
    #[error("Processing error: {0}")]
    ProcessingError(#[from] Error),
    #[error("Repository error: {0}")]
    RepositoryError(#[from] RepositoryError),
}

pub struct MessageService {
    repository: Arc<dyn MessageRepository>,
    queue: Arc<dyn MessageQueue>,
    messenger_factory: Arc<MessengerAdapterFactory>,
}

impl MessageService {
    pub fn new(
        repository: Arc<dyn MessageRepository>,
        queue: Arc<dyn MessageQueue>,
        messenger_factory: Arc<MessengerAdapterFactory>,
    ) -> Self {
        Self {
            repository,
            queue,
            messenger_factory,
        }
    }

    pub async fn get_message_details(&self, message_id: Uuid) -> Result<Message, MessageServiceError> {
        self.repository
            .find_by_id(message_id)
            .await?
            .ok_or(MessageServiceError::MessageNotFound(message_id))
    }

    pub async fn retry_failed_message(&self, destination_id: Uuid) -> Result<(), MessageServiceError> {
        tracing::info!("Manual retry requested for destination: {}", destination_id);

        let message = self.repository.find_by_destination_id(destination_id).await?
            .ok_or(MessageServiceError::MessageNotFound(destination_id))?;

        let destination = message.destinations
            .iter()
            .find(|d| d.id == destination_id)
            .ok_or(MessageServiceError::MessageNotFound(destination_id))?;

        tracing::info!("Processing manual retry for destination {} (retry count: {})", destination_id, destination.retry_count);

        let mut updated_destination = destination.clone();
        updated_destination.status = DeliveryStatus::Pending;
        updated_destination.retry_count += 1;
        updated_destination.last_attempt = Some(chrono::Utc::now());
        updated_destination.error_message = None;

        self.repository.update_destination(&updated_destination).await?;

        let delay = chrono::Duration::minutes(2_i64.pow(updated_destination.retry_count));
        self.queue.requeue_with_delay(
            message.id,
            destination_id,
            destination.messenger_type,
            delay,
        ).await
        .map_err(|e| MessageServiceError::ProcessingError(anyhow::anyhow!(e)))?;

        tracing::info!("Re-queued destination {} with delay of {} minutes", destination_id, delay.num_minutes());

        Ok(())
    }

    pub async fn auto_retry_failed_message(&self, destination_id: Uuid) -> Result<(), MessageServiceError> {
        tracing::info!("Auto retry attempt for destination: {}", destination_id);

        let message = self.repository.find_by_destination_id(destination_id).await?
            .ok_or(MessageServiceError::MessageNotFound(destination_id))?;

        let destination = message.destinations
            .iter()
            .find(|d| d.id == destination_id)
            .ok_or(MessageServiceError::MessageNotFound(destination_id))?;

        // Auto retry - respect retry limit
        if destination.retry_count >= 3 {
            tracing::warn!("Destination {} has exhausted retry limit, no more auto retries", destination_id);
            return Ok(());
        }

        tracing::info!("Processing auto retry for destination {} (retry count: {})", destination_id, destination.retry_count);

        let mut updated_destination = destination.clone();
        updated_destination.status = DeliveryStatus::Pending;
        updated_destination.retry_count += 1;
        updated_destination.last_attempt = Some(chrono::Utc::now());
        updated_destination.error_message = None;

        self.repository.update_destination(&updated_destination).await?;

        // Re-queue with exponential backoff delay
        let delay = chrono::Duration::minutes(2_i64.pow(updated_destination.retry_count));
        self.queue.requeue_with_delay(
            message.id,
            destination_id,
            destination.messenger_type,
            delay,
        ).await
        .map_err(|e| MessageServiceError::ProcessingError(anyhow::anyhow!(e)))?;

        tracing::info!("Auto-requeued destination {} with delay of {} minutes", destination_id, delay.num_minutes());

        Ok(())
    }

    pub async fn process_pending_message(&self, messenger_type: MessengerType) -> Result<(), MessageServiceError> {
        while let Some((message_id, destination_id)) = self.queue.dequeue(messenger_type).await
            .map_err(|e| MessageServiceError::ProcessingError(anyhow::anyhow!(e)))? {
            if let Err(e) = self.process_single_message(message_id, destination_id, messenger_type).await {
                tracing::error!("Failed to process message {}: {}", message_id, e);
            }
        }
        Ok(())
    }

    async fn process_single_message(
        &self,
        message_id: Uuid,
        destination_id: Uuid,
        messenger_type: MessengerType,
    ) -> Result<(), MessageServiceError> {
        let message = self.repository.find_by_id(message_id).await?
            .ok_or(MessageServiceError::MessageNotFound(message_id))?;

        let destination = message.destinations
            .iter()
            .find(|d| d.id == destination_id)
            .ok_or(MessageServiceError::MessageNotFound(destination_id))?;

        // Get messenger adapter
        let adapter = self.messenger_factory.create_adapter(&messenger_type)
            .map_err(|e| MessageServiceError::ProcessingError(anyhow::anyhow!(e)))?;

        // Send message
        let message_content = MessageContent {
            text: message.payload.content().to_string(),
            format: Some(message.payload.format()),
        };
        let result = adapter.send_message(&destination.chat_id, &message_content).await;

        match result {
            Ok(_) => {
                // Update destination status to sent
                let mut updated_destination = destination.clone();
                updated_destination.status = DeliveryStatus::Sent;
                updated_destination.sent_at = Some(chrono::Utc::now());
                self.repository.update_destination(&updated_destination).await
                    .map_err(|e| MessageServiceError::ProcessingError(anyhow::anyhow!(e)))?;
            }
            Err(e) => {
                // Update destination status to failed
                let mut updated_destination = destination.clone();
                updated_destination.status = DeliveryStatus::Failed;
                updated_destination.last_attempt = Some(chrono::Utc::now());
                updated_destination.error_message = Some(e.to_string());
                self.repository.update_destination(&updated_destination).await
                    .map_err(|e| MessageServiceError::ProcessingError(anyhow::anyhow!(e)))?;

                // Schedule auto retry if not exhausted
                if updated_destination.retry_count < 3 {
                    // Use the auto retry method that respects retry limits
                    if let Err(e) = self.auto_retry_failed_message(destination_id).await {
                        tracing::error!("Failed to auto-retry destination {}: {}", destination_id, e);
                    }
                } else {
                    // Mark as retry exhausted (use Failed status)
                    updated_destination.status = DeliveryStatus::Failed;
                    self.repository.update_destination(&updated_destination).await
                        .map_err(|e| MessageServiceError::ProcessingError(anyhow::anyhow!(e)))?;
                }
            }
        }

        Ok(())
    }
}

  