//! [![github]](https://github.com/marekvospel/axum-media)
//!
//! [github]: https://img.shields.io/badge/github-8da0cb?labelColor=555555&logo=github
//!
//! <br>
//!
//! This crate provides a simple way to use multiple mime types for serializing and
//! deserializing structs within the axum ecosystem. Inspired by axum's Json
//! extractor.
//!
//!
//! ## Example

//! ```rust
//! use axum_media::AnyMedia;

//! #[tokio::main]
//! async fn main() {
//!   let app = axum::Router::new()
//!     .route("/", axum::routing::get(index));
//!     //.route("/login", axum::routing::post(login))
//!   axum::Server::bind(&"127.0.0.1:3000".parse().unwrap())
//!     .serve(app.into_make_service());
//!  }
//!
//! async fn index(headers: axum::http::HeaderMap) -> impl axum::response::IntoResponse {
//!   AnyMedia(serde_json::json!({
//!     "routes": ["/", "/login"]
//!    }))
//!    // Chooses the right serializer based on the Accept header
//!      .with_mime_str(
//!        headers
//!          .get("accept")
//!          .map(|v| v.to_str().unwrap_or(""))
//!          .unwrap_or(""),
//!      )
//! }
//!
//! #[derive(serde::Deserialize)]
//! struct LoginData {
//!   email: String,
//!   password: String,
//! }
//!
//! // Automatically chooses the right deserializer based on the Content-Type header (To be implemented)
//! // async fn login(AnyMedia(data): AnyMedia<LoginData>) -> String {
//! //   data.email
//! // }
//!
//! ```

pub(crate) use axum::{
    http::{header, StatusCode},
    response::IntoResponse,
};
pub(crate) use bytes::{BufMut, BytesMut};
pub(crate) use mime::Mime;
pub(crate) use serde::Serialize;

pub(crate) mod mimetypes;

#[derive(Debug, Clone, Default)]
pub struct AnyMedia<T>(pub T);

impl<T> AnyMedia<T> {
    pub fn with_mime_str(self, mime: &str) -> AnyMediaIntoResponse<T> {
        let mime = mime.parse().ok();

        AnyMediaIntoResponse {
            data: self.0,
            mime: mime,
        }
    }

    pub fn with_mime(self, mime: Mime) -> AnyMediaIntoResponse<T> {
        AnyMediaIntoResponse {
            data: self.0,
            mime: Some(mime),
        }
    }
}

#[derive(Clone)]
pub struct AnyMediaIntoResponse<T> {
    data: T,
    mime: Option<Mime>,
}

impl<T> IntoResponse for AnyMedia<T>
where
    T: Serialize,
{
    fn into_response(self) -> axum::response::Response {
        AnyMediaIntoResponse {
            data: self.0,
            mime: None,
        }
        .into_response()
    }
}

impl<T> IntoResponse for AnyMediaIntoResponse<T>
where
    T: Serialize,
{
    fn into_response(self) -> axum::response::Response {
        let mime = self.mime.unwrap_or(mime::APPLICATION_JSON);
        let mut buf = BytesMut::with_capacity(128).writer();

        let mut result: Option<Result<(), AnyMediaError>> = match (mime.type_(), mime.subtype()) {
            (mime::APPLICATION, mime::JSON) => {
                Some(mimetypes::serialize_json(&self.data, &mut buf))
            }
            #[cfg(feature = "urlencoded")]
            (mime::APPLICATION, mime::WWW_FORM_URLENCODED) => {
                Some(mimetypes::serialize_urlencoded(&self.data, &mut buf))
            }
            _ => None,
        };

        if let None = result {
            result = match (mime.type_(), mime.suffix()) {
                #[cfg(feature = "urlencoded")]
                (mime::APPLICATION, Some(mime::WWW_FORM_URLENCODED)) => {
                    Some(mimetypes::serialize_urlencoded(&self.data, &mut buf))
                }
                _ => Some(mimetypes::serialize_json(&self.data, &mut buf)),
            }
        }

        if let Err(_) = result.unwrap() {
            // TODO: handle error properly?
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                [(header::CONTENT_TYPE, mime.to_string())],
            )
                .into_response();
        }

        (
            [(header::CONTENT_TYPE, mime.to_string())],
            buf.into_inner().freeze(),
        )
            .into_response()
    }
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum AnyMediaError {
    #[error("{0}")]
    JsonError(#[from] serde_json::Error),
    #[cfg(feature = "urlencoded")]
    #[error("{0}")]
    UrlEncodedError(#[from] serde_urlencoded::ser::Error),
}
