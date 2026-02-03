use maedic::database::MaedicHealth;

use crate::helpers::spawn_app;

#[tokio::test]
async fn health_check_works() {
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
    assert!(json.to_string().contains("healthy"));
}

#[tokio::test]
async fn config_endpoint_works() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/v1/config", app.address))
        .send()
        .await
        .expect("Failed to execute request");

    assert!(response.status().is_success());
}
