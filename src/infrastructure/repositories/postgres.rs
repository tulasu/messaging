use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{FromRow, Pool, Postgres, Row};
use uuid::Uuid;

use crate::domain::{
    models::{
        MessageAttempt, MessageContent, MessageHistoryEntry, MessageStatus, MessageType, MessengerToken,
        MessengerTokenStatus, MessengerType, RequestedBy, User,
    },
    repositories::{MessageHistoryRepository, MessengerTokenRepository, UserRepository},
};

pub type PgPool = Pool<Postgres>;

#[derive(Clone)]
pub struct PostgresUserRepository {
    pool: PgPool,
}

impl PostgresUserRepository {
    pub fn new(pool: PgPool) -> Arc<Self> {
        Arc::new(Self { pool })
    }
}

#[async_trait]
impl UserRepository for PostgresUserRepository {
    async fn find_by_email(&self, email: &str) -> anyhow::Result<Option<User>> {
        let record = sqlx::query_as::<_, UserRecord>(
            r#"SELECT id, email, display_name, created_at, updated_at FROM users WHERE email = $1"#,
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await?;
        Ok(record.map(User::from))
    }

    async fn get(&self, id: &Uuid) -> anyhow::Result<Option<User>> {
        let record = sqlx::query_as::<_, UserRecord>(
            r#"SELECT id, email, display_name, created_at, updated_at FROM users WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(record.map(User::from))
    }

    async fn upsert(&self, user: &User) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            INSERT INTO users (id, email, display_name, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (id) DO UPDATE
            SET email = EXCLUDED.email,
                display_name = EXCLUDED.display_name,
                updated_at = EXCLUDED.updated_at
            "#,
        )
        .bind(user.id)
        .bind(&user.email)
        .bind(&user.display_name)
        .bind(user.created_at)
        .bind(user.updated_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

#[derive(Clone)]
pub struct PostgresMessengerTokenRepository {
    pool: PgPool,
}

impl PostgresMessengerTokenRepository {
    pub fn new(pool: PgPool) -> Arc<Self> {
        Arc::new(Self { pool })
    }
}

#[async_trait]
impl MessengerTokenRepository for PostgresMessengerTokenRepository {
    async fn upsert(&self, mut token: MessengerToken) -> anyhow::Result<MessengerToken> {
        token.updated_at = Utc::now();
        let status = token_status_to_str(token.status);
        let record = sqlx::query_as::<_, MessengerTokenRecord>(
            r#"
            INSERT INTO messenger_tokens (
                id,
                user_id,
                messenger,
                access_token,
                refresh_token,
                status,
                created_at,
                updated_at
            ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8)
            ON CONFLICT (id) DO UPDATE
            SET access_token = EXCLUDED.access_token,
                refresh_token = EXCLUDED.refresh_token,
                status = EXCLUDED.status,
                updated_at = EXCLUDED.updated_at
            RETURNING
                id,
                user_id,
                messenger,
                access_token,
                refresh_token,
                status,
                created_at,
                updated_at
            "#,
        )
        .bind(token.id)
        .bind(token.user_id)
        .bind(token.messenger.as_str())
        .bind(&token.access_token)
        .bind(&token.refresh_token)
        .bind(status)
        .bind(token.created_at)
        .bind(token.updated_at)
        .fetch_one(&self.pool)
        .await?;

        Ok(record.try_into()?)
    }

    async fn find_active(
        &self,
        user_id: &Uuid,
        messenger: MessengerType,
    ) -> anyhow::Result<Option<MessengerToken>> {
        let record = sqlx::query_as::<_, MessengerTokenRecord>(
            r#"
            SELECT id, user_id, messenger, access_token, refresh_token, status, created_at, updated_at
            FROM messenger_tokens
            WHERE user_id = $1
              AND messenger = $2
              AND status = 'active'
            ORDER BY updated_at DESC
            LIMIT 1
            "#,
        )
        .bind(user_id)
        .bind(messenger.as_str())
        .fetch_optional(&self.pool)
        .await?;
        record.map(|record| record.try_into()).transpose()
    }

    async fn list_by_user(&self, user_id: &Uuid) -> anyhow::Result<Vec<MessengerToken>> {
        let rows = sqlx::query_as::<_, MessengerTokenRecord>(
            r#"
            SELECT id, user_id, messenger, access_token, refresh_token, status, created_at, updated_at
            FROM messenger_tokens
            WHERE user_id = $1
            ORDER BY updated_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(|record| record.try_into()).collect()
    }
}

#[derive(Clone)]
pub struct PostgresMessageHistoryRepository {
    pool: PgPool,
}

impl PostgresMessageHistoryRepository {
    pub fn new(pool: PgPool) -> Arc<Self> {
        Arc::new(Self { pool })
    }
}

#[async_trait]
impl MessageHistoryRepository for PostgresMessageHistoryRepository {
    async fn insert(
        &self,
        user_id: Uuid,
        messenger: MessengerType,
        recipient: String,
        content: MessageContent,
        requested_by: RequestedBy,
    ) -> anyhow::Result<MessageHistoryEntry> {
        let id = Uuid::new_v4();
        let status = MessageStatus::Pending;
        let now = Utc::now();
        let (status_str, reason) = message_status_to_fields(&status);
        let requested_by = requested_by_to_str(&requested_by);

        let row = sqlx::query(
            r#"
            INSERT INTO message_history (
                id, user_id, messenger, recipient, body, message_type, status, status_reason,
                attempts, requested_by, created_at, updated_at
            )
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12)
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(user_id)
        .bind(messenger.as_str())
        .bind(&recipient)
        .bind(&content.body)
        .bind(message_type_to_str(&content.message_type))
        .bind(status_str)
        .bind(reason)
        .bind(0_i32)
        .bind(requested_by)
        .bind(now)
        .bind(now)
        .fetch_one(&self.pool)
        .await?;

        MessageHistoryEntry::try_from(row)
    }

    async fn update_status(
        &self,
        message_id: Uuid,
        status: MessageStatus,
        attempts: u32,
    ) -> anyhow::Result<()> {
        let (status_str, reason) = message_status_to_fields(&status);
        sqlx::query(
            r#"
            UPDATE message_history
            SET status = $2,
                status_reason = $3,
                attempts = $4,
                updated_at = $5
            WHERE id = $1
            "#,
        )
        .bind(message_id)
        .bind(status_str)
        .bind(reason)
        .bind(attempts as i32)
        .bind(Utc::now())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn get(&self, message_id: Uuid) -> anyhow::Result<Option<MessageHistoryEntry>> {
        let row = sqlx::query(
            r#"
            SELECT *
            FROM message_history
            WHERE id = $1
            "#,
        )
        .bind(message_id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(MessageHistoryEntry::try_from).transpose()
    }

    async fn list_by_user(
        &self,
        user_id: Uuid,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> anyhow::Result<(Vec<MessageHistoryEntry>, bool)> {
        let limit = limit.unwrap_or(50).min(200) as i32;
        let offset = offset.unwrap_or(0) as i32;

        // Get one extra to check if there are more
        let rows = sqlx::query(
            r#"
            SELECT *
            FROM message_history
            WHERE user_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(user_id)
        .bind(limit + 1)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        let has_more = rows.len() > limit as usize;
        let entries: Vec<MessageHistoryEntry> = rows
            .into_iter()
            .take(limit as usize)
            .map(MessageHistoryEntry::try_from)
            .collect::<Result<Vec<_>, _>>()?;

        Ok((entries, has_more))
    }

    async fn log_attempt(
        &self,
        message_id: Uuid,
        attempt_number: u32,
        status: MessageStatus,
        requested_by: RequestedBy,
    ) -> anyhow::Result<()> {
        let (status_str, reason) = message_status_to_fields(&status);
        sqlx::query(
            r#"
            INSERT INTO message_attempts (
                id, message_id, attempt_number, status, status_reason, requested_by, created_at
            )
            VALUES (gen_random_uuid(), $1, $2, $3, $4, $5, NOW())
            "#,
        )
        .bind(message_id)
        .bind(attempt_number as i32)
        .bind(status_str)
        .bind(reason)
        .bind(requested_by_to_str(&requested_by))
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn get_attempts(&self, message_id: Uuid) -> anyhow::Result<Vec<MessageAttempt>> {
        let rows = sqlx::query(
            r#"
            SELECT id, message_id, attempt_number, status, status_reason, requested_by, created_at
            FROM message_attempts
            WHERE message_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(message_id)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|row| {
                let status_str: String = row.get("status");
                let reason: Option<String> = row.get("status_reason");
                let status = message_status_from_str(&status_str, reason)?;
                let requested_by_str: String = row.get("requested_by");
                let requested_by = requested_by_from_str(&requested_by_str)?;

                Ok(MessageAttempt {
                    id: row.get("id"),
                    message_id: row.get("message_id"),
                    attempt_number: row.get::<i32, _>("attempt_number") as u32,
                    status,
                    requested_by,
                    created_at: row.get("created_at"),
                })
            })
            .collect()
    }
}

#[derive(FromRow)]
struct UserRecord {
    id: Uuid,
    email: String,
    display_name: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<UserRecord> for User {
    fn from(value: UserRecord) -> Self {
        Self {
            id: value.id,
            email: value.email,
            display_name: value.display_name,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

#[derive(FromRow)]
struct MessengerTokenRecord {
    id: Uuid,
    user_id: Uuid,
    messenger: String,
    access_token: String,
    refresh_token: Option<String>,
    status: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl TryFrom<MessengerTokenRecord> for MessengerToken {
    type Error = anyhow::Error;

    fn try_from(value: MessengerTokenRecord) -> Result<Self, Self::Error> {
        let messenger = MessengerType::from_str(&value.messenger)
            .ok_or_else(|| anyhow::anyhow!("unknown messenger {}", value.messenger))?;
        let status = match value.status.as_str() {
            "active" => MessengerTokenStatus::Active,
            "inactive" => MessengerTokenStatus::Inactive,
            other => anyhow::bail!("unknown token status {other}"),
        };
        Ok(Self {
            id: value.id,
            user_id: value.user_id,
            messenger,
            access_token: value.access_token,
            refresh_token: value.refresh_token,
            status,
            created_at: value.created_at,
            updated_at: value.updated_at,
        })
    }
}

impl TryFrom<sqlx::postgres::PgRow> for MessageHistoryEntry {
    type Error = anyhow::Error;

    fn try_from(row: sqlx::postgres::PgRow) -> Result<Self, Self::Error> {
        let messenger_str: String = row.try_get("messenger")?;
        let messenger = MessengerType::from_str(&messenger_str)
            .ok_or_else(|| anyhow::anyhow!("unknown messenger {}", messenger_str))?;
        let message_type = row.try_get::<String, _>("message_type")?;
        let content = MessageContent {
            body: row.try_get("body")?,
            message_type: str_to_message_type(&message_type)?,
        };
        let status_str: String = row.try_get("status")?;
        let status_reason: Option<String> = row.try_get("status_reason")?;
        let attempts: i32 = row.try_get("attempts")?;
        let status = message_status_from_fields(&status_str, status_reason, attempts)?;
        let requested_by_str: String = row.try_get("requested_by")?;
        let requested_by = str_to_requested_by(&requested_by_str)?;

        Ok(MessageHistoryEntry {
            id: row.try_get("id")?,
            user_id: row.try_get("user_id")?,
            messenger,
            recipient: row.try_get("recipient")?,
            content,
            status,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
            attempts: attempts as u32,
            requested_by,
        })
    }
}

fn token_status_to_str(status: MessengerTokenStatus) -> &'static str {
    match status {
        MessengerTokenStatus::Active => "active",
        MessengerTokenStatus::Inactive => "inactive",
    }
}

fn message_type_to_str(message_type: &MessageType) -> &'static str {
    match message_type {
        MessageType::PlainText => "plain_text",
    }
}

fn str_to_message_type(value: &str) -> anyhow::Result<MessageType> {
    match value {
        "plain_text" => Ok(MessageType::PlainText),
        other => anyhow::bail!("unknown message type {other}"),
    }
}

fn requested_by_to_str(value: &RequestedBy) -> &'static str {
    match value {
        RequestedBy::System => "system",
        RequestedBy::User => "user",
    }
}

fn str_to_requested_by(value: &str) -> anyhow::Result<RequestedBy> {
    match value {
        "system" => Ok(RequestedBy::System),
        "user" => Ok(RequestedBy::User),
        other => anyhow::bail!("unknown requested_by {other}"),
    }
}

fn requested_by_from_str(value: &str) -> anyhow::Result<RequestedBy> {
    str_to_requested_by(value)
}

fn message_status_to_fields(status: &MessageStatus) -> (&'static str, Option<String>) {
    match status {
        MessageStatus::Pending => ("pending", None),
        MessageStatus::Scheduled => ("scheduled", None),
        MessageStatus::InFlight => ("in_flight", None),
        MessageStatus::Sent => ("sent", None),
        MessageStatus::Retrying { reason, .. } => ("retrying", Some(reason.clone())),
        MessageStatus::Failed { reason, .. } => ("failed", Some(reason.clone())),
        MessageStatus::Cancelled => ("cancelled", None),
    }
}

fn message_status_from_fields(
    status: &str,
    reason: Option<String>,
    attempts: i32,
) -> anyhow::Result<MessageStatus> {
    Ok(match status {
        "pending" => MessageStatus::Pending,
        "scheduled" => MessageStatus::Scheduled,
        "in_flight" => MessageStatus::InFlight,
        "sent" => MessageStatus::Sent,
        "retrying" => MessageStatus::Retrying {
            reason: reason.unwrap_or_else(|| "retrying".to_string()),
            attempts: attempts as u32,
        },
        "failed" => MessageStatus::Failed {
            reason: reason.unwrap_or_else(|| "failed".to_string()),
            attempts: attempts as u32,
        },
        "cancelled" => MessageStatus::Cancelled,
        other => anyhow::bail!("unknown message status {other}"),
    })
}

fn message_status_from_str(status: &str, reason: Option<String>) -> anyhow::Result<MessageStatus> {
    // For attempts, we use 0 as default since we don't store attempts in message_attempts table
    message_status_from_fields(status, reason, 0)
}
