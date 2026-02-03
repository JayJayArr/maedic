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
    // dbg!(&response);

    assert!(response.status().is_success());
    assert_eq!(response.content_length(), Some(58));
}
