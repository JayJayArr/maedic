use prometheus_client::encoding::EncodeLabelValue;
use strum_macros::EnumIter;

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
