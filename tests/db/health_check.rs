use crate::helpers::spawn_app;
use maedic::database::MaedicHealth;

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
}
