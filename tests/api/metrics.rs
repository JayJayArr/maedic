use crate::api::helpers::spawn_app;

#[tokio::test]
async fn test_config_endpoint_works() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/v1/metrics", app.address))
        .send()
        .await
        .expect("Failed to execute request");

    assert!(response.status().is_success());
}
