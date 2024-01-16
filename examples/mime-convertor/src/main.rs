use axum::{http::HeaderMap, response::IntoResponse, routing::post, Router};
use axum_media::AnyMedia;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let app = Router::new().route("/convert", post(handler));

    let tcp = TcpListener::bind("127.0.0.1:3000").await.unwrap();
    axum::serve(tcp, app.into_make_service()).await.unwrap()
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
