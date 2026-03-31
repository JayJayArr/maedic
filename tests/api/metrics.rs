use crate::api::helpers::TestApplication;

#[tokio::test]
async fn test_metrics_database_sizes() {
    let app = TestApplication::spawn_app().await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/v1/metrics", app.address))
        .send()
        .await
        .expect("Failed to execute request");

    assert!(response.status().is_success());
    let text = response
        .text()
        .await
        .expect("Could not convert response to text");

    assert!(text.contains("HELP tablesize Number of database objects"));

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
async fn test_metrics_card_states() {
    let app = TestApplication::spawn_app().await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/v1/metrics", app.address))
        .send()
        .await
        .expect("Failed to execute request");

    assert!(response.status().is_success());
    let text = response
        .text()
        .await
        .expect("Could not convert response to text");

    assert!(text.contains("HELP card_state State of cards"));

    assert!(text.contains("Active"));
    assert!(text.contains("Disabled"));
}
