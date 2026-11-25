use rstest::rstest;

use crate::api::helpers::{DbVersion, TestApplication};
#[tokio::test]
#[rstest]
#[case(DbVersion::NoMigration, "/livez")]
#[case(DbVersion::NoMigration, "/readyz")]
async fn test_health_endpoints_return_200_ok(
    #[case] db_version: DbVersion,
    #[case] endpoint: &str,
) {
    let app = TestApplication::spawn_app(db_version).await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}{}", app.address, endpoint))
        .send()
        .await
        .expect("Failed to execute request");

    assert!(response.status().is_success());
}
