use axum::http::StatusCode;
use axum::response::IntoResponse;
use prometheus_client::encoding::EncodeLabelSet;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

use crate::{configuration::LimitSettings, database::DatabaseConnectionState};

/// Health components of the connected PW instance
///
/// Featuring checks for:
/// - The HI_QUEUE (a builtin task queue)
/// - Spool Files (unfinished downloads to the hardware)
/// - Service_State (the status of the PW Windows Service)
/// - Checks for CPU and RAM usage
/// - Health of Maedic itself, checking the DB Connection
#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct PWHealth {
    pub maedic_health: MaedicHealth,
    pub service_state: Option<ServiceState>,
    pub global_cpu_usage_percentage: Option<f32>,
    pub used_memory_percentage: Option<f32>,
    pub hi_queue_size: Option<i32>,
    pub unhealthy_spool_files: Option<Vec<SpoolFileCount>>,
}

/// The Health of Maedic itself
/// Checks for a healthy Database connection
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct MaedicHealth {
    pub database_connection: DatabaseConnectionState,
    pub version_number: String,
}

/// Default values for MaedicHealth
impl MaedicHealth {
    pub fn healthy() -> Self {
        Self {
            database_connection: DatabaseConnectionState::Healthy,
            version_number: env!("CARGO_PKG_VERSION").to_string(),
        }
    }

    pub fn unhealthy() -> Self {
        Self {
            database_connection: DatabaseConnectionState::Unhealthy,
            version_number: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

impl IntoResponse for MaedicHealth {
    fn into_response(self) -> axum::response::Response {
        match self.database_connection {
            DatabaseConnectionState::Healthy => (StatusCode::OK, self.to_string()).into_response(),
            DatabaseConnectionState::Unhealthy => {
                (StatusCode::SERVICE_UNAVAILABLE, self.to_string()).into_response()
            }
        }
    }
}

impl Display for MaedicHealth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.database_connection {
            DatabaseConnectionState::Healthy => write!(f, "database_connection: healthy"),
            DatabaseConnectionState::Unhealthy => write!(f, "database_connection: unhealthy"),
        }
    }
}

/// Health of the underlying Operating System
#[derive(Serialize, Clone, Debug)]
pub struct SystemHealth {
    pub service_state: ServiceState,
    pub global_cpu_usage_percentage: f32,
    pub used_memory_percentage: f32,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct SpoolFileCount {
    pub spool_file_count: i32,
    pub description: String,
    pub directory: String,
}

impl From<tiberius::Row> for SpoolFileCount {
    fn from(val: tiberius::Row) -> Self {
        SpoolFileCount {
            description: val.get::<&str, &str>("description").unwrap().to_string(),
            spool_file_count: val.get("spool_file_count").unwrap(),
            directory: val.get::<&str, &str>("directory").unwrap().to_string(),
        }
    }
}

impl From<&tiberius::Row> for SpoolFileCount {
    fn from(val: &tiberius::Row) -> Self {
        SpoolFileCount {
            description: val.get::<&str, &str>("description").unwrap().to_string(),
            spool_file_count: val.get("spool_file_count").unwrap(),
            directory: val.get::<&str, &str>("directory").unwrap().to_string(),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct HiQueueCount {
    pub hi_queue_count: i32,
    pub description: String,
}

impl From<tiberius::Row> for HiQueueCount {
    fn from(val: tiberius::Row) -> Self {
        HiQueueCount {
            description: val.get::<&str, &str>("description").unwrap().to_string(),
            hi_queue_count: val.get("hi_queue_count").unwrap(),
        }
    }
}

impl From<&tiberius::Row> for HiQueueCount {
    fn from(val: &tiberius::Row) -> Self {
        HiQueueCount {
            description: val.get::<&str, &str>("description").unwrap().to_string(),
            hi_queue_count: val.get("hi_queue_count").unwrap(),
        }
    }
}

#[derive(Default, Deserialize, Serialize, Debug, PartialEq, EncodeLabelSet, Eq, Hash, Clone)]
pub struct PanelInstalled {
    pub description: String,
    pub firmware_major_version: i64,
    pub firmware_minor_version: i64,
    pub installed: i64,
}

impl From<tiberius::Row> for PanelInstalled {
    fn from(val: tiberius::Row) -> Self {
        let split: Vec<&str> = val
            .get::<&str, &str>("firmware_version")
            .unwrap()
            .split_terminator(".")
            .collect();
        PanelInstalled {
            description: val.get::<&str, &str>("description").unwrap().to_string(),
            installed: if val.get::<&str, &str>("installed").unwrap() == "Y" {
                1
            } else {
                0
            },
            firmware_major_version: split[0].parse::<i64>().unwrap(),
            firmware_minor_version: split[1].parse::<i64>().unwrap(),
        }
    }
}

impl From<&tiberius::Row> for PanelInstalled {
    fn from(val: &tiberius::Row) -> Self {
        let split: Vec<&str> = val
            .get::<&str, &str>("firmware_version")
            .unwrap()
            .split_terminator(".")
            .collect();
        PanelInstalled {
            description: val.get::<&str, &str>("description").unwrap().to_string(),
            installed: if val.get::<&str, &str>("installed").unwrap() == "Y" {
                1
            } else {
                0
            },
            firmware_major_version: split[0].parse::<i64>().unwrap(),
            firmware_minor_version: split[1].parse::<i64>().unwrap(),
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub enum ServiceState {
    Up,
    Down,
}

#[tracing::instrument(name = "Determine Health Status with gathered parameters", skip_all)]
pub fn health_is_good(health: &PWHealth, limits: &LimitSettings) -> bool {
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

#[cfg(test)]
mod tests {
    use crate::health::SpoolFileCount;

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
