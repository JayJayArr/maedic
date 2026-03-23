use crate::run::AppState;
use axum::http::StatusCode;
use axum::{extract::State, response::IntoResponse};

#[tracing::instrument(name = "Check self health", skip(state))]
pub async fn metrics_handler(State(state): State<AppState>) -> impl IntoResponse {
    StatusCode::OK
}
