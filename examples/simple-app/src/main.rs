use axum::{http::HeaderMap, response::IntoResponse, routing::get, Router, Server};
use axum_media::AnyMedia;
use serde_json::json;

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(handler));

    Server::bind(&"127.0.0.1:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn handler(headers: HeaderMap) -> impl IntoResponse {
    AnyMedia(json!({ "test": true, "key": "value" })).with_mime_str(
        headers
            .get("accept")
            .map(|v| v.to_str().unwrap_or(""))
            .unwrap_or(""),
    )
}
