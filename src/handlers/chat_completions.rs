use crate::{errors::AzureError, proxy::HttpClient};
use axum::{
    body::Body,
    extract::{Request, State},
    http::{StatusCode, Uri},
    response::IntoResponse,
};

/// Supported Azure AI API versions
///
/// Reference: https://learn.microsoft.com/en-us/rest/api/aifoundry/model-inference/get-chat-completions/get-chat-completions
const API_VERSIONS: &[&str] = &["2024-05-01-preview", "2025-04-01"];

/// This function proxies the requests to `/chat/completion` to the underlying `/v1/chat/completion`,
/// making sure that the I/O schemas are Azure AI compliant. This function handles that the
/// `api-version` query parameter is provided, builds the URI for the underlying service, and
/// proxies the request to `/v1/chat/completions`.
pub async fn chat_completions_handler(
    State(client): State<HttpClient>,
    mut req: Request<Body>,
) -> Result<impl IntoResponse, AzureError> {
    // TODO: most likely we need to parse the `api-version` more than once so may make sense to
    // move this over to a `utils.rs` file
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

    // TODO: doesn't make sense to calculate this over and over, let's create a shared state with
    // the information (also useful to propagate the information from the CLI)
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
