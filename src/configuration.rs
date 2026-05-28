use bb8::Pool;
use bb8_tiberius::ConnectionManager;
use config::{Config, ConfigError};
use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use serde_aux::field_attributes::deserialize_number_from_string;

pub fn get_configuration(name: String) -> Result<Settings, ConfigError> {
    let base_path = std::env::current_dir().expect("Failed to determine current directory.");
    let config_directory = base_path.join("configuration");

    Config::builder()
        .add_source(config::File::from(config_directory.join(name)).required(true))
        .build()?
        .try_deserialize()
}

/// `Settings` collects the complete Options provided in the config file
#[derive(Deserialize, Serialize, Clone, Debug, Default)]
pub struct Settings {
    pub application: ApplicationSettings,
    pub database: DatabaseSettings,
    pub limits: LimitSettings,
}

/// Settings for the Application itself
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ApplicationSettings {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
    pub logfile_path: String,
    pub log_level: String,
    pub service_name: String,
    pub expose_config: bool,
}

/// Settings for the Database Connection Pool
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct DatabaseSettings {
    pub hostname: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub auth_method: DBAuthMethod,
    pub username: String,
    #[serde(skip_serializing)]
    pub password: SecretString,
    pub database_name: String,
    pub trust_cert: bool,
}

/// Limits for the `PWHealth` values
/// Each numeric value is the maximum GOOD condition
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct LimitSettings {
    pub hi_queue_count: i32,
    pub spool_file_count: i32,
    pub max_cpu_percentage: f32,
    pub max_ram_percentage: f32,
    pub check_local_service: bool,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub enum DBAuthMethod {
    Basic,
    Windows,
    Integrated,
}

impl Default for DatabaseSettings {
    fn default() -> Self {
        Self {
            port: 1433,
            hostname: "0.0.0.0".to_string(),
            username: "sa".into(),
            auth_method: DBAuthMethod::Basic,
            password: "Charlie 13".into(),
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
            log_level: "info".to_string(),
            service_name: "micserver.exe".into(),
            expose_config: false,
        }
    }
}

pub type DBConnectionPool = Pool<ConnectionManager>;

#[cfg(test)]
mod tests {
    use crate::configuration::get_configuration;
    use rstest::rstest;

    #[rstest]
    #[case("test")]
    #[case("base")]
    fn test_configuration_for_tests_is_a_valid_configuration(#[case] configname: String) {
        let config = get_configuration(configname);
        assert!(config.is_ok());
    }

    #[test]
    fn test_application_exits_on_bad_config() {
        let config = get_configuration("../tests/bad_config.yaml".to_string());
        dbg!(&config);
        assert!(config.is_err());
        match config {
            Ok(_config) => panic!("this config should not be accepted as good"),
            Err(err) => assert_eq!(
                err.to_string(),
                "missing configuration field \"database.trust_cert\"".to_string()
            ),
        }
    }
}
