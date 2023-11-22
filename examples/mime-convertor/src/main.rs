use axum::{http::HeaderMap, response::IntoResponse, routing::post, Router, Server};
use axum_media::AnyMedia;

#[tokio::main]
async fn main() {
    let app = Router::new().route("/convert", post(handler));

    Server::bind(&"127.0.0.1:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

// Takes any Content-Type body and converts it to a different mime type based on the Accept header.
async fn handler(
    headers: HeaderMap,
    AnyMedia(value, _): AnyMedia<serde_json::Value>,
) -> impl IntoResponse {
    AnyMedia(
        value,
        headers
            .get("accept")
            .map(|v| v.to_str().unwrap_or(""))
            .unwrap_or("")
            .to_owned()
            .into(),
    )
}
