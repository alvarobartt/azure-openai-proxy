use axum::{
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use serde_json::json;
use thiserror::Error;

/// Azure AI specific errors
///
/// Reference: https://github.com/microsoft/api-guidelines/blob/vNext/azure/Guidelines.md#handling-errors
#[derive(Debug, Error)]
pub enum AzureError {
    #[error("The api-version query parameter (?api-version=) is required for all requests.")]
    MissingApiVersionParameter,

    #[error("Unsupported api-version '{0}'. The supported api-versions are '{1}'.")]
    UnsupportedApiVersionValue(String, String),

    #[error("Internal proxy parsing error with: '{0}'.")]
    InternalParsing(String),

    #[error("Upstream error: '{0}' (status {1}).")]
    Upstream(StatusCode, String),
}

impl IntoResponse for AzureError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            Self::MissingApiVersionParameter => (
                StatusCode::BAD_REQUEST,
                "MissingApiVersionParameter",
                self.to_string(),
            ),
            Self::UnsupportedApiVersionValue(ver, supported) => (
                StatusCode::BAD_REQUEST,
                "UnsupportedApiVersionValue",
                format!(
                    "Unsupported api-version '{0}'. The supported api-versions are '{1}'.",
                    ver, supported
                ),
            ),
            Self::InternalParsing(message) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "InternalProxyParsing",
                message,
            ),
            Self::Upstream(status, message) => (status, "UpstreamApi", message),
        };

        let body = Json(json!({
            "error": {
                "code": code,
                "message": message
            }
        }));

        (status, body).into_response()
    }
}
