use crate::api::helpers::TestApplication;
#[tokio::test]
async fn test_metrics_endpoint_works() {
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

    assert!(text.contains("Number of database objects"));
    assert!(text.contains("HiQueue"));
}
