use axum::routing::{get, post};
use axum::Router;

use crate::handlers::message_handler::MessageHandler;
use crate::AppState;

pub fn message_routes() -> Router<AppState> {
    Router::new()
        .route("/messages", post(MessageHandler::send_message))
        .route("/messages/:id/status", get(MessageHandler::get_message_status))
        .route("/destinations/:id/retry", post(MessageHandler::retry_message))
}