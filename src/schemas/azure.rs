use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub struct QueryParameters {
    #[serde(rename = "api-version")]
    pub api_version: Option<String>,
}

/// https://learn.microsoft.com/en-us/rest/api/aifoundry/model-inference/get-chat-completions/get-chat-completions?view=rest-aifoundry-model-inference-2025-04-01&tabs=HTTP#extraparameters
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub enum ExtraParameters {
    /// The service will pass extra parameters to the back-end AI model.
    PassThrough,

    /// The service will ignore (drop) extra parameters in the request payload. It will only pass
    /// the known parameters to the back-end AI model.
    Drop,

    /// The service will error if it detected extra parameters in the request payload. This is the
    /// service default.
    Error,
}
