mod common;
use common::*;
use serde_json::json;

#[tokio::test]
async fn test_login_success() {
    let server = TestServer::new().await;
    let client = make_client().await;

    let response = client
        .post(&server.url("/api/login"))
        .json(&json!({
            "name": "test_user"
        }))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 200);

    let json: serde_json::Value = response.json().await.expect("Failed to parse JSON");
    assert!(json["id"].is_string());
    assert_eq!(json["name"], "test_user");
    assert_eq!(json["wallet"], 1000);
}

#[tokio::test]
async fn test_login_duplicate_user() {
    let server = TestServer::new().await;
    let client = make_client().await;

    // First login should succeed
    let response1 = client
        .post(&server.url("/api/login"))
        .json(&json!({
            "name": "duplicate_user"
        }))
        .send()
        .await
        .expect("Failed to send request");
    assert_eq!(response1.status(), 200);

    // Second login with same name should fail
    let response2 = client
        .post(&server.url("/api/login"))
        .json(&json!({
            "name": "duplicate_user"
        }))
        .send()
        .await
        .expect("Failed to send request");
    assert_eq!(response2.status(), 400);
}
