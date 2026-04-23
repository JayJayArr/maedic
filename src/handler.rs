use axum::body::Body;
use axum::http::header::CONTENT_TYPE;
use axum::http::{Response, StatusCode};
use axum::response::IntoResponse;
use axum::{Json, extract::State};
use prometheus_client::encoding::text::encode;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::warn;

use crate::metrics::{Endpoint, collect_metrics};
use crate::{
    checks::{check_local_service, get_cpu_load, get_ram_load},
    configuration::LimitSettings,
    database::{DatabaseConnectionState, get_db_status, get_table_count, get_unhealthy_spoolfiles},
    error::ApplicationError,
    health::{MaedicHealth, PWHealth, health_is_good},
    run::AppState,
};

/// Handler to check the Health of PW
#[tracing::instrument(name = "check PW health", skip_all)]
pub async fn check_health(
    State(state): State<Arc<Mutex<AppState>>>,
) -> Result<(StatusCode, Json<PWHealth>), ApplicationError> {
    let mut state = state.lock().await;
    state.metrics.inc_requests(Endpoint::Health);
    let limits = state.config.limits.clone();
    // HI_QUEUE
    let hi_queue_size = if limits.hi_queue_count == 0 {
        None
    } else {
        Some(get_table_count(state.pool.clone(), "hi_queue".to_string()).await?)
    };
    //Spool Files
    let unhealthy_spool_files = if limits.spool_file_count == 0 {
        None
    } else {
        Some(get_unhealthy_spoolfiles(state.pool.clone(), limits.spool_file_count).await?)
    };
    state.sys.refresh_all();
    let service_state = if !limits.check_local_service {
        None
    } else {
        Some(check_local_service(&state.sys, &state.config.application.service_name).await)
    };
    let global_cpu_usage_percentage = if limits.max_cpu_percentage == 0.0 {
        None
    } else {
        Some(get_cpu_load(&state.sys).await)
    };

    let used_memory_percentage = if limits.max_ram_percentage == 0.0 {
        None
    } else {
        Some(get_ram_load(&state.sys).await)
    };

    let maedic_health = match get_db_status(state.pool.clone()).await {
        Ok(state) => match state {
            DatabaseConnectionState::Healthy => MaedicHealth::healthy(),
            DatabaseConnectionState::Unhealthy => MaedicHealth::unhealthy(),
        },
        Err(_) => MaedicHealth::unhealthy(),
    };

    let health = PWHealth {
        unhealthy_spool_files,
        hi_queue_size,
        service_state,
        global_cpu_usage_percentage,
        used_memory_percentage,
        maedic_health,
    };
    if !health_is_good(&health, &limits) {
        warn!("App reported unhealthy status {:?}", health);
        Ok((StatusCode::SERVICE_UNAVAILABLE, Json(health)))
    } else {
        Ok((StatusCode::OK, Json(health)))
    }
}

/// Exposing the `LimitSettings` for the health check endpoint
#[tracing::instrument(name = "Getting exposed config", skip_all)]
pub async fn get_config_handler(
    State(state): State<Arc<Mutex<AppState>>>,
) -> Result<(StatusCode, Json<LimitSettings>), StatusCode> {
    let state = state.lock().await;
    state.metrics.inc_requests(Endpoint::Config);
    if state.config.application.expose_config {
        Ok((StatusCode::OK, Json(state.config.limits.clone())))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

/// Exposing Prometheus style metrics collected from the database
#[tracing::instrument(name = "Scrape metrics", skip(state))]
pub async fn metrics_handler(State(state): State<Arc<Mutex<AppState>>>) -> impl IntoResponse {
    let state = state.lock().await;
    state.metrics.inc_requests(Endpoint::Metrics);
    collect_metrics(state.pool.clone(), &state.metrics)
        .await
        .unwrap();
    let mut buffer = String::new();
    encode(&mut buffer, &state.registry).unwrap();
    Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, "text/plain; version=0.0.4; charset=utf-8")
        .body(Body::from(buffer))
        .unwrap()
}
