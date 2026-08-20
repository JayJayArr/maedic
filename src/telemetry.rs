use tracing::subscriber::set_global_default;
use tracing_log::LogTracer;
use tracing_subscriber::{EnvFilter, Layer, Registry, filter::Directive, layer::SubscriberExt};

/// Initialize `Tracing` and a `tracing_subscriber
pub fn initialize_tracing(env_filter: String, path: String) -> anyhow::Result<()> {
    //This format fails non-existing log-levels early
    format!("maedic={}", env_filter).parse::<Directive>()?;

    let env_filter = EnvFilter::new(env_filter);
    let fmt_layer_file = tracing_subscriber::fmt::layer()
        .with_ansi(false)
        .with_writer(
            std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
                .expect("Could not create or open logfile"),
        )
        .with_filter(env_filter.clone());
    let fmt_layer_log = tracing_subscriber::fmt::layer()
        .with_ansi(true)
        .with_filter(env_filter.clone());

    let subscriber = Registry::default()
        .with(env_filter)
        .with(fmt_layer_file)
        .with(fmt_layer_log);

    LogTracer::init().expect("Failed to set logger");
    set_global_default(subscriber).expect("Failed to set subscriber");
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
