use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::messenger::MessengerType;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessengerToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub messenger: MessengerType,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub status: MessengerTokenStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum MessengerTokenStatus {
    Active,
    Inactive,
}
