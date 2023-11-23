use std::ops::{Deref, DerefMut};

use axum::{
    body::HttpBody,
    extract::FromRequest,
    http::{header, HeaderValue, Request, StatusCode},
    response::IntoResponse,
    BoxError,
};
use bytes::{BufMut, Bytes, BytesMut};
use serde::Serialize;
use tracing::error;

use crate::{mimetypes, AnyMediaDeserializeError, AnyMediaRejection, AnyMediaSerializeError};

/// Automatic data extractor / response.
///
/// When used as an extractor it parses the request's `Content-Type` header and tries to [`serde::Deserialize`]
/// the body into `<T>`. When extracting second field will always be `None`. If the `Content-Type`
/// is missing, invalid or not supported, `application/json` will be used.
///
/// [`AnyMediaRejection`] will be returned in case the body is not valid or `<T>` cannot be deserialized.
///
/// When used as a response, second field is used to determine the type to Serialize to. If `None`,
/// `application/json` will be used. Meant to be used with [`crate::Accept`] extractor.
///
/// ## Extractor example
///
/// ```rust,no_run
/// use axum::{response::IntoResponse, routing::{get, post}};
/// use axum_media::{AnyMedia, Accept};
///
/// #[tokio::main]
/// async fn main() {
///   let app = axum::Router::new()
///     .route("/login", post(login_handler));
///
///   axum::Server::bind(&"127.0.0.1:3000".parse().unwrap())
///     .serve(app.into_make_service())
///     .await.unwrap()
/// }
///
/// #[derive(serde::Deserialize)]
/// struct LoginData {
///   email: String,
///   password: String,
/// }
///
/// // Uses the `Content-Type` to determine what to try to Deserialize.
/// async fn login_handler(data: AnyMedia<LoginData>) -> String {
///   format!("Authorization: {}:{}", data.email, data.password)
/// }
///
/// ```
///
/// ## Response example
///
/// ```rust,no_run
/// use axum::{response::IntoResponse, routing::{get, post}};
/// use axum_media::{AnyMedia, Accept};
///
/// #[tokio::main]
/// async fn main() {
///   let app = axum::Router::new()
///     .route("/posts", get(returns_anything_handler))
///     .route("/always_json", get(always_json_handler));
///
///   axum::Server::bind(&"127.0.0.1:3000".parse().unwrap())
///     .serve(app.into_make_service())
///     .await.unwrap()
/// }
///
/// // Always returns json, because second parameter is none.
/// async fn always_json_handler() -> impl IntoResponse {
///    AnyMedia(serde_json::json!({ "version": "1.0.0" }), None)
/// }
///
/// // Can be used with the Accept extractor to return values based on the `Accept` header.
/// async fn returns_anything_handler(accept: Accept) -> impl IntoResponse {
///   AnyMedia(
///     serde_json::json!({
///       "posts": [
///         {
///           "title": "MyPost",
///           "description": "A short about how dogs are better than cats"
///         }
///       ]
///     }),
///     accept.into(),
///   )
/// }
/// ```
///
#[derive(Debug, Clone, Default)]
pub struct AnyMedia<T>(pub T, pub Option<String>);

impl<T> From<T> for AnyMedia<T> {
    fn from(data: T) -> Self {
        AnyMedia(data, None)
    }
}

impl<T> Deref for AnyMedia<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for AnyMedia<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> IntoResponse for AnyMedia<T>
where
    T: Serialize,
{
    fn into_response(self) -> axum::response::Response {
        let mime = self
            .1
            .map(|s| s.parse().unwrap_or(mime::APPLICATION_JSON))
            .unwrap_or(mime::APPLICATION_JSON);
        let mut buf = BytesMut::with_capacity(128).writer();

        let mut result: Option<Result<(), AnyMediaSerializeError>> =
            match (mime.type_(), mime.subtype()) {
                (mime::APPLICATION, mime::JSON) => {
                    Some(mimetypes::serialize_json(&self.0, &mut buf))
                }
                #[cfg(feature = "urlencoded")]
                (mime::APPLICATION, mime::WWW_FORM_URLENCODED) => {
                    Some(mimetypes::serialize_urlencoded(&self.0, &mut buf))
                }
                _ => None,
            };

        if let None = result {
            result = match (mime.type_(), mime.suffix()) {
                #[cfg(feature = "urlencoded")]
                (mime::APPLICATION, Some(mime::WWW_FORM_URLENCODED)) => {
                    Some(mimetypes::serialize_urlencoded(&self.0, &mut buf))
                }
                _ => Some(mimetypes::serialize_json(&self.0, &mut buf)),
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

        let result = match (mime.type_(), mime.subtype()) {
            #[cfg(feature = "urlencoded")]
            (mime::APPLICATION, mime::WWW_FORM_URLENCODED) => {
                mimetypes::deserialize_urlencoded(&bytes)
            }
            _ => mimetypes::deserialize_json(&bytes),
        };

        match result {
            Ok(data) => Ok(AnyMedia(data, None)),
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
                }
            }
        }
    }
}
