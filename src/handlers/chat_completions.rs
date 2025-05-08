use crate::{
    errors::AzureError,
    proxy::ProxyState,
    schemas::{ChatRequest, QueryParameters},
    utils::append_path_to_uri,
};
use axum::{
    body::Body,
    extract::{Json, Query, Request, State},
    http::{HeaderMap, Method, StatusCode},
    response::IntoResponse,
};

/// This function proxies the requests to `/chat/completion` to the underlying `/v1/chat/completion`,
/// making sure that the I/O schemas are Azure AI compliant. This function handles that the
/// `api-version` query parameter is provided, builds the URI for the underlying service, and
/// proxies the request to `/v1/chat/completions`.
pub async fn chat_completions_handler(
    method: Method,
    headers: HeaderMap,
    Query(query): Query<QueryParameters>,
    State(state): State<ProxyState>,
    Json(payload): Json<ChatRequest>,
) -> Result<impl IntoResponse, AzureError> {
    // Checks that the `api-version` query parameter is provided and valid
    // check_api_version(parameters.api_version)?;
    tracing::info!("query contains {:?}", query);
    tracing::info!("headers contains {:?}", headers);

    // Updates the request URI whilst keeping the headers, parameters, etc.
    let uri = append_path_to_uri(state.uri, "/v1/chat/completions");

    // Forwards request to the underlying upstream API
    tracing::info!("Proxying {} request to {}", method, uri);

    // Build request again preserving the method, body and headers
    let mut req: Request<Body> = Request::builder()
        .method(method)
        .uri(uri)
        .body(payload.into())
        .map_err(|e| AzureError::InternalParsing(e.to_string()))?;

    *req.headers_mut() = headers;

    state
        .client
        .request(req)
        .await
        .map_err(|e| AzureError::Upstream(StatusCode::BAD_GATEWAY, e.to_string()))
        .map(|res| res.into_response())
}
