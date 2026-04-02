use crate::{
    configuration::{LimitSettings, SystemState},
    database::{DatabaseConnectionState, get_db_status, get_table_count, get_unhealthy_spoolfiles},
    error::ApplicationError,
    health::{MaedicHealth, PWHealth, ServiceState, health_is_good},
    run::AppState,
};
use axum::{Json, extract::State, http::StatusCode};
use std::sync::Arc;
use sysinfo::Process;
use tokio::sync::Mutex;
use tracing::warn;

/// Handler to check the Health of PW
#[tracing::instrument(name = "check PW health", skip_all)]
pub async fn check_health(
    State(state): State<Arc<Mutex<AppState>>>,
) -> Result<(StatusCode, Json<PWHealth>), ApplicationError> {
    let state = state.lock().await;
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

#[tracing::instrument(name = "Check CPU load", skip_all)]
async fn get_cpu_load(sys: &SystemState) -> f32 {
    let mut system = sys.lock().await;
    system.refresh_all();
    (system.global_cpu_usage() * 100.0).ceil() / 100.0
}

// check the local RAM usage
#[tracing::instrument(name = "Check RAM load", skip_all)]
async fn get_ram_load(sys: &SystemState) -> f32 {
    let mut system = sys.lock().await;
    system.refresh_all();

    ((system.used_memory() as f32 / system.total_memory() as f32) * 10000.0).ceil() / 100.0
}

#[tracing::instrument(name = "Check if local service is running", skip_all)]
async fn check_local_service(sys: &SystemState, service_name: &String) -> ServiceState {
    let mut system = sys.lock().await;
    system.refresh_all();

    let matchin_process_list: Vec<&Process> =
        system.processes_by_name(service_name.as_ref()).collect();
    if !matchin_process_list.is_empty() {
        ServiceState::Up
    } else {
        ServiceState::Down
    }
}

#[tracing::instrument(name = "Getting exposed config", skip_all)]
pub async fn get_config_handler(
    State(state): State<Arc<Mutex<AppState>>>,
) -> Result<(StatusCode, Json<LimitSettings>), StatusCode> {
    let state = state.lock().await;
    if state.config.application.expose_config {
        Ok((StatusCode::OK, Json(state.config.limits.clone())))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}
