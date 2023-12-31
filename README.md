# axum-media
[![](https://img.shields.io/badge/marekvospel%2Faxum-media?logo=github&labelColor=555555&color=8da0cb)](https://github.com/marekvospel/axum-media)
[![](https://img.shields.io/crates/v/axum_media.svg?color=fc8d62&logo=rust)](https://crates.io/crates/axum_media)
[![](https://img.shields.io/badge/docs.rs-axum--media-66c2a5?labelColor=555555&logo=docs.rs)](https://docs.rs/axum_media/latest/axum_media)

This library provides a way to use multiple mime types for serializing and
deserializing structs within the axum ecosystem. Inspired by axum's Json
extractor.

```toml
[dependencies]
# Enable features such as urlencoded
axum-media = { version = "0.2.0", features = ["urlencoded"]}
```

## Example

```rust
use axum_media::{AnyMedia, ContentType};

#[tokio::main]
async fn main() {
  let app = axum::Router::new()
    .route("/", get(index))
    .route("/login", post(login))
}

async fn index(content_type: ContentType) -> impl IntoResponse {
  // Chooses the right serializer based on the Accept header
  AnyMedia(
    serde_json::json!({
      "routes": ["/", "/login"],
    }),
    content_type,
  )
}

#[derive(Deserialize)]
struct LoginData {
  email: String
  password: String
}

// Automatically chooses the right deserializer based on the Content-Type header
async fn login(AnyMedia(data, _): AnyMedia<LoginData>) -> String {
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