pub struct PanelInstalled {
    pub description: String,
    pub firmware_major_version: i64,
    pub firmware_minor_version: i64,
    pub installed: bool,
}

impl From<tiberius::Row> for PanelInstalled {
    fn from(val: tiberius::Row) -> Self {
        let split: Vec<&str> = val
            .get::<&str, &str>("firmware_version")
            .unwrap_or_default()
            .split_terminator(".")
            .collect();
        let mut iter = split.iter();
        PanelInstalled {
            description: val
                .get::<&str, &str>("description")
                .unwrap_or_default()
                .to_string(),
            installed: if val.get::<&str, &str>("installed").unwrap_or_default() == "Y" {
                true
            } else {
                false
            },
            firmware_major_version: iter
                .next()
                .unwrap_or(&"0")
                .parse::<i64>()
                .unwrap_or_default(),
            firmware_minor_version: iter
                .next()
                .unwrap_or(&"0")
                .parse::<i64>()
                .unwrap_or_default(),
        }
    }
}

impl From<&tiberius::Row> for PanelInstalled {
    fn from(val: &tiberius::Row) -> Self {
        let split: Vec<&str> = val
            .get::<&str, &str>("firmware_version")
            .unwrap_or_default()
            .split_terminator(".")
            .collect();
        let mut iter = split.iter();
        PanelInstalled {
            description: val
                .get::<&str, &str>("description")
                .unwrap_or_default()
                .to_string(),
            installed: if val.get::<&str, &str>("installed").unwrap_or_default() == "Y" {
                true
            } else {
                false
            },
            firmware_major_version: iter
                .next()
                .unwrap_or(&"0")
                .parse::<i64>()
                .unwrap_or_default(),
            firmware_minor_version: iter
                .next()
                .unwrap_or(&"0")
                .parse::<i64>()
                .unwrap_or_default(),
        }
    }
}
