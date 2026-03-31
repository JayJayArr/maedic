use crate::configuration::DBConnectionPool;
use crate::database::{get_card_state, get_table_count};
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
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use tokio::sync::Mutex;

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelValue, EnumIter, strum_macros::Display)]
pub enum TableSizes {
    #[strum(to_string = "badge")]
    Badges,
    #[strum(to_string = "badge_c")]
    Cards,
    #[strum(to_string = "panel")]
    Panels,
    #[strum(to_string = "channel")]
    Channels,
    #[strum(to_string = "spanel")]
    Subpanels,
    #[strum(to_string = "reader")]
    Readers,
    #[strum(to_string = "hi_queue")]
    HiQueue,
    #[strum(to_string = "unack_Al")]
    UnackAlarms,
    #[strum(to_string = "ev_log")]
    Events,
    #[strum(to_string = "uid")]
    Users,
    #[strum(to_string = "wrkst")]
    Workstations,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelValue, EnumIter, strum_macros::Display)]
pub enum CardStates {
    #[strum(to_string = "A")]
    Active,
    #[strum(to_string = "D")]
    Disabled,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct CardStateLabels {
    pub status: CardStates,
}
#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct TableSizeLabels {
    pub table: TableSizes,
}

#[derive(Debug, Default)]
pub struct Metrics {
    table: Family<TableSizeLabels, Gauge>,
    status: Family<CardStateLabels, Gauge>,
}

impl Metrics {
    pub fn set_table_size(&self, size: TableSizes, value: i64) {
        self.table
            .get_or_create(&TableSizeLabels { table: size })
            .set(value);
    }

    pub fn set_card_state(&self, state: CardStates, value: i64) {
        self.status
            .get_or_create(&CardStateLabels { status: state })
            .set(value);
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
    //Collect Table sizes
    for count in TableSizes::iter() {
        let countvalue = get_table_count(pool.clone(), count.to_string()).await?;
        metrics.set_table_size(count, countvalue.into());
    }

    for state in CardStates::iter() {
        let countvalue = get_card_state(pool.clone(), state.to_string()).await?;
        metrics.set_card_state(state, countvalue.into());
    }
    Ok(())
}

pub async fn setup_metrics_registry() -> (Registry, Metrics) {
    let metrics = Metrics {
        table: Family::default(),
        status: Family::default(),
    };
    let mut registry = Registry::default();
    registry.register(
        "tablesize",
        "Number of database objects",
        metrics.table.clone(),
    );
    registry.register("card_state", "State of cards", metrics.status.clone());
    (registry, metrics)
}
