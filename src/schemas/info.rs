use serde::{Deserialize, Serialize};

/// https://learn.microsoft.com/en-us/rest/api/aifoundry/model-inference/get-model-info/get-model-info?view=rest-aifoundry-model-inference-2025-04-01&tabs=HTTP#modeltype
#[derive(Serialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub enum ModelType {
    /// A model capable of taking chat-formatted messages and generate responses
    ChatCompletion,

    /// A model capable of generating embeddings from a text
    #[allow(unused)]
    Embeddings,
}

#[derive(Serialize, Debug)]
pub struct AzureInfoResponse {
    pub model_name: String,
    pub model_type: ModelType,
    pub model_provider_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ModelInfo {
    pub id: String,
}

#[derive(Deserialize, Debug)]
pub struct InfoResponse {
    pub data: Vec<ModelInfo>,
}
