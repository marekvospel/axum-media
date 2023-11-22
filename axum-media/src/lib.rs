//! [![github]](https://github.com/marekvospel/axum-media)
//! [![crates.io]](https://crates.io/crates/axum_media)
//! [![docs.rs]](https://docs.rs/axum_media/latest/axum_media)
//!
//!
//! [github]: https://img.shields.io/badge/github-8da0cb?labelColor=555555&logo=github
//! [crates.io]: https://img.shields.io/crates/v/axum_media.svg?color=fc8d62&logo=rust
//! [docs.rs]: https://img.shields.io/badge/docs.rs-axum--media-66c2a5?labelColor=555555&logo=docs.rs
//!
//! <br>
//!
//! This crate provides a simple way to use multiple mime types for serializing and
//! deserializing structs within the axum ecosystem. Inspired by axum's Json
//! extractor.
//!
//!
//! ## Example

//! ```rust,no_run
//! use axum_media::{AnyMedia, Accept};

//! #[tokio::main]
//! async fn main() {
//!   let app = axum::Router::new()
//!     .route("/", axum::routing::get(index))
//!     .route("/login", axum::routing::post(login));
//!
//!   axum::Server::bind(&"127.0.0.1:3000".parse().unwrap())
//!     .serve(app.into_make_service())
//!     .await.unwrap();
//! }
//!
//! async fn index(accept: Accept) -> impl axum::response::IntoResponse {
//!   // Chooses the right serializer based on the Accept header
//!   AnyMedia(
//!     serde_json::json!({
//!       "routes": ["/", "/login"],
//!     }),
//!     accept.into(),
//!   )
//! }
//!
//! #[derive(serde::Deserialize)]
//! struct LoginData {
//!   email: String,
//!   password: String,
//! }
//!
//! // Automatically chooses the right deserializer based on the Content-Type header
//! async fn login(AnyMedia(data, _): AnyMedia<LoginData>) -> String {
//!   data.email
//! }
//!
//! ```

pub(crate) use axum::{
    extract::rejection::BytesRejection,
    http::{header, StatusCode},
    response::IntoResponse,
};

pub(crate) mod accept;
pub(crate) mod anymedia;
pub(crate) mod mimetypes;

pub use accept::Accept;
pub use anymedia::AnyMedia;

#[derive(Debug, thiserror::Error)]
pub enum AnyMediaRejection {
    #[error("Failed to deserialize the JSON body into the target type: {0}")]
    JsonDataError(serde_path_to_error::Error<serde_json::Error>),
    #[error("Failed to parse the request body as JSON: {0}")]
    JsonSyntaxError(serde_path_to_error::Error<serde_json::Error>),
    #[error("{0}")]
    BytesRejection(#[from] BytesRejection),
    #[cfg(feature = "urlencoded")]
    #[error("{0}")]
    UrlEncodedError(#[from] serde_urlencoded::de::Error),
}

impl IntoResponse for AnyMediaRejection {
    fn into_response(self) -> axum::response::Response {
        (
            StatusCode::BAD_REQUEST,
            [(header::CONTENT_TYPE, mime::UTF_8.to_string())],
            format!("{self}"),
        )
            .into_response()
    }
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum AnyMediaSerializeError {
    #[error("{0}")]
    JsonError(#[from] serde_json::Error),
    #[cfg(feature = "urlencoded")]
    #[error("{0}")]
    UrlEncodedError(#[from] serde_urlencoded::ser::Error),
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum AnyMediaDeserializeError {
    #[error("{0}")]
    JsonError(#[from] serde_path_to_error::Error<serde_json::Error>),
    #[cfg(feature = "urlencoded")]
    #[error("{0}")]
    UrlEncodedError(#[from] serde_urlencoded::de::Error),
}
