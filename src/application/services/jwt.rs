use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::Context;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::models::User;

#[derive(Clone)]
pub struct JwtServiceConfig {
    pub secret: String,
    pub expiration: Duration,
}

#[derive(Clone)]
pub struct JwtService {
    encoding: EncodingKey,
    decoding: DecodingKey,
    validation: Validation,
    config: JwtServiceConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub email: String,
    pub exp: usize,
    pub iat: usize,
}

impl JwtService {
    pub fn new(config: JwtServiceConfig) -> Self {
        let validation = Validation::default();
        let encoding = EncodingKey::from_secret(config.secret.as_bytes());
        let decoding = DecodingKey::from_secret(config.secret.as_bytes());

        Self {
            encoding,
            decoding,
            validation,
            config,
        }
    }

    pub fn issue(&self, user: &User) -> anyhow::Result<String> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .context("failed to calculate current timestamp")?;
        let exp = now + self.config.expiration;
        let claims = Claims {
            sub: user.id,
            email: user.email.clone(),
            exp: exp.as_secs() as usize,
            iat: now.as_secs() as usize,
        };

        jsonwebtoken::encode(&Header::default(), &claims, &self.encoding)
            .context("failed to encode JWT")
    }

    pub fn verify(&self, token: &str) -> anyhow::Result<Claims> {
        jsonwebtoken::decode::<Claims>(token, &self.decoding, &self.validation)
            .map(|data| data.claims)
            .context("failed to verify JWT")
    }
}

