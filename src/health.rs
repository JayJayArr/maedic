use crate::{
    configuration::{LimitSettings, SystemState},
    database::{DatabaseConnectionState, get_db_status, get_table_count, get_unhealthy_spoolfiles},
    error::ApplicationError,
    indicators::{MaedicHealth, PWHealth, ServiceState},
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

#[tracing::instrument(name = "Determine Health Status with gathered parameters", skip_all)]
fn health_is_good(health: &PWHealth, limits: &LimitSettings) -> bool {
    // HI_QUEUE
    if let Some(hi_queue_size) = health.hi_queue_size
        && hi_queue_size > limits.hi_queue_count
    {
        return false;
    };

    // Spool Files
    if let Some(unhealthy_spool_files) = &health.unhealthy_spool_files
        && !unhealthy_spool_files.is_empty()
    {
        return false;
    };

    // Service State
    if let Some(service_state) = &health.service_state
        && service_state != &ServiceState::Up
    {
        return false;
    };

    // CPU
    if let Some(cpu_value) = health.global_cpu_usage_percentage
        && cpu_value > limits.max_cpu_percentage
    {
        return false;
    };

    // RAM
    if let Some(ram_value) = health.used_memory_percentage
        && ram_value > limits.max_ram_percentage
    {
        return false;
    };
    true
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

#[cfg(test)]
mod tests {
    use crate::indicators::SpoolFileCount;

    use super::*;
    use rstest::rstest;
    impl Default for PWHealth {
        fn default() -> Self {
            Self {
                hi_queue_size: Some(0),
                unhealthy_spool_files: Some(Vec::new()),
                service_state: Some(ServiceState::Up),
                global_cpu_usage_percentage: Some(5.0),
                used_memory_percentage: Some(5.0),
                maedic_health: MaedicHealth {
                    database_connection: DatabaseConnectionState::Healthy,
                    version_number: env!("CARGO_PKG_VERSION").to_string(),
                },
            }
        }
    }

    #[test]
    fn is_good_with_perfect_health() {
        assert!(health_is_good(
            &PWHealth::default(),
            &LimitSettings::default()
        ));
    }

    #[test]
    fn should_error_on_service_down() {
        assert!(!health_is_good(
            &PWHealth {
                service_state: Some(ServiceState::Down),
                ..Default::default()
            },
            &LimitSettings::default()
        ));
    }

    #[test]
    fn should_error_on_big_hi_queue() {
        assert!(!health_is_good(
            &PWHealth {
                hi_queue_size: Some(1001),
                ..Default::default()
            },
            &LimitSettings::default()
        ));
    }

    #[test]
    fn should_error_on_unhealthy_spool_files() {
        assert!(!health_is_good(
            &PWHealth {
                unhealthy_spool_files: vec![SpoolFileCount {
                    spool_file_count: 11,
                    description: "yeet".to_string(),
                    directory: "C:\\Yeet\\ProWatch".to_string(),
                }]
                .into(),
                ..Default::default()
            },
            &LimitSettings::default()
        ));
    }

    #[test]
    fn should_error_on_high_cpu_usage() {
        assert!(!health_is_good(
            &PWHealth {
                used_memory_percentage: Some(81.0),
                ..Default::default()
            },
            &LimitSettings::default()
        ));
    }

    #[test]
    fn should_error_on_high_ram_usage() {
        assert!(!health_is_good(
            &PWHealth {
                global_cpu_usage_percentage: Some(81.0),
                ..Default::default()
            },
            &LimitSettings::default()
        ));
    }

    #[rstest]
    #[case(PWHealth {unhealthy_spool_files: None, ..Default::default()})]
    #[case(PWHealth {hi_queue_size: None, ..Default::default()})]
    #[case(PWHealth {service_state: None, ..Default::default()})]
    #[case(PWHealth {global_cpu_usage_percentage: None, ..Default::default()})]
    #[case(PWHealth {used_memory_percentage: None, ..Default::default()})]
    fn ignoring_any_health_checks_yields_healthy_results(#[case] health: PWHealth) {
        assert!(health_is_good(&health, &LimitSettings::default()));
    }
}
