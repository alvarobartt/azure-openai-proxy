use crate::errors::AzureError;
use axum::http::Uri;

/// Supported Azure AI Model Inference API versions
///
/// Reference: https://learn.microsoft.com/en-us/rest/api/aifoundry/model-inference/get-chat-completions/get-chat-completions
const API_VERSIONS: &[&str] = &["2024-05-01-preview", "2025-04-01"];

/// Function to check that the `api-version` query parameter is preset within the request, and
/// that the `api-version` value is a valid value on Azure AI Model Inference API.
pub fn check_api_version(query: Option<&str>) -> Result<(), AzureError> {
    let api_version = query
        .and_then(|q| {
            q.split('&')
                .find(|p| p.starts_with("api-version="))
                .and_then(|p| p.split('=').nth(1))
        })
        .ok_or(AzureError::MissingApiVersionParameter)?;

    if !API_VERSIONS.contains(&api_version) {
        return Err(AzureError::UnsupportedApiVersionValue(
            api_version.to_string(),
            API_VERSIONS.join(", ").into(),
        ));
    }

    Ok(())
}

/// Function to append a path (route) to an URI
pub fn append_path_to_uri(uri: Uri, path: &str) -> Uri {
    let mut uri = uri.to_string();
    uri.push_str(path);
    Uri::try_from(uri).unwrap()
}
