use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;

use crate::domain::models::{MessageContent, MessengerChat, MessengerToken, MessengerType};

#[derive(Debug, Clone, Copy)]
pub struct PaginationParams {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            limit: Some(50),
            offset: Some(0),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PaginatedChats {
    pub chats: Vec<MessengerChat>,
    pub has_more: bool,
    pub next_offset: Option<u32>,
}

#[async_trait]
pub trait MessengerClient: Send + Sync {
    fn messenger(&self) -> MessengerType;
    async fn send(
        &self,
        token: &MessengerToken,
        recipient: &str,
        content: &MessageContent,
    ) -> anyhow::Result<()>;
    async fn list_chats(
        &self,
        token: &MessengerToken,
        pagination: PaginationParams,
    ) -> anyhow::Result<PaginatedChats>;
}

#[derive(Clone)]
pub struct MessengerGateway {
    clients: HashMap<MessengerType, Arc<dyn MessengerClient>>,
}

impl MessengerGateway {
    pub fn new(clients: Vec<Arc<dyn MessengerClient>>) -> Self {
        let mut map = HashMap::new();
        for client in clients {
            map.insert(client.messenger(), client);
        }
        Self { clients: map }
    }

    pub fn get(&self, messenger: MessengerType) -> Option<Arc<dyn MessengerClient>> {
        self.clients.get(&messenger).cloned()
    }
}
