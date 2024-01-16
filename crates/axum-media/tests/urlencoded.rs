use crate::utils::string_body;
use axum::{body::Body, http::Request, response::IntoResponse, routing::get, Router};
use axum_media::{Accept, AnyMedia};
use serde::{Deserialize, Serialize};
use tower::ServiceExt;

mod utils;

#[derive(Serialize, Deserialize)]
struct TestData {
    test: bool,
}

#[tokio::test]
async fn it_should_serialize_urlencoded() {
    async fn handler(accept: Accept) -> impl IntoResponse {
        AnyMedia(TestData { test: true }, accept.into())
    }

    let app: Router = Router::new().route("/", get(handler));

    let res = app
        .oneshot(
            Request::builder()
                .uri("/")
                .header("Accept", "application/x-www-form-urlencoded")
                .body(Body::from(()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    assert_eq!(string_body(res).await.unwrap(), "test=true")
}

#[tokio::test]
async fn it_should_deserialize_urlencoded() {
    async fn handler(AnyMedia(data, _): AnyMedia<TestData>) -> impl IntoResponse {
        data.test.to_string()
    }

    let app: Router = Router::new().route("/", get(handler));

    let res = app
        .oneshot(
            Request::builder()
                .uri("/")
                .header("Content-Type", "application/x-www-form-urlencoded")
                .body(Body::from("test=true"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    assert_eq!(string_body(res).await.unwrap(), "true")
}
