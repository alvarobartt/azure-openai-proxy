use crate::{
    errors::AzureError,
    proxy::ProxyState,
    utils::{append_path_to_uri, check_api_version},
};
use axum::{
    body::{to_bytes, Body},
    extract::{Request, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct ModelInfo {
    id: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct InfoResponse {
    data: Vec<ModelInfo>,
}

// Reference: https://learn.microsoft.com/en-us/rest/api/aifoundry/model-inference/get-model-info/get-model-info?view=rest-aifoundry-model-inference-2025-04-01&tabs=HTTP#modeltype
#[derive(Serialize, Debug)]
#[serde(rename_all = "kebab-case")]
enum ModelType {
    /// A model capable of taking chat-formatted messages and generate responses
    ChatCompletion,

    /// A model capable of generating embeddings from a text
    #[allow(unused)]
    Embeddings,
}

#[derive(Serialize, Debug)]
pub struct AzureInfoResponse {
    model_name: String,
    model_type: ModelType,
    model_provider_name: String,
}

pub async fn info_handler(
    State(state): State<ProxyState>,
    mut req: Request<Body>,
) -> Result<Json<AzureInfoResponse>, AzureError> {
    // Checks that the `api-version` query parameter is provided and valid
    check_api_version(req.uri().query())?;

    // Updates the request URI whilst keeping the headers, parameters, etc.
    *req.uri_mut() = append_path_to_uri(state.uri, "/info");

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
