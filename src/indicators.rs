use serde::{Deserialize, Serialize};

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

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub enum ServiceState {
    Up,
    Down,
}
