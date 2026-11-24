use maedic::configuration::{ApplicationSettings, LimitSettings};
use rstest::rstest;
use serde_json::Value;

use crate::api::helpers::{DbVersion, TestApplication};
#[tokio::test]
#[rstest]
#[case(DbVersion::NoMigration)]
async fn test_config_contains_no_password(#[case] db_version: DbVersion) {
    let app = TestApplication::spawn_app(db_version).await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/v1/config", app.address))
        .send()
        .await
        .expect("Failed to execute request");

    assert!(response.status().is_success());
    //this makes sure that the exposed config are valid Settings
    let text = response.text().await.unwrap();
    assert!(!text.contains("password"));
}

#[tokio::test]
#[rstest]
#[case(DbVersion::NoMigration)]
async fn test_config_endpoint_works(#[case] db_version: DbVersion) {
    let app = TestApplication::spawn_app(db_version).await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/v1/config", app.address))
        .send()
        .await
        .expect("Failed to execute request");

    assert!(response.status().is_success());
    //this makes sure that the exposed config are valid Settings
    let json = response.json::<Value>().await.unwrap();
    let application_settings: ApplicationSettings =
        serde_json::from_value(json.get("application").unwrap().clone()).unwrap();
    let limit_settings: LimitSettings =
        serde_json::from_value(json.get("limits").unwrap().clone()).unwrap();
    let config = app.config.clone();
    assert_eq!(application_settings, config.application);
    assert_eq!(limit_settings, config.limits);
}
