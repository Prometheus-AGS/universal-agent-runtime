//! Version-one request and response models.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Secret-free configured provider returned by UAR administration APIs.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProviderView {
    pub id: String,
    pub display_name: String,
    pub base_url: String,
    pub protocol: Value,
    pub default_model: Option<String>,
    #[serde(default)]
    pub models: Vec<ProviderModelConfig>,
    pub enabled: bool,
    pub credential_configured: bool,
}

/// Provider model route with UAR-owned enablement.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProviderModelConfig {
    pub id: String,
    pub display_name: Option<String>,
    pub context_window: Option<u32>,
    pub supports_vision: bool,
    pub supports_tools: bool,
    #[serde(default)]
    pub supports_reasoning: bool,
    #[serde(default)]
    pub supports_structured_output: bool,
    #[serde(default = "sdk_default_true")]
    pub supports_streaming: bool,
    pub max_output_tokens: Option<u32>,
    #[serde(default = "sdk_default_true")]
    pub enabled: bool,
}

const fn sdk_default_true() -> bool {
    true
}

/// Provider configuration mutation. API keys are write-only.
#[derive(Debug, Clone, Serialize)]
pub struct SaveProviderConfig {
    pub id: String,
    pub display_name: String,
    pub base_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    pub protocol: Value,
    pub default_model: Option<String>,
    pub models: Vec<ProviderModelConfig>,
    pub enabled: bool,
}

/// Configured-provider list envelope.
#[derive(Debug, Clone, Deserialize)]
pub struct ProvidersResponse {
    pub providers: Vec<ProviderView>,
    pub default_id: Option<String>,
}

/// Result from a live provider credential/endpoint validation call.
#[derive(Debug, Clone, Deserialize)]
pub struct ProviderTestResponse {
    pub ok: bool,
    pub provider_id: String,
    pub model_id: String,
    pub latency_ms: u128,
    pub received_text: bool,
}

/// OpenAI-compatible chat message.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChatMessage {
    /// Role (`system`, `user`, `assistant`, or `tool`).
    pub role: String,
    /// Text or structured content.
    pub content: Value,
}

impl ChatMessage {
    /// Construct a text message.
    pub fn text(role: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            role: role.into(),
            content: Value::String(content.into()),
        }
    }
}

/// Chat completion request, including tool and structured-output controls.
#[derive(Debug, Clone, Serialize, Default)]
pub struct ChatCompletionRequest {
    /// Conversation messages.
    pub messages: Vec<ChatMessage>,
    /// Optional provider/model selector.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Stream response chunks.
    #[serde(default)]
    pub stream: bool,
    /// OpenAI-compatible tool declarations.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tools: Vec<Value>,
    /// Structured-output declaration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<Value>,
    /// Stable conversation/session id used for history and scoped policy.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    /// Persisted UAR agent id. Omit to use the protected orchestrator/default.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    /// Typed per-turn scoped run policy.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_policy: Option<Value>,
    /// Streaming vocabulary (`openai`, `agui`, `dual`, or `agui_spec`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_mode: Option<String>,
}

/// Chat completion response. Unknown provider fields are retained.
#[derive(Debug, Clone, Deserialize)]
pub struct ChatCompletionResponse {
    /// Completion identifier.
    #[serde(default)]
    pub id: String,
    /// Provider response body.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, Value>,
}

/// Embedding request.
#[derive(Debug, Clone, Serialize)]
pub struct EmbeddingRequest {
    /// Input strings.
    pub input: Vec<String>,
    /// Optional embedding model.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
}
/// One embedding vector.
#[derive(Debug, Clone, Deserialize)]
pub struct Embedding {
    /// Position in input.
    pub index: usize,
    /// Vector values.
    pub embedding: Vec<f32>,
}
/// Embedding response.
#[derive(Debug, Clone, Deserialize)]
pub struct EmbeddingResponse {
    /// Generated vectors.
    pub data: Vec<Embedding>,
}

/// Run creation request.
#[derive(Debug, Clone, Serialize)]
pub struct CreateRunRequest {
    /// Compiled agent artifact.
    pub artifact: Value,
    /// User input.
    pub input: String,
    /// Optional session identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
}
/// Run creation/resume response.
#[derive(Debug, Clone, Deserialize)]
pub struct RunResponse {
    /// New run identifier.
    pub run_id: String,
    /// Relative stream URL.
    pub stream_url: String,
    /// Source run when resumed.
    #[serde(default)]
    pub resumed_from_run_id: Option<String>,
    /// Source checkpoint.
    #[serde(default)]
    pub checkpoint_id: Option<String>,
}
/// Resume request.
#[derive(Debug, Clone, Serialize)]
pub struct ResumeRunRequest {
    /// Compiled agent artifact.
    pub artifact: Value,
    /// Optional replacement input.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<String>,
    /// Optional session.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
}
/// Cancellation result.
#[derive(Debug, Clone, Deserialize)]
pub struct CancelRunResponse {
    /// Whether a live run was cancelled.
    pub cancelled: bool,
}
/// Response submitted for a UAR A2UI input artifact.
#[derive(Debug, Clone, Serialize)]
pub struct ArtifactResponseRequest {
    /// Artifact id that generated the input request.
    pub artifact_id: String,
    /// User response payload.
    pub response: Value,
}
/// Acknowledgement for an accepted A2UI artifact response.
#[derive(Debug, Clone, Deserialize)]
pub struct ArtifactResponseAck {
    /// Owning run id.
    pub run_id: String,
    /// Artifact id that accepted the response.
    pub artifact_id: String,
    /// New run that continues the same conversation after the interaction.
    pub continuation_run_id: String,
    /// Acceptance status.
    pub status: String,
}
/// Standard A2UI user action sent back to a rendered surface.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct A2uiActionRequest {
    pub surface_id: String,
    pub name: String,
    pub source_component_id: String,
    pub timestamp: Option<String>,
    pub context: Value,
    pub a2ui_client_data_model: Option<Value>,
}

/// Acknowledgement containing the real continuation run.
#[derive(Debug, Clone, Deserialize)]
pub struct A2uiActionAck {
    pub run_id: String,
    pub continuation_run_id: String,
    pub status: String,
}
/// Persisted run checkpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    /// Checkpoint identifier.
    pub id: String,
    /// Owning run.
    pub run_id: String,
    /// Graph node.
    pub node_id: String,
    /// Iteration number.
    pub iteration: usize,
    /// Saved graph state.
    pub state: Value,
    /// Creation timestamp.
    pub created_at: String,
}
/// Checkpoint listing.
#[derive(Debug, Clone, Deserialize)]
pub struct CheckpointListResponse {
    /// Owning run.
    pub run_id: String,
    /// Ordered checkpoints.
    pub checkpoints: Vec<Checkpoint>,
}

/// A decoded server-sent event.
#[derive(Debug, Clone, PartialEq)]
pub struct StreamEvent {
    /// Optional event identifier.
    pub id: Option<String>,
    /// Optional event type.
    pub event: Option<String>,
    /// Parsed JSON payload or string fallback.
    pub data: Value,
}

/// Tool execution request.
#[derive(Debug, Clone, Serialize)]
pub struct ToolCallRequest {
    /// Tool arguments.
    pub arguments: Value,
}
/// Tool execution result.
#[derive(Debug, Clone, Deserialize)]
pub struct ToolCallResponse {
    /// Result body.
    #[serde(flatten)]
    pub result: serde_json::Map<String, Value>,
}

/// Knowledge base configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct KnowledgeBaseConfig {
    /// Embedding provider.
    #[serde(default)]
    pub embedding_provider: String,
    /// Embedding model.
    #[serde(default)]
    pub embedding_model: String,
    /// Vector dimensions.
    pub vector_dimensions: Option<usize>,
    /// File processor.
    #[serde(default)]
    pub file_processor: String,
    /// Chunk strategy.
    #[serde(default)]
    pub chunk_strategy: String,
}
/// Knowledge base.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeBase {
    /// Identifier.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Optional description.
    pub description: Option<String>,
    /// Configuration.
    #[serde(default)]
    pub config: KnowledgeBaseConfig,
    /// Created timestamp.
    pub created_at: String,
    /// Updated timestamp.
    pub updated_at: String,
    /// Stored documents.
    #[serde(default)]
    pub document_count: usize,
}
/// Create a knowledge base.
#[derive(Debug, Clone, Serialize)]
pub struct CreateKnowledgeBaseRequest {
    /// Name.
    pub name: String,
    /// Description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Optional config.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<KnowledgeBaseConfig>,
}
/// Update a knowledge base.
#[derive(Debug, Clone, Serialize, Default)]
pub struct UpdateKnowledgeBaseRequest {
    /// New name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// New description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// New config.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<KnowledgeBaseConfig>,
}
/// Knowledge document.
#[derive(Debug, Clone, Deserialize)]
pub struct Document {
    /// Identifier.
    pub id: String,
    /// Knowledge-base identifier.
    pub kb_id: String,
    /// Filename.
    pub filename: String,
    /// MIME type.
    pub mime_type: Option<String>,
    /// Chunk count.
    pub chunk_count: usize,
    /// Processing status.
    pub status: String,
    /// Failure detail.
    pub error_message: Option<String>,
}
/// Search request.
#[derive(Debug, Clone, Serialize)]
pub struct SearchRequest {
    /// Query text.
    pub query: String,
    /// Maximum results.
    pub limit: usize,
    /// Minimum score.
    pub min_score: f32,
}
/// Search result.
#[derive(Debug, Clone, Deserialize)]
pub struct SearchResult {
    /// Chunk content.
    pub content: String,
    /// Similarity score.
    pub score: f32,
    /// Metadata.
    pub metadata: Value,
    /// Source document.
    pub document_id: Option<String>,
}
/// Search response.
#[derive(Debug, Clone, Deserialize)]
pub struct SearchResponse {
    /// Matching chunks.
    pub results: Vec<SearchResult>,
}
/// Ingest request.
#[derive(Debug, Clone, Serialize)]
pub struct IngestRequest {
    /// Source content.
    pub content: String,
    /// Optional metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
}
/// Ingest response (runtime fields are retained for compatibility).
#[derive(Debug, Clone, Deserialize)]
pub struct IngestResponse {
    /// Runtime response.
    #[serde(flatten)]
    pub result: serde_json::Map<String, Value>,
}

#[cfg(test)]
mod tests {
    use super::ProviderModelConfig;

    #[test]
    fn provider_model_capabilities_round_trip() {
        let model = ProviderModelConfig {
            id: "gemma-4-e2b".to_owned(),
            display_name: Some("Gemma 4 E2B".to_owned()),
            context_window: Some(8_192),
            supports_vision: false,
            supports_tools: true,
            supports_reasoning: true,
            supports_structured_output: true,
            supports_streaming: true,
            max_output_tokens: Some(1_024),
            enabled: true,
        };

        let json = serde_json::to_value(&model).expect("model should serialize");
        let restored: ProviderModelConfig =
            serde_json::from_value(json).expect("model should deserialize");

        assert_eq!(restored, model);
    }
}
