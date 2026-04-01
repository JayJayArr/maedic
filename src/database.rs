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
    indicators::{HiQueueCount, PanelInstalled, SpoolFileCount},
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
        .ok_or(ApplicationError::EmptyResult)?
        .get::<i32, &str>("connection_state")
        .ok_or(DatabaseConnectionState::Unhealthy)
    {
        Ok(1) => Ok(DatabaseConnectionState::Healthy),
        _ => Ok(DatabaseConnectionState::Unhealthy),
    }
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

    let spool_file_counts: Vec<SpoolFileCount> =
        queryresult[0].iter().map(|row| row.into()).collect();

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
        .ok_or(ApplicationError::EmptyResult)?
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
        .ok_or(ApplicationError::EmptyResult)?
        .get::<i32, &str>("COUNT")
        .ok_or(ApplicationError::Conversion(
            "Failed to convert COUNT".to_string(),
        ))?;
    Ok(size)
}

#[tracing::instrument(name = "Get Version & build number", skip(pool))]
pub async fn get_version_number(
    pool: DBConnectionPool,
) -> Result<(u8, u8, u8, i32), ApplicationError> {
    let mut client = pool.get().await?;
    let tablesize = client
        .simple_query("SELECT COUNT(*) as COUNT FROM db_version")
        .await?
        .into_row()
        .await?
        .ok_or(ApplicationError::EmptyResult)?
        .get::<i32, &str>("COUNT")
        .ok_or(ApplicationError::Conversion(
            "Failed to convert COUNT".to_string(),
        ))?;
    if tablesize != 0 {
        let result = client
            .simple_query(
                "SELECT VER_MAJOR_NUM, VER_MINOR_NUM, VER_SP_NUM, Build_No FROM db_version",
            )
            .await?
            .into_row()
            .await?
            .ok_or(ApplicationError::EmptyResult)?;

        let major = result
            .get::<u8, &str>("VER_MAJOR_NUM")
            .ok_or(ApplicationError::Conversion(
                "Failed to convert Version".to_string(),
            ))?;
        let minor = result
            .get::<u8, &str>("VER_MINOR_NUM")
            .ok_or(ApplicationError::Conversion(
                "Failed to convert Build_no".to_string(),
            ))?;
        let patch = result
            .get::<u8, &str>("VER_SP_NUM")
            .ok_or(ApplicationError::Conversion(
                "Failed to convert Version".to_string(),
            ))?;
        let build_no = result
            .get::<i32, &str>("Build_No")
            .ok_or(ApplicationError::Conversion(
                "Failed to convert Build_no".to_string(),
            ))?;
        return Ok((major, minor, patch, build_no));
    }
    Ok((0, 0, 0, 0))
}

#[tracing::instrument(name = "Get Hi_Queue per Panel", skip(pool))]
pub async fn get_hiqueue_count_per_panel(
    pool: DBConnectionPool,
) -> Result<Vec<HiQueueCount>, ApplicationError> {
    let mut client = pool.get().await?;
    let panel_tablesize = match client
        .simple_query("SELECT COUNT(*) as COUNT FROM Panel")
        .await?
        .into_row()
        .await?
    {
        Some(result) => result
            .get::<i32, &str>("COUNT")
            .ok_or(ApplicationError::Conversion(
                "Failed to convert COUNT".to_string(),
            ))?,
        None => {
            return Err(ApplicationError::Conversion(
                "No result received".to_string(),
            ));
        }
    };
    let hi_queue_tablesize = match client
        .simple_query("SELECT COUNT(*) as COUNT FROM HI_QUEUE")
        .await?
        .into_row()
        .await?
    {
        Some(result) => result
            .get::<i32, &str>("COUNT")
            .ok_or(ApplicationError::Conversion(
                "Failed to convert COUNT".to_string(),
            ))?,
        None => {
            return Err(ApplicationError::Conversion(
                "No result received".to_string(),
            ));
        }
    };
    if panel_tablesize != 0 && hi_queue_tablesize != 0 {
        let result = client
            .simple_query(
            "select DESCRP as 'description', COUNT(*) as 'hi_queue_count' from (
	            select Panel.DESCRP, HI_QUEUE.ID from HI_QUEUE inner join Panel on HI_QUEUE.CPAR2 = Panel.ID
	            union all
	            select Panel.Descrp, HI_QUEUE.ID from HI_QUEUE inner join Panel on LEFT(HI_QUEUE.CPAR1,(CHARINDEX(':', CPAR1) + 7)) = PANEL.ID

            ) as interims
            group by DESCRP"
            )
            .await?
            .into_results()
            .await?;
        let hi_queue_count: Vec<HiQueueCount> = result[0].iter().map(|row| row.into()).collect();
        return Ok(hi_queue_count);
    }
    Ok(Vec::new())
}

#[tracing::instrument(name = "Get Firmware Records", skip(pool))]
pub async fn get_panel_state(
    pool: DBConnectionPool,
) -> Result<Vec<PanelInstalled>, ApplicationError> {
    let mut client = pool.get().await?;
    let panel_tablesize = match client
        .simple_query("SELECT COUNT(*) as COUNT FROM Panel")
        .await?
        .into_row()
        .await?
    {
        Some(result) => result
            .get::<i32, &str>("COUNT")
            .ok_or(ApplicationError::Conversion(
                "Failed to convert COUNT".to_string(),
            ))?,
        None => {
            return Err(ApplicationError::Conversion(
                "No result received".to_string(),
            ));
        }
    };
    if panel_tablesize != 0 {
        let result = client
            .simple_query(
                "select DESCRP as 'description', FIRMWARE_VERSION as 'firmware_version', INSTALLED as 'installed' from Panel",
            )
            .await?
            .into_results()
            .await?;
        let hi_queue_count: Vec<PanelInstalled> = result[0].iter().map(|row| row.into()).collect();
        return Ok(hi_queue_count);
    }
    Ok(Vec::new())
}
