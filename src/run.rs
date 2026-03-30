use crate::{
    configuration::{DBConnectionPool, Settings, SystemState},
    database::self_health,
    health::{check_health, get_config_handler},
    metrics::metrics_handler,
};
use axum::{
    Router,
    extract::{ConnectInfo, MatchedPath, Request, connect_info::IntoMakeServiceWithConnectInfo},
    middleware::AddExtension,
};
use axum::{routing::get, serve::Serve};
use prometheus_client::registry::Registry;
use std::{net::SocketAddr, sync::Arc};
use tokio::{net::TcpListener, sync::Mutex};
use tower_governor::{GovernorLayer, governor::GovernorConfigBuilder};
use tower_http::trace::TraceLayer;
use tracing::{info, info_span};

#[derive(Debug)]
pub struct AppState {
    pub pool: DBConnectionPool,
    pub config: Settings,
    pub sys: SystemState,
    pub registry: Registry,
}

pub async fn run(
    listener: TcpListener,
    state: AppState,
    configuration: Settings,
) -> Result<
    Serve<
        TcpListener,
        IntoMakeServiceWithConnectInfo<Router, SocketAddr>,
        AddExtension<Router, ConnectInfo<SocketAddr>>,
    >,
    anyhow::Error,
> {
    let governor_conf = GovernorConfigBuilder::default()
        .per_second(1)
        .burst_size(5)
        .finish()
        .unwrap();

    let app = Router::new()
        .route("/v1/health", get(check_health))
        .route("/v1/config", get(get_config_handler))
        .route("/v1/self", get(self_health))
        .route("/v1/metrics", get(metrics_handler))
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
        .with_state(Arc::new(Mutex::new(state)));

    info!(
        "Starting maedic version {} with config: {:?}",
        env!("CARGO_PKG_VERSION"),
        configuration
    );

    Ok(axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    ))
}
