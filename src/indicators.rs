use serde::Serialize;

#[derive(Serialize, Clone, Debug)]
pub struct SystemHealth {
    pub service_state: ServiceState,
    pub global_cpu_usage_percentage: f32,
    pub used_memory_percentage: f32,
}

#[derive(Serialize, Debug)]
pub struct SpoolFileCount {
    pub spool_file_count: i32,
    pub description: String,
    pub directory: String,
}

impl Into<SpoolFileCount> for tiberius::Row {
    fn into(self) -> SpoolFileCount {
        return SpoolFileCount {
            description: self.get::<&str, &str>("description").unwrap().to_string(),
            spool_file_count: self.get("spool_file_count").unwrap(),
            directory: self.get::<&str, &str>("directory").unwrap().to_string(),
        };
    }
}

#[derive(Serialize, Clone, Debug, PartialEq)]
pub enum ServiceState {
    Up,
    Down,
}
