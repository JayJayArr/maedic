use config::Config;
use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use serde_aux::field_attributes::deserialize_number_from_string;
use std::sync::Arc;
use sysinfo::System;
use tiberius::Client;
use tokio::sync::Mutex;
use tokio_util::compat::Compat;

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let base_path = std::env::current_dir().expect("Failed to determine current directory.");
    let config_directory = base_path;

    Config::builder()
        .add_source(config::File::from(config_directory.join("base")).required(true))
        .build()?
        .try_deserialize()
}

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
pub struct Settings {
    pub application: ApplicationSettings,
    pub database: DatabaseSettings,
    pub limits: LimitSettings,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ApplicationSettings {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
    pub logfile_path: String,
    pub service_name: String,
    pub expose_config: bool,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct DatabaseSettings {
    pub host: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub username: String,
    #[serde(skip_serializing)]
    pub password: SecretString,
    pub database_name: String,
    pub trust_cert: bool,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct LimitSettings {
    pub hi_queue_count: i32,
    pub spool_file_count: i32,
    pub max_cpu_percentage: f32,
    pub max_ram_percentage: f32,
    pub check_local_service: bool,
}

impl Default for DatabaseSettings {
    fn default() -> Self {
        Self {
            port: 1433,
            host: "0.0.0.0".to_string(),
            username: "sa".into(),
            password: "Charlie".into(),
            database_name: "PWNT".into(),
            trust_cert: true,
        }
    }
}

impl Default for LimitSettings {
    fn default() -> Self {
        Self {
            hi_queue_count: 1000,
            spool_file_count: 10,
            max_cpu_percentage: 80.0,
            max_ram_percentage: 80.0,
            check_local_service: false,
        }
    }
}
impl Default for ApplicationSettings {
    fn default() -> Self {
        Self {
            port: 3000,
            host: "0.0.0.0".into(),
            logfile_path: "./maedic.log".into(),
            service_name: "micserver.exe".into(),
            expose_config: false,
        }
    }
}

pub type DbClient = Arc<Mutex<Client<Compat<tokio::net::TcpStream>>>>;
pub type SystemState = Arc<Mutex<System>>;

#[derive(Clone)]
pub struct AppState {
    pub db_client: DbClient,
    pub config: Settings,
    pub sys: SystemState,
}
