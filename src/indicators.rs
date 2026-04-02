use axum::http::StatusCode;
use axum::response::IntoResponse;
use prometheus_client::encoding::EncodeLabelSet;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

use crate::database::DatabaseConnectionState;

/// Health components of the connected PW instance
///
/// Featuring checks for:
/// - The HI_QUEUE (a builtin task queue)
/// - Spool Files (unfinished downloads to the hardware)
/// - Service_State (the status of the PW Windows Service)
/// - Checks for CPU and RAM usage
#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct PWHealth {
    pub hi_queue_size: Option<i32>,
    pub unhealthy_spool_files: Option<Vec<SpoolFileCount>>,
    pub service_state: Option<ServiceState>,
    pub global_cpu_usage_percentage: Option<f32>,
    pub used_memory_percentage: Option<f32>,
    pub maedic_health: MaedicHealth,
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
