use poem_openapi::{OpenApi, payload::PlainText};

use crate::presentation::http::endpoints::root::{Endpoints, EndpointsTags};

#[OpenApi]
impl Endpoints {
    #[oai(path = "/health", method = "get", tag = EndpointsTags::Health)]
    pub async fn health(&self) -> PlainText<&'static str> {
        PlainText("OK")
    }
}
