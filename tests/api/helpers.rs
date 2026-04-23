use axum::Router;
use axum::extract::ConnectInfo;
use axum::extract::connect_info::IntoMakeServiceWithConnectInfo;
use axum::middleware::AddExtension;
use axum::serve::Serve;
use maedic::metrics::setup_metrics_registry;
use maedic::run::run;
use maedic::{
    configuration::{DBConnectionPool, DatabaseSettings, Settings, get_configuration},
    database::setup_database_pool,
    run::AppState,
    telemetry::initialize_tracing,
};
use once_cell::sync::Lazy;
use secrecy::ExposeSecret;
use std::net::SocketAddr;
use sysinfo::System;
use tiberius::{AuthMethod, Client, Config};
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio_util::compat::{Compat, TokioAsyncWriteCompatExt};
use tracing::info;
use uuid::Uuid;

#[allow(dead_code)]
pub struct TestApplication {
    pub address: String,
    pub pool: DBConnectionPool,
    pub config: Settings,
    pub db_version: DbVersion,
}

static TRACING: Lazy<()> = Lazy::new(|| {
    initialize_tracing("info".to_string(), "maedic.log".to_string()).unwrap();
});

impl TestApplication {
    pub async fn spawn_app(db_version: DbVersion) -> TestApplication {
        Lazy::force(&TRACING);

        let configuration = {
            let mut c =
                get_configuration("test".to_string()).expect("Failed to read configuration");
            c.database.database_name = Uuid::new_v4().to_string();
            c.application.port = 0;
            c
        };
        let creation_client = create_db_client(&configuration.database, false).await;
        create_database(&configuration.database, creation_client).await;

        let mut migration_client = create_db_client(&configuration.database, true).await;
        configure_database(&mut migration_client, db_version.clone()).await;

        let pool = setup_database_pool(configuration.database.clone())
            .await
            .unwrap();

        let application = TestServer::build(configuration.clone())
            .await
            .expect("Failed to build Application.");
        let address = format!("http://127.0.0.1:{}", application.port());

        let app = TestApplication {
            pool,
            address,
            config: configuration,
            db_version,
        };
        let _handle = tokio::spawn(application.run_until_stopped());
        app
    }
}

/// `TestServer` which sets up a fresh Database for each test
pub struct TestServer {
    port: u16,
    server: Serve<
        TcpListener,
        IntoMakeServiceWithConnectInfo<Router, SocketAddr>,
        AddExtension<Router, ConnectInfo<SocketAddr>>,
    >,
}

impl TestServer {
    pub async fn build(configuration: Settings) -> Result<Self, anyhow::Error> {
        let connection_pool = setup_database_pool(configuration.database.clone())
            .await
            .unwrap();
        let listener = TcpListener::bind(format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        ))
        .await
        .expect("could not bind port");
        let port = listener.local_addr().unwrap().port();
        let (registry, metrics) = setup_metrics_registry().await;
        info!(
            "Starting app on {:?}:{:?}",
            configuration.application.host, configuration.application.port
        );
        //Start the application
        let server = run(
            listener,
            AppState {
                pool: connection_pool,
                config: configuration.clone(),
                sys: System::new_all(),
                registry,
                metrics,
            },
            configuration,
        )
        .await?;
        Ok(Self { port, server })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

/// Client to handle easy requests to the `TestApplication`
pub struct TestClient {
    client: reqwest::Client,
}

impl TestClient {
    pub async fn get_endpoint(&self, address: String, endpoint: &str) -> reqwest::Response {
        self.client
            .get(format!("{}{}", address, endpoint))
            .send()
            .await
            .expect("Failed to execute request")
    }

    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

pub async fn create_db_client(
    db_config: &DatabaseSettings,
    set_db_name: bool,
) -> Client<Compat<tokio::net::TcpStream>> {
    let mut config = Config::new();

    config.host(db_config.host.clone());
    config.port(db_config.port);
    config.authentication(AuthMethod::sql_server(
        db_config.username.clone(),
        db_config.password.expose_secret(),
    ));
    if set_db_name {
        config.database(db_config.database_name.clone());
    }
    if db_config.trust_cert {
        config.trust_cert();
    }
    let tcp = TcpStream::connect(config.get_addr()).await.unwrap();
    tcp.set_nodelay(true).unwrap();
    Client::connect(config.clone(), tcp.compat_write())
        .await
        .unwrap()
}

pub async fn configure_database(
    client: &mut Client<Compat<tokio::net::TcpStream>>,
    db_version: DbVersion,
) {
    match db_version {
        DbVersion::V652SP1 => {
            refinery::embed_migrations!("tests/migrations/V652SP1");
            migrations::runner().run_async(client).await.unwrap();
        }
        DbVersion::V66SP1 => {
            refinery::embed_migrations!("tests/migrations/V66SP1");
            migrations::runner().run_async(client).await.unwrap();
        }
    }
}

pub async fn create_database(
    db_config: &DatabaseSettings,
    mut client: Client<Compat<tokio::net::TcpStream>>,
) {
    let query = format!("CREATE DATABASE \"{}\"", db_config.database_name);
    client.execute(query, &[]).await.unwrap();
    client.close().await.unwrap();
}

#[derive(Clone, Debug)]
pub enum DbVersion {
    /// Build No. 17618
    V652SP1,
    /// Build No. 18204
    V66SP1,
}
