use std::sync::Arc;

use poem_openapi::{OpenApi, payload::PlainText};

use crate::presentation::http::endpoints::root::{ApiState, EndpointsTags};

#[derive(Clone)]
pub struct HealthEndpoints {
    state: Arc<ApiState>,
}

impl HealthEndpoints {
    pub fn new(state: Arc<ApiState>) -> Self {
        Self { state }
    }
}

#[OpenApi]
impl HealthEndpoints {
    #[oai(path = "/health", method = "get", tag = EndpointsTags::Health)]
    pub async fn health(&self) -> PlainText<&'static str> {
        let _ = &self.state;
        PlainText("OK")
    }
}
