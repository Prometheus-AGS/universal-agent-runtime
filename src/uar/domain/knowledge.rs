// Use String for ISO8601/RFC3339 to avoid chrono serde feature dominance issues

use serde::{Deserialize, Deserializer, Serialize};
use uuid::Uuid;

/// SurrealDB / older clients may persist JSON `null` for string columns. Plain `String`
/// rejects `null`; these helpers coerce null and empty to defaults so reads stay compatible.
fn deserialize_kb_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    Option::<String>::deserialize(deserializer).map(|o| o.unwrap_or_default())
}

fn deserialize_kb_rfc3339<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    Option::<String>::deserialize(deserializer).map(|o| {
        o.filter(|s| !s.is_empty())
            .unwrap_or_else(|| "1970-01-01T00:00:00Z".to_string())
    })
}

fn deserialize_embedding_provider<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    Option::<String>::deserialize(deserializer).map(|o| match o {
        None => KbConfig::default_embedding_provider(),
        Some(s) if s.trim().is_empty() => KbConfig::default_embedding_provider(),
        Some(s) => s,
    })
}

fn deserialize_embedding_model<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    Option::<String>::deserialize(deserializer).map(|o| match o {
        None => KbConfig::default_embedding_model(),
        Some(s) if s.trim().is_empty() => KbConfig::default_embedding_model(),
        Some(s) => s,
    })
}

fn deserialize_file_processor<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    Option::<String>::deserialize(deserializer).map(|o| match o {
        None => KbConfig::default_file_processor(),
        Some(s) if s.trim().is_empty() => KbConfig::default_file_processor(),
        Some(s) => s,
    })
}

fn default_chunk_strategy() -> crate::uar::rag::chunking::ChunkingStrategy {
    crate::uar::rag::chunking::ChunkingStrategy::Recursive { size: 512 }
}

fn deserialize_chunk_strategy<'de, D>(
    deserializer: D,
) -> Result<crate::uar::rag::chunking::ChunkingStrategy, D::Error>
where
    D: Deserializer<'de>,
{
    Option::<crate::uar::rag::chunking::ChunkingStrategy>::deserialize(deserializer)
        .map(|o| o.unwrap_or_else(default_chunk_strategy))
}

fn deserialize_kb_config_flex<'de, D>(deserializer: D) -> Result<KbConfig, D::Error>
where
    D: Deserializer<'de>,
{
    Option::<KbConfig>::deserialize(deserializer).map(|o| o.unwrap_or_default())
}

fn deserialize_document_status_flex<'de, D>(deserializer: D) -> Result<DocumentStatus, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error as _;

    let value = serde_json::Value::deserialize(deserializer)?;
    if value.is_null() {
        return Ok(DocumentStatus::Pending);
    }

    if let Ok(status) = serde_json::from_value::<DocumentStatus>(value.clone()) {
        return Ok(status);
    }

    let label = value
        .as_str()
        .or_else(|| value.get("status").and_then(serde_json::Value::as_str))
        .unwrap_or_default();

    match label {
        "pending" | "Pending" => Ok(DocumentStatus::Pending),
        "processing" | "Processing" => Ok(DocumentStatus::Processing),
        "indexed" | "Indexed" => Ok(DocumentStatus::Indexed),
        "failed" | "Failed" => {
            let error = value
                .get("error")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default()
                .to_string();
            Ok(DocumentStatus::Failed { error })
        }
        other => Err(D::Error::custom(format!(
            "unknown document status '{other}'"
        ))),
    }
}

/// A named knowledge base container for RAG document scoping.
/// Each KB has its own embedding model, chunking strategy, and document collection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeBase {
    #[serde(default, deserialize_with = "deserialize_kb_string")]
    pub id: String,
    /// Unique human-readable name (e.g., "default", "technical-docs")
    #[serde(default, deserialize_with = "deserialize_kb_string")]
    pub name: String,
    /// Optional description of the knowledge base
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default, deserialize_with = "deserialize_kb_config_flex")]
    pub config: KbConfig,
    #[serde(
        default = "default_kb_rfc3339",
        deserialize_with = "deserialize_kb_rfc3339"
    )]
    pub created_at: String, // RFC3339
    #[serde(
        default = "default_kb_rfc3339",
        deserialize_with = "deserialize_kb_rfc3339"
    )]
    pub updated_at: String, // RFC3339
}

fn default_kb_rfc3339() -> String {
    "1970-01-01T00:00:00Z".to_string()
}

/// Configuration for a knowledge base's processing pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KbConfig {
    /// Embedding provider: "fastembed", "openai", "mistral"
    #[serde(
        default = "KbConfig::default_embedding_provider",
        deserialize_with = "deserialize_embedding_provider"
    )]
    pub embedding_provider: String,
    /// Model ID for the embedding provider
    #[serde(
        default = "KbConfig::default_embedding_model",
        deserialize_with = "deserialize_embedding_model"
    )]
    pub embedding_model: String,
    /// Vector dimensions (None = use model default)
    #[serde(default)]
    pub vector_dimensions: Option<usize>,
    /// File processor: "auto", "unstructured", "mistral", "kreuzberg"
    #[serde(
        default = "KbConfig::default_file_processor",
        deserialize_with = "deserialize_file_processor"
    )]
    pub file_processor: String,
    /// Chunking strategy for document processing
    #[serde(
        default = "default_chunk_strategy",
        deserialize_with = "deserialize_chunk_strategy"
    )]
    pub chunk_strategy: crate::uar::rag::chunking::ChunkingStrategy,
}

impl KbConfig {
    /// Default embedding provider (fastembed for local inference)
    pub fn default_embedding_provider() -> String {
        "fastembed".to_string()
    }

    /// Default embedding model
    pub fn default_embedding_model() -> String {
        "BAAI/bge-small-en-v1.5".to_string()
    }

    /// Default file processor
    pub fn default_file_processor() -> String {
        "kreuzberg".to_string()
    }
}

impl Default for KbConfig {
    fn default() -> Self {
        Self {
            embedding_provider: Self::default_embedding_provider(),
            embedding_model: Self::default_embedding_model(),
            vector_dimensions: None,
            file_processor: Self::default_file_processor(),
            chunk_strategy: crate::uar::rag::chunking::ChunkingStrategy::Recursive { size: 512 },
        }
    }
}

/// A chunk of knowledge from a processed document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeChunk {
    pub id: Uuid,
    pub kb_id: String,
    /// Optional reference to the source document
    #[serde(default)]
    pub document_id: Option<String>,
    pub content: String,
    pub metadata: Option<serde_json::Value>,
    #[serde(default)]
    pub embedding: Vec<f32>,
    pub created_at: String, // RFC3339
}

/// A search result matching a knowledge chunk.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeMatch {
    pub chunk: KnowledgeChunk,
    pub score: f32,
}

/// Tracks a source document ingested into a knowledge base.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeDocument {
    pub id: String,
    pub kb_id: String,
    pub filename: String,
    #[serde(default)]
    pub file_path: Option<String>,
    pub mime_type: Option<String>,
    #[serde(default)]
    pub chunk_count: usize,
    #[serde(default, deserialize_with = "deserialize_document_status_flex")]
    pub status: DocumentStatus,
    pub created_at: String, // RFC3339
    pub updated_at: String, // RFC3339
}

/// Status of document processing.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum DocumentStatus {
    #[default]
    Pending,
    Processing,
    Indexed,
    Failed {
        error: String,
    },
}
