use std::sync::Arc;
use std::time::Duration;

use async_nats::jetstream::{
    self,
    consumer::{AckPolicy, PullConsumer, pull},
};
use tokio::task::JoinHandle;
use tokio_stream::StreamExt;

use crate::{
    application::{
        handlers::message_dispatcher::MessageDispatchHandler, services::event_bus::MessageBus,
    },
    domain::events::OutboundMessageEvent,
};

#[derive(Clone)]
pub struct JetstreamConfig {
    pub url: String,
    pub stream: String,
    pub subject: String,
    pub durable: String,
    pub pull_batch: usize,
    pub ack_wait_seconds: u64,
    pub max_deliver: i64,
}

pub struct JetstreamBus {
    context: jetstream::Context,
    subject: String,
}

impl JetstreamBus {
    pub async fn new(config: &JetstreamConfig) -> anyhow::Result<(Arc<Self>, JetstreamWorker)> {
        let client = async_nats::connect(&config.url).await?;
        let context = jetstream::new(client);

        let stream = context
            .get_or_create_stream(jetstream::stream::Config {
                name: config.stream.clone(),
                subjects: vec![config.subject.clone()],
                ..Default::default()
            })
            .await?;

        let consumer = stream
            .get_or_create_consumer(
                &config.durable,
                pull::Config {
                    durable_name: Some(config.durable.clone()),
                    ack_policy: AckPolicy::Explicit,
                    ack_wait: Duration::from_secs(config.ack_wait_seconds),
                    max_deliver: config.max_deliver,
                    ..Default::default()
                },
            )
            .await?;

        let bus = Arc::new(Self {
            context: context.clone(),
            subject: config.subject.clone(),
        });

        let worker = JetstreamWorker {
            consumer,
            pull_batch: config.pull_batch,
        };

        Ok((bus, worker))
    }
}

#[async_trait::async_trait]
impl MessageBus for JetstreamBus {
    async fn publish(&self, event: OutboundMessageEvent) -> anyhow::Result<()> {
        let payload = serde_json::to_vec(&event)?;
        self.context
            .publish(self.subject.clone(), payload.into())
            .await?;
        Ok(())
    }
}

pub struct JetstreamWorker {
    consumer: PullConsumer,
    pull_batch: usize,
}

impl JetstreamWorker {
    pub fn spawn(
        self,
        handler: Arc<MessageDispatchHandler>,
        bus: Arc<JetstreamBus>,
    ) -> JoinHandle<()> {
        tokio::spawn(async move {
            if let Err(err) = self.run(handler, bus).await {
                eprintln!("jetstream worker stopped: {err:?}");
            }
        })
    }

    async fn run(
        self,
        handler: Arc<MessageDispatchHandler>,
        bus: Arc<JetstreamBus>,
    ) -> anyhow::Result<()> {
        loop {
            let mut batch = self
                .consumer
                .batch()
                .max_messages(self.pull_batch)
                .messages()
                .await?;
            while let Some(message) = batch.next().await {
                match message {
                    Ok(msg) => {
                        if let Err(err) =
                            Self::process_message(msg, handler.clone(), bus.clone()).await
                        {
                            eprintln!("failed to process message: {err:?}");
                        }
                    }
                    Err(err) => {
                        eprintln!("jetstream batch error: {err:?}");
                    }
                }
            }
        }
    }

    async fn process_message(
        message: jetstream::Message,
        handler: Arc<MessageDispatchHandler>,
        bus: Arc<JetstreamBus>,
    ) -> anyhow::Result<()> {
        let event: OutboundMessageEvent = serde_json::from_slice(&message.payload)?;
        match handler.handle(event.clone()).await {
            Ok(_) => {
                if let Err(e) = message.ack().await {
                    return Err(anyhow::anyhow!("failed to ack message: {}", e));
                }
            }
            Err(err) => {
                if event.attempt >= event.max_attempts {
                    if let Err(e) = message.ack().await {
                        return Err(anyhow::anyhow!("failed to ack message: {}", e));
                    }
                } else {
                    let mut next = event;
                    next.attempt += 1;
                    bus.publish(next).await?;
                    if let Err(e) = message.ack().await {
                        return Err(anyhow::anyhow!("failed to ack message: {}", e));
                    }
                }
                eprintln!("dispatcher error: {err:?}");
            }
        }
        Ok(())
    }
}
