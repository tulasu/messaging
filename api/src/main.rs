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

use messaging::infrastructure::RedisMessageQueue;
use messaging::infrastructure::persistence::PostgresMessageRepository;

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

    // Initialize repositories
    let _message_repository = Arc::new(PostgresMessageRepository::new(pool));
    let _message_queue = Arc::new(RedisMessageQueue::new(redis_client));

    // TODO: Set up routes and handlers
    let app = Router::new().layer(
        ServiceBuilder::new()
            .layer(TraceLayer::new_for_http())
            .layer(CorsLayer::permissive()),
    );

    // Run server
    let port = var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()?;

    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;

    tracing::info!("Server listening on port {}", port);
    axum::serve(listener, app).await?;

    Ok(())
}
