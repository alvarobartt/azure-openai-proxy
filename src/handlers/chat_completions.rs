use crate::{
    errors::AzureError,
    proxy::ProxyState,
    schemas::{
        azure::{ExtraParameters, QueryParameters},
        chat_completions::ChatRequest,
    },
    utils::{append_path_to_uri, check_api_version},
};
use axum::{
    body::Body,
    extract::{Json, Query, Request, State},
    http::{
        header::{CONNECTION, CONTENT_LENGTH, TRANSFER_ENCODING},
        HeaderMap, Method, StatusCode,
    },
    response::IntoResponse,
};
use std::collections::HashMap;

/// This function proxies the requests to `/chat/completions` to the underlying `/v1/chat/completions`,
/// making sure that the I/O schemas are compliant with the Azure AI Model Inference API
/// specification. This function handles that the `api-version` query parameter is provided, builds
/// the URI for the underlying service, and proxies the request to `/v1/chat/completions`.
pub async fn chat_completions_handler(
    method: Method,
    mut headers: HeaderMap,
    Query(query): Query<QueryParameters>,
    State(state): State<ProxyState>,
    Json(mut payload): Json<ChatRequest>,
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

    // If `extra-parameters` exists, then remove it as there's no need to forward it
    headers.remove("extra-parameters");
    // And, also remove the headers: CONNECTION, CONTENT_LENGTH, and TRANSFER_ENCODING, to prevent
    // issues when forwarding the request, the HTTP client will automatically recalculate those
    headers.remove(CONNECTION);
    headers.remove(CONTENT_LENGTH);
    headers.remove(TRANSFER_ENCODING);

    // Based on `extra-parameters` define what to do with the `extra_parameters` field
    match extra_parameters {
        ExtraParameters::Drop => {
            payload.extra_parameters = HashMap::new();
        }
        ExtraParameters::Error => {
            if !payload.extra_parameters.is_empty() {
                let fields = payload
                    .extra_parameters
                    .keys()
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(",");

                return Err(AzureError::InternalParsing(format!(
                    "As the header `extra-parameters` is set to `error`, since the following parameters have been provided {}, and those are not defined within the Azure AI Model Inference API specification.",
                    fields
                )));
            }
        }
        ExtraParameters::PassThrough => (),
    };

    // Updates the request URI whilst keeping the headers, parameters, etc.
    let uri = append_path_to_uri(state.uri, "/v1/chat/completions");

    // Forwards request to the underlying upstream API
    tracing::info!("Proxying {} request to {} with {:?}", method, uri, payload);

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
