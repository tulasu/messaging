use crate::application::ports::{EventDispatcher, EventError};
use crate::domain::DomainEvent;
use async_trait::async_trait;
use futures::StreamExt;
use redis::{AsyncCommands, Client};
use serde_json;
use tracing::{error, info};

pub struct RedisEventDispatcher {
    client: Client,
}

impl RedisEventDispatcher {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    fn get_channel_name(event: &DomainEvent) -> String {
        match event {
            DomainEvent::MessageCreated(_) => "events:message.created".to_string(),
            DomainEvent::MessageQueued(_) => "events:message.queued".to_string(),
            DomainEvent::MessageProcessing(_) => "events:message.processing".to_string(),
            DomainEvent::MessageSent(_) => "events:message.sent".to_string(),
            DomainEvent::MessageFailed(_) => "events:message.failed".to_string(),
            DomainEvent::MessageRetryScheduled(_) => "events:message.retry_scheduled".to_string(),
        }
    }
}

#[async_trait]
impl EventDispatcher for RedisEventDispatcher {
    async fn dispatch(&self, event: DomainEvent) -> Result<(), EventError> {
        let channel_name = Self::get_channel_name(&event);
        let event_json = serde_json::to_string(&event)?;

        let mut conn = self.client.get_async_connection().await?;

        let _: () = conn
            .publish(&channel_name, &event_json)
            .await
            .map_err(|e| EventError::DispatchFailed(e.to_string()))?;

        info!("Dispatched event to channel: {}", channel_name);

        Ok(())
    }

    async fn dispatch_batch(&self, events: Vec<DomainEvent>) -> Result<(), EventError> {
        for event in events {
            self.dispatch(event).await?;
        }
        Ok(())
    }
}

pub struct RedisEventSubscriber {
    client: Client,
}

impl RedisEventSubscriber {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub async fn subscribe<F>(
        &self,
        channel_pattern: &str,
        callback: F,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        F: Fn(DomainEvent) + Send + Sync + 'static,
    {
        let mut pubsub = self.client.get_async_connection().await?.into_pubsub();

        pubsub.subscribe(channel_pattern).await?;

        info!("Subscribed to channel pattern: {}", channel_pattern);

        let mut stream = pubsub.on_message();

        while let Some(msg) = stream.next().await {
            if let Ok(payload) = msg.get_payload::<String>() {
                match serde_json::from_str::<DomainEvent>(&payload) {
                    Ok(event) => {
                        callback(event);
                    }
                    Err(e) => {
                        error!("Failed to parse event: {}", e);
                    }
                }
            }
        }
        Ok(())
    }
}
