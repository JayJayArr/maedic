use std::str::FromStr;

use prometheus_client::encoding::EncodeLabelValue;
use strum_macros::EnumIter;

use crate::error::ApplicationError;

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

impl FromStr for CardStates {
    type Err = ApplicationError;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "A" => Ok(CardStates::Active),
            "D" => Ok(CardStates::Disabled),
            "O" => Ok(CardStates::AutoDisabled),
            "X" => Ok(CardStates::Expired),
            "L" => Ok(CardStates::Lost),
            "S" => Ok(CardStates::Stolen),
            "T" => Ok(CardStates::Terminated),
            "U" => Ok(CardStates::Unaccounted),
            "V" => Ok(CardStates::Void),
            _ => Err(ApplicationError::Conversion(format!(
                "Could not convert {} to a CardState",
                value
            ))),
        }
    }
}
