use std::str::FromStr;

use tracing_subscriber::{
    EnvFilter, Layer, filter::Directive, layer::SubscriberExt, util::SubscriberInitExt,
};

/// Initialize `Tracing` and a `tracing_subscriber
pub fn initialize_tracing(env_filter: String, path: String) -> anyhow::Result<()> {
    let log_level: Directive = Directive::from_str(env_filter.as_str())?;
    //Filter unnecessary info
    let filter = EnvFilter::new(&env_filter)
        .add_directive(format!("reqwest={}", log_level).parse()?)
        .add_directive(format!("tiberius={}", log_level).parse()?);

    //Create a format layer with the appropriate filter for standard output
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_ansi(false)
        .with_filter(filter.clone());

    //Create a format layer with the appropriate filter for logging to file
    let file_log_layer = tracing_subscriber::fmt::layer()
        .with_ansi(false)
        .with_writer(
            std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
                .expect("Could not create or open logfile"),
        )
        .with_filter(filter);

    tracing_subscriber::registry()
        // install standard output layer
        .with(fmt_layer)
        // install logging to file
        .with(file_log_layer)
        .init();
    Ok(())
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use crate::telemetry::initialize_tracing;

    #[rstest]
    #[case("error")]
    #[case("warn")]
    #[case("info")]
    #[case("debug")]
    #[case("trace")]
    #[case("ERROR")]
    #[case("WARN")]
    #[case("INFO")]
    #[case("DEBUG")]
    #[case("TRACE")]
    fn test_log_level_is_accepted(#[case] log_level: String) {
        assert!(initialize_tracing(log_level, "maedic.log".to_string()).is_ok())
    }

    #[rstest]
    #[case("logs")]
    #[case("warnings")]
    #[case("errors")]
    #[case("traces")]
    #[case("infos")]
    #[case("debugs")]
    fn test_reject_wrong_log_level(#[case] log_level: String) {
        assert!(initialize_tracing(log_level, "maedic.log".to_string()).is_err())
    }
}
