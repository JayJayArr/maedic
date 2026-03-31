use crate::api::helpers::{TestApplication, TestClient};

#[tokio::test]
async fn test_rate_limiter_on_health_endpoint_works() {
    let app = TestApplication::spawn_app().await;
    let client = TestClient::new();

    //Create 6 quick requests
    for _ in 1..6 {
        client.get_endpoint(app.address.clone(), "/v1/health").await;
    }
    //sixth request should yield a rate limit response
    let response = client.get_endpoint(app.address, "/v1/health").await;

    assert_eq!(response.status(), 429);
}

#[tokio::test]
async fn test_rate_limiter_on_metrics_endpoint_works() {
    let app = TestApplication::spawn_app().await;
    let client = TestClient::new();

    //Create 6 quick requests
    for _ in 1..6 {
        client
            .get_endpoint(app.address.clone(), "/v1/metrics")
            .await;
    }
    //sixth request should yield a rate limit response
    let response = client.get_endpoint(app.address, "/v1/metrics").await;

    assert_eq!(response.status(), 429);
}

#[tokio::test]
async fn test_rate_limiter_on_config_endpoint_works() {
    let app = TestApplication::spawn_app().await;
    let client = TestClient::new();

    //Create 6 quick requests
    for _ in 1..6 {
        client.get_endpoint(app.address.clone(), "/v1/config").await;
    }
    //sixth request should yield a rate limit response
    let response = client.get_endpoint(app.address, "/v1/config").await;

    assert_eq!(response.status(), 429);
}

#[tokio::test]
async fn test_rate_limiter_on_self_endpoint_works() {
    let app = TestApplication::spawn_app().await;
    let client = TestClient::new();

    //Create 6 quick requests
    for _ in 1..6 {
        client.get_endpoint(app.address.clone(), "/v1/self").await;
    }
    //sixth request should yield a rate limit response
    let response = client.get_endpoint(app.address, "/v1/self").await;

    assert_eq!(response.status(), 429);
}

#[tokio::test]
async fn test_rate_limiter_is_endpoint_global() {
    let app = TestApplication::spawn_app().await;
    let client = TestClient::new();

    //Create 6 quick requests
    for _ in 1..6 {
        client.get_endpoint(app.address.clone(), "/v1/health").await;
    }
    //sixth request to another endpoint still yields an error
    let response = client.get_endpoint(app.address, "/v1/metrics").await;

    assert_eq!(response.status(), 429);
}
