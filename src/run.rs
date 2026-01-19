use std::net::SocketAddr;

use crate::{
    configuration::{AppState, Settings},
    database::self_health,
    health::{check_health, get_config_handler},
};
use axum::Router;
use axum::routing::get;
use tokio::net::TcpListener;
use tower_governor::{GovernorLayer, governor::GovernorConfigBuilder};
use tracing::info;

pub async fn run(
    listener: TcpListener,
    state: AppState,
    configuration: Settings,
) -> anyhow::Result<()> {
    let governor_conf = GovernorConfigBuilder::default()
        .per_second(1)
        .burst_size(5)
        .finish()
        .unwrap();

    let app = Router::new()
        .route("/v1/health", get(check_health))
        .route("/v1/config", get(get_config_handler))
        .route("/v1/self", get(self_health))
        .with_state(state)
        .layer(GovernorLayer::new(governor_conf));
    info!(
        "Starting maedic version {} on port: {}",
        env!("CARGO_PKG_VERSION"),
        configuration.application.port
    );
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;
    Ok(())
}
