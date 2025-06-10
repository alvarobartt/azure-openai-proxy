use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum EmbeddingInput {
    Single(String),
    Batch(Vec<String>),
}

/// Specifies the types of embeddings to generate. Compressed embeddings types like uint8, int8,
/// ubinary and binary, may reduce storage costs without sacrificing the integrity of the data.
///
/// Reference: https://learn.microsoft.com/en-us/rest/api/aifoundry/model-inference/get-embeddings/get-embeddings?view=rest-aifoundry-model-inference-2024-05-01-preview&tabs=HTTP#embeddingencodingformat
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum EmbeddingEncodingFormat {
    /// Get back binary representation of the embeddings encoded as Base64 string. OpenAI Python
    /// library retrieves embeddings from the API as encoded binary data, rather than using
    /// intermediate decimal representations as is usually done.
    Base64,

    /// Get back signed binary embeddings
    Binary,

    /// Get back full precision embeddings
    Float,

    /// Get back signed int8 embeddings
    Int8,

    /// Get back unsigned binary embeddings
    UBinary,

    /// Get back unsigned int8 embeddings
    UInt8,
}

/// Represents the input types used for embedding search.
///
/// Reference: https://learn.microsoft.com/en-us/rest/api/aifoundry/model-inference/get-embeddings/get-embeddings?view=rest-aifoundry-model-inference-2024-05-01-preview&tabs=HTTP#embeddinginputtype
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum EmbeddingInputType {
    /// Indicates the input represents a document that is stored in a vector database.
    Document,

    /// Indicates the input represents a search query to find the most relevant documents in your
    /// vector database.
    Query,

    /// Indicates the input is a general text input.
    Text,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EmbeddingsRequest {
    /// Input text to embed, encoded as a string or array of tokens. To embed multiple inputs in a
    /// single request, pass an array of strings or array of token arrays.
    input: EmbeddingInput,

    /// ID of the specific AI model to use, if more than one model is available on the endpoint.
    model: String,

    /// The number of dimensions the resulting output embeddings should have. Passing null causes
    /// the model to use its default value. Returns a 422 error if the model doesn't support the
    /// value or parameter.
    #[serde(skip_serializing_if = "Option::is_none")]
    dimensions: Option<i32>,

    /// The desired format for the returned embeddings.
    #[serde(skip_serializing_if = "Option::is_none")]
    encoding_format: Option<EmbeddingEncodingFormat>,

    /// The type of the input. Returns a 422 error if the model doesn't support the value or parameter.
    #[serde(skip_serializing_if = "Option::is_none")]
    input_type: Option<EmbeddingInputType>,

    /// Placeholder for the extra parameters to be provided if the `extra-parameters` header
    /// contains the value `pass-through`, meaning that the extra parameters within the payload
    /// won't be ignored (default `serde` behavior), but rather kept and passed through to the
    /// underlying API
    #[serde(flatten, skip_serializing_if = "HashMap::is_empty")]
    pub extra_parameters: HashMap<String, serde_json::Value>,
}

impl Into<axum::body::Body> for EmbeddingsRequest {
    fn into(self) -> axum::body::Body {
        let bytes = serde_json::to_vec(&self).unwrap();
        tracing::debug!(
            "Serialized EmbeddingsRequest JSON: {}",
            String::from_utf8_lossy(&bytes)
        );
        axum::body::Body::from(bytes)
    }
}
