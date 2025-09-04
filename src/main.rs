use std::sync::Arc;

use crate::{
    configuration::{Settings, get_configuration},
    database::setup_database_client,
    health::check_health,
};
use axum::{Router, routing::get};
use tiberius::Client;
use tokio::sync::Mutex;
use tokio_util::compat::Compat;
use tracing::info;

mod configuration;
mod database;
mod health;

#[derive(Clone)]
struct AppState {
    pub db_client: Arc<Mutex<Client<Compat<tokio::net::TcpStream>>>>,
    pub config: Settings,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let configuration = get_configuration()?;
    info!(
        "Starting maedic with the following config {:?}",
        configuration
    );

    let listener = tokio::net::TcpListener::bind(format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    ))
    .await
    .expect("Could not bind to port");

    let client = setup_database_client(configuration.database.clone()).await?;
    let state = AppState {
        db_client: Arc::new(Mutex::new(client)),
        config: configuration.clone(),
    };

    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/health", get(check_health))
        .with_state(state);
    info!(
        "Starting maedic on port: {}",
        configuration.application.port
    );

    axum::serve(listener, app).await?;

    Ok(())
}
