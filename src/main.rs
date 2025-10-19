use std::io::Error;

use poem::{Route, Server, listener::TcpListener};
use poem_openapi::OpenApiService;
use tokio::main;

use crate::{config::Config, presentation::http::endpoints::root::Endpoints};

mod config;
mod presentation;

#[main]
async fn main() -> Result<(), Error> {
    let config = Config::try_parse().map_err(Error::other)?;

    let server_url = format!("{}://{}:{}", config.scheme, config.host, config.port);

    println!("Starting server at {}", server_url);

    let api_service = OpenApiService::new(Endpoints, "Messaging API", "0.1.0")
        .server(format!("{}/api", server_url));
    let ui = api_service.swagger_ui();
    let app = Route::new().nest("/api", api_service).nest("/", ui);

    Server::new(TcpListener::bind(format!("localhost:{}", config.port)))
        .run(app)
        .await
}
