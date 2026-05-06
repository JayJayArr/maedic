use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct HiQueueCount {
    pub hi_queue_count: i32,
    pub description: String,
}

impl From<tiberius::Row> for HiQueueCount {
    fn from(val: tiberius::Row) -> Self {
        HiQueueCount {
            description: val
                .get::<&str, &str>("description")
                .unwrap_or_default()
                .to_string(),
            hi_queue_count: val.get("hi_queue_count").unwrap_or_default(),
        }
    }
}

impl From<&tiberius::Row> for HiQueueCount {
    fn from(val: &tiberius::Row) -> Self {
        HiQueueCount {
            description: val
                .get::<&str, &str>("description")
                .unwrap_or_default()
                .to_string(),
            hi_queue_count: val.get("hi_queue_count").unwrap_or_default(),
        }
    }
}
