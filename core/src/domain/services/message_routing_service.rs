use crate::domain::ChatId;
use crate::domain::events::{DomainEvent, MessageCreated, MessageQueued};
use crate::domain::models::{Message, MessengerType};
use chrono::{Duration, Utc};

pub trait MessageRoutingService {
    fn route_message(&self, message: &Message) -> Vec<DomainEvent>;
    fn calculate_retry_schedule(&self, retry_count: u32) -> Duration;
}

pub struct DefaultMessageRoutingService;

impl MessageRoutingService for DefaultMessageRoutingService {
    fn route_message(&self, message: &Message) -> Vec<DomainEvent> {
        let mut events = Vec::new();

        // Create MessageCreated event
        let destinations: Vec<(MessengerType, ChatId)> = message
            .destinations
            .iter()
            .map(|dest| (dest.messenger_type.clone(), dest.chat_id.clone()))
            .collect();

        events.push(DomainEvent::MessageCreated(MessageCreated {
            message_id: message.id,
            destinations,
            occurred_at: Utc::now(),
        }));

        // Create MessageQueued events for each destination
        for destination in &message.destinations {
            events.push(DomainEvent::MessageQueued(MessageQueued {
                message_id: message.id,
                destination_id: destination.id,
                messenger_type: destination.messenger_type.clone(),
                occurred_at: Utc::now(),
            }));
        }

        events
    }

    fn calculate_retry_schedule(&self, retry_count: u32) -> Duration {
        // Exponential backoff: 1min, 2min, 4min, 8min, 16min, max 32min
        let minutes = 60 * 2_u64.pow(retry_count.min(4));
        Duration::seconds(minutes as i64)
    }
}
