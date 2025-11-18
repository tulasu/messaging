use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::models::{
    MessageAttempt, MessageContent, MessageHistoryEntry, MessageStatus, MessengerToken, MessengerType, RequestedBy,
    User,
};

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_email(&self, email: &str) -> anyhow::Result<Option<User>>;
    async fn get(&self, id: &Uuid) -> anyhow::Result<Option<User>>;
    async fn upsert(&self, user: &User) -> anyhow::Result<()>;
}

#[async_trait]
pub trait MessengerTokenRepository: Send + Sync {
    async fn upsert(&self, token: MessengerToken) -> anyhow::Result<MessengerToken>;
    async fn find_active(
        &self,
        user_id: &Uuid,
        messenger: MessengerType,
    ) -> anyhow::Result<Option<MessengerToken>>;
    async fn list_by_user(&self, user_id: &Uuid) -> anyhow::Result<Vec<MessengerToken>>;
}

#[async_trait]
pub trait MessageHistoryRepository: Send + Sync {
    async fn insert(
        &self,
        user_id: Uuid,
        messenger: MessengerType,
        recipient: String,
        content: MessageContent,
        requested_by: RequestedBy,
    ) -> anyhow::Result<MessageHistoryEntry>;

    async fn update_status(
        &self,
        message_id: Uuid,
        status: MessageStatus,
        attempts: u32,
    ) -> anyhow::Result<()>;

    async fn get(&self, message_id: Uuid) -> anyhow::Result<Option<MessageHistoryEntry>>;

    async fn list_by_user(
        &self,
        user_id: Uuid,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> anyhow::Result<(Vec<MessageHistoryEntry>, bool)>;

    async fn log_attempt(
        &self,
        message_id: Uuid,
        attempt_number: u32,
        status: MessageStatus,
        requested_by: RequestedBy,
    ) -> anyhow::Result<()>;

    async fn get_attempts(&self, message_id: Uuid) -> anyhow::Result<Vec<MessageAttempt>>;
}
