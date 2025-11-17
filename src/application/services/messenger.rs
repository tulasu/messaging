use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;

use crate::domain::models::{MessageContent, MessengerToken, MessengerType};

#[async_trait]
pub trait MessengerClient: Send + Sync {
    fn messenger(&self) -> MessengerType;
    async fn send(
        &self,
        token: &MessengerToken,
        recipient: &str,
        content: &MessageContent,
    ) -> anyhow::Result<()>;
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
