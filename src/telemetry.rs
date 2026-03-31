use tracing_subscriber::{EnvFilter, Layer, layer::SubscriberExt, util::SubscriberInitExt};

/// Initialize `Tracing` and a `tracing_subscriber
pub fn initialize_tracing(env_filter: String) -> anyhow::Result<()> {
    //Filter unnecessary info
    let filter = EnvFilter::new(env_filter.clone())
        .add_directive(format!("reqwest={}", env_filter.clone()).parse().unwrap())
        .add_directive(format!("tiberius={}", env_filter).parse().unwrap());

    //Create a format layer with the appropriate filter
    let fmt_layer = tracing_subscriber::fmt::layer().with_filter(filter);

    tracing_subscriber::registry()
        //install standard output layer
        .with(fmt_layer)
        .init();
    Ok(())
}
