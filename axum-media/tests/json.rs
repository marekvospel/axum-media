use axum::{routing::get, Router};
use axum_media::AnyMedia;
use axum_test_helper::TestClient;
use serde_json::{json, Value};

#[tokio::test]
async fn it_should_serialize_json() {
    let app = Router::new().route(
        "/",
        get(move || async { AnyMedia(json!({ "test": true, "key": "value" })) }),
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
