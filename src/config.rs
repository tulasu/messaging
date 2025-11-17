use std::env::var;

use dotenvy::dotenv;

pub struct Config {
    pub port: u16,
    pub scheme: String,
    pub host: String,
    pub jwt_secret: String,
    pub jwt_ttl_seconds: u64,
    pub nats_url: String,
    pub nats_stream: String,
    pub nats_subject: String,
    pub nats_durable: String,
    pub nats_pull_batch: usize,
    pub nats_ack_wait_seconds: u64,
    pub nats_max_deliver: i64,
    pub system_retry_limit: u32,
}

impl Config {
    pub fn try_parse() -> Result<Config, &'static str> {
        let _ = dotenv();

        Ok(Config {
            port: read_var("PORT")?.parse::<u16>().map_err(|_| "invalid PORT")?,
            scheme: read_var("SCHEME")?,
            host: read_var("HOST")?,
            jwt_secret: read_var("JWT_SECRET")?,
            jwt_ttl_seconds: read_var("JWT_TTL_SECONDS")?
                .parse::<u64>()
                .map_err(|_| "invalid JWT_TTL_SECONDS")?,
            nats_url: read_var("NATS_URL")?,
            nats_stream: read_var_or_default("NATS_STREAM", "MESSAGING"),
            nats_subject: read_var_or_default("NATS_SUBJECT", "messaging.outbound"),
            nats_durable: read_var_or_default("NATS_DURABLE", "messaging-worker"),
            nats_pull_batch: read_var_or_default("NATS_PULL_BATCH", "32")
                .parse::<usize>()
                .map_err(|_| "invalid NATS_PULL_BATCH")?,
            nats_ack_wait_seconds: read_var_or_default("NATS_ACK_WAIT_SECONDS", "30")
                .parse::<u64>()
                .map_err(|_| "invalid NATS_ACK_WAIT_SECONDS")?,
            nats_max_deliver: read_var_or_default("NATS_MAX_DELIVER", "10")
                .parse::<i64>()
                .map_err(|_| "invalid NATS_MAX_DELIVER")?,
            system_retry_limit: read_var_or_default("SYSTEM_RETRY_LIMIT", "3")
                .parse::<u32>()
                .map_err(|_| "invalid SYSTEM_RETRY_LIMIT")?,
        })
    }
}

fn read_var(name: &str) -> Result<String, &'static str> {
    var(name).map_err(|_| "failed to read env var")
}

fn read_var_or_default(name: &str, default: &str) -> String {
    var(name).unwrap_or_else(|_| default.to_string())
}
