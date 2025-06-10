use serde::{Deserialize, Serialize};

/// The type of AI model.
/// Reference: https://learn.microsoft.com/en-us/rest/api/aifoundry/model-inference/get-model-info/get-model-info?view=rest-aifoundry-model-inference-2025-04-01&tabs=HTTP#modeltype
#[derive(Serialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub enum ModelType {
    /// A model capable of taking chat-formatted messages and generate responses
    ChatCompletion,

    /// A model capable of generating embeddings from a text
    #[allow(unused)]
    Embeddings,
}

/// Represents some basic information about the AI model.
/// Reference: https://learn.microsoft.com/en-us/rest/api/aifoundry/model-inference/get-model-info/get-model-info?view=rest-aifoundry-model-inference-2024-05-01-preview&tabs=HTTP#modelinfo
#[derive(Serialize, Debug)]
pub struct InfoResponse {
    /// The name of the AI model. For example: Phi21
    pub model_name: String,

    /// The type of the AI model. A Unique identifier for the profile.
    pub model_type: ModelType,

    /// The model provider name. For example: Microsoft
    pub model_provider_name: String,
}

/// Describes an OpenAI model offering that can be used with the API.
/// Reference: https://platform.openai.com/docs/api-reference/models/object
#[derive(Serialize, Deserialize, Debug)]
pub struct OpenAIModelInfo {
    /// The model identifier, which can be referenced in the API endpoints.
    pub id: String,

    /// The object type, which is always "model".
    #[allow(unused)]
    object: String,

    /// The Unix timestamp (in seconds) when the model was created.
    #[allow(unused)]
    created: u32,

    /// The organization that owns the model.
    #[allow(unused)]
    owned_by: String,
}

/// List with the currently available models, and provides basic information about each one.
/// https://platform.openai.com/docs/api-reference/models/object
#[derive(Deserialize, Debug)]
pub struct OpenAIInfoResponse {
    /// The object type, which is always "list".
    #[allow(unused)]
    object: String,

    /// A list of model objects.
    pub data: Vec<OpenAIModelInfo>,
}
