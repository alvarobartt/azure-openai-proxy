use axum::{
    body::Body,
    extract::{Request, State},
    http::{StatusCode, Uri},
    response::IntoResponse,
};
use hyper_util::client::legacy::{connect::HttpConnector, Client};

use crate::errors::AzureError;

type HttpClient = Client<HttpConnector, Body>;

const API_VERSIONS: &[&str] = &["2024-05-01-preview", "2025-04-01"];

pub async fn chat_completions_handler(
    State(client): State<HttpClient>,
    mut req: Request<Body>,
) -> Result<impl IntoResponse, AzureError> {
    let version = req
        .uri()
        .query()
        .and_then(|q| {
            q.split('&')
                .find(|p| p.starts_with("api-version="))
                .and_then(|p| p.split('=').nth(1))
        })
        .ok_or(AzureError::MissingApiVersionParameter)?;

    if !API_VERSIONS.contains(&version) {
        return Err(AzureError::UnsupportedApiVersionValue(
            version.to_string(),
            API_VERSIONS.join(", ").into(),
        ));
    }

    *req.uri_mut() = {
        let host = std::env::var("UPSTREAM_HOST").unwrap_or_else(|_| "http://localhost".into());
        let port = std::env::var("UPSTREAM_PORT").ok();

        let mut uri = host.replace("/v1", "");
        if let Some(p) = port {
            uri.push_str(&format!(":{}", p));
        }
        uri.push_str("/v1/chat/completions");
        Uri::try_from(uri).unwrap()
    };

    tracing::info!("Proxying {} request to {}", req.method(), req.uri());

    client
        .request(req)
        .await
        .map_err(|e| AzureError::Upstream(StatusCode::BAD_GATEWAY, e.to_string()))
        .map(|res| res.into_response())
}
