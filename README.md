# axum-media

This library provides a way to use multiple mime types for serializing and
deserializing structs within the axum ecosystem. Inspired by axum's Json
extractor.

## Example

```rs
use axum_media::AnyMedia;

#[tokio::main]
async fn main() {
  let app = axum::Router::new()
    .route("/", get(index))
    .route("/login", post(login))
}

async fn index(headers: HeaderMap) -> impl IntoResponse {
  AnyMedia(serde_json::json!({
    "routes": ["/", "/login"]
  }))
  // Chooses the right serializer based on the Accept header
    .with_mime_str(
      headers
        .get("accept")
        .map(|v| v.to_str().unwrap_or(""))
        .unwrap_or(""),
    )
}

#[derive(Deserialize)]
struct LoginData {
  email: String
  password: String
}

// Automatically chooses the right deserializer based on the Content-Type header (To be implemented)
async fn login(AnyMedia(data): AnyMedia<LoginData>) -> String {
  data.email
}

```

#### License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
</sub>