use crate::application::ports::{MessageQueue, MessageRepository, EventDispatcher};
use crate::domain::models::{MessageDestination, DeliveryStatus};
use crate::domain::events::DomainEvent;
use crate::infrastructure::messengers::{MessengerAdapterFactory, MessengerAdapter};
use crate::domain::services::MessageRoutingService;
use tokio::time::{sleep, Duration};
use tracing::{info, error, warn};

pub struct MessageQueueWorker<MQ, MR, ED> {
    message_queue: Arc<MQ>,
    message_repository: Arc<MR>,
    event_dispatcher: Arc<ED>,
    messenger_factory: MessengerAdapterFactory,
    routing_service: MessageRoutingService,
}

impl<MQ, MR, ED> MessageQueueWorker<MQ, MR, ED>
where
    MQ: MessageQueue + Send + Sync + 'static,
    MR: MessageRepository + Send + Sync + 'static,
    ED: EventDispatcher + Send + Sync + 'static,
{
    pub fn new(
        message_queue: Arc<MQ>,
        message_repository: Arc<MR>,
        event_dispatcher: Arc<ED>,
    ) -> Self {
        Self {
            message_queue,
            message_repository,
            event_dispatcher,
            messenger_factory: MessengerAdapterFactory::new(),
            routing_service: MessageRoutingService::default(),
        }
    }

    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Starting message queue worker");

        loop {
            match self.process_message_queue().await {
                Ok(_) => {
                    // Small delay between iterations to prevent busy loop
                    sleep(Duration::from_millis(100)).await;
                }
                Err(e) => {
                    error!("Error processing message queue: {}", e);
                    // Wait longer on error before retrying
                    sleep(Duration::from_secs(5)).await;
                }
            }
        }
    }

    async fn process_message_queue(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Get queued messages
        let queued_destinations = self.message_queue.get_queued_messages(10).await?;

        if queued_destinations.is_empty() {
            return Ok(());
        }

        info!("Processing {} queued destinations", queued_destinations.len());

        for destination in queued_destinations {
            if let Err(e) = self.process_destination(&destination).await {
                error!("Failed to process destination {}: {}", destination.id, e);

                // Update destination status to failed
                let mut failed_destination = destination.clone();
                failed_destination.status = DeliveryStatus::Failed;
                failed_destination.error_message = Some(e.to_string());

                if let Err(update_err) = self.message_repository.update_destination(&failed_destination).await {
                    error!("Failed to update destination status: {}", update_err);
                }
            }
        }

        Ok(())
    }

    async fn process_destination(&self, destination: &MessageDestination) -> Result<(), Box<dyn std::error::Error>> {
        // Mark as processing
        let mut processing_destination = destination.clone();
        processing_destination.status = DeliveryStatus::Queued;
        processing_destination.last_attempt = Some(chrono::Utc::now());

        self.message_repository.update_destination(&processing_destination).await?;

        // Get messenger adapter
        let adapter = self.messenger_factory.create_adapter(&destination.messenger_type)?;

        // Get message content
        let message = self.message_repository.find_by_id(destination.message_id).await?
            .ok_or("Message not found")?;

        let content = match &message.payload {
            crate::domain::models::Payload::Plain { content } => {
                crate::domain::value_objects::MessageContent::new(content.clone())
            }
            crate::domain::models::Payload::Formatted { content, format } => {
                crate::domain::value_objects::MessageContent::with_format(
                    content.clone(),
                    format.clone()
                )
            }
        };

        // Send message
        let sent_message = adapter.send_message(&destination.chat_id, &content).await?;

        // Update destination status to sent
        let mut sent_destination = destination.clone();
        sent_destination.status = DeliveryStatus::Sent;
        sent_destination.sent_at = Some(sent_message.timestamp);
        sent_destination.last_attempt = Some(chrono::Utc::now());

        self.message_repository.update_destination(&sent_destination).await?;

        // Dispatch message sent event
        let event = DomainEvent::MessageSent(crate::domain::events::MessageSent {
            message_id: destination.message_id,
            destination_id: destination.id,
            messenger_type: destination.messenger_type.clone(),
            platform_message_id: sent_message.platform_message_id,
            occurred_at: sent_message.timestamp,
        });

        self.event_dispatcher.dispatch(&event).await?;

        info!("Successfully sent message {} to {}", destination.message_id, destination.chat_id.as_str());

        Ok(())
    }

    pub async fn start_retry_processor(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Starting retry processor");

        loop {
            match self.process_retries().await {
                Ok(_) => {
                    // Check for retries every minute
                    sleep(Duration::from_secs(60)).await;
                }
                Err(e) => {
                    error!("Error processing retries: {}", e);
                    sleep(Duration::from_secs(60)).await;
                }
            }
        }
    }

    async fn process_retries(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Get failed destinations for retry
        let failed_destinations = self.message_repository.find_pending_retries(10).await?;

        if failed_destinations.is_empty() {
            return Ok(());
        }

        info!("Processing {} failed destinations for retry", failed_destinations.len());

        for destination in failed_destinations {
            let should_retry = self.should_retry_destination(&destination).await?;

            if should_retry {
                if let Err(e) = self.schedule_retry(&destination).await {
                    error!("Failed to schedule retry for destination {}: {}", destination.id, e);
                }
            }
        }

        Ok(())
    }

    async fn should_retry_destination(&self, destination: &MessageDestination) -> Result<bool, Box<dyn std::error::Error>> {
        // Check if retry count exceeds limit
        if destination.retry_count >= 5 {
            warn!("Destination {} exceeded max retry count", destination.id);
            return Ok(false);
        }

        // Check if enough time has passed since last attempt
        if let Some(last_attempt) = destination.last_attempt {
            let retry_delay = self.routing_service.calculate_retry_schedule(destination.retry_count);
            let time_since_last_attempt = chrono::Utc::now() - last_attempt;

            if time_since_last_attempt < retry_delay {
                return Ok(false);
            }
        }

        Ok(true)
    }

    async fn schedule_retry(&self, destination: &MessageDestination) -> Result<(), Box<dyn std::error::Error>> {
        // Update destination for retry
        let mut retry_destination = destination.clone();
        retry_destination.status = DeliveryStatus::RetryScheduled;
        retry_destination.retry_count += 1;
        retry_destination.last_attempt = Some(chrono::Utc::now());

        self.message_repository.update_destination(&retry_destination).await?;

        // Re-queue the message
        self.message_queue.enqueue_message(&retry_destination).await?;

        // Dispatch retry scheduled event
        let event = DomainEvent::MessageRetryScheduled(crate::domain::events::MessageRetryScheduled {
            message_id: destination.message_id,
            destination_id: destination.id,
            retry_count: retry_destination.retry_count,
            scheduled_at: chrono::Utc::now(),
            occurred_at: chrono::Utc::now(),
        });

        self.event_dispatcher.dispatch(&event).await?;

        info!("Scheduled retry {} for destination {}", retry_destination.retry_count, destination.id);

        Ok(())
    }
}