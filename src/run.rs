use std::{net::SocketAddr, sync::Arc};

use crate::{
    configuration::{AppState, Settings},
    database::{self_health, setup_database_pool},
    health::{check_health, get_config_handler},
};
use axum::{
    Router,
    extract::{ConnectInfo, MatchedPath, Request, connect_info::IntoMakeServiceWithConnectInfo},
    middleware::AddExtension,
};
use axum::{routing::get, serve::Serve};
use sysinfo::System;
use tokio::{net::TcpListener, sync::Mutex};
use tower_governor::{GovernorLayer, governor::GovernorConfigBuilder};
use tower_http::trace::TraceLayer;
use tracing::{info, info_span};

pub struct Application {
    port: u16,
    server: Serve<
        TcpListener,
        IntoMakeServiceWithConnectInfo<Router, SocketAddr>,
        AddExtension<Router, ConnectInfo<SocketAddr>>,
    >,
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
    Ok(axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    ))
}

impl Application {
    pub async fn build(configuration: Settings) -> Result<Self, anyhow::Error> {
        let connection_pool = setup_database_pool(configuration.database.clone())
            .await
            .unwrap();
        let listener = TcpListener::bind(format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        ))
        .await
        .expect("could not bind port");
        let port = listener.local_addr().unwrap().port();
        info!(
            "Starting app on {:?}:{:?}",
            configuration.application.host, configuration.application.port
        );
        //Start the application
        let server = run(
            listener,
            AppState {
                pool: connection_pool,
                config: configuration.clone(),
                sys: Arc::new(Mutex::new(System::new_all())),
            },
            configuration,
        )
        .await?;
        Ok(Self { port, server })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}
