use crate::{errors::AzureError, proxy::HttpClient};
use axum::{
    body::Body,
    extract::{Request, State},
    http::{StatusCode, Uri},
    response::IntoResponse,
};
use std::str::FromStr;

use crate::utils::check_api_version;

/// This function proxies the requests to `/chat/completion` to the underlying `/v1/chat/completion`,
/// making sure that the I/O schemas are Azure AI compliant. This function handles that the
/// `api-version` query parameter is provided, builds the URI for the underlying service, and
/// proxies the request to `/v1/chat/completions`.
pub async fn chat_completions_handler(
    State(client): State<HttpClient>,
    mut req: Request<Body>,
) -> Result<impl IntoResponse, AzureError> {
    check_api_version(req.uri().query())?;

    // TODO: doesn't make sense to calculate this over and over, let's create a shared state with
    // the information (also useful to propagate the information from the CLI)
    *req.uri_mut() = Uri::from_str("http://localhost/v1/chat/completions").unwrap();

    tracing::info!("Proxying {} request to {}", req.method(), req.uri());

    client
        .request(req)
        .await
        .map_err(|e| AzureError::Upstream(StatusCode::BAD_GATEWAY, e.to_string()))
        .map(|res| res.into_response())
}
