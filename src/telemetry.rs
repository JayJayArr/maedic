use tracing_subscriber::{EnvFilter, Layer, layer::SubscriberExt, util::SubscriberInitExt};

pub fn initialize_tracing() -> anyhow::Result<()> {
    //Filter unnecessary info
    let filter = EnvFilter::new("info")
        .add_directive("reqwest=info".parse().unwrap())
        .add_directive("tiberius=info".parse().unwrap());

    //Create a format layer with the appropriate filter
    let fmt_layer = tracing_subscriber::fmt::layer().with_filter(filter);

    tracing_subscriber::registry()
        //install standard output layer
        .with(fmt_layer)
        .init();
    Ok(())
}
