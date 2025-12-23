use axum::{http::StatusCode, response::IntoResponse};

#[derive(thiserror::Error, Debug)]
pub enum HealthError {
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),

    #[error(transparent)]
    Database(#[from] tiberius::error::Error),

    #[error("{0}")]
    Conversion(String),
}

impl IntoResponse for HealthError {
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
            Self::Conversion(err) => {
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
