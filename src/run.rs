use crate::{
    configuration::{AppState, Settings},
    database::self_health,
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
        .route("/v1/health", get(check_health))
        .route("/v1/config", get(get_config_handler))
        .route("/v1/self", get(self_health))
        .with_state(state);
    info!(
        "Starting maedic version {} on port: {}",
        env!("CARGO_PKG_VERSION"),
        configuration.application.port
    );
    axum::serve(listener, app).await?;
    Ok(())
}
