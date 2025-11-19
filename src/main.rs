use std::io::Error;
use std::sync::Arc;
use std::time::Duration;

use poem::{EndpointExt, Route, Server, listener::TcpListener, middleware::CookieJarManager};
use poem_openapi::OpenApiService;
use tokio::main;

use crate::{
    application::{
        handlers::message_dispatcher::MessageDispatchHandler,
        services::{event_bus::MessageBus, jwt::JwtServiceConfig, messenger::MessengerGateway},
        usecases::{
            authenticate_user::AuthenticateUserUseCase,
            get_message_attempts::GetMessageAttemptsUseCase,
            list_chats::ListChatsUseCase,
            list_messages::ListMessagesUseCase,
            list_tokens::ListTokensUseCase,
            register_token::RegisterTokenUseCase,
            retry_message::RetryMessageUseCase,
            schedule_message::{ScheduleMessageConfig, ScheduleMessageUseCase},
        },
    },
    config::Config,
    domain::repositories::{MessageHistoryRepository, MessengerTokenRepository, UserRepository},
    infrastructure::{
        messaging::{
            jetstream::{JetstreamBus, JetstreamConfig},
            telegram::TelegramClient,
            vk::VkClient,
        },
        repositories::postgres::{
            PostgresMessageHistoryRepository, PostgresMessengerTokenRepository,
            PostgresUserRepository,
        },
    },
    presentation::http::endpoints::{
        auth::AuthEndpoints, chats::ChatsEndpoints, health::HealthEndpoints,
        messages::MessagesEndpoints, root::ApiState, tokens::TokensEndpoints,
    },
};
use sqlx::postgres::PgPoolOptions;

mod application;
mod config;
mod domain;
mod infrastructure;
mod presentation;

#[main]
async fn main() -> Result<(), Error> {
    let config = Config::try_parse().map_err(Error::other)?;

    let pool = PgPoolOptions::new()
        .max_connections(config.database_max_connections)
        .connect(&config.database_url)
        .await
        .map_err(Error::other)?;
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .map_err(Error::other)?;

    // infrastructure
    let user_repo: Arc<dyn UserRepository> = PostgresUserRepository::new(pool.clone());
    let token_repo: Arc<dyn MessengerTokenRepository> =
        PostgresMessengerTokenRepository::new(pool.clone());
    let history_repo: Arc<dyn MessageHistoryRepository> =
        PostgresMessageHistoryRepository::new(pool.clone());

    let messenger_gateway = MessengerGateway::new(vec![TelegramClient::new(), VkClient::new()]);

    let jwt_config = JwtServiceConfig {
        secret: config.jwt_secret.clone(),
        expiration: Duration::from_secs(config.jwt_ttl_seconds),
        refresh_expiration: Duration::from_secs(config.jwt_refresh_ttl_seconds),
    };

    let schedule_config = ScheduleMessageConfig {
        max_attempts: config.system_retry_limit,
    };

    let (bus_impl, worker) = JetstreamBus::new(&JetstreamConfig {
        url: config.nats_url.clone(),
        stream: config.nats_stream.clone(),
        subject: config.nats_subject.clone(),
        durable: config.nats_durable.clone(),
        pull_batch: config.nats_pull_batch,
        ack_wait_seconds: config.nats_ack_wait_seconds,
        max_deliver: config.nats_max_deliver,
    })
    .await
    .map_err(Error::other)?;

    let bus: Arc<dyn MessageBus> = bus_impl.clone();

    // use-cases
    let auth_usecase = Arc::new(AuthenticateUserUseCase::new(
        user_repo.clone(),
        jwt_config.clone(),
    ));
    let register_token_usecase = Arc::new(RegisterTokenUseCase::new(token_repo.clone()));
    let list_tokens_usecase = Arc::new(ListTokensUseCase::new(token_repo.clone()));
    let list_chats_usecase = Arc::new(ListChatsUseCase::new(
        token_repo.clone(),
        messenger_gateway.clone(),
    ));
    let schedule_message_usecase = Arc::new(ScheduleMessageUseCase::new(
        token_repo.clone(),
        history_repo.clone(),
        bus.clone(),
        schedule_config,
    ));
    let list_messages_usecase = Arc::new(ListMessagesUseCase::new(history_repo.clone()));
    let retry_message_usecase = Arc::new(RetryMessageUseCase::new(
        history_repo.clone(),
        schedule_message_usecase.clone(),
    ));
    let get_message_attempts_usecase =
        Arc::new(GetMessageAttemptsUseCase::new(history_repo.clone()));

    let dispatcher = Arc::new(MessageDispatchHandler::new(
        token_repo,
        history_repo.clone(),
        messenger_gateway.clone(),
    ));
    let _worker_handle = worker.spawn(dispatcher, bus_impl);

    let api_state = Arc::new(ApiState {
        auth_usecase,
        register_token_usecase,
        list_tokens_usecase,
        list_chats_usecase,
        schedule_message_usecase,
        list_messages_usecase,
        retry_message_usecase,
        get_message_attempts_usecase,
        jwt_config,
    });

    let server_url = format!("{}://{}:{}", config.scheme, config.host, config.port);

    println!("Starting server at {}", server_url);

    let apis = (
        HealthEndpoints::new(api_state.clone()),
        AuthEndpoints::new(api_state.clone()),
        TokensEndpoints::new(api_state.clone()),
        MessagesEndpoints::new(api_state.clone()),
        ChatsEndpoints::new(api_state.clone()),
    );

    let api_service =
        OpenApiService::new(apis, "Messaging API", "0.1.0").server(format!("{}/api", server_url));
    let ui = api_service.swagger_ui();
    let route = Route::new().nest("/api", api_service).nest("/", ui);
    let app = route.with(CookieJarManager::new());

    Server::new(TcpListener::bind(format!("0.0.0.0:{}", config.port)))
        .run(app)
        .await
}
