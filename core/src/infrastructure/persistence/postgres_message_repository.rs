use crate::application::ports::{MessageRepository, RepositoryError};
use crate::domain::{
    ChatId, DeliveryStatus, Message, MessageDestination, MessengerType, Payload, TextFormat,
};
use async_trait::async_trait;
use serde_json;
use sqlx::{PgPool, Row};
use uuid::Uuid;

pub struct PostgresMessageRepository {
    pool: PgPool,
}

impl PostgresMessageRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl MessageRepository for PostgresMessageRepository {
    async fn save(&self, message: &Message) -> Result<(), RepositoryError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        // Insert message
        let payload_type = match &message.payload {
            Payload::Plain { .. } => "plain",
            Payload::Formatted { format, .. } => match format {
                TextFormat::Plain => "plain",
                TextFormat::Markdown => "markdown",
                TextFormat::Html => "html",
            },
        };

        let payload_data = serde_json::to_value(&message.payload)
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        sqlx::query("INSERT INTO messages (id, payload_type, payload_data) VALUES ($1, $2, $3)")
            .bind(message.id)
            .bind(payload_type)
            .bind(payload_data)
            .execute(&mut *tx)
            .await
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        // Insert destinations
        for destination in &message.destinations {
            sqlx::query("INSERT INTO message_destinations (id, message_id, messenger_type, chat_id, status, retry_count) VALUES ($1, $2, $3, $4, $5, $6)")
                .bind(destination.id)
                .bind(destination.message_id)
                .bind(destination.messenger_type.to_string())
                .bind(destination.chat_id.as_str())
                .bind(serde_json::to_value(&destination.status).unwrap())
                .bind(destination.retry_count as i32)
                .execute(&mut *tx)
                .await
                .map_err(|e| RepositoryError::Database(e.to_string()))?;
        }

        tx.commit()
            .await
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Message>, RepositoryError> {
        let message_row = sqlx::query(
            "SELECT id, payload_type, payload_data, created_at FROM messages WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        let message_row = match message_row {
            Some(row) => row,
            None => return Ok(None),
        };

        // Parse payload
        let payload: Payload = match message_row.try_get::<&str, _>("payload_type").unwrap_or("") {
            "plain" => Payload::Plain {
                content: message_row
                    .try_get::<serde_json::Value, _>("payload_data")
                    .map_err(|e| {
                        RepositoryError::Database(format!("Failed to parse plain payload: {}", e))
                    })?
                    .as_str()
                    .unwrap_or("")
                    .to_string(),
            },
            "formatted" => {
                let formatted_data: serde_json::Value = message_row
                    .try_get::<serde_json::Value, _>("payload_data")
                    .map_err(|e| {
                        RepositoryError::Database(format!("Failed to get payload data: {}", e))
                    })?;

                Payload::Formatted {
                    content: formatted_data["content"].as_str().unwrap_or("").to_string(),
                    format: match formatted_data["format"].as_str().unwrap_or("plain") {
                        "markdown" => TextFormat::Markdown,
                        "html" => TextFormat::Html,
                        _ => TextFormat::Plain,
                    },
                }
            }
            _ => {
                return Err(RepositoryError::Database(format!(
                    "Unknown payload type: {}",
                    message_row.try_get::<&str, _>("payload_type").unwrap_or("")
                )));
            }
        };

        // Fetch destinations
        let destination_rows = sqlx::query("SELECT id, messenger_type, chat_id, status, retry_count, last_attempt, sent_at, error_message FROM message_destinations WHERE message_id = $1 ORDER BY created_at")
            .bind(id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        let mut destinations = Vec::new();
        for row in destination_rows {
            let messenger_type = match row.try_get::<&str, _>("messenger_type").unwrap_or("") {
                "telegram" => MessengerType::Telegram,
                "vk" => MessengerType::VK,
                "max" => MessengerType::MAX,
                _ => {
                    return Err(RepositoryError::Database(format!(
                        "Unknown messenger type: {}",
                        row.try_get::<&str, _>("messenger_type").unwrap_or("")
                    )));
                }
            };

            let status_json: serde_json::Value = row
                .try_get::<serde_json::Value, _>("status")
                .map_err(|e| RepositoryError::Database(format!("Failed to get status: {}", e)))?;
            let status: DeliveryStatus = serde_json::from_value(status_json)
                .map_err(|e| RepositoryError::Database(format!("Failed to parse status: {}", e)))?;

            let chat_id_string: String = row
                .try_get::<_, _>("chat_id")
                .map_err(|e| RepositoryError::Database(format!("Failed to get chat_id: {}", e)))?;
            let chat_id = ChatId::new(chat_id_string)
                .map_err(|e| RepositoryError::Database(format!("Invalid chat ID: {}", e)))?;

            destinations.push(MessageDestination {
                id: row
                    .try_get::<_, _>("id")
                    .map_err(|e| RepositoryError::Database(format!("Failed to get id: {}", e)))?,
                message_id: row.try_get::<_, _>("message_id").map_err(|e| {
                    RepositoryError::Database(format!("Failed to get message_id: {}", e))
                })?,
                messenger_type,
                chat_id,
                status,
                retry_count: row.try_get::<i32, _>("retry_count").map_err(|e| {
                    RepositoryError::Database(format!("Failed to get retry_count: {}", e))
                })? as u32,
                last_attempt: row
                    .try_get::<Option<chrono::DateTime<chrono::Utc>>, _>("last_attempt")
                    .map_err(|e| {
                        RepositoryError::Database(format!("Failed to get last_attempt: {}", e))
                    })?,
                sent_at: row
                    .try_get::<Option<chrono::DateTime<chrono::Utc>>, _>("sent_at")
                    .map_err(|e| {
                        RepositoryError::Database(format!("Failed to get sent_at: {}", e))
                    })?,
                error_message: row
                    .try_get::<Option<String>, _>("error_message")
                    .map_err(|e| {
                        RepositoryError::Database(format!("Failed to get error_message: {}", e))
                    })?,
            });
        }

        Ok(Some(Message {
            id: message_row.try_get::<_, _>("id").map_err(|e| {
                RepositoryError::Database(format!("Failed to get message id: {}", e))
            })?,
            payload,
            destinations,
            created_at: message_row.try_get::<_, _>("created_at").map_err(|e| {
                RepositoryError::Database(format!("Failed to get created_at: {}", e))
            })?,
        }))
    }

    async fn update_destination(
        &self,
        destination: &MessageDestination,
    ) -> Result<(), RepositoryError> {
        sqlx::query("UPDATE message_destinations SET status = $1, retry_count = $2, last_attempt = $3, sent_at = $4, error_message = $5, updated_at = NOW() WHERE id = $6")
            .bind(serde_json::to_value(&destination.status).unwrap())
            .bind(destination.retry_count as i32)
            .bind(destination.last_attempt)
            .bind(destination.sent_at)
            .bind(destination.error_message.as_ref())
            .bind(destination.id)
            .execute(&self.pool)
            .await
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(())
    }

    async fn find_pending_retries(
        &self,
        limit: u32,
    ) -> Result<Vec<MessageDestination>, RepositoryError> {
        let rows = sqlx::query("SELECT id, message_id, messenger_type, chat_id, status, retry_count, last_attempt, sent_at, error_message FROM message_destinations WHERE status = 'failed' AND retry_count < 5 ORDER BY last_attempt ASC LIMIT $1")
            .bind(limit as i32)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        let mut destinations = Vec::new();
        for row in rows {
            let messenger_type = match row.try_get::<&str, _>("messenger_type").unwrap_or("") {
                "telegram" => MessengerType::Telegram,
                "vk" => MessengerType::VK,
                "max" => MessengerType::MAX,
                _ => {
                    return Err(RepositoryError::Database(format!(
                        "Unknown messenger type: {}",
                        row.try_get::<&str, _>("messenger_type").unwrap_or("")
                    )));
                }
            };

            let status_json: serde_json::Value = row
                .try_get::<serde_json::Value, _>("status")
                .map_err(|e| RepositoryError::Database(format!("Failed to get status: {}", e)))?;
            let status: DeliveryStatus = serde_json::from_value(status_json)
                .map_err(|e| RepositoryError::Database(format!("Failed to parse status: {}", e)))?;

            let chat_id_string: String = row
                .try_get::<_, _>("chat_id")
                .map_err(|e| RepositoryError::Database(format!("Failed to get chat_id: {}", e)))?;
            let chat_id = ChatId::new(chat_id_string)
                .map_err(|e| RepositoryError::Database(format!("Invalid chat ID: {}", e)))?;

            destinations.push(MessageDestination {
                id: row
                    .try_get::<_, _>("id")
                    .map_err(|e| RepositoryError::Database(format!("Failed to get id: {}", e)))?,
                message_id: row.try_get::<_, _>("message_id").map_err(|e| {
                    RepositoryError::Database(format!("Failed to get message_id: {}", e))
                })?,
                messenger_type,
                chat_id,
                status,
                retry_count: row.try_get::<i32, _>("retry_count").map_err(|e| {
                    RepositoryError::Database(format!("Failed to get retry_count: {}", e))
                })? as u32,
                last_attempt: row
                    .try_get::<Option<chrono::DateTime<chrono::Utc>>, _>("last_attempt")
                    .map_err(|e| {
                        RepositoryError::Database(format!("Failed to get last_attempt: {}", e))
                    })?,
                sent_at: row
                    .try_get::<Option<chrono::DateTime<chrono::Utc>>, _>("sent_at")
                    .map_err(|e| {
                        RepositoryError::Database(format!("Failed to get sent_at: {}", e))
                    })?,
                error_message: row
                    .try_get::<Option<String>, _>("error_message")
                    .map_err(|e| {
                        RepositoryError::Database(format!("Failed to get error_message: {}", e))
                    })?,
            });
        }

        Ok(destinations)
    }

    async fn find_destinations_by_message_id(
        &self,
        message_id: Uuid,
    ) -> Result<Vec<MessageDestination>, RepositoryError> {
        let rows = sqlx::query("SELECT id, messenger_type, chat_id, status, retry_count, last_attempt, sent_at, error_message FROM message_destinations WHERE message_id = $1 ORDER BY created_at")
            .bind(message_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        let mut destinations = Vec::new();
        for row in rows {
            let messenger_type = match row.try_get::<&str, _>("messenger_type").unwrap_or("") {
                "telegram" => MessengerType::Telegram,
                "vk" => MessengerType::VK,
                "max" => MessengerType::MAX,
                _ => {
                    return Err(RepositoryError::Database(format!(
                        "Unknown messenger type: {}",
                        row.try_get::<&str, _>("messenger_type").unwrap_or("")
                    )));
                }
            };

            let status_json: serde_json::Value = row
                .try_get::<serde_json::Value, _>("status")
                .map_err(|e| RepositoryError::Database(format!("Failed to get status: {}", e)))?;
            let status: DeliveryStatus = serde_json::from_value(status_json)
                .map_err(|e| RepositoryError::Database(format!("Failed to parse status: {}", e)))?;

            let chat_id_string: String = row
                .try_get::<_, _>("chat_id")
                .map_err(|e| RepositoryError::Database(format!("Failed to get chat_id: {}", e)))?;
            let chat_id = ChatId::new(chat_id_string)
                .map_err(|e| RepositoryError::Database(format!("Invalid chat ID: {}", e)))?;

            destinations.push(MessageDestination {
                id: row
                    .try_get::<_, _>("id")
                    .map_err(|e| RepositoryError::Database(format!("Failed to get id: {}", e)))?,
                message_id: row.try_get::<_, _>("message_id").map_err(|e| {
                    RepositoryError::Database(format!("Failed to get message_id: {}", e))
                })?,
                messenger_type,
                chat_id,
                status,
                retry_count: row.try_get::<i32, _>("retry_count").map_err(|e| {
                    RepositoryError::Database(format!("Failed to get retry_count: {}", e))
                })? as u32,
                last_attempt: row
                    .try_get::<Option<chrono::DateTime<chrono::Utc>>, _>("last_attempt")
                    .map_err(|e| {
                        RepositoryError::Database(format!("Failed to get last_attempt: {}", e))
                    })?,
                sent_at: row
                    .try_get::<Option<chrono::DateTime<chrono::Utc>>, _>("sent_at")
                    .map_err(|e| {
                        RepositoryError::Database(format!("Failed to get sent_at: {}", e))
                    })?,
                error_message: row
                    .try_get::<Option<String>, _>("error_message")
                    .map_err(|e| {
                        RepositoryError::Database(format!("Failed to get error_message: {}", e))
                    })?,
            });
        }

        Ok(destinations)
    }
}
