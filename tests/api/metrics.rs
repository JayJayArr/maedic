use crate::api::helpers::{DbVersion, TestApplication, TestClient};
use rstest::rstest;

#[tokio::test]
#[rstest]
#[case(DbVersion::V652SP1)]
#[case(DbVersion::V66SP1)]
async fn test_metrics_database_sizes(#[case] db_version: DbVersion) {
    let app = TestApplication::spawn_app(db_version).await;
    let client = TestClient::new();

    let response = client.get_endpoint(app.address, "/v1/metrics").await;

    assert!(response.status().is_success());
    let text = response
        .text()
        .await
        .expect("Could not convert response to text");

    assert!(text.contains("# HELP maedic_tablesize Number of database objects."));

    assert!(text.contains("Badges"));
    assert!(text.contains("Cards"));
    assert!(text.contains("Panels"));
    assert!(text.contains("Channels"));
    assert!(text.contains("Subpanels"));
    assert!(text.contains("Readers"));
    assert!(text.contains("HiQueue"));
    assert!(text.contains("UnackAlarms"));
    assert!(text.contains("Events"));
    assert!(text.contains("Users"));
    assert!(text.contains("Workstations"));
}

#[tokio::test]
#[rstest]
#[case(DbVersion::V652SP1)]
#[case(DbVersion::V66SP1)]
async fn test_metrics_card_states(#[case] db_version: DbVersion) {
    let app = TestApplication::spawn_app(db_version).await;
    let client = TestClient::new();

    let response = client.get_endpoint(app.address, "/v1/metrics").await;

    assert!(response.status().is_success());
    let text = response
        .text()
        .await
        .expect("Could not convert response to text");

    assert!(text.contains("# HELP maedic_card_state State of cards."));

    assert!(text.contains("Active"));
    assert!(text.contains("Disabled"));
    assert!(text.contains("AutoDisabled"));
    assert!(text.contains("Expired"));
    assert!(text.contains("Lost"));
    assert!(text.contains("Stolen"));
    assert!(text.contains("Terminated"));
    assert!(text.contains("Unaccounted"));
    assert!(text.contains("Void"));
}

#[tokio::test]
#[rstest]
#[case(DbVersion::V652SP1)]
#[case(DbVersion::V66SP1)]
async fn test_metrics_version_numbers(#[case] db_version: DbVersion) {
    let app = TestApplication::spawn_app(db_version).await;
    let client = TestClient::new();

    let response = client.get_endpoint(app.address, "/v1/metrics").await;

    assert!(response.status().is_success());
    let text = response
        .text()
        .await
        .expect("Could not convert response to text");

    assert!(text.contains("# HELP maedic_pw_version_number Version numbers."));

    assert!(text.contains("Major"));
    assert!(text.contains("Minor"));
    assert!(text.contains("Patch"));
    assert!(text.contains("BuildNo"));
}

#[tokio::test]
#[rstest]
#[case(DbVersion::V652SP1)]
#[case(DbVersion::V66SP1)]
async fn test_metrics_panel_installed(#[case] db_version: DbVersion) {
    let app = TestApplication::spawn_app(db_version).await;
    let client = TestClient::new();

    let response = client.get_endpoint(app.address, "/v1/metrics").await;

    assert!(response.status().is_success());
    let text = response
        .text()
        .await
        .expect("Could not convert response to text");

    assert!(text.contains("# TYPE maedic_panel_installed gauge"));

    assert!(text.contains("panel"));
    assert!(text.contains("major_version"));
    assert!(text.contains("minor_version"));
}

#[tokio::test]
#[rstest]
#[case(DbVersion::V652SP1)]
#[case(DbVersion::V66SP1)]
async fn test_metrics_content_type(#[case] db_version: DbVersion) {
    let app = TestApplication::spawn_app(db_version).await;
    let client = TestClient::new();

    let response = client.get_endpoint(app.address, "/v1/metrics").await;

    assert!(response.status().is_success());
    let headermap = response.headers();
    let content_type = headermap.get("content-type").unwrap();
    assert_eq!(content_type, "text/plain; version=0.0.4; charset=utf-8");
}
