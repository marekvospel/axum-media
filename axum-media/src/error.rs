use axum::{
    extract::rejection::BytesRejection,
    http::{header, StatusCode},
    response::IntoResponse,
};

#[derive(Debug, thiserror::Error)]
pub enum AnyMediaRejection {
    #[error("Failed to deserialize the JSON body into the target type: {0}")]
    JsonDataError(serde_path_to_error::Error<serde_json::Error>),
    #[error("Failed to parse the request body as JSON: {0}")]
    JsonSyntaxError(serde_path_to_error::Error<serde_json::Error>),
    #[cfg(feature = "urlencoded")]
    #[error("Failed to parse the request body as form urlencoded: {0}")]
    UrlEncodedError(#[from] serde_urlencoded::de::Error),
    #[cfg(feature = "yaml")]
    #[error("Failed to deserialize the yaml body into the target type: {0}")]
    YamlDataError(serde_path_to_error::Error<serde_yaml::Error>),
    #[error("{0}")]
    BytesRejection(#[from] BytesRejection),
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
    #[error(transparent)]
    JsonError(#[from] serde_json::Error),
    #[cfg(feature = "urlencoded")]
    #[error(transparent)]
    UrlEncodedError(#[from] serde_urlencoded::ser::Error),
    #[cfg(feature = "yaml")]
    #[error(transparent)]
    YamlError(#[from] serde_yaml::Error),
    #[cfg(feature = "xml")]
    #[error(transparent)]
    XmlError(#[from] serde_xml_rs::Error),
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum AnyMediaDeserializeError {
    #[error(transparent)]
    JsonError(#[from] serde_path_to_error::Error<serde_json::Error>),
    #[cfg(feature = "urlencoded")]
    #[error(transparent)]
    UrlEncodedError(#[from] serde_urlencoded::de::Error),
    #[cfg(feature = "yaml")]
    #[error(transparent)]
    YamlError(#[from] serde_path_to_error::Error<serde_yaml::Error>),
}
