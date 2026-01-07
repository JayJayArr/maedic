use axum::{http::StatusCode, response::IntoResponse};
use bb8::RunError;

#[derive(thiserror::Error, Debug)]
pub enum ApplicationError {
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),

    #[error(transparent)]
    Database(#[from] tiberius::error::Error),

    #[error(transparent)]
    DatabaseConnection(#[from] RunError<bb8_tiberius::Error>),

    #[error("{0}")]
    Conversion(String),
}

impl IntoResponse for ApplicationError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            Self::Unexpected(err) => {
                tracing::error!("{:?}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
            }
            Self::Database(err) => {
                tracing::error!("{:?}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
            }
            Self::DatabaseConnection(err) => {
                tracing::error!("{:?}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
            }
            Self::Conversion(err) => {
                tracing::error!("{:?}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
            }
        };
        (status, message).into_response()
    }
}
