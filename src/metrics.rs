use crate::run::AppState;
use axum::http::StatusCode;
use axum::{extract::State, response::IntoResponse};
use std::sync::Arc;
use tokio::sync::Mutex;

#[tracing::instrument(name = "Check self health", skip(state))]
pub async fn metrics_handler(State(state): State<Arc<Mutex<AppState>>>) -> impl IntoResponse {
    StatusCode::OK
}
