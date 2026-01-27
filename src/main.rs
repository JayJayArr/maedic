use maedic::{
    configuration::{AppState, get_configuration},
    database::setup_database_pool,
    run::run,
    telemetry::initialize_tracing,
};
use std::sync::Arc;
use sysinfo::System;
use tokio::sync::Mutex;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let configuration = get_configuration()?;

    initialize_tracing()?;

    info!(
        "Starting maedic version {} with config: {:?}",
        env!("CARGO_PKG_VERSION"),
        configuration
    );

    let listener = tokio::net::TcpListener::bind(format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    ))
    .await
    .expect("Could not bind to port");

    let pool = setup_database_pool(configuration.database.clone()).await?;
    let state = AppState {
        pool,
        config: configuration.clone(),
        sys: Arc::new(Mutex::new(System::new_all())),
    };

    run(listener, state, configuration)
        .await
        .expect("Failed to start application");

    Ok(())
}
