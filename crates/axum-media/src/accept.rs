use std::ops::{Deref, DerefMut};

use axum::{extract::FromRequestParts, http::request::Parts};

/// Accept header extractor.
///
/// Gets `Option<String>` from the request's `Accept` header, so it can be later used with [`crate::AnyMedia`].
/// Thanks to [`axum::extract::FromRequestParts`] it can be used with other extractors including [`axum::Json`] or [`crate::AnyMedia`].
///
/// The request should never be rejected, as if the header is not present `None` will be used as the internal value.
///
/// ## Example
/// ```rust,no_run
/// use axum_media::Accept;
///
/// async fn handler(accept: Accept) -> impl axum::response::IntoResponse {
///   // Can directly be used as Option<String>
///   println!("Request accepts {}", accept.as_deref().unwrap());
///
///   axum_media::AnyMedia(
///     serde_json::json!({}),
///     accept.into(),
///   )
/// }
///
/// #[tokio::main]
/// async fn main() {
///   let app = axum::Router::new().route("/", axum::routing::get(handler));
///
///   let tcp = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
///   axum::serve(tcp, app.into_make_service()).await.unwrap()
/// }
/// ```
#[derive(Debug, Clone)]
pub struct Accept(Option<String>);

#[axum::async_trait]
impl<S> FromRequestParts<S> for Accept
where
    S: Send + Sync,
{
    type Rejection = ();

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        Ok(Accept(
            parts
                .headers
                .get("accept")
                .and_then(|h| h.to_str().ok().map(|s| s.to_owned())),
        ))
    }
}

impl Deref for Accept {
    type Target = Option<String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Accept {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Accept> for Option<String> {
    fn from(content_type: Accept) -> Self {
        content_type.0
    }
}
