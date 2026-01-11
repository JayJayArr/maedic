use opentelemetry::trace::TracerProvider;
use opentelemetry_otlp::{Protocol, WithExportConfig};
use opentelemetry_sdk::{Resource, trace::SdkTracerProvider};
use tracing_subscriber::{EnvFilter, Layer, layer::SubscriberExt, util::SubscriberInitExt};

pub fn init_tracing() -> anyhow::Result<()> {
    //init SpanExporter to opentelemetry
    let otlp_exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_http()
        .with_protocol(Protocol::HttpJson)
        .with_endpoint("http://localhost:4318/v1/traces")
        .build()?;

    //Filter unnecessary info
    let filter = EnvFilter::new("info")
        .add_directive("reqwest=info".parse().unwrap())
        .add_directive("tiberius=info".parse().unwrap());

    let tracer_provider = SdkTracerProvider::builder()
        .with_resource(Resource::builder().with_service_name("maedic").build())
        .with_batch_exporter(otlp_exporter)
        .build();
    let tracer = tracer_provider.tracer("maedic");

    // Create a tracing layer with the configured tracer
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    let fmt_layer = tracing_subscriber::fmt::layer().with_filter(filter);

    tracing_subscriber::registry()
        //install opentelemetry layer
        .with(telemetry)
        //install standard output layer
        .with(fmt_layer)
        .init();
    Ok(())
}
