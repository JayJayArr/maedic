use crate::api::helpers::{DbVersion, TestApplication, TestClient};
use rstest::rstest;

#[tokio::test]
async fn test_rate_limiter_is_global() {
    let app = TestApplication::spawn_app(DbVersion::V652).await;
    let client = TestClient::new();

    //Create 5 quick requests
    for _ in 1..5 {
        let response = client.get_endpoint(app.address.clone(), "/v1/health").await;
        assert_eq!(response.status(), 200);
    }
    //sixth request to another endpoint still yields an error
    let response = client.get_endpoint(app.address, "/v1/metrics").await;

    assert_eq!(response.status(), 429);
}
#[rstest]
#[case("/v1/health")]
#[case("/v1/metrics")]
#[case("/v1/config")]
#[tokio::test]
async fn test_rate_limiter_is_applied_to_endpoint(#[case] endpoint: &str) {
    let app = TestApplication::spawn_app(DbVersion::V652).await;
    let client = TestClient::new();

    //Create 5 quick requests
    for _ in 1..5 {
        let response = client.get_endpoint(app.address.clone(), endpoint).await;
        assert_eq!(response.status(), 200);
    }
    //sixth request to another endpoint still yields an error
    let response = client.get_endpoint(app.address, endpoint).await;

    assert_eq!(response.status(), 429);
}
