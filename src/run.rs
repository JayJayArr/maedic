use crate::{
    configuration::{DBConnectionPool, Settings},
    handler::{
        check_health, get_config_handler, handle_timeout_error, handler_404, metrics_handler,
    },
    metrics::Metrics,
};
use axum::{
    Router,
    error_handling::HandleErrorLayer,
    extract::{ConnectInfo, DefaultBodyLimit, connect_info::IntoMakeServiceWithConnectInfo},
    middleware::AddExtension,
};
use axum::{routing::get, serve::Serve};
use prometheus_client::registry::Registry;
use std::{net::SocketAddr, sync::Arc, time::Duration};
use sysinfo::System;
use tokio::{net::TcpListener, sync::Mutex};
use tower::ServiceBuilder;
use tower_governor::{GovernorLayer, governor::GovernorConfigBuilder};
use tower_http::{
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, RequestId, SetRequestIdLayer},
    trace::TraceLayer,
};
use tracing::{info, info_span};

const MAX_BODY_BYTES: usize = 1024 * 1024;
const REQUEST_BODY_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(16);

/// The current running state of the Application
#[derive(Debug)]
pub struct AppState {
    pub pool: DBConnectionPool,
    pub settings: Settings,
    pub sys: System,
    pub registry: Registry,
    pub metrics: Metrics,
}

/// Start the Application with specific `Settings` and `AppState`
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
        .per_second(configuration.application.rate_limit_per_second)
        .burst_size(configuration.application.rate_limit_burst)
        .finish()
        .expect("Failed to create RateLimiter Settings");

    let app = Router::new()
        .route("/v1/health", get(check_health))
        .route("/v1/config", get(get_config_handler))
        .route("/metrics", get(metrics_handler))
        .fallback(handler_404)
        .layer(DefaultBodyLimit::max(MAX_BODY_BYTES))
        .layer(tower_http::timeout::RequestBodyTimeoutLayer::new(
            REQUEST_BODY_TIMEOUT,
        ))
        .layer(tower_http::catch_panic::CatchPanicLayer::new())
        .layer(GovernorLayer::new(governor_conf))
        .with_state(Arc::new(Mutex::new(state)))
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(handle_timeout_error))
                .timeout(Duration::from_secs(
                    configuration.application.request_time_limit_seconds,
                )),
        )
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|req: &axum::extract::Request| {
                    let request_id = req
                        .extensions()
                        .get::<RequestId>()
                        .and_then(|id| id.header_value().to_str().ok())
                        .unwrap_or_default();
                    info_span!("request",
                        %request_id,
                        method= %req.method(),
                        path = %req.uri().path()
                    )
                })
                .on_request(())
                .on_response(())
                .on_failure(()),
        )
        .layer(PropagateRequestIdLayer::x_request_id())
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid));

    info!(
        "Starting maedic version {} with config: {:#?}",
        env!("CARGO_PKG_VERSION"),
        configuration
    );

    Ok(axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    ))
}
