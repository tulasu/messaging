use thiserror::Error;

#[derive(Debug, Error)]
pub enum MaxApiError {
    #[error("Authentication error: {0}")]
    Authentication(String),

    #[error("API error: {0}")]
    Api(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Invalid chat ID: {0}")]
    InvalidChatId(String),

    #[error("Serialization error: {0}")]
    Serialization(String),
}
