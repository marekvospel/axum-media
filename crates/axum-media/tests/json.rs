use crate::utils::{json_body, string_body};
use axum::{body::Body, http::Request, response::IntoResponse, routing::get, Router};
use axum_media::AnyMedia;
use serde::Deserialize;
use serde_json::json;
use tower::ServiceExt;

mod utils;

#[tokio::test]
async fn it_should_serialize_json() {
    let app: Router = Router::new().route(
        "/",
        get(|| async { AnyMedia(json!({ "test": true, "key": "value" }), None.into()) }),
    );

    let res = app
        .oneshot(Request::builder().uri("/").body(Body::from(())).unwrap())
        .await
        .unwrap();

    assert_eq!(
        res.headers().get("content-type").unwrap(),
        "application/json"
    );
    assert_eq!(
        json_body::<serde_json::Value>(res).await.unwrap(),
        json!({ "test": true, "key": "value" })
    )
}

#[derive(Deserialize, Clone, Copy)]
struct TestData {
    test: bool,
}

#[tokio::test]
async fn it_should_deserialize_json() {
    async fn handler(AnyMedia(json, _): AnyMedia<TestData>) -> impl IntoResponse {
        json.test.to_string()
    }

    let app: Router = Router::new().route("/", get(handler));

    let res = app
        .oneshot(
            Request::builder()
                .uri("/")
                .body(Body::from(json!({ "test": true }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    assert_eq!(string_body(res).await.unwrap(), "true".to_string());
}

#[tokio::test]
async fn it_should_reject_invalid_json() {
    async fn handler(AnyMedia(json, _): AnyMedia<TestData>) -> impl IntoResponse {
        json.test.to_string()
    }

    let app: Router = Router::new().route("/", get(handler));

    let res = app
        .oneshot(
            Request::builder()
                .uri("/")
                .body(Body::from("{ 'test': true }"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), 400);
    assert_ne!(string_body(res).await.unwrap(), "true");
}
