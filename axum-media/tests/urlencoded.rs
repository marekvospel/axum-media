use axum::{response::IntoResponse, routing::get, Router};
use axum_media::{Accept, AnyMedia};
use axum_test_helper::TestClient;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct TestData {
    test: bool,
}

#[tokio::test]
async fn it_should_serialize_urlencoded() {
    async fn handler(accept: Accept) -> impl IntoResponse {
        AnyMedia(TestData { test: true }, accept.into())
    }

    let app = Router::new().route("/", get(handler));

    let client = TestClient::new(app);
    let res = client
        .get("/")
        .header("Accept", "application/x-www-form-urlencoded")
        .send()
        .await;

    assert_eq!(res.status(), 200);
    assert_eq!(res.text().await, "test=true")
}

#[tokio::test]
async fn it_should_deserialize_urlencoded() {
    async fn handler(AnyMedia(data, _): AnyMedia<TestData>) -> impl IntoResponse {
        data.test.to_string()
    }

    let app = Router::new().route("/", get(handler));

    let client = TestClient::new(app);
    let res = client
        .get("/")
        .body("test=true")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .send()
        .await;

    assert_eq!(res.status(), 200);
    assert_eq!(res.text().await, "true")
}
