use config::Config;
use secrecy::SecretString;
use serde::Deserialize;
use serde_aux::field_attributes::deserialize_number_from_string;

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let base_path = std::env::current_dir().expect("Failed to determine current directory.");
    let config_directory = base_path;

    Config::builder()
        .add_source(config::File::from(config_directory.join("base")).required(true))
        .build()?
        .try_deserialize()
}

#[derive(Deserialize, Clone, Debug)]
pub struct Settings {
    pub application: ApplicationSettings,
    pub database: DatabaseSettings,
    pub limits: LimitSettings,
}

#[derive(Deserialize, Clone, Debug)]
pub struct ApplicationSettings {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
    pub logfile_path: String,
    pub service_name: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct DatabaseSettings {
    pub host: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub username: String,
    pub password: SecretString,
    pub database_name: String,
    pub trust_cert: bool,
}

#[derive(Deserialize, Clone, Debug)]
pub struct LimitSettings {
    pub hi_queue_count: i32,
    pub spool_file_count: i32,
    pub max_cpu_percentage: f32,
    pub max_ram_percentage: f32,
}
