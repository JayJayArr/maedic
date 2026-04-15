use crate::api::helpers::{DbVersion, TestApplication};
use maedic::{
    configuration::LimitSettings,
    database::DatabaseConnectionState,
    health::{MaedicHealth, PWHealth},
};
use rstest::rstest;

#[tokio::test]
#[rstest]
#[case(DbVersion::V652)]
async fn test_config_endpoint_works(#[case] db_version: DbVersion) {
    let app = TestApplication::spawn_app(db_version).await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/v1/config", app.address))
        .send()
        .await
        .expect("Failed to execute request");

    assert!(response.status().is_success());
    let json = response.json::<LimitSettings>().await.unwrap();
    let limit_config = app.config.limits.clone();

    assert_eq!(json, limit_config);
}

#[tokio::test]
#[rstest]
#[case(DbVersion::V652)]
async fn test_pw_health_endpoint_works_with_db(#[case] db_version: DbVersion) {
    let app = TestApplication::spawn_app(db_version).await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/v1/health", app.address))
        .send()
        .await
        .expect("Failed to execute request");

    assert!(response.status().is_success());
    let json = response.json::<PWHealth>().await.unwrap();
    let perfect_health: PWHealth = PWHealth {
        unhealthy_spool_files: Some(Vec::new()),
        hi_queue_size: Some(0),
        global_cpu_usage_percentage: None,
        used_memory_percentage: None,
        service_state: None,
        maedic_health: MaedicHealth {
            database_connection: DatabaseConnectionState::Healthy,
            version_number: env!("CARGO_PKG_VERSION").to_string(),
        },
    };
    assert_eq!(json, perfect_health)
}

#[tokio::test]
#[rstest]
#[case(DbVersion::V652)]
async fn test_version_number_is_correct(#[case] db_version: DbVersion) {
    let app = TestApplication::spawn_app(db_version).await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/v1/health", app.address))
        .send()
        .await
        .expect("Failed to execute request");

    assert!(response.status().is_success());

    let version_number = env!("CARGO_PKG_VERSION").to_string();
    let text = response.text().await.unwrap();

    assert!(text.contains(&version_number))
}
