use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde::Serialize;
use tracing::error;

use crate::{AppState, DbClient, configuration::Settings};

#[derive(Serialize, Debug)]
pub struct Health {
    pub hi_queue_size: i32,
    pub spool_file_count: Vec<SpoolFileCount>,
}

pub async fn check_health(
    State(state): State<AppState>,
) -> Result<(StatusCode, Json<Health>), HealthError> {
    let hi_queue_size = get_hiqueue_count(state.db_client.clone()).await?;
    let spool_file_count =
        get_spoolfile_count(state.db_client, state.config.limits.spool_file_count).await?;

    let health = Health {
        spool_file_count,
        hi_queue_size: hi_queue_size,
    };

    if !health_is_good(&health, &state.config) {
        error!("App reported unhealthy status {:?}", health);
        return Ok((StatusCode::SERVICE_UNAVAILABLE, Json(health)));
    } else {
        return Ok((StatusCode::OK, Json(health)));
    }
}

fn health_is_good(health: &Health, config: &Settings) -> bool {
    if health.hi_queue_size > config.limits.hi_queue_count.into()
        || health.spool_file_count.len() > 0
    {
        return false;
    }
    true
}

async fn get_hiqueue_count(client: DbClient) -> Result<i32, HealthError> {
    let mut client = client.lock().await;
    let size = client
        .simple_query("SELECT COUNT(*) as HIQUEUECOUNT FROM HI_QUEUE")
        .await?
        .into_row()
        .await?
        .unwrap()
        .get::<i32, &str>("HIQUEUECOUNT")
        .ok_or(HealthError::ConversionError(
            "Failed to convert HIQUEUECOUNT".to_string(),
        ))?;
    Ok(size)
}

async fn get_spoolfile_count(
    client: DbClient,
    limit_per_channel: i32,
) -> Result<Vec<SpoolFileCount>, HealthError> {
    let mut client = client.lock().await;
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

#[derive(thiserror::Error, Debug)]
pub enum HealthError {
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),

    #[error(transparent)]
    DatabaseError(#[from] tiberius::error::Error),

    #[error("{0}")]
    ConversionError(String),
}

impl IntoResponse for HealthError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            Self::UnexpectedError(err) => {
                tracing::error!("{:?}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Something went wrong".to_owned(),
                )
            }
            Self::DatabaseError(err) => {
                tracing::error!("{:?}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Something went wrong with the database queries".to_owned(),
                )
            }
            Self::ConversionError(err) => {
                tracing::error!("{:?}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Error when converting a DB value".to_owned(),
                )
            }
        };
        (status, message).into_response()
    }
}

#[derive(Serialize, Debug)]
pub struct SpoolFileCount {
    spool_file_count: i32,
    description: String,
    directory: String,
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
