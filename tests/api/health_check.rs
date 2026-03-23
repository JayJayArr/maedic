use maedic::{
    configuration::LimitSettings,
    database::{DatabaseConnectionState, MaedicHealth},
    indicators::PWHealth,
};

use crate::api::helpers::spawn_app;

#[tokio::test]
async fn test_self_health_works() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/v1/self", app.address))
        .send()
        .await
        .expect("Failed to execute request");
    assert!(response.status().is_success());
    assert_eq!(response.content_length(), Some(58));
    let json = response.json::<MaedicHealth>().await.unwrap();
    let perfect_health: MaedicHealth = MaedicHealth {
        database_connection: DatabaseConnectionState::Healthy,
        version_number: env!("CARGO_PKG_VERSION").to_string(),
    };

    assert_eq!(json, perfect_health);
    assert!(json.to_string().contains("healthy"));
}

#[tokio::test]
async fn test_config_endpoint_works() {
    let app = spawn_app().await;
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
async fn test_pw_health_endpoint_works_with_db() {
    let app = spawn_app().await;
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
    };
    assert_eq!(json, perfect_health)
}
