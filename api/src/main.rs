use std::env::var;
use std::error::Error;
use axum::Router;
use std::sync::Arc;
use redis::Client;
use sqlx::postgres::PgPool;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use messaging::infrastructure::{RedisMessageQueue, PostgresMessageRepository};
use messaging::application::services::MessageService;
use messaging::application::usecases::{SendMessageUseCase, SendMessageUseCaseImpl};
use messaging::infrastructure::MessengerAdapterFactory;
use messaging::domain::services::DefaultMessageRoutingService;

mod handlers;
mod routes;
mod dtos;
mod error;

use routes::message_routes;

#[derive(Clone)]
pub struct AppState {
    pub message_service: Arc<MessageService>,
    pub send_message_use_case: Arc<dyn SendMessageUseCase>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Database connection
    let database_url = var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPool::connect(&database_url).await?;

    // Redis connection
    let redis_url =
        var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());
    let redis_client = Client::open(redis_url)?;

    // Initialize infrastructure components
    let message_repository = Arc::new(PostgresMessageRepository::new(pool.clone()));
    let message_queue = Arc::new(RedisMessageQueue::new(redis_client.clone()));
    let messenger_factory = Arc::new(MessengerAdapterFactory::new());
    let routing_service = Arc::new(DefaultMessageRoutingService);

    // Initialize application services
    let message_service = Arc::new(MessageService::new(
        message_repository.clone(),
        message_queue.clone(),
        messenger_factory.clone(),
    ));

    let send_message_use_case: Arc<dyn SendMessageUseCase> = Arc::new(
        SendMessageUseCaseImpl::new(
            message_repository.clone(),
            message_queue.clone(),
            routing_service.clone(),
        )
    );

    let app_state = AppState {
        message_service: message_service.clone(),
        send_message_use_case: send_message_use_case.clone(),
    };

    // Set up routes
    let app = Router::new()
        .nest("/api/v1", message_routes())
        .route("/health", axum::routing::get(health_check))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive()),
        )
        .with_state(app_state);

    // Run server
    let port = var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()?;

    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;

    tracing::info!("Server listening on port {}", port);
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> &'static str {
    "OK"
}
