use crate::configuration::get_configuration;
use axum::{Router, routing::get};
use tracing::info;

mod configuration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let configuration = get_configuration()?;

    let listener = tokio::net::TcpListener::bind(format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    ))
    .await
    .expect("Could not bind port");
    info!(
        "Starting maedic on port: {}",
        configuration.application.port
    );

    let app = Router::new().route("/", get(|| async { "Hello, World!" }));

    axum::serve(listener, app).await?;

    Ok(())
}
