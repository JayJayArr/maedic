use maedic::{
    configuration::{AppState, get_configuration},
    database::setup_database_pool,
    run::run,
};
use opentelemetry_otlp::Protocol;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{Resource, trace::SdkTracerProvider};
use std::sync::Arc;
use sysinfo::System;
use tokio::sync::Mutex;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use opentelemetry::trace::TracerProvider;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let configuration = get_configuration()?;
    let otlp_exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_http()
        .with_protocol(Protocol::HttpJson)
        .with_endpoint("http://localhost:4318/v1/traces")
        .build()?;
    //
    // let provider = SdkLoggerProvider::builder()
    //     .with_resource(Resource::builder().with_service_name("maedic").build())
    //     .with_batch_exporter(otlp_exporter)
    //     .build();
    //
    // let filter_otel = EnvFilter::new("debug")
    //     .add_directive("hyper=off".parse().unwrap())
    //     .add_directive("tonic=off".parse().unwrap())
    //     .add_directive("h2=off".parse().unwrap())
    //     .add_directive("reqwest=off".parse().unwrap())
    //     .add_directive("tiberius=off".parse().unwrap());
    // let otel_layer = layer::OpenTelemetryTracingBridge::new(&provider).with_filter(filter_otel);
    // let filter_fmt = EnvFilter::new("info").add_directive("opentelemetry=debug".parse().unwrap());
    // let fmt_layer = tracing_subscriber::fmt::layer()
    //     .with_thread_names(true)
    //     .with_filter(filter_fmt);
    //
    // tracing_subscriber::registry()
    //     .with(otel_layer)
    //     .with(fmt_layer)
    //     .init();

    let provider = SdkTracerProvider::builder()
        .with_resource(Resource::builder().with_service_name("maedic").build())
        .with_batch_exporter(otlp_exporter)
        .build();
    let tracer = provider.tracer("maedic");

    // Create a tracing layer with the configured tracer
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    // let subscriber = Registry::default().with(telemetry);
    tracing_subscriber::registry()
        .with(telemetry)
        // .with(otel_layer)
        // .with(fmt_layer)
        .init();

    info!(
        "Starting maedic version {} with config: {:?}",
        env!("CARGO_PKG_VERSION"),
        configuration
    );

    let listener = tokio::net::TcpListener::bind(format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    ))
    .await
    .expect("Could not bind to port");

    let pool = setup_database_pool(configuration.database.clone()).await?;
    let state = AppState {
        pool,
        config: configuration.clone(),
        sys: Arc::new(Mutex::new(System::new_all())),
    };

    run(listener, state, configuration)
        .await
        .expect("Failed to start application");

    Ok(())
}
