use axum::{response::IntoResponse, routing::get, Router};
use axum_media::AnyMedia;
use axum_test_helper::TestClient;
use serde::Deserialize;
use serde_json::{json, Value};

#[tokio::test]
async fn it_should_serialize_json() {
    let app = Router::new().route(
        "/",
        get(|| async { AnyMedia(json!({ "test": true, "key": "value" })) }),
    );

    let client = TestClient::new(app);
    let res = client.get("/").send().await;

    assert_eq!(
        res.headers().get("content-type").unwrap(),
        "application/json"
    );
    assert_eq!(
        res.json::<Value>().await,
        json!({ "test": true, "key": "value" })
    )
}

#[derive(Deserialize, Clone, Copy)]
struct TestData {
    test: bool,
}

#[tokio::test]
async fn it_should_deserialize_json() {
    async fn handler(AnyMedia(json): AnyMedia<TestData>) -> impl IntoResponse {
        json.test.to_string()
    }

    let app = Router::new().route("/", get(handler));

    let client = TestClient::new(app);
    let res = client
        .get("/")
        .json(&json!({
            "test": true
        }))
        .send()
        .await;

    assert_eq!(res.status(), 200);
    assert_eq!(res.text().await, "true".to_string());
}

#[tokio::test]
async fn it_should_reject_invalid_json() {
    async fn handler(AnyMedia(json): AnyMedia<TestData>) -> impl IntoResponse {
        json.test.to_string()
    }

    let app = Router::new().route("/", get(handler));

    let client = TestClient::new(app);
    let res = client.get("/").body("{ 'test': true }").send().await;

    assert_eq!(res.status(), 400);
    assert_ne!(res.text().await, "true")
}
