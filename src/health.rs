use crate::{
    configuration::{AppState, DbClient, Settings, SystemState},
    indicators::{ServiceState, SpoolFileCount, SystemHealth},
};
use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde::Serialize;
use sysinfo::Process;
use tracing::error;

#[derive(Serialize, Debug)]
pub struct Health {
    pub hi_queue_size: i32,
    pub unhealthy_spool_files: Vec<SpoolFileCount>,
    pub sysinfo_health: SystemHealth,
}

pub async fn check_health(
    State(state): State<AppState>,
) -> Result<(StatusCode, Json<Health>), HealthError> {
    let hi_queue_size = get_hiqueue_count(state.db_client.clone()).await?;
    let unhealthy_spool_files =
        get_unhealthy_spoolfiles(state.db_client, state.config.limits.spool_file_count).await?;
    let system_health = get_system_health(&state.sys, &state.config).await;

    let health = Health {
        unhealthy_spool_files,
        hi_queue_size,
        sysinfo_health: system_health,
    };
    if !health_is_good(&health, &state.config) {
        error!("App reported unhealthy status {:?}", health);
        Ok((StatusCode::SERVICE_UNAVAILABLE, Json(health)))
    } else {
        Ok((StatusCode::OK, Json(health)))
    }
}

fn health_is_good(health: &Health, config: &Settings) -> bool {
    if health.hi_queue_size > config.limits.hi_queue_count
        || !health.unhealthy_spool_files.is_empty()
        || health.sysinfo_health.used_memory_percentage > config.limits.max_ram_percentage
        || health.sysinfo_health.global_cpu_usage_percentage > config.limits.max_cpu_percentage
        || health.sysinfo_health.service_state != ServiceState::Up
    {
        return false;
    }
    true
}

async fn get_system_health(sys: &SystemState, config: &Settings) -> SystemHealth {
    let mut system = sys.lock().await;
    system.refresh_all();

    let matchin_process_list: Vec<&Process> = system
        .processes_by_name(config.application.service_name.as_ref())
        .collect();
    let service_state = if !matchin_process_list.is_empty() {
        ServiceState::Up
    } else {
        ServiceState::Down
    };

    SystemHealth {
        service_state,
        global_cpu_usage_percentage: (system.global_cpu_usage() * 100.0).ceil() / 100.0,
        used_memory_percentage: ((system.used_memory() as f32 / system.total_memory() as f32)
            * 10000.0)
            .ceil()
            / 100.0,
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

pub async fn get_config_handler(State(state): State<AppState>) -> (StatusCode, Json<Settings>) {
    (StatusCode::OK, Json(state.config))
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
    const GOOD_HEALTH: Health = Health {
        hi_queue_size: 0,
        unhealthy_spool_files: Vec::new(),
        sysinfo_health: SystemHealth {
            service_state: ServiceState::Up,
            global_cpu_usage_percentage: 5.0,
            used_memory_percentage: 5.0,
        },
    };

    #[test]
    fn is_good_with_perfect_health() {
        assert!(health_is_good(&GOOD_HEALTH, &Settings::default()));
    }

    #[test]
    fn should_error_on_service_down() {
        let mut health = GOOD_HEALTH;
        health.sysinfo_health.service_state = ServiceState::Down;
        assert!(!health_is_good(&health, &Settings::default()));
    }

    #[test]
    fn should_error_on_big_hi_queue() {
        let mut health = GOOD_HEALTH;
        health.hi_queue_size = 1001;
        assert!(!health_is_good(&health, &Settings::default()));
    }

    #[test]
    fn should_error_on_unhealthy_spool_files() {
        let mut health = GOOD_HEALTH;
        health.unhealthy_spool_files = vec![SpoolFileCount {
            spool_file_count: 11,
            description: "yeet".to_string(),
            directory: "C:\\Yeet\\ProWatch".to_string(),
        }];
        assert!(!health_is_good(&health, &Settings::default()));
    }

    #[test]
    fn should_error_on_high_cpu_usage() {
        let mut health = GOOD_HEALTH;
        health.sysinfo_health.used_memory_percentage = 81.0;
        assert!(!health_is_good(&health, &Settings::default()));
    }

    #[test]
    fn should_error_on_high_ram_usage() {
        let mut health = GOOD_HEALTH;
        health.sysinfo_health.used_memory_percentage = 81.0;
        assert!(!health_is_good(&health, &Settings::default()));
    }
}
