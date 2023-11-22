use std::ops::{Deref, DerefMut};

use axum::{extract::FromRequestParts, http::request::Parts};

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
