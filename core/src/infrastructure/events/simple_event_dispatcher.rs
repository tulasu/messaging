use crate::application::ports::{EventDispatcher, EventError};
use crate::domain::DomainEvent;
use async_trait::async_trait;
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::Mutex;

pub type EventHandler = Arc<
    dyn Fn(DomainEvent) -> Pin<Box<dyn Future<Output = Result<(), EventError>> + Send>>
        + Send
        + Sync,
>;

pub struct SimpleEventDispatcher {
    handlers: Arc<Mutex<HashMap<String, Vec<EventHandler>>>>,
}

impl SimpleEventDispatcher {
    pub fn new() -> Self {
        Self {
            handlers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn register_handler<F, Fut>(&self, event_type: &str, handler: F)
    where
        F: Fn(DomainEvent) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<(), EventError>> + Send + 'static,
    {
        let handler = Arc::new(move |event: DomainEvent| {
            Box::pin(handler(event)) as Pin<Box<dyn Future<Output = Result<(), EventError>> + Send>>
        });

        let mut handlers = self.handlers.blocking_lock();
        handlers
            .entry(event_type.to_string())
            .or_insert_with(Vec::new)
            .push(handler);
    }
}

impl Default for SimpleEventDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl EventDispatcher for SimpleEventDispatcher {
    async fn dispatch(&self, event: DomainEvent) -> Result<(), EventError> {
        let event_type = match &event {
            DomainEvent::MessageCreated(_) => "MessageCreated",
            DomainEvent::MessageQueued(_) => "MessageQueued",
            DomainEvent::MessageProcessing(_) => "MessageProcessing",
            DomainEvent::MessageSent(_) => "MessageSent",
            DomainEvent::MessageFailed(_) => "MessageFailed",
            DomainEvent::MessageRetryScheduled(_) => "MessageRetryScheduled",
        };

        let handlers = self.handlers.lock().await;
        if let Some(event_handlers) = handlers.get(event_type) {
            let mut results = Vec::new();

            for handler in event_handlers {
                let result = handler(event.clone()).await;
                results.push(result);
            }

            // Check if any handler failed
            for result in results {
                if let Err(e) = result {
                    return Err(e);
                }
            }
        }

        Ok(())
    }

    async fn dispatch_batch(&self, events: Vec<DomainEvent>) -> Result<(), EventError> {
        for event in events {
            self.dispatch(event).await?;
        }
        Ok(())
    }
}
