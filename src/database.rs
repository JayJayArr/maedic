use axum::{extract::State, http::StatusCode, response::IntoResponse};
use secrecy::ExposeSecret;
use tiberius::{AuthMethod, Client, Config};
use tokio::net::TcpStream;
use tokio_util::compat::{Compat, TokioAsyncWriteCompatExt};

use crate::configuration::{AppState, DatabaseSettings, DbClient};

#[derive(Debug, Clone)]
enum DatabaseConnectionState {
    Up,
    Down,
}

pub async fn setup_database_client(
    db_config: DatabaseSettings,
) -> Result<Client<Compat<tokio::net::TcpStream>>, tiberius::error::Error> {
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

    Client::connect(config, tcp.compat_write()).await
}

pub async fn self_health(State(state): State<AppState>) -> Result<StatusCode, SelfHealthError> {
    match get_db_status(state.db_client).await {
        //TODO: Reconnect the DB Client on failure
        Ok(state) => match state {
            DatabaseConnectionState::Up => Ok(StatusCode::OK),
            DatabaseConnectionState::Down => Err(SelfHealthError::Disconnected),
        },
        Err(_) => Err(SelfHealthError::Disconnected),
    }
}

async fn get_db_status(client: DbClient) -> Result<DatabaseConnectionState, SelfHealthError> {
    match client
        .lock()
        .await
        .simple_query("SELECT 1 as connection_state")
        .await?
        .into_row()
        .await?
        .unwrap()
        .get::<i32, &str>("connection_state")
        .ok_or(DatabaseConnectionState::Down)
    {
        Ok(1) => Ok(DatabaseConnectionState::Up),
        _ => Ok(DatabaseConnectionState::Down),
    }
}

#[derive(thiserror::Error, Debug)]
pub enum SelfHealthError {
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),

    #[error(transparent)]
    Database(#[from] tiberius::error::Error),

    #[error("Database Disconnect")]
    Disconnected,
}

impl IntoResponse for SelfHealthError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            Self::Unexpected(err) => {
                tracing::error!("{:?}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Something went wrong".to_owned(),
                )
            }
            Self::Database(err) => {
                tracing::error!("{:?}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Something went wrong with the database queries".to_owned(),
                )
            }
            Self::Disconnected => {
                tracing::error!("Database disconnected, trying reconnect...");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Database disconnected, trying to reconnect...".to_owned(),
                )
            }
        };
        (status, message).into_response()
    }
}
