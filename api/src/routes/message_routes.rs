use axum::routing::{get, post};
use axum::Router;
use crate::handlers::message_handler::MessageHandler;
use crate::application::ports::MessageRepository;
use std::sync::Arc;

pub fn message_routes<R: MessageRepository + 'static>(
    handler: Arc<MessageHandler<R>>,
) -> Router {
    Router::new()
        .route("/messages", post(MessageHandler::<R>::send_message))
        .route("/messages", get(MessageHandler::<R>::get_messages))
        .route("/messages/:id", get(MessageHandler::<R>::get_message_status))
        .route("/messages/:id/retry", post(MessageHandler::<R>::retry_message))
        .with_state(handler)
}