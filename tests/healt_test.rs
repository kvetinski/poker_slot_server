mod common;
use common::*;

#[tokio::test]
async fn test_health_endpoint() {
    let server = TestServer::new().await;
    let client = make_client().await;

    let response = client
        .get(&server.url("/"))
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), 200);

    let json: serde_json::Value = response.json().await.expect("Failed to parse JSON");
    assert_eq!(json["status"], "ok");
    assert_eq!(json["service"], "poker-server");
    assert_eq!(json["version"], "0.1");
}
