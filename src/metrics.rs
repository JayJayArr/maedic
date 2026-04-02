use crate::configuration::DBConnectionPool;
use crate::database::{
    get_card_state, get_hiqueue_count_per_panel, get_panel_state, get_table_count,
    get_unhealthy_spoolfiles, get_version_number,
};
use crate::error::ApplicationError;
use crate::run::AppState;
use axum::body::Body;
use axum::http::header::CONTENT_TYPE;
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

/// `TableSizes` lists the Tables where the size is used in the metrics
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

/// `CardStates` lists the possible States of a saved card
/// e.g. `Active` or `Disabled`, each state being saved as a single char in the db
#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelValue, EnumIter, strum_macros::Display)]
pub enum CardStates {
    #[strum(to_string = "A")]
    Active,
    #[strum(to_string = "D")]
    Disabled,
    #[strum(to_string = "O")]
    AutoDisabled,
    #[strum(to_string = "X")]
    Expired,
    #[strum(to_string = "L")]
    Lost,
    #[strum(to_string = "S")]
    Stolen,
    #[strum(to_string = "T")]
    Terminated,
    #[strum(to_string = "U")]
    Unaccounted,
    #[strum(to_string = "V")]
    Void,
}

/// `CardStates` lists the possible States of a saved card
/// e.g. `Active` or `Disabled`, each state being saved as a single char in the db
#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelValue, EnumIter)]
pub enum VersionComponents {
    Major,
    Minor,
    Patch,
    BuildNo,
}

/// `CardStateLabels`` is the displayed label for each Metric in the family
#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct CardStateLabels {
    pub status: CardStates,
}

/// `TableSizeLabels` is the displayed label for each Metric in the family
#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct TableSizeLabels {
    pub table: TableSizes,
}

/// `VersionLabels` is the displayed label for the Version numbers
#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct VersionLabels {
    pub value: VersionComponents,
}
/// `SpoolFileLabel` is the displayed label for the Spool Files
#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct SpoolFileLabel {
    pub panel: String,
}

/// `HiQueueLabel` is the displayed label for the hi_queue counts
#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct HiQueueLabel {
    pub channel: String,
}

/// `PanelFirmwareLabel` is the displayed label for the panel_firmware
#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct PanelInstalledLabel {
    pub panel: String,
    pub major_version: i64,
    pub minor_version: i64,
}

/// `Metrics` is the complete collection of all exposed metrics
#[derive(Debug, Default)]
pub struct Metrics {
    table: Family<TableSizeLabels, Gauge>,
    status: Family<CardStateLabels, Gauge>,
    version: Family<VersionLabels, Gauge>,
    spool_files: Family<SpoolFileLabel, Gauge>,
    hi_queue_counts: Family<HiQueueLabel, Gauge>,
    panel_installed: Family<PanelInstalledLabel, Gauge>,
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

    pub fn set_version(&self, version: VersionComponents, value: i64) {
        self.version
            .get_or_create(&VersionLabels { value: version })
            .set(value);
    }

    pub fn set_spool_file_count(&self, panel: String, value: i64) {
        self.spool_files
            .get_or_create(&SpoolFileLabel { panel })
            .set(value);
    }

    pub fn set_hi_queue_count(&self, channel: String, value: i64) {
        self.hi_queue_counts
            .get_or_create(&HiQueueLabel { channel })
            .set(value);
    }

    pub fn set_panel_firmware(&self, panel: String, major: i64, minor: i64, installed: i64) {
        self.panel_installed
            .get_or_create(&PanelInstalledLabel {
                panel,
                major_version: major,
                minor_version: minor,
            })
            .set(installed);
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
        .header(CONTENT_TYPE, "text/plain; version=0.0.4; charset=utf-8")
        .body(Body::from(buffer))
        .unwrap()
}

#[tracing::instrument(name = "Collect metrics", skip(pool, metrics))]
pub async fn collect_metrics(
    pool: DBConnectionPool,
    metrics: &Metrics,
) -> Result<(), ApplicationError> {
    // Collect Version numbers
    let (major, minor, patch, build_no) = get_version_number(pool.clone()).await?;
    metrics.set_version(VersionComponents::Major, major.into());
    metrics.set_version(VersionComponents::Minor, minor.into());
    metrics.set_version(VersionComponents::Patch, patch.into());
    metrics.set_version(VersionComponents::BuildNo, build_no.into());

    // Collect Table sizes
    for count in TableSizes::iter() {
        let countvalue = get_table_count(pool.clone(), count.to_string()).await?;
        metrics.set_table_size(count, countvalue.into());
    }

    // Collect Card Status Metrics
    for state in CardStates::iter() {
        let countvalue = get_card_state(pool.clone(), state.to_string()).await?;
        metrics.set_card_state(state, countvalue.into());
    }

    // Collect and set spool_file metrics
    let spool_files = get_unhealthy_spoolfiles(pool.clone(), -1).await?;
    for spool_file in spool_files {
        metrics.set_spool_file_count(spool_file.description, spool_file.spool_file_count.into());
    }

    // Collect and set hi_queue_counts
    let hi_queue_counts = get_hiqueue_count_per_panel(pool.clone()).await?;
    for hi_queue_count in hi_queue_counts {
        metrics.set_hi_queue_count(
            hi_queue_count.description,
            hi_queue_count.hi_queue_count.into(),
        );
    }

    // Collect and set panel_firmware
    let panel_firmware_records = get_panel_state(pool.clone()).await?;
    for panel_firmware in panel_firmware_records {
        metrics.set_panel_firmware(
            panel_firmware.description,
            panel_firmware.firmware_major_version,
            panel_firmware.firmware_minor_version,
            panel_firmware.installed,
        );
    }

    Ok(())
}

pub async fn setup_metrics_registry() -> (Registry, Metrics) {
    let metrics = Metrics {
        table: Family::default(),
        status: Family::default(),
        version: Family::default(),
        spool_files: Family::default(),
        hi_queue_counts: Family::default(),
        panel_installed: Family::default(),
    };
    let mut registry = Registry::default();
    registry.register(
        "pw_version_number",
        "Version numbers",
        metrics.version.clone(),
    );
    registry.register(
        "tablesize",
        "Number of database objects",
        metrics.table.clone(),
    );
    registry.register("card_state", "State of cards", metrics.status.clone());
    registry.register(
        "spool_files",
        "Spool files per Channel",
        metrics.spool_files.clone(),
    );
    registry.register(
        "hi_queue_counts",
        "Actions queued per Channel",
        metrics.hi_queue_counts.clone(),
    );
    registry.register(
        "panel_installed",
        "Installation Status of each Panel, 1=UP, 0=DOWN",
        metrics.panel_installed.clone(),
    );
    (registry, metrics)
}
