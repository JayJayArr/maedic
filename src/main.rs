use std::{fs::OpenOptions, sync::Arc};

use crate::{
    configuration::{Settings, get_configuration},
    database::setup_database_client,
    health::check_health,
};
use axum::{Router, routing::get};
use tiberius::Client;
use tokio::sync::Mutex;
use tokio_util::compat::Compat;
use tracing::{Level, info};
use tracing_subscriber::{Layer, Registry, filter, fmt, layer::SubscriberExt};

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
    let configuration = get_configuration()?;
    let logfile = OpenOptions::new()
        .append(true)
        .create(true)
        .open(configuration.application.logfile_path.clone())
        .expect("could not create log file");

    let subscriber = Registry::default()
        //default stdout logger
        .with(
            fmt::layer()
                .with_ansi(true)
                .with_filter(filter::LevelFilter::from_level(Level::DEBUG)),
        )
        //logging to file
        .with(
            fmt::layer()
                .json()
                .with_writer(logfile)
                .with_ansi(true)
                .with_filter(filter::LevelFilter::from_level(Level::DEBUG)),
        );

    tracing::subscriber::set_global_default(subscriber).unwrap();

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
