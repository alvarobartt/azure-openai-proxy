use crate::{
    errors::AzureError,
    proxy::ProxyState,
    schemas::{
        azure::QueryParameters,
        info::{InfoResponse, ModelType, OpenAIInfoResponse},
    },
    utils::{append_path_to_uri, check_api_version},
    UpstreamType,
};
use axum::{
    body::{to_bytes, Body},
    extract::{Query, Request, State},
    http::{HeaderMap, Method, StatusCode},
    response::{IntoResponse, Json},
};

pub async fn info_handler(
    method: Method,
    headers: HeaderMap,
    Query(query): Query<QueryParameters>,
    State(state): State<ProxyState>,
) -> Result<Json<InfoResponse>, AzureError> {
    // Checks that the `api-version` query parameter is provided and valid
    check_api_version(query.api_version)?;

    // Updates the request URI whilst keeping the headers, parameters, etc.
    let uri = append_path_to_uri(state.uri, "/v1/models");

    // Forwards request to the underlying upstream API
    tracing::info!("Proxying {} request to {}", method, uri);

    // Build request again preserving the method, body and headers
    let mut req: Request<Body> = Request::builder()
        .method(method)
        .uri(uri)
        .body(Body::empty())
        .map_err(|e| AzureError::InternalParsing(e.to_string()))?;

    *req.headers_mut() = headers;

    let body = state
        .client
        .request(req)
        .await
        .map_err(|e| AzureError::Upstream(StatusCode::BAD_GATEWAY, e.to_string()))
        .map(|r| r.into_response())?;

    // Parsing response body into Azure AI Model Inference compliant JSON
    let body_bytes = to_bytes(body.into_body(), std::usize::MAX)
        .await
        .map_err(|e| AzureError::InternalParsing(e.to_string()))?;

    let info: OpenAIInfoResponse = serde_json::from_slice(&body_bytes)
        .map_err(|e| AzureError::InternalParsing(e.to_string()))?;

    let (model_provider_name, model_name) = info.data[0]
        .id
        .split_once("/")
        // Necessary to prevent that if the split fails for some reason as e.g. the `id` is
        // internally set to a path, then the original `id` information is preserved and
        // returned even if "not correct"; when working with models from the Hugging Face Hub
        .unwrap_or((&info.data[0].id, &info.data[0].id));

    let model_type = match &state.upstream_type {
        UpstreamType::ChatCompletions => ModelType::ChatCompletion,
        UpstreamType::Embeddings => ModelType::Embeddings,
    };

    Ok(Json(InfoResponse {
        model_name: model_name.to_string(),
        model_type,
        model_provider_name: model_provider_name.to_string(),
    }))
}
