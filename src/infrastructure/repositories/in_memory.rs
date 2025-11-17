use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::domain::{
    models::{
        MessageContent, MessageHistoryEntry, MessageStatus, MessengerToken, MessengerTokenStatus,
        MessengerType, RequestedBy, User,
    },
    repositories::{MessageHistoryRepository, MessengerTokenRepository, UserRepository},
};

#[derive(Default)]
pub struct InMemoryUserRepository {
    users: Arc<RwLock<HashMap<Uuid, User>>>,
}

impl InMemoryUserRepository {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl UserRepository for InMemoryUserRepository {
    async fn find_by_email(&self, email: &str) -> anyhow::Result<Option<User>> {
        let users = self.users.read().await;
        Ok(users.values().find(|u| u.email == email).cloned())
    }

    async fn get(&self, id: &Uuid) -> anyhow::Result<Option<User>> {
        let users = self.users.read().await;
        Ok(users.get(id).cloned())
    }

    async fn upsert(&self, user: &User) -> anyhow::Result<()> {
        let mut users = self.users.write().await;
        users.insert(user.id, user.clone());
        Ok(())
    }
}

#[derive(Default)]
pub struct InMemoryMessengerTokenRepository {
    tokens: Arc<RwLock<HashMap<Uuid, MessengerToken>>>,
}

impl InMemoryMessengerTokenRepository {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl MessengerTokenRepository for InMemoryMessengerTokenRepository {
    async fn upsert(&self, mut token: MessengerToken) -> anyhow::Result<MessengerToken> {
        token.updated_at = Utc::now();
        let mut tokens = self.tokens.write().await;

        // deactivate previous active token for the same messenger/user
        for existing in tokens.values_mut() {
            if existing.user_id == token.user_id
                && existing.messenger == token.messenger
                && existing.status == MessengerTokenStatus::Active
            {
                existing.status = MessengerTokenStatus::Inactive;
                existing.updated_at = Utc::now();
            }
        }

        tokens.insert(token.id, token.clone());
        Ok(token)
    }

    async fn find_active(
        &self,
        user_id: &Uuid,
        messenger: MessengerType,
    ) -> anyhow::Result<Option<MessengerToken>> {
        let tokens = self.tokens.read().await;
        Ok(tokens
            .values()
            .find(|t| {
                t.user_id == *user_id
                    && t.messenger == messenger
                    && t.status == MessengerTokenStatus::Active
            })
            .cloned())
    }

    async fn list_by_user(&self, user_id: &Uuid) -> anyhow::Result<Vec<MessengerToken>> {
        let tokens = self.tokens.read().await;
        Ok(tokens
            .values()
            .filter(|t| &t.user_id == user_id)
            .cloned()
            .collect())
    }
}

#[derive(Default)]
pub struct InMemoryMessageHistoryRepository {
    messages: Arc<RwLock<HashMap<Uuid, MessageHistoryEntry>>>,
}

impl InMemoryMessageHistoryRepository {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl MessageHistoryRepository for InMemoryMessageHistoryRepository {
    async fn insert(
        &self,
        user_id: Uuid,
        messenger: MessengerType,
        recipient: String,
        content: MessageContent,
        requested_by: RequestedBy,
    ) -> anyhow::Result<MessageHistoryEntry> {
        let now = Utc::now();
        let entry = MessageHistoryEntry {
            id: Uuid::new_v4(),
            user_id,
            messenger,
            recipient,
            content,
            status: MessageStatus::Pending,
            created_at: now,
            updated_at: now,
            attempts: 0,
            requested_by,
        };
        let mut messages = self.messages.write().await;
        messages.insert(entry.id, entry.clone());
        Ok(entry)
    }

    async fn update_status(
        &self,
        message_id: Uuid,
        status: MessageStatus,
        attempts: u32,
    ) -> anyhow::Result<()> {
        let mut messages = self.messages.write().await;
        if let Some(entry) = messages.get_mut(&message_id) {
            entry.status = status;
            entry.updated_at = Utc::now();
            entry.attempts = attempts;
        }
        Ok(())
    }

    async fn get(&self, message_id: Uuid) -> anyhow::Result<Option<MessageHistoryEntry>> {
        let messages = self.messages.read().await;
        Ok(messages.get(&message_id).cloned())
    }

    async fn list_by_user(&self, user_id: Uuid) -> anyhow::Result<Vec<MessageHistoryEntry>> {
        let messages = self.messages.read().await;
        Ok(messages
            .values()
            .filter(|m| m.user_id == user_id)
            .cloned()
            .collect())
    }
}
