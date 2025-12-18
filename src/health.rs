use crate::{
    configuration::{AppState, DbClient, LimitSettings, SystemState},
    indicators::{ServiceState, SpoolFileCount},
};
use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde::Serialize;
use sysinfo::Process;
use tracing::error;

#[derive(Serialize, Debug)]
pub struct Health {
    pub hi_queue_size: Option<i32>,
    pub unhealthy_spool_files: Option<Vec<SpoolFileCount>>,
    pub service_state: Option<ServiceState>,
    pub global_cpu_usage_percentage: Option<f32>,
    pub used_memory_percentage: Option<f32>,
}

pub async fn check_health(
    State(state): State<AppState>,
) -> Result<(StatusCode, Json<Health>), HealthError> {
    let limits = state.config.limits;
    // HIQUEUE
    let hi_queue_size = if limits.hi_queue_count == 0 {
        None
    } else {
        Some(get_hiqueue_count(state.db_client.clone()).await?)
    };
    //Spool Files
    let unhealthy_spool_files = if limits.spool_file_count == 0 {
        None
    } else {
        Some(get_unhealthy_spoolfiles(state.db_client, limits.spool_file_count).await?)
    };
    let service_state = if !limits.check_local_service {
        None
    } else {
        Some(check_local_service(&state.sys, &state.config.application.service_name).await)
    };
    // let system_health = get_system_health(&state.sys, &state.config).await;
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

    let health = Health {
        unhealthy_spool_files,
        hi_queue_size,
        service_state,
        global_cpu_usage_percentage,
        used_memory_percentage,
    };
    if !health_is_good(&health, &limits) {
        error!("App reported unhealthy status {:?}", health);
        Ok((StatusCode::SERVICE_UNAVAILABLE, Json(health)))
    } else {
        Ok((StatusCode::OK, Json(health)))
    }
}

fn health_is_good(health: &Health, limits: &LimitSettings) -> bool {
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

// check the local cpu load
async fn get_cpu_load(sys: &SystemState) -> f32 {
    let mut system = sys.lock().await;
    system.refresh_all();
    (system.global_cpu_usage() * 100.0).ceil() / 100.0
}

// check the local RAM usage
async fn get_ram_load(sys: &SystemState) -> f32 {
    let mut system = sys.lock().await;
    system.refresh_all();

    ((system.used_memory() as f32 / system.total_memory() as f32) * 10000.0).ceil() / 100.0
}

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

async fn get_hiqueue_count(client: DbClient) -> Result<i32, HealthError> {
    let mut client = client.lock().await;
    let size = client
        .simple_query("SELECT COUNT(*) as HIQUEUECOUNT FROM HI_QUEUE")
        .await?
        .into_row()
        .await?
        .unwrap()
        .get::<i32, &str>("HIQUEUECOUNT")
        .ok_or(HealthError::Conversion(
            "Failed to convert HIQUEUECOUNT".to_string(),
        ))?;
    Ok(size)
}

async fn get_unhealthy_spoolfiles(
    client: DbClient,
    limit_per_channel: i32,
) -> Result<Vec<SpoolFileCount>, HealthError> {
    let mut client = client.lock().await;
    let queryresult = client
        .query("select DESCRP as description, SPOOl_FILE_COUNT as spool_file_count, SPOOL_DIR as directory from CHANNEL where Installed = 'Y' and SPOOl_FILE_COUNT > @P1", &[&limit_per_channel])
        .await?.into_results().await?;

    let spool_file_counts = queryresult[0]
        .iter()
        .map(|row| SpoolFileCount {
            description: row.get::<&str, &str>("description").unwrap().to_string(),
            spool_file_count: row.get("spool_file_count").unwrap(),
            directory: row.get::<&str, &str>("directory").unwrap().to_string(),
        })
        .collect();

    Ok(spool_file_counts)
}

pub async fn get_config_handler(
    State(state): State<AppState>,
) -> Result<(StatusCode, Json<LimitSettings>), StatusCode> {
    if state.config.application.expose_config {
        Ok((StatusCode::OK, Json(state.config.limits)))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum HealthError {
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),

    #[error(transparent)]
    Database(#[from] tiberius::error::Error),

    #[error("{0}")]
    Conversion(String),
}

impl IntoResponse for HealthError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            Self::Unexpected(err) => {
                tracing::error!("{:?}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Something went wrong".to_owned(),
                )
            }
            Self::Database(err) => {
                tracing::error!("{:?}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Something went wrong with the database queries".to_owned(),
                )
            }
            Self::Conversion(err) => {
                tracing::error!("{:?}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Error when converting a DB value".to_owned(),
                )
            }
        };
        (status, message).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    impl Default for Health {
        fn default() -> Self {
            Self {
                hi_queue_size: Some(0),
                unhealthy_spool_files: Some(Vec::new()),
                service_state: Some(ServiceState::Up),
                global_cpu_usage_percentage: Some(5.0),
                used_memory_percentage: Some(5.0),
            }
        }
    }

    #[test]
    fn is_good_with_perfect_health() {
        assert!(health_is_good(
            &Health::default(),
            &LimitSettings::default()
        ));
    }

    #[test]
    fn should_error_on_service_down() {
        assert!(!health_is_good(
            &Health {
                service_state: Some(ServiceState::Down),
                ..Default::default()
            },
            &LimitSettings::default()
        ));
    }

    #[test]
    fn should_error_on_big_hi_queue() {
        assert!(!health_is_good(
            &Health {
                hi_queue_size: Some(1001),
                ..Default::default()
            },
            &LimitSettings::default()
        ));
    }

    #[test]
    fn should_error_on_unhealthy_spool_files() {
        assert!(!health_is_good(
            &Health {
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
            &Health {
                used_memory_percentage: Some(81.0),
                ..Default::default()
            },
            &LimitSettings::default()
        ));
    }

    #[test]
    fn should_error_on_high_ram_usage() {
        assert!(!health_is_good(
            &Health {
                global_cpu_usage_percentage: Some(81.0),
                ..Default::default()
            },
            &LimitSettings::default()
        ));
    }

    #[rstest]
    #[case(Health {unhealthy_spool_files: None, ..Default::default()})]
    #[case(Health {hi_queue_size: None, ..Default::default()})]
    #[case(Health {service_state: None, ..Default::default()})]
    #[case(Health {global_cpu_usage_percentage: None, ..Default::default()})]
    #[case(Health {used_memory_percentage: None, ..Default::default()})]
    fn ignoring_any_health_checks_yields_healthy_results(#[case] health: Health) {
        assert!(health_is_good(&health, &LimitSettings::default()));
    }
}
