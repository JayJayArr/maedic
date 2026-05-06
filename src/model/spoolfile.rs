use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, PartialEq)]

pub struct SpoolFileCount {
    pub spool_file_count: i32,
    pub description: String,
    pub directory: String,
}

impl From<tiberius::Row> for SpoolFileCount {
    fn from(val: tiberius::Row) -> Self {
        SpoolFileCount {
            description: val.get::<&str, &str>("description").unwrap().to_string(),
            spool_file_count: val.get("spool_file_count").unwrap(),
            directory: val.get::<&str, &str>("directory").unwrap().to_string(),
        }
    }
}

impl From<&tiberius::Row> for SpoolFileCount {
    fn from(val: &tiberius::Row) -> Self {
        SpoolFileCount {
            description: val
                .get::<&str, &str>("description")
                .unwrap_or_default()
                .to_string(),
            spool_file_count: val.get("spool_file_count").unwrap_or_default(),
            directory: val
                .get::<&str, &str>("directory")
                .unwrap_or_default()
                .to_string(),
        }
    }
}
