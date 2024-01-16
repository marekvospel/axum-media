#![allow(dead_code)]

use axum::{body::Body, http::Response};
use http_body_util::BodyExt;
use serde::de::DeserializeOwned;

pub async fn string_body(res: Response<Body>) -> anyhow::Result<String> {
    Ok(String::from_utf8(
        res.into_body().collect().await?.to_bytes().to_vec(),
    )?)
}

pub async fn json_body<T: DeserializeOwned>(res: Response<Body>) -> anyhow::Result<T> {
    Ok(serde_json::from_str::<T>(
        String::from_utf8(res.into_body().collect().await?.to_bytes().to_vec())?.as_str(),
    )?)
}
