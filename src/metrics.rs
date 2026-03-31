use crate::configuration::DBConnectionPool;
use crate::database::get_hiqueue_count;
use crate::error::ApplicationError;
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
pub enum Counts {
    USERS,
    CARDS,
    PANELS,
    CHANNELS,
    SUBPANELS,
    READERS,
    HiQueue,
    UnackAlarms,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct CountLabels {
    pub size: Counts,
}

#[derive(Debug, Default)]
pub struct Metrics {
    sizes: Family<CountLabels, Gauge>,
}

impl Metrics {
    pub fn set_size(&self, size: Counts, value: i64) {
        self.sizes.get_or_create(&CountLabels { size }).set(value);
    }
}

#[tracing::instrument(name = "Scrape metrics", skip(state))]
pub async fn metrics_handler(State(state): State<Arc<Mutex<AppState>>>) -> impl IntoResponse {
    let state = state.lock().await;
    collect_metrics(state.pool.clone(), &state.metrics)
        .await
        .unwrap();
    let mut buffer = String::new();
    encode(&mut buffer, &state.registry).unwrap();
    Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(buffer))
        .unwrap()
}

#[tracing::instrument(name = "Collect metrics", skip(pool, metrics))]
pub async fn collect_metrics(
    pool: DBConnectionPool,
    metrics: &Metrics,
) -> Result<(), ApplicationError> {
    let hi_queue_count = get_hiqueue_count(pool).await?;
    metrics.set_size(Counts::HiQueue, hi_queue_count.into());
    Ok(())
}

pub async fn setup_metrics_registry() -> (Registry, Metrics) {
    let metrics = Metrics {
        sizes: Family::default(),
    };
    let mut registry = Registry::default();
    registry.register(
        "counts",
        "Number of database objects",
        metrics.sizes.clone(),
    );
    (registry, metrics)
}
