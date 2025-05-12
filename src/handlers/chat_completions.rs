use crate::schemas::ExtraParameters;
use crate::{
    errors::AzureError,
    proxy::ProxyState,
    schemas::{ChatRequest, QueryParameters},
    utils::{append_path_to_uri, check_api_version},
};
use axum::{
    body::Body,
    extract::{Query, Request, State},
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
    body: String,
) -> Result<impl IntoResponse, AzureError> {
    // Checks that the `api-version` query parameter is provided and valid
    check_api_version(query.api_version)?;

    // Checks if the `extra-parameters` header is there, and applies the necessary filtering to
    // the payload to be forwarded to the underlying API
    let extra_parameters: ExtraParameters = headers
        .get("extra-parameters")
        .and_then(|value| value.to_str().ok())
        .map(|s| {
            serde_json::from_str::<ExtraParameters>(&format!("\"{}\"", s))
                .unwrap_or(ExtraParameters::PassThrough)
        })
        .unwrap_or(ExtraParameters::PassThrough);

    tracing::debug!(
        "Reading body {:?} with extra-parameters {:?}",
        body,
        extra_parameters
    );
    let payload = ChatRequest::from_str(body.as_str(), extra_parameters)
        .map_err(|e| AzureError::InternalParsing(e.to_string()))?;
    tracing::debug!("Body parsed as JSON as {:?}", payload);

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
