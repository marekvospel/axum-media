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

//! ```rust
//! use axum_media::AnyMedia;

//! #[tokio::main]
//! async fn main() {
//!   let app = axum::Router::new()
//!     .route("/", axum::routing::get(index))
//!     .route("/login", axum::routing::post(login));
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
//! // Automatically chooses the right deserializer based on the Content-Type header
//! async fn login(AnyMedia(data): AnyMedia<LoginData>) -> String {
//!   data.email
//! }
//!
//! ```

pub(crate) use axum::{
    body::HttpBody,
    extract::FromRequest,
    http::{HeaderValue, Request},
    BoxError,
};
pub(crate) use axum::{
    http::{header, StatusCode},
    response::IntoResponse,
};
use bytes::Bytes;
pub(crate) use bytes::{BufMut, BytesMut};
pub(crate) use error::{AnyMediaDeserializeError, AnyMediaSerializeError};
pub(crate) use mime::Mime;
pub(crate) use serde::Serialize;
pub(crate) use tracing::error;

pub(crate) mod error;
pub(crate) mod mimetypes;

// Re-export
pub use error::AnyMediaRejection;

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

        let mut result: Option<Result<(), AnyMediaSerializeError>> =
            match (mime.type_(), mime.subtype().as_str()) {
                (mime::APPLICATION, "json") => {
                    Some(mimetypes::serialize_json(&self.data, &mut buf))
                }
                #[cfg(feature = "urlencoded")]
                (mime::APPLICATION, "x-www-form-urlencoded") => {
                    Some(mimetypes::serialize_urlencoded(&self.data, &mut buf))
                }
                #[cfg(feature = "yaml")]
                (mime::APPLICATION, "yaml") => {
                    Some(mimetypes::serialize_yaml(&self.data, &mut buf))
                }
                #[cfg(feature = "xml")]
                (mime::APPLICATION, "xml") => Some(mimetypes::serialize_xml(&self.data, &mut buf)),
                _ => None,
            };

        if let None = result {
            result = match (mime.type_(), mime.suffix().map(|m| m.as_str())) {
                #[cfg(feature = "urlencoded")]
                (mime::APPLICATION, Some("x-www-form-urlencoded")) => {
                    Some(mimetypes::serialize_urlencoded(&self.data, &mut buf))
                }
                #[cfg(feature = "yaml")]
                (mime::APPLICATION, Some("yaml")) => {
                    Some(mimetypes::serialize_yaml(&self.data, &mut buf))
                }
                #[cfg(feature = "xml")]
                (mime::APPLICATION, Some("xml")) => {
                    Some(mimetypes::serialize_xml(&self.data, &mut buf))
                }
                _ => Some(mimetypes::serialize_json(&self.data, &mut buf)),
            }
        }

        if let Err(err) = result.unwrap() {
            error!("{}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                [(
                    header::CONTENT_TYPE,
                    HeaderValue::from_static(mime::TEXT_PLAIN_UTF_8.as_ref()),
                )],
                err.to_string(),
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

#[axum::async_trait]
impl<T, S, B> FromRequest<S, B> for AnyMedia<T>
where
    T: serde::de::DeserializeOwned,
    B: HttpBody + Send + 'static,
    B::Data: Send,
    B::Error: Into<BoxError>,
    S: Send + Sync,
{
    type Rejection = AnyMediaRejection;

    async fn from_request(req: Request<B>, state: &S) -> Result<Self, Self::Rejection> {
        let mime = req
            .headers()
            .get("content-type")
            .map(|h| h.to_str().unwrap_or(""))
            .unwrap_or("")
            .parse()
            .unwrap_or(mime::APPLICATION_JSON);

        let bytes = Bytes::from_request(req, state).await?;

        let result = match (mime.type_(), mime.subtype().as_str()) {
            #[cfg(feature = "urlencoded")]
            (mime::APPLICATION, "x-www-form-urlencoded") => {
                mimetypes::deserialize_urlencoded(&bytes)
            }
            #[cfg(feature = "yaml")]
            (mime::APPLICATION, "yaml") => mimetypes::deserialize_yaml(&bytes),
            _ => mimetypes::deserialize_json(&bytes),
        };

        match result {
            Ok(data) => Ok(AnyMedia(data)),
            Err(err) => {
                error!("{}", err);
                match err {
                    AnyMediaDeserializeError::JsonError(err) => match err.inner().classify() {
                        serde_json::error::Category::Data => {
                            Err(AnyMediaRejection::JsonDataError(err))
                        }
                        serde_json::error::Category::Syntax | serde_json::error::Category::Eof => {
                            Err(AnyMediaRejection::JsonSyntaxError(err))
                        }
                        serde_json::error::Category::Io => unreachable!(),
                    },
                    #[cfg(feature = "urlencoded")]
                    AnyMediaDeserializeError::UrlEncodedError(err) => Err(err.into()),
                    // TODO: Implement yaml error handling
                    #[cfg(feature = "yaml")]
                    AnyMediaDeserializeError::YamlError(_err) => unimplemented!(),
                }
            }
        }
    }
}
