mod common;
use common::*;
use serde_json::json;

async fn create_and_discard(server: &TestServer, client: &reqwest::Client) -> (String, String) {
    let response = client
        .post(&server.url("/api/login"))
        .json(&json!({
            "name": "reveal_test_user"
        }))
        .send()
        .await
        .expect("Failed to send request");

    let json: serde_json::Value = response.json().await.expect("Failed to parse JSON");
    let user_id = json["id"].as_str().unwrap().to_string();

    let response = client
        .post(&server.url("/api/start"))
        .json(&json!({
            "user_id": &user_id,
            "ante": 10
        }))
        .send()
        .await
        .expect("Failed to send request");

    let json: serde_json::Value = response.json().await.expect("Failed to parse JSON");
    let round_id = json["round_id"].as_str().unwrap().to_string();

    (user_id, round_id)
}

#[tokio::test]
async fn test_reveal_success() {
    let server = TestServer::new().await;
    let client = make_client().await;
    let (user_id, round_id) = create_and_discard(&server, &client).await;

    let response = client
        .post(&server.url("/api/reveal"))
        .json(&json!({
            "user_id": user_id,
            "round_id": round_id
        }))
        .send()
        .await
        .expect("Failed to send request");

    // Reveal should always succeed for valid round
    assert!(response.status().is_success());

    let json: serde_json::Value = response.json().await.expect("Failed to parse JSON");
    assert!(json["hand_rank"].is_string());
    assert!(json["multiplier"].is_number());
    assert!(json["payout"].is_number());
    assert!(json["wallet"].is_number());
    assert!(json["win_pool"].is_number());
    assert!(json["house_profit"].is_number());
}

#[tokio::test]
async fn test_reveal_twice_fails() {
    let server = TestServer::new().await;
    let client = make_client().await;
    let (user_id, round_id) = create_and_discard(&server, &client).await;

    // First reveal should succeed
    let response1 = client
        .post(&server.url("/api/reveal"))
        .json(&json!({
            "user_id": user_id,
            "round_id": round_id
        }))
        .send()
        .await
        .expect("Failed to send request");
    assert!(response1.status().is_success());

    // Second reveal should fail
    let response2 = client
        .post(&server.url("/api/reveal"))
        .json(&json!({
            "user_id": user_id,
            "round_id": round_id
        }))
        .send()
        .await
        .expect("Failed to send request");
    assert_eq!(response2.status(), 400);
}
