use serde::de::{self, MapAccess, Unexpected, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;

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

#[derive(Serialize, Deserialize, Debug)]
pub struct ModelInfo {
    pub id: String,
}

#[derive(Deserialize, Debug)]
pub struct InfoResponse {
    pub data: Vec<ModelInfo>,
}

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

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatRequestAudioReference {
    id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FunctionCall {
    arguments: String,
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FunctionDefinition {
    description: String,
    name: String,
    parameters: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase", tag = "type")]
pub enum ChatCompletionsToolCall {
    Function { id: String, function: FunctionCall },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase", tag = "role")]
pub enum ChatRequestMessage {
    System {
        content: String,
    },
    User {
        content: String,
    },
    Assistant {
        audio: Option<ChatRequestAudioReference>,
        content: String,
        tool_calls: Option<Vec<ChatCompletionsToolCall>>,
    },
    Tool {
        content: String,
        tool_call_id: String,
    },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum ChatCompletionsResponseFormat {
    Text,
    JsonObject,
    JsonSchema {
        description: String,
        name: String,
        schema: serde_json::Value,
        strict: bool,
    },
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum ChatCompletionsModality {
    Text,
    Audio,
}

impl ChatCompletionsModality {
    fn default() -> Option<Vec<Self>> {
        Some(vec![ChatCompletionsModality::Text])
    }
}

#[derive(Debug, Serialize)]
pub enum ToolChoice {
    None,
    Auto,
    Required,
    Function(FunctionTool),
}

impl<'de> Deserialize<'de> for ToolChoice {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ToolChoiceVisitor;

        impl<'de> Visitor<'de> for ToolChoiceVisitor {
            type Value = ToolChoice;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str(r#""none", "auto", "required", or a function tool object"#)
            }

            fn visit_str<E>(self, value: &str) -> Result<ToolChoice, E>
            where
                E: de::Error,
            {
                match value {
                    "none" => Ok(ToolChoice::None),
                    "auto" => Ok(ToolChoice::Auto),
                    "required" => Ok(ToolChoice::Required),
                    _ => Err(de::Error::invalid_value(Unexpected::Str(value), &self)),
                }
            }

            fn visit_map<M>(self, map: M) -> Result<ToolChoice, M::Error>
            where
                M: MapAccess<'de>,
            {
                let tool = FunctionTool::deserialize(de::value::MapAccessDeserializer::new(map))?;
                Ok(ToolChoice::Function(tool))
            }
        }

        deserializer.deserialize_any(ToolChoiceVisitor)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FunctionTool {
    #[serde(rename = "type")]
    tool_type: String,
    function: FunctionSpec,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FunctionSpec {
    name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChatRequest {
    /// ID of the specific AI model to use, if more than one model is available on the endpoint.
    #[serde(skip_serializing_if = "Option::is_none")]
    model: Option<String>,

    /// The collection of context messages associated with this chat completions request. Typical
    /// usage begins with a chat message for the System role that provides instructions for the
    /// behavior of the assistant, followed by alternating messages between the User and Assistant
    /// roles.
    messages: Vec<ChatRequestMessage>,

    /// A value that influences the probability of generated tokens appearing based on their
    /// cumulative frequency in generated text. Positive values will make tokens less likely to
    /// appear as their frequency increases and decrease the likelihood of the model repeating the
    /// same statements verbatim. Supported range is [-2, 2].
    #[serde(skip_serializing_if = "Option::is_none")]
    frequency_penalty: Option<f32>,

    /// The maximum number of tokens to generate.
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,

    /// The modalities that the model is allowed to use for the chat completions response. The
    /// default modality is text. Indicating an unsupported modality combination results in an 422
    /// error.
    #[allow(unused)]
    #[serde(skip_serializing, default = "ChatCompletionsModality::default")]
    modalities: Option<Vec<ChatCompletionsModality>>,

    /// A value that influences the probability of generated tokens appearing based on their
    /// existing presence in generated text. Positive values will make tokens less likely to appear
    /// when they already exist and increase the model's likelihood to output new topics. Supported
    /// range is [-2, 2].
    #[serde(skip_serializing_if = "Option::is_none")]
    presence_penalty: Option<f32>,

    /// An alternative to sampling with temperature called nucleus sampling. This value causes the
    /// model to consider the results of tokens with the provided probability mass. As an example,
    /// a value of 0.15 will cause only the tokens comprising the top 15% of probability mass to be
    /// considered. It is not recommended to modify temperature and top_p for the same completions
    /// request as the interaction of these two settings is difficult to predict. Supported range
    /// is [0, 1].
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,

    /// If specified, the system will make a best effort to sample deterministically such that
    /// repeated requests with the same seed and parameters should return the same result.
    /// Determinism is not guaranteed.
    #[serde(skip_serializing_if = "Option::is_none")]
    seed: Option<i64>,

    /// A collection of textual sequences that will end completions generation.
    #[serde(skip_serializing_if = "Option::is_none")]
    stop: Option<Vec<String>>,

    /// A value indicating whether chat completions should be streamed for this request.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    stream: Option<bool>,

    /// The sampling temperature to use that controls the apparent creativity of generated
    /// completions. Higher values will make output more random while lower values will make
    /// results more focused and deterministic. It is not recommended to modify temperature and
    /// top_p for the same completions request as the interaction of these two settings is
    /// difficult to predict. Supported range is [0, 1].
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,

    /// An object specifying the format that the model must output.
    /// - Setting to { "type": "json_schema", "json_schema": {...} } enables Structured Outputs
    ///   which ensures the model will match your supplied JSON schema.
    /// - Setting to { "type": "json_object" } enables JSON mode, which ensures the message the
    ///   model generates is valid JSON.
    /// Important: when using JSON mode, you must also instruct the model to produce JSON yourself
    /// via a system or user message. Without this, the model may generate an unending stream of
    /// whitespace until the generation reaches the token limit, resulting in a long-running and
    /// seemingly "stuck" request. Also note that the message content may be partially cut off if
    /// finish_reason="length", which indicates the generation exceeded max_tokens or the
    /// conversation exceeded the max context length.
    #[serde(skip_serializing_if = "Option::is_none")]
    response_format: Option<ChatCompletionsResponseFormat>,

    /// If specified, the model will configure which of the provided tools it can use for the chat
    /// completions response.
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_choice: Option<ToolChoice>,

    /// Placeholder for the extra parameters to be provided if the `extra-parameters` header
    /// contains the value `pass-through`, meaning that the extra parameters within the payload
    /// won't be ignored (default `serde` behavior), but rather kept and passed through to the
    /// underlying API
    #[serde(flatten, skip_serializing_if = "HashMap::is_empty")]
    pub extra_parameters: HashMap<String, serde_json::Value>,
}

impl Into<axum::body::Body> for ChatRequest {
    fn into(self) -> axum::body::Body {
        let bytes = serde_json::to_vec(&self).unwrap();
        tracing::debug!(
            "Serialized ChatRequest JSON: {}",
            String::from_utf8_lossy(&bytes)
        );
        axum::body::Body::from(bytes)
    }
}

impl ChatRequest {
    pub fn from_str(
        value: &str,
        extra_parameters: ExtraParameters,
    ) -> Result<Self, serde_json::Error> {
        let mut payload: Self = serde_json::from_str(value)?;

        match extra_parameters {
            ExtraParameters::Error => {
                if payload.extra_parameters.is_empty() {
                    let fields = payload
                        .extra_parameters
                        .keys()
                        .cloned()
                        .collect::<Vec<_>>()
                        .join(",");

                    return Err(serde::de::Error::custom(format!(
                        "As the header `extra-parameters` is set to `error`, since the following parameters have been provided {}, and those are not defined within the Azure AI Model Inference API specification have been provided, the request failed!",
                        fields
                    )));
                }
                payload.extra_parameters = HashMap::new();
            }
            ExtraParameters::Drop => {
                payload.extra_parameters = HashMap::new();
            }
            ExtraParameters::PassThrough => (),
        }
        Ok(payload)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_full_structure() {
        let payload = json!({
            "messages": [
                {"role": "system", "content": "You are a pirate"},
                {"role": "user", "content": "Where's Paris?"},
                {"role": "tool", "content": "In France", "tool_call_id": "456"}
            ]
        });

        let input: ChatRequest = serde_json::from_value(payload.clone()).unwrap();
        assert_eq!(input.messages.len(), 3);

        let serialized = serde_json::to_value(input).unwrap();
        assert_eq!(serialized, payload);
    }
}
