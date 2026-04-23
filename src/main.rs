use maedic::{
    configuration::get_configuration,
    database::setup_database_pool,
    metrics::setup_metrics_registry,
    run::{AppState, run},
    telemetry::initialize_tracing,
};
use sysinfo::System;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let configuration = get_configuration("base".to_string())?;

    initialize_tracing(
        configuration.application.log_level.clone(),
        configuration.application.logfile_path.clone(),
    )?;

    let listener = tokio::net::TcpListener::bind(format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    ))
    .await
    .expect("Could not bind to port");

    let pool = setup_database_pool(configuration.database.clone())
        .await
        .expect("Could not establish database connection");
    let (registry, metrics) = setup_metrics_registry().await;
    let state = AppState {
        pool,
        config: configuration.clone(),
        sys: System::new_all(),
        registry,
        metrics,
    };

    run(listener, state, configuration)
        .await
        .expect("Failed to build application")
        .await
        .expect("Failed to start application");

    Ok(())
}
