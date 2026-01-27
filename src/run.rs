use std::net::SocketAddr;

use crate::{
    configuration::{AppState, Settings},
    database::self_health,
    health::{check_health, get_config_handler},
};
use axum::routing::get;
use axum::{
    Router,
    extract::{MatchedPath, Request},
};
use tokio::net::TcpListener;
use tower_governor::{GovernorLayer, governor::GovernorConfigBuilder};
use tower_http::trace::TraceLayer;
use tracing::{info, info_span};

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
        .layer(GovernorLayer::new(governor_conf))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &Request<_>| {
                    let request_id = uuid::Uuid::new_v4();
                    let matched_path = request
                        .extensions()
                        .get::<MatchedPath>()
                        .map(MatchedPath::as_str);

                    info_span!(
                        "http_request",
                        method = ?request.method(),
                        matched_path,
                        request_id = tracing::field::display(request_id),
                    )
                })
                .on_failure(()),
        )
        .with_state(state);
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
