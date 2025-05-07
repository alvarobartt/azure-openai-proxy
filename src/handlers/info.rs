use crate::{
    errors::AzureError,
    proxy::ProxyState,
    schemas::{AzureInfoResponse, InfoResponse, ModelType},
    utils::{append_path_to_uri, check_api_version},
};
use axum::{
    body::{to_bytes, Body},
    extract::{Request, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};

pub async fn info_handler(
    State(state): State<ProxyState>,
    mut req: Request<Body>,
) -> Result<Json<AzureInfoResponse>, AzureError> {
    // Checks that the `api-version` query parameter is provided and valid
    check_api_version(req.uri().query())?;

    // Updates the request URI whilst keeping the headers, parameters, etc.
    *req.uri_mut() = append_path_to_uri(state.uri, "/v1/models");

    // Forwards request to the underlying upstream API
    tracing::info!("Proxying {} request to {}", req.method(), req.uri());
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

    let info: InfoResponse = serde_json::from_slice(&body_bytes)
        .map_err(|e| AzureError::InternalParsing(e.to_string()))?;

    let (model_provider_name, model_name) = info.data[0]
        .id
        .split_once("/")
        // Necessary to prevent that if the split fails for some reason as e.g. the `id` is
        // internally set to a path, then the original `id` information is preserved and
        // returned even if "not correct"
        .unwrap_or((&info.data[0].id, &info.data[0].id));

    Ok(Json(AzureInfoResponse {
        model_name: model_name.to_string(),
        model_type: ModelType::ChatCompletion,
        model_provider_name: model_provider_name.to_string(),
    }))
}
