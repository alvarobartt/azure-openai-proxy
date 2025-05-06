use crate::{
    errors::AzureError,
    proxy::ProxyState,
    utils::{append_path_to_uri, check_api_version},
};
use axum::{
    body::Body,
    extract::{Request, State},
    http::StatusCode,
    response::IntoResponse,
};

/// This function proxies the requests to `/chat/completion` to the underlying `/v1/chat/completion`,
/// making sure that the I/O schemas are Azure AI compliant. This function handles that the
/// `api-version` query parameter is provided, builds the URI for the underlying service, and
/// proxies the request to `/v1/chat/completions`.
pub async fn chat_completions_handler(
    State(state): State<ProxyState>,
    mut req: Request<Body>,
) -> Result<impl IntoResponse, AzureError> {
    // Checks that the `api-version` query parameter is provided and valid
    check_api_version(req.uri().query())?;

    // Updates the request URI whilst keeping the headers, parameters, etc.
    *req.uri_mut() = append_path_to_uri(state.uri, "v1/chat/completions");
    tracing::info!("Now the query contains: {:?}", req.uri().query());

    // Forwards request to the underlying upstream API
    tracing::info!("Proxying {} request to {}", req.method(), req.uri());
    state
        .client
        .request(req)
        .await
        .map_err(|e| AzureError::Upstream(StatusCode::BAD_GATEWAY, e.to_string()))
        .map(|res| res.into_response())
}
