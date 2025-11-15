use crate::{
    configuration::{AppState, Settings},
    health::{check_health, get_config_handler},
};
use axum::Router;
use axum::routing::get;
use tokio::net::TcpListener;
use tracing::info;

pub async fn run(
    listener: TcpListener,
    state: AppState,
    configuration: Settings,
) -> anyhow::Result<()> {
    let app = Router::new()
        .route("/health", get(check_health))
        .route("/config", get(get_config_handler))
        .with_state(state);
    info!(
        "Starting maedic on port: {}",
        configuration.application.port
    );
    axum::serve(listener, app).await?;
    Ok(())
}
