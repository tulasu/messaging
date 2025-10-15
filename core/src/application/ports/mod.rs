use crate::domain::{DomainEvent, Message, MessageDestination, MessengerType};
use async_trait::async_trait;
use chrono::Duration;
use redis::RedisError;
use serde_json::Error;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error("Database error: {0}")]
    Database(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Connection error")]
    Connection,
}

#[derive(Debug, Error)]
pub enum EventError {
    #[error("Event dispatch failed: {0}")]
    DispatchFailed(String),
    #[error("Invalid event format")]
    InvalidFormat,
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Redis error: {0}")]
    Redis(String),
}

impl From<Error> for EventError {
    fn from(err: Error) -> Self {
        EventError::Serialization(err.to_string())
    }
}

impl From<RedisError> for EventError {
    fn from(err: RedisError) -> Self {
        EventError::Redis(err.to_string())
    }
}

#[derive(Debug, Error)]
pub enum QueueError {
    #[error("Queue operation failed: {0}")]
    OperationFailed(String),
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    #[error("Queue connection failed")]
    ConnectionFailed,
}

#[async_trait]
pub trait MessageRepository: Send + Sync {
    async fn save(&self, message: &Message) -> Result<(), RepositoryError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Message>, RepositoryError>;
    async fn find_by_destination_id(&self, destination_id: Uuid) -> Result<Option<Message>, RepositoryError>;
    async fn update_destination(
        &self,
        destination: &MessageDestination,
    ) -> Result<(), RepositoryError>;
    async fn find_pending_retries(
        &self,
        limit: u32,
    ) -> Result<Vec<MessageDestination>, RepositoryError>;
    async fn find_destinations_by_message_id(
        &self,
        message_id: Uuid,
    ) -> Result<Vec<MessageDestination>, RepositoryError>;
}

#[async_trait]
pub trait EventDispatcher: Send + Sync {
    async fn dispatch(&self, event: DomainEvent) -> Result<(), EventError>;
    async fn dispatch_batch(&self, events: Vec<DomainEvent>) -> Result<(), EventError>;
}

#[async_trait]
pub trait MessageQueue: Send + Sync {
    async fn enqueue(
        &self,
        message_id: Uuid,
        destination_id: Uuid,
        messenger_type: MessengerType,
    ) -> Result<(), QueueError>;
    async fn dequeue(
        &self,
        messenger_type: MessengerType,
    ) -> Result<Option<(Uuid, Uuid)>, QueueError>;
    async fn requeue_with_delay(
        &self,
        message_id: Uuid,
        destination_id: Uuid,
        messenger_type: MessengerType,
        delay: Duration,
    ) -> Result<(), QueueError>;
}
