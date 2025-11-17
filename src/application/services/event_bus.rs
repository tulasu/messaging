use async_trait::async_trait;

use crate::domain::events::OutboundMessageEvent;

#[async_trait]
pub trait MessageBus: Send + Sync {
    async fn publish(&self, event: OutboundMessageEvent) -> anyhow::Result<()>;
}
