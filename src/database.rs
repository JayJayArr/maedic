use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use bb8::Pool;
use bb8_tiberius::ConnectionManager;
use secrecy::ExposeSecret;
use serde::Serialize;
use std::fmt::Display;
use tiberius::{AuthMethod, Config};
use tokio::net::TcpStream;

use crate::{
    configuration::{AppState, DBConnectionPool, DatabaseSettings},
    error::ApplicationError,
};

#[derive(Debug, Clone, Serialize)]
enum DatabaseConnectionState {
    Healthy,
    Unhealthy,
}

/// The Health of Maedic itself
/// Checks for a healthy Database connection
#[derive(Clone, Debug, Serialize)]
pub struct MaedicHealth {
    database_connection: DatabaseConnectionState,
    version_number: String,
}

/// Default values for MaedicHealth
impl MaedicHealth {
    fn healthy() -> Self {
        Self {
            database_connection: DatabaseConnectionState::Healthy,
            version_number: env!("CARGO_PKG_VERSION").to_string(),
        }
    }

    fn unhealthy() -> Self {
        Self {
            database_connection: DatabaseConnectionState::Unhealthy,
            version_number: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

pub async fn setup_database_pool(
    db_config: DatabaseSettings,
) -> Result<Pool<ConnectionManager>, tiberius::error::Error> {
    let mut config = Config::new();

    config.host(db_config.host);
    config.port(db_config.port);
    config.authentication(AuthMethod::sql_server(
        db_config.username,
        db_config.password.expose_secret(),
    ));
    if db_config.trust_cert {
        config.trust_cert();
    }
    config.database(db_config.database_name);
    // Connection should always be readonly as we are just monitoring
    config.readonly(true);

    let tcp = TcpStream::connect(config.get_addr()).await?;
    tcp.set_nodelay(true)?;
    let mgr = bb8_tiberius::ConnectionManager::build(config).unwrap();
    let pool = bb8::Pool::builder().max_size(2).build(mgr).await.unwrap();

    Ok(pool)
}

#[tracing::instrument(name = "Check self health", skip(state))]
pub async fn self_health(State(state): State<AppState>) -> Json<MaedicHealth> {
    match get_db_status(state.pool).await {
        Ok(state) => match state {
            DatabaseConnectionState::Healthy => Json(MaedicHealth::healthy()),
            DatabaseConnectionState::Unhealthy => Json(MaedicHealth::unhealthy()),
        },
        Err(_) => Json(MaedicHealth::unhealthy()),
    }
}

#[tracing::instrument(name = "check database connection", skip(pool))]
async fn get_db_status(
    pool: DBConnectionPool,
) -> Result<DatabaseConnectionState, ApplicationError> {
    match pool
        .get()
        .await?
        .simple_query("SELECT 1 as connection_state")
        .await?
        .into_row()
        .await?
        .unwrap()
        .get::<i32, &str>("connection_state")
        .ok_or(DatabaseConnectionState::Unhealthy)
    {
        Ok(1) => Ok(DatabaseConnectionState::Healthy),
        _ => Ok(DatabaseConnectionState::Unhealthy),
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
