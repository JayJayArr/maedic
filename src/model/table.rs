use prometheus_client::encoding::EncodeLabelValue;
use strum_macros::EnumIter;

/// `Table` lists the Tables where the size is used in the metrics
#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelValue, EnumIter, strum_macros::Display)]
pub enum Tables {
    #[strum(to_string = "audit_log")]
    AuditChanges,
    #[strum(to_string = "badge")]
    Badges,
    #[strum(to_string = "badge_c")]
    Cards,
    #[strum(to_string = "panel")]
    Panels,
    #[strum(to_string = "channel")]
    Channels,
    #[strum(to_string = "reader")]
    Readers,
    #[strum(to_string = "input")]
    Inputs,
    #[strum(to_string = "output")]
    Outputs,
    #[strum(to_string = "logical_dev")]
    LogicalDevices,
    #[strum(to_string = "company")]
    Companies,
    #[strum(to_string = "clear")]
    ClearanceCodes,
    #[strum(to_string = "hi_queue")]
    HiQueue,
    #[strum(to_string = "unack_Al")]
    UnacknowledgedAlarms,
    #[strum(to_string = "ev_log")]
    Events,
    #[strum(to_string = "parti")]
    Partitions,
    #[strum(to_string = "uid")]
    SoftwareUsers,
    #[strum(to_string = "spanel")]
    Subpanels,
    #[strum(to_string = "wrkst")]
    Workstations,
}

