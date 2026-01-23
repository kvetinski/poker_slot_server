mod common;
use common::*;
use serde_json::json;

async fn create_test_user(server: &TestServer, client: &reqwest::Client) -> String {
    let response = client
        .post(&server.url("/api/login"))
        .json(&json!({
            "name": "game_test_user"
        }))
        .send()
        .await
        .expect("Failed to send request");

    let json: serde_json::Value = response.json().await.expect("Failed to parse JSON");
    json["id"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn test_start_game_success() {
    let server = TestServer::new().await;
    let client = make_client().await;
    let user_id = create_test_user(&server, &client).await;

    let response = client
        .post(&server.url("/api/start"))
        .json(&json!({
            "user_id": user_id,
            "ante": 10
        }))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 200);

    let json: serde_json::Value = response.json().await.expect("Failed to parse JSON");
    assert!(json["round_id"].is_string());
    assert_eq!(json["cards"].as_array().unwrap().len(), 5);
    assert_eq!(json["wallet"], 990); // 1000 - 10
    assert!(json["win_pool"].is_number());
}

#[tokio::test]
async fn test_start_game_insufficient_wallet() {
    let server = TestServer::new().await;
    let client = make_client().await;
    let user_id = create_test_user(&server, &client).await;

    let response = client
        .post(&server.url("/api/start"))
        .json(&json!({
            "user_id": user_id,
            "ante": 2000
        }))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 400);
}

#[tokio::test]
async fn test_start_game_invalid_user() {
    let server = TestServer::new().await;
    let client = make_client().await;

    let response = client
        .post(&server.url("/api/start"))
        .json(&json!({
            "user_id": "invalid-user-id",
            "ante": 10
        }))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 400);
}
