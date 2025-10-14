use crate::application::ports::{MessageQueue, QueueError};
use crate::domain::MessengerType;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use redis::{AsyncCommands, Client};
use serde_json;
use uuid::Uuid;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct QueueItem {
    message_id: Uuid,
    destination_id: Uuid,
    scheduled_at: Option<DateTime<Utc>>,
}

pub struct RedisMessageQueue {
    client: Client,
}

impl RedisMessageQueue {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    fn get_queue_key(&self, messenger_type: &MessengerType) -> String {
        format!("queue:{}", messenger_type)
    }

    fn get_delayed_queue_key(&self, messenger_type: &MessengerType) -> String {
        format!("queue_delayed:{}", messenger_type)
    }
}

#[async_trait]
impl MessageQueue for RedisMessageQueue {
    async fn enqueue(
        &self,
        message_id: Uuid,
        destination_id: Uuid,
        messenger_type: MessengerType,
    ) -> Result<(), QueueError> {
        let mut conn = self
            .client
            .get_async_connection()
            .await
            .map_err(|_| QueueError::ConnectionFailed)?;

        let queue_item = QueueItem {
            message_id,
            destination_id,
            scheduled_at: None,
        };

        let serialized = serde_json::to_string(&queue_item)
            .map_err(|e| QueueError::OperationFailed(format!("Serialization failed: {}", e)))?;

        let queue_key = self.get_queue_key(&messenger_type);
        conn.lpush::<_, _, ()>(queue_key, serialized)
            .await
            .map_err(|e| QueueError::OperationFailed(format!("Redis lpush failed: {}", e)))?;

        Ok(())
    }

    async fn dequeue(
        &self,
        messenger_type: MessengerType,
    ) -> Result<Option<(Uuid, Uuid)>, QueueError> {
        let mut conn = self
            .client
            .get_async_connection()
            .await
            .map_err(|_| QueueError::ConnectionFailed)?;

        // First check for delayed items that are ready to be processed
        let delayed_queue_key = self.get_delayed_queue_key(&messenger_type);
        let queue_key = self.get_queue_key(&messenger_type);

        // Process delayed items
        loop {
            let delayed_item: Option<String> = conn
                .lrange(&delayed_queue_key, 0, 0)
                .await
                .map_err(|e| QueueError::OperationFailed(format!("Redis lrange failed: {}", e)))?;

            if let Some(serialized) = delayed_item {
                let queue_item: QueueItem = serde_json::from_str(&serialized).map_err(|e| {
                    QueueError::OperationFailed(format!("Deserialization failed: {}", e))
                })?;

                if let Some(scheduled_at) = queue_item.scheduled_at {
                    if scheduled_at <= Utc::now() {
                        // Move to main queue
                        conn.lpop::<_, Option<String>>(&delayed_queue_key, None)
                            .await
                            .map_err(|e| {
                                QueueError::OperationFailed(format!("Redis lpop failed: {}", e))
                            })?;

                        conn.lpush::<_, _, ()>(&queue_key, serialized)
                            .await
                            .map_err(|e| {
                                QueueError::OperationFailed(format!("Redis lpush failed: {}", e))
                            })?;
                        continue;
                    }
                }
                break;
            } else {
                break;
            }
        }

        // Dequeue from main queue
        let serialized: Option<String> = conn
            .rpop(&queue_key, None)
            .await
            .map_err(|e| QueueError::OperationFailed(format!("Redis rpop failed: {}", e)))?;

        match serialized {
            Some(s) => {
                let queue_item: QueueItem = serde_json::from_str(&s).map_err(|e| {
                    QueueError::OperationFailed(format!("Deserialization failed: {}", e))
                })?;

                Ok(Some((queue_item.message_id, queue_item.destination_id)))
            }
            None => Ok(None),
        }
    }

    async fn requeue_with_delay(
        &self,
        message_id: Uuid,
        destination_id: Uuid,
        messenger_type: MessengerType,
        delay: chrono::Duration,
    ) -> Result<(), QueueError> {
        let mut conn = self
            .client
            .get_async_connection()
            .await
            .map_err(|_| QueueError::ConnectionFailed)?;

        let queue_item = QueueItem {
            message_id,
            destination_id,
            scheduled_at: Some(Utc::now() + delay),
        };

        let serialized = serde_json::to_string(&queue_item)
            .map_err(|e| QueueError::OperationFailed(format!("Serialization failed: {}", e)))?;

        let delayed_queue_key = self.get_delayed_queue_key(&messenger_type);
        conn.lpush::<_, _, ()>(delayed_queue_key, serialized)
            .await
            .map_err(|e| QueueError::OperationFailed(format!("Redis lpush failed: {}", e)))?;

        Ok(())
    }
}
