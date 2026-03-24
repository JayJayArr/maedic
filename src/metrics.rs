use crate::run::AppState;
use axum::body::Body;
use axum::http::{Response, StatusCode};
use axum::{extract::State, response::IntoResponse};
use prometheus_client::encoding::text::encode;
use prometheus_client::encoding::{EncodeLabelSet, EncodeLabelValue};
use prometheus_client::metrics::family::Family;
use prometheus_client::metrics::gauge::Gauge;
use prometheus_client::registry::Registry;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelValue)]
pub enum Size {
    USERS,
    CARDS,
    PANELS,
    CHANNELS,
    SUBPANELS,
    READERS,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct SizeLabels {
    pub size: Size,
}

#[derive(Debug, Default)]
pub struct Metrics {
    sizes: Family<SizeLabels, Gauge>,
}

impl Metrics {
    pub fn set_size(&self, size: Size, value: i64) {
        self.sizes.get_or_create(&SizeLabels { size }).set(value);
    }
}

#[tracing::instrument(name = "Scrape metrics", skip(state))]
pub async fn metrics_handler(State(state): State<Arc<Mutex<AppState>>>) -> impl IntoResponse {
    let state = state.lock().await;
    let mut buffer = String::new();
    encode(&mut buffer, &state.registry).unwrap();
    Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(buffer))
        .unwrap()
}

pub async fn collect_metrics() {
    todo!()
}

pub async fn setup_metrics_registry() -> Registry {
    let metrics = Metrics {
        sizes: Family::default(),
    };
    let mut registry = Registry::default();
    registry.register(
        "requests",
        "Number of database objects",
        metrics.sizes.clone(),
    );
    registry
}
