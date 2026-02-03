use maedic::{
    configuration::{DBConnectionPool, DatabaseSettings, Settings, get_configuration},
    database::setup_database_pool,
    run::Application,
    telemetry::initialize_tracing,
};
use once_cell::sync::Lazy;
use secrecy::ExposeSecret;
use tiberius::{AuthMethod, Client, Config};
use tokio::net::TcpStream;
use tokio_util::compat::{Compat, TokioAsyncWriteCompatExt};
use uuid::Uuid;

#[allow(dead_code)]
pub struct TestApp {
    pub address: String,
    pub port: u16,
    pub pool: DBConnectionPool,
    pub config: Settings,
}

static TRACING: Lazy<()> = Lazy::new(|| {
    initialize_tracing().unwrap();
});

pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    let configuration = {
        let mut c = get_configuration("test".to_string()).expect("Failed to read configuration");
        c.database.database_name = Uuid::new_v4().to_string();
        c.application.port = 0;
        c
    };

    create_database(&configuration.database).await;

    let pool = setup_database_pool(configuration.database.clone())
        .await
        .unwrap();

    let application = Application::build(configuration.clone())
        .await
        .expect("Failed to build Application.");
    let application_port = application.port();
    let address = format!("http://127.0.0.1:{}", application.port());
    let _handle = tokio::spawn(application.run_until_stopped());

    TestApp {
        pool,
        port: application_port,
        address,
        config: configuration,
    }
}

mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("migrations");
}

pub async fn configure_database(mut client: Client<Compat<tokio::net::TcpStream>>) {
    embedded::migrations::runner()
        .run_async(&mut client)
        .await
        .unwrap();
}

pub async fn create_database(db_config: &DatabaseSettings) {
    let mut config = Config::new();

    config.host(db_config.host.clone());
    config.port(db_config.port);
    config.authentication(AuthMethod::sql_server(
        db_config.username.clone(),
        db_config.password.expose_secret(),
    ));
    if db_config.trust_cert {
        config.trust_cert();
    }
    {
        let tcp = TcpStream::connect(config.get_addr()).await.unwrap();
        tcp.set_nodelay(true).unwrap();
        let mut client = Client::connect(config.clone(), tcp.compat_write())
            .await
            .unwrap();

        let query = format!("CREATE DATABASE \"{}\"", db_config.database_name);
        client.execute(query, &[]).await.unwrap();
        client.close().await.unwrap();
    }
    config.database(db_config.database_name.clone());
    let tcp = TcpStream::connect(config.get_addr()).await.unwrap();
    let client = Client::connect(config.clone(), tcp.compat_write())
        .await
        .unwrap();

    configure_database(client).await;
}
