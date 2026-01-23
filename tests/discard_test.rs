mod common;
use common::*;
use serde_json::json;

async fn create_game(server: &TestServer, client: &reqwest::Client) -> (String, String) {
    let response = client
        .post(&server.url("/api/login"))
        .json(&json!({
            "name": "discard_test_user"
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
async fn test_discard_success() {
    let server = TestServer::new().await;
    let client = make_client().await;
    let (user_id, round_id) = create_game(&server, &client).await;

    let response = client
        .post(&server.url("/api/discard"))
        .json(&json!({
            "user_id": user_id,
            "round_id": round_id,
            "discard_indices": [0, 2]
        }))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 200);

    let json: serde_json::Value = response.json().await.expect("Failed to parse JSON");
    assert_eq!(json["cards"].as_array().unwrap().len(), 5);
    assert_eq!(json["wallet"], 980); // 1000 - 10 (ante) - 10 (2 cards * 0.5*10 ante = 10)
    assert_eq!(json["total_bet"], 10);
}

#[tokio::test]
async fn test_discard_invalid_round() {
    let server = TestServer::new().await;
    let client = make_client().await;
    let (user_id, _) = create_game(&server, &client).await;

    let response = client
        .post(&server.url("/api/discard"))
        .json(&json!({
            "user_id": user_id,
            "round_id": "invalid-round-id",
            "discard_indices": [0, 1]
        }))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 400);
}
