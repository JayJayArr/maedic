use axum::{Json, extract::State};
use bb8::Pool;
use bb8_tiberius::ConnectionManager;
use secrecy::ExposeSecret;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tiberius::{AuthMethod, Config};
use tokio::net::TcpStream;
use tokio::sync::Mutex;

use crate::{
    configuration::{DBConnectionPool, DatabaseSettings},
    error::ApplicationError,
    health::MaedicHealth,
    indicators::SpoolFileCount,
    run::AppState,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DatabaseConnectionState {
    Healthy,
    Unhealthy,
}

#[tracing::instrument(name = "Setup Database connection pool", skip_all)]
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
pub async fn self_health(State(state): State<Arc<Mutex<AppState>>>) -> Json<MaedicHealth> {
    let state = state.lock().await;
    match get_db_status(state.pool.clone()).await {
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

#[tracing::instrument(name = "Check HI_QUEUE Table", skip_all)]
pub async fn get_hiqueue_count(pool: DBConnectionPool) -> Result<i32, ApplicationError> {
    let mut client = pool.get().await?;
    let size = client
        .simple_query("SELECT COUNT(*) as HIQUEUECOUNT FROM HI_QUEUE")
        .await?
        .into_row()
        .await?
        .unwrap()
        .get::<i32, &str>("HIQUEUECOUNT")
        .ok_or(ApplicationError::Conversion(
            "Failed to convert HIQUEUECOUNT".to_string(),
        ))?;
    Ok(size)
}

#[tracing::instrument(name = "Check unhealthy spoolfiles", skip_all)]
pub async fn get_unhealthy_spoolfiles(
    pool: DBConnectionPool,
    limit_per_channel: i32,
) -> Result<Vec<SpoolFileCount>, ApplicationError> {
    let mut client = pool.get().await?;
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

#[tracing::instrument(name = "Check Table Size", skip(pool))]
pub async fn get_table_count(
    pool: DBConnectionPool,
    tablename: String,
) -> Result<i32, ApplicationError> {
    let mut client = pool.get().await?;
    let size = client
        .simple_query(format!("SELECT COUNT(*) as COUNT FROM {}", tablename))
        .await?
        .into_row()
        .await?
        .unwrap()
        .get::<i32, &str>("COUNT")
        .ok_or(ApplicationError::Conversion(
            "Failed to convert COUNT".to_string(),
        ))?;
    Ok(size)
}

#[tracing::instrument(name = "Check Card Status", skip(pool))]
pub async fn get_card_state(
    pool: DBConnectionPool,
    status: String,
) -> Result<i32, ApplicationError> {
    let mut client = pool.get().await?;
    let size = client
        .simple_query(format!(
            "SELECT COUNT(*) as COUNT FROM Badge_C where STAT_COD = '{}'",
            status
        ))
        .await?
        .into_row()
        .await?
        .unwrap()
        .get::<i32, &str>("COUNT")
        .ok_or(ApplicationError::Conversion(
            "Failed to convert COUNT".to_string(),
        ))?;
    Ok(size)
}
