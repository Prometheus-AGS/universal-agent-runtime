use crate::uar::runtime::matching::ClassifierConfig;
use clap::Parser;
use config::{Config, Environment};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::time::Duration;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Config file path
    #[arg(short, long, env = "CONFIG_FILE")]
    pub config: Option<String>,

    /// Port to listen on
    #[arg(long, env = "PORT")]
    pub port: Option<u16>,

    /// Require JWT authentication
    #[arg(long, env = "JWT_REQUIRED")]
    pub jwt_required: Option<bool>,

    /// Enable rate limiting
    #[arg(long, env = "RATE_LIMIT_ENABLED")]
    pub rate_limit_enabled: Option<bool>,

    /// Disable timeout middleware
    #[arg(long, env = "TIMEOUT_DISABLED")]
    pub timeout_disabled: Option<bool>,

    /// Enable external cache (Redis)
    #[arg(long, env = "EXTERNAL_CACHE_ENABLED")]
    pub external_cache_enabled: Option<bool>,

    /// Default LLM model (provider/model format, e.g., "openai/gpt-4o")
    #[arg(long, env = "LLM_MODEL")]
    pub llm_model: Option<String>,

    /// LLM API key
    #[arg(long, env = "LLM_API_KEY")]
    pub llm_api_key: Option<String>,

    /// LLM base URL override
    #[arg(long, env = "LLM_BASE_URL")]
    pub llm_base_url: Option<String>,

    /// LLM protocol (auto, chat, responses)
    #[arg(long, env = "LLM_PROTOCOL")]
    pub llm_protocol: Option<String>,

    /// LLM budget limit (USD)
    #[arg(long, env = "UAR_LLM__BUDGET__GLOBAL_LIMIT")]
    pub llm_budget_limit: Option<f64>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub security: SecurityConfig,
    pub resilience: ResilienceConfig,
    pub persistence: PersistenceConfig,
    #[serde(default)]
    pub file_processing: FileProcessingConfig,
    #[serde(default)]
    pub unstructured: Option<UnstructuredConfig>,
    #[serde(default)]
    pub mistral_ocr: Option<MistralConfig>,
    #[serde(default)]
    pub kreuzberg: Option<KreuzbergConfig>,
    #[serde(default)]
    pub vision: VisionConfig,
    /// Model files configuration (tokenizer, embeddings, etc.)
    #[serde(default)]
    pub models: ModelsConfig,
    #[serde(default)]
    pub knowledge_bases: KnowledgeBasesConfig,
    /// Intent classifier configuration for skill matching
    #[serde(default)]
    pub intent_classifier: ClassifierConfig,
    /// LLM configuration (liter-llm unified client)
    #[serde(default)]
    pub llm: LlmConfig,
    /// LLM provider configurations (multi-provider support)
    #[serde(default)]
    pub providers: Vec<crate::llm::registry::ProviderConfig>,
    /// Agent memory system configuration
    #[serde(default)]
    pub memory: MemoryConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub port: u16,
    pub host: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SecurityConfig {
    pub jwt_required: bool,
    pub jwt_secret: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ResilienceConfig {
    pub rate_limit_enabled: bool,
    #[serde(default)]
    pub timeout_disabled: bool,
    pub requests_per_second: f32,
    pub burst_size: f32,
    #[serde(default = "ResilienceConfig::default_request_timeout_ms")]
    pub request_timeout_ms: u64,
    #[serde(default = "ResilienceConfig::default_stream_start_timeout_ms")]
    pub stream_start_timeout_ms: u64,
    #[serde(default = "ResilienceConfig::default_retries_enabled")]
    pub retries_enabled: bool,
    #[serde(default = "ResilienceConfig::default_retry_max_attempts")]
    pub retry_max_attempts: u32,
    #[serde(default = "ResilienceConfig::default_retry_base_delay_ms")]
    pub retry_base_delay_ms: u64,
    #[serde(default = "ResilienceConfig::default_retry_backoff_multiplier")]
    pub retry_backoff_multiplier: f32,
    #[serde(default = "ResilienceConfig::default_retry_max_delay_ms")]
    pub retry_max_delay_ms: u64,
    #[serde(default = "ResilienceConfig::default_retry_jitter_mode")]
    pub retry_jitter_mode: String,
    #[serde(default = "ResilienceConfig::default_retry_respect_retry_after")]
    pub retry_respect_retry_after: bool,
    #[serde(default = "ResilienceConfig::default_retryable_http_statuses")]
    pub retryable_http_statuses: Vec<u16>,
    #[serde(default = "ResilienceConfig::default_retryable_transport_errors")]
    pub retryable_transport_errors: bool,
    #[serde(default = "ResilienceConfig::default_retry_budget_ms")]
    pub retry_budget_ms: u64,
}

impl ResilienceConfig {
    fn default_request_timeout_ms() -> u64 {
        30_000
    }

    fn default_stream_start_timeout_ms() -> u64 {
        15_000
    }

    fn default_retries_enabled() -> bool {
        true
    }

    fn default_retry_max_attempts() -> u32 {
        3
    }

    fn default_retry_base_delay_ms() -> u64 {
        1_000
    }

    fn default_retry_backoff_multiplier() -> f32 {
        2.0
    }

    fn default_retry_max_delay_ms() -> u64 {
        10_000
    }

    fn default_retry_jitter_mode() -> String {
        "full".to_string()
    }

    fn default_retry_respect_retry_after() -> bool {
        true
    }

    fn default_retryable_http_statuses() -> Vec<u16> {
        vec![408, 425, 429, 500, 502, 503, 504]
    }

    fn default_retryable_transport_errors() -> bool {
        true
    }

    fn default_retry_budget_ms() -> u64 {
        20_000
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct PersistenceConfig {
    pub provider: String,
    pub database_url: String,
    pub vector_dimension: usize,
    pub external_cache_enabled: bool,
    /// Optional username for SurrealDB server mode (ws:// / wss:// / http:// / https://).
    /// Ignored for embedded (file/rocksdb) endpoints.
    #[serde(default)]
    pub surreal_user: Option<String>,
    /// Optional password for SurrealDB server mode.
    /// Ignored for embedded (file/rocksdb) endpoints.
    #[serde(default)]
    pub surreal_pass: Option<String>,
}

/// Configuration for file processing and uploads.
#[derive(Debug, Deserialize, Clone)]
pub struct FileProcessingConfig {
    /// Provider to use: "unstructured", "mistral", "kreuzberg" (local), "auto"
    pub provider: String,
    /// Directory where uploaded files are saved before processing
    pub upload_dir: String,
    /// Maximum number of files per prompt
    pub max_files_per_prompt: usize,
    /// Maximum file size per file in bytes
    pub max_file_size: usize,
    /// Maximum total upload size for all files in a single prompt
    pub max_total_size: usize,
    /// Allowed MIME types (empty = allow all supported types)
    #[serde(default)]
    pub allowed_mime_types: Vec<String>,
}

impl Default for FileProcessingConfig {
    fn default() -> Self {
        Self {
            provider: "auto".to_string(),
            upload_dir: std::env::temp_dir()
                .join("uar-uploads")
                .to_string_lossy()
                .to_string(),
            max_files_per_prompt: 10,
            max_file_size: 50 * 1024 * 1024,   // 50MB
            max_total_size: 100 * 1024 * 1024, // 100MB
            allowed_mime_types: Vec::new(),
        }
    }
}

/// Unstructured.io configuration (hosted or self-hosted).
#[derive(Debug, Deserialize, Clone)]
pub struct UnstructuredConfig {
    /// API URL (default: hosted service)
    #[serde(default = "UnstructuredConfig::default_api_url")]
    pub api_url: String,
    /// API key for the hosted service
    pub api_key: Option<String>,
}

impl UnstructuredConfig {
    fn default_api_url() -> String {
        "https://api.unstructured.io/general/v0/general".to_string()
    }
}

impl Default for UnstructuredConfig {
    fn default() -> Self {
        Self {
            api_url: Self::default_api_url(),
            api_key: None,
        }
    }
}

/// Mistral OCR configuration.
#[derive(Debug, Deserialize, Clone)]
pub struct MistralConfig {
    /// Mistral API key
    pub api_key: Option<String>,
    /// OCR model to use
    #[serde(default = "MistralConfig::default_ocr_model")]
    pub ocr_model: String,
}

impl MistralConfig {
    fn default_ocr_model() -> String {
        "mistral-ocr-latest".to_string()
    }
}

impl Default for MistralConfig {
    fn default() -> Self {
        Self {
            api_key: None,
            ocr_model: Self::default_ocr_model(),
        }
    }
}

/// Kreuzberg local file processing configuration.
/// Kreuzberg is a high-performance document intelligence framework with a Rust core.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Deserialize, Clone)]
pub struct KreuzbergConfig {
    /// Enable OCR for scanned documents and images
    #[serde(default = "KreuzbergConfig::default_ocr_enabled")]
    pub ocr_enabled: bool,
    /// Force OCR even if text layer exists in PDFs
    #[serde(default)]
    pub force_ocr: bool,
    /// OCR backend: "tesseract" (default), "easyocr", "paddleocr"
    #[serde(default = "KreuzbergConfig::default_ocr_backend")]
    pub ocr_backend: String,
    /// OCR language(s), e.g., "eng", "eng+deu" for multiple
    #[serde(default = "KreuzbergConfig::default_ocr_language")]
    pub ocr_language: String,
    /// PDF rendering DPI for OCR (higher = better quality but more memory)
    #[serde(default = "KreuzbergConfig::default_pdf_dpi")]
    pub pdf_dpi: u32,
    /// Extract tables from documents
    #[serde(default = "KreuzbergConfig::default_extract_tables")]
    pub extract_tables: bool,
    /// Extract metadata from documents
    #[serde(default = "KreuzbergConfig::default_extract_metadata")]
    pub extract_metadata: bool,
    /// Output format: "markdown" or "text"
    #[serde(default = "KreuzbergConfig::default_output_format")]
    pub output_format: String,
}

impl KreuzbergConfig {
    fn default_ocr_enabled() -> bool {
        true
    }
    fn default_ocr_backend() -> String {
        "tesseract".to_string()
    }
    fn default_ocr_language() -> String {
        "eng".to_string()
    }
    fn default_pdf_dpi() -> u32 {
        300
    }
    fn default_extract_tables() -> bool {
        true
    }
    fn default_extract_metadata() -> bool {
        true
    }
    fn default_output_format() -> String {
        "markdown".to_string()
    }
}

impl Default for KreuzbergConfig {
    fn default() -> Self {
        Self {
            ocr_enabled: Self::default_ocr_enabled(),
            force_ocr: false,
            ocr_backend: Self::default_ocr_backend(),
            ocr_language: Self::default_ocr_language(),
            pdf_dpi: Self::default_pdf_dpi(),
            extract_tables: Self::default_extract_tables(),
            extract_metadata: Self::default_extract_metadata(),
            output_format: Self::default_output_format(),
        }
    }
}

/// Vision/Image processing configuration.
#[derive(Debug, Deserialize, Clone)]
pub struct VisionConfig {
    /// Explicit vision model (overrides auto-detection)
    pub model: Option<String>,
    /// Auto-detect vision capability from default LLM model
    #[serde(default = "VisionConfig::default_auto_detect")]
    pub auto_detect: bool,
}

impl VisionConfig {
    fn default_auto_detect() -> bool {
        true
    }
}

impl Default for VisionConfig {
    fn default() -> Self {
        Self {
            model: None,
            auto_detect: true,
        }
    }
}

/// Model files configuration (tokenizer, embeddings, etc.).
#[derive(Debug, Deserialize, Clone)]
pub struct ModelsConfig {
    /// Directory containing model files (tokenizer.json, etc.)
    #[serde(default = "ModelsConfig::default_models_dir")]
    pub models_dir: String,
    /// Vector matching threshold for similarity matching
    #[serde(default = "ModelsConfig::default_vector_threshold")]
    pub vector_threshold: f32,
}

impl ModelsConfig {
    fn default_models_dir() -> String {
        // Default to the development path, will be overridden in Docker
        "src/uar/runtime/matching/models".to_string()
    }

    fn default_vector_threshold() -> f32 {
        0.75
    }
}

impl Default for ModelsConfig {
    fn default() -> Self {
        Self {
            models_dir: Self::default_models_dir(),
            vector_threshold: Self::default_vector_threshold(),
        }
    }
}

// =============================================================================
// KNOWLEDGE BASES CONFIGURATION
// =============================================================================

/// Top-level configuration for knowledge bases.
#[derive(Debug, Deserialize, Clone, Default)]
pub struct KnowledgeBasesConfig {
    /// Default knowledge base configuration (always exists)
    #[serde(default)]
    pub default: Option<KnowledgeBaseConfig>,
    /// Additional named knowledge bases
    #[serde(default)]
    pub named: HashMap<String, KnowledgeBaseConfig>,
}

/// Configuration for a single knowledge base.
#[derive(Debug, Deserialize, Clone)]
pub struct KnowledgeBaseConfig {
    /// Unique name for the knowledge base
    pub name: String,
    /// Description of the knowledge base
    #[serde(default)]
    pub description: Option<String>,
    /// Embedding provider: "fastembed", "openai", "mistral"
    #[serde(default = "KnowledgeBaseConfig::default_embedding_provider")]
    pub embedding_provider: String,
    /// Model ID for the embedding provider
    #[serde(default = "KnowledgeBaseConfig::default_embedding_model")]
    pub embedding_model: String,
    /// Vector dimensions (None = use model default)
    #[serde(default)]
    pub vector_dimensions: Option<usize>,
    /// File processor: "auto", "unstructured", "mistral", "kreuzberg"
    #[serde(default = "KnowledgeBaseConfig::default_file_processor")]
    pub file_processor: String,
    /// Chunking strategy configuration
    #[serde(default)]
    pub chunking: ChunkingConfig,
}

impl KnowledgeBaseConfig {
    fn default_embedding_provider() -> String {
        "fastembed".to_string()
    }

    fn default_embedding_model() -> String {
        "BAAI/bge-small-en-v1.5".to_string()
    }

    fn default_file_processor() -> String {
        "auto".to_string()
    }
}

/// Chunking strategy configuration.
#[derive(Debug, Deserialize, Clone)]
pub struct ChunkingConfig {
    /// Strategy: "fixed", "recursive", "token", "sentence", "semantic", "document"
    #[serde(default = "ChunkingConfig::default_strategy")]
    pub strategy: String,
    /// Chunk size in characters (for fixed/recursive) or tokens
    #[serde(default = "ChunkingConfig::default_chunk_size")]
    pub chunk_size: usize,
    /// Semantic similarity threshold (for semantic strategy only)
    #[serde(default)]
    pub semantic_threshold: Option<f32>,
}

impl ChunkingConfig {
    fn default_strategy() -> String {
        "recursive".to_string()
    }

    fn default_chunk_size() -> usize {
        512
    }
}

impl Default for ChunkingConfig {
    fn default() -> Self {
        Self {
            strategy: Self::default_strategy(),
            chunk_size: Self::default_chunk_size(),
            semantic_threshold: None,
        }
    }
}

// =============================================================================
// MEMORY CONFIGURATION
// =============================================================================

/// Configuration for the in-process agent memory system, backed by surreal-memory + SurrealDB/RocksDB.
#[derive(Debug, Deserialize, Clone)]
pub struct MemoryConfig {
    /// Enable the memory system (default: false — opt-in).
    #[serde(default)]
    pub enabled: bool,
    /// Path to the RocksDB data directory for embedded mode.
    #[serde(default = "MemoryConfig::default_db_path")]
    pub db_path: String,
    /// Embedding provider: "openai" | "cohere" | "local"
    #[serde(default = "MemoryConfig::default_embedding_provider")]
    pub embedding_provider: String,
    /// Embedding model identifier (provider-specific).
    #[serde(default = "MemoryConfig::default_embedding_model")]
    pub embedding_model: String,
    /// OpenAI API key for embeddings (used when embedding_provider = "openai").
    #[serde(default)]
    pub openai_api_key: Option<String>,
    /// Cohere API key for embeddings (used when embedding_provider = "cohere").
    #[serde(default)]
    pub cohere_api_key: Option<String>,
    /// Auto-capture memories from conversations after each assistant turn.
    #[serde(default = "MemoryConfig::default_auto_capture")]
    pub auto_capture: bool,
    /// weight applied to the vector-similarity score in hybrid search (0.0–1.0).
    #[serde(default = "MemoryConfig::default_vector_weight")]
    pub vector_weight: f32,
    /// Weight applied to the BM25 full-text score in hybrid search (0.0–1.0).
    #[serde(default = "MemoryConfig::default_bm25_weight")]
    pub bm25_weight: f32,
    /// Inject relevant memory context into LLM prompts before each call.
    #[serde(default = "MemoryConfig::default_inject_context")]
    pub inject_context: bool,
    /// Maximum tokens to include in an injected memory context block.
    #[serde(default = "MemoryConfig::default_max_context_tokens")]
    pub max_context_tokens: u32,
    /// Expose the memory MCP tools over HTTP at `mcp_http_path`.
    #[serde(default = "MemoryConfig::default_mcp_http_enabled")]
    pub mcp_http_enabled: bool,
    /// URL path for the streamable-HTTP MCP memory endpoint.
    #[serde(default = "MemoryConfig::default_mcp_http_path")]
    pub mcp_http_path: String,
    /// SurrealDB namespace for memory storage.
    #[serde(default = "MemoryConfig::default_namespace")]
    pub namespace: String,
    /// SurrealDB database name for memory storage.
    #[serde(default = "MemoryConfig::default_database")]
    pub database: String,
    /// External SurrealDB endpoint (ws:// or wss://). When set, overrides embedded mode.
    /// Env var: UAR_MEMORY__SURREAL_ENDPOINT
    #[serde(default)]
    pub surreal_endpoint: Option<String>,
    /// Username for the external SurrealDB instance.
    /// Env var: UAR_MEMORY__SURREAL_USER
    #[serde(default)]
    pub surreal_user: Option<String>,
    /// Password for the external SurrealDB instance.
    /// Env var: UAR_MEMORY__SURREAL_PASS
    #[serde(default)]
    pub surreal_pass: Option<String>,
}

impl MemoryConfig {
    fn default_db_path() -> String {
        "./data/memory.db".to_string()
    }
    fn default_embedding_provider() -> String {
        "openai".to_string()
    }
    fn default_embedding_model() -> String {
        "text-embedding-3-small".to_string()
    }
    fn default_auto_capture() -> bool {
        true
    }
    fn default_vector_weight() -> f32 {
        0.7
    }
    fn default_bm25_weight() -> f32 {
        0.3
    }
    fn default_inject_context() -> bool {
        true
    }
    fn default_max_context_tokens() -> u32 {
        2000
    }
    fn default_mcp_http_enabled() -> bool {
        true
    }
    fn default_mcp_http_path() -> String {
        "/mcp/memory".to_string()
    }
    fn default_namespace() -> String {
        "uar".to_string()
    }
    fn default_database() -> String {
        "memory".to_string()
    }
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            db_path: Self::default_db_path(),
            embedding_provider: Self::default_embedding_provider(),
            embedding_model: Self::default_embedding_model(),
            openai_api_key: None,
            cohere_api_key: None,
            auto_capture: Self::default_auto_capture(),
            vector_weight: Self::default_vector_weight(),
            bm25_weight: Self::default_bm25_weight(),
            inject_context: Self::default_inject_context(),
            max_context_tokens: Self::default_max_context_tokens(),
            mcp_http_enabled: Self::default_mcp_http_enabled(),
            mcp_http_path: Self::default_mcp_http_path(),
            namespace: Self::default_namespace(),
            database: Self::default_database(),
            surreal_endpoint: None,
            surreal_user: None,
            surreal_pass: None,
        }
    }
}

impl AppConfig {
    /// Load config using the already-parsed `Cli` struct.
    ///
    /// Call this from `main` *after* `Cli::parse()` so that clap's built-in
    /// early-exit flags (`--version`, `--help`) are handled before any config
    /// validation or env-var lookups occur.
    pub fn load_with_cli(cli: Cli) -> Result<Self, config::ConfigError> {
        let mut builder = Config::builder();

        // 1. Default Defaults
        builder = builder
            .set_default("server.port", 3000)?
            .set_default("server.host", "0.0.0.0")?
            .set_default("security.jwt_required", true)?
            .set_default(
                "security.jwt_secret",
                "fallback_secret_change_in_production",
            )?
            .set_default("resilience.rate_limit_enabled", true)?
            .set_default("resilience.timeout_disabled", false)? // Default enabled (timeout_disabled=false)
            .set_default("resilience.requests_per_second", 10.0)?
            .set_default("resilience.burst_size", 20.0)?
            .set_default("resilience.request_timeout_ms", 30_000_i64)?
            .set_default("resilience.stream_start_timeout_ms", 15_000_i64)?
            .set_default("resilience.retries_enabled", true)?
            .set_default("resilience.retry_max_attempts", 3_i64)?
            .set_default("resilience.retry_base_delay_ms", 1_000_i64)?
            .set_default("resilience.retry_backoff_multiplier", 2.0)?
            .set_default("resilience.retry_max_delay_ms", 10_000_i64)?
            .set_default("resilience.retry_jitter_mode", "full")?
            .set_default("resilience.retry_respect_retry_after", true)?
            .set_default(
                "resilience.retryable_http_statuses",
                vec![
                    408_i64, 425_i64, 429_i64, 500_i64, 502_i64, 503_i64, 504_i64,
                ],
            )?
            .set_default("resilience.retryable_transport_errors", true)?
            .set_default("resilience.retry_budget_ms", 20_000_i64)?
            .set_default("persistence.external_cache_enabled", false)?
            // 1536 = text-embedding-3-small; override via UAR_PERSISTENCE__VECTOR_DIMENSION
            .set_default("persistence.vector_dimension", 1536_i64)?
            // File processing defaults
            .set_default("file_processing.provider", "auto")?
            .set_default(
                "file_processing.upload_dir",
                std::env::temp_dir()
                    .join("uar-uploads")
                    .to_string_lossy()
                    .to_string(),
            )?
            .set_default("file_processing.max_files_per_prompt", 10_i64)?
            .set_default("file_processing.max_file_size", 52_428_800_i64)?
            .set_default("file_processing.max_total_size", 104_857_600_i64)?
            // Vision defaults
            .set_default("vision.auto_detect", true)?
            // Models defaults
            .set_default("models.models_dir", "src/uar/runtime/matching/models")?
            .set_default("models.vector_threshold", 0.75)?
            // Memory system defaults
            .set_default("memory.enabled", false)?
            .set_default("memory.db_path", "./data/memory.db")?
            .set_default("memory.embedding_provider", "openai")?
            .set_default("memory.embedding_model", "text-embedding-3-small")?
            .set_default("memory.auto_capture", true)?
            .set_default("memory.vector_weight", 0.7)?
            .set_default("memory.bm25_weight", 0.3)?
            .set_default("memory.inject_context", true)?
            .set_default("memory.max_context_tokens", 2000_i64)?
            .set_default("memory.mcp_http_enabled", true)?
            .set_default("memory.mcp_http_path", "/mcp/memory")?
            .set_default("memory.namespace", "uar")?
            .set_default("memory.database", "memory")?
            // LLM defaults
            .set_default("llm.model", "openai/gpt-4o")?
            .set_default("llm.protocol", "auto")?
            .set_default("llm.timeout_secs", 60_i64)?
            .set_default("llm.max_retries", 3_i64)?
            .set_default("llm.cost_tracking", false)?
            .set_default("llm.tracing", true)?;

        // 2. Config File Loading (Explicit > Implicit)
        if let Some(config_path) = &cli.config {
            builder = builder.add_source(config::File::with_name(config_path));
        } else {
            // implicit: ./config.yaml if exists
            let cwd_config = std::path::Path::new("config.yaml");
            if cwd_config.exists() {
                builder = builder.add_source(config::File::from(cwd_config));
            } else if let Some(mut home_config) = dirs::home_dir() {
                // implicit 2: ~/.uar/config.yaml
                home_config.push(".uar");
                home_config.push("config.yaml");
                if home_config.exists() {
                    builder = builder.add_source(config::File::from(home_config));
                }
            }
        }
        // 4. Manual CLI Overrides
        // ...
        if let Some(rl) = cli.rate_limit_enabled {
            builder = builder.set_override("resilience.rate_limit_enabled", rl)?;
        }
        if let Some(td) = cli.timeout_disabled {
            builder = builder.set_override("resilience.timeout_disabled", td)?;
        }
        if let Some(cache) = cli.external_cache_enabled {
            builder = builder.set_override("persistence.external_cache_enabled", cache)?;
        }

        // LLM CLI overrides
        if let Some(ref model) = cli.llm_model {
            builder = builder.set_override("llm.model", model.clone())?;
        }
        if let Some(ref api_key) = cli.llm_api_key {
            builder = builder.set_override("llm.api_key", api_key.clone())?;
        }
        if let Some(ref base_url) = cli.llm_base_url {
            builder = builder.set_override("llm.base_url", base_url.clone())?;
        }
        if let Some(ref protocol) = cli.llm_protocol {
            builder = builder.set_override("llm.protocol", protocol.clone())?;
        }
        if let Some(budget) = cli.llm_budget_limit {
            builder = builder.set_override("llm.budget.global_limit", budget)?;
        }

        // 4. Manual Environment Overrides
        // ...
        if let Ok(val) = env::var("UAR_RESILIENCE__RATE_LIMIT_ENABLED")
            && let Ok(bool_val) = val.parse::<bool>()
        {
            builder = builder.set_override("resilience.rate_limit_enabled", bool_val)?;
        }
        if let Ok(val) = env::var("UAR_RESILIENCE__TIMEOUT_DISABLED")
            && let Ok(bool_val) = val.parse::<bool>()
        {
            builder = builder.set_override("resilience.timeout_disabled", bool_val)?;
        }
        if let Ok(val) = env::var("UAR_PERSISTENCE__PROVIDER") {
            // ...
            builder = builder.set_override("persistence.provider", val)?;
        }
        if let Ok(val) = env::var("UAR_PERSISTENCE__DATABASE_URL") {
            builder = builder.set_override("persistence.database_url", val)?;
        }
        if let Ok(val) = env::var("UAR_MODELS_DIR") {
            builder = builder.set_override("models.models_dir", val)?;
        }

        // 5. Environment Variables (prefixed with UAR_) - for any keys not explicitly overridden above
        // E.g. UAR_SERVER__PORT=8000, UAR_SECURITY__JWT_SECRET=...
        //
        // prefix_separator("_") is required: config 0.13+ defaults the prefix separator to the
        // same value as `separator` (i.e. "__"), which would only match "UAR__*" vars.
        // Setting it to "_" makes the library strip "UAR_" from the front and then split
        // on "__" for nesting — matching the "UAR_SECTION__KEY" convention used throughout.
        builder = builder.add_source(
            Environment::with_prefix("UAR")
                .prefix_separator("_")
                .separator("__")
                .try_parsing(true),
        );

        // Implicit provider override if DB URL looks like a path or ws://?
        // No, let users set UAR_PERSISTENCE__PROVIDER explicitly.

        // Backward-compat: LLM_* legacy env vars (lower priority than UAR_LLM__* and CLI).
        // These are kept so existing deployments don't break after migrating to liter-llm.
        // Precedence: UAR_LLM__* (via Environment source above) > LLM_* (here) > config file > defaults.
        if let Ok(val) = env::var("LLM_MODEL") {
            builder = builder.set_override("llm.model", val)?;
        }
        if let Ok(val) = env::var("LLM_API_KEY") {
            builder = builder.set_override("llm.api_key", val)?;
        }
        if let Ok(val) = env::var("LLM_BASE_URL") {
            builder = builder.set_override("llm.base_url", val)?;
        }
        if let Ok(val) = env::var("LLM_PROTOCOL") {
            builder = builder.set_override("llm.protocol", val)?;
        }
        // Provider-specific API key shortcuts — map to llm.api_key if llm.api_key not set
        // (UAR_LLM__API_KEY or LLM_API_KEY take precedence).
        for provider_key in &[
            "OPENAI_API_KEY",
            "ANTHROPIC_API_KEY",
            "GROQ_API_KEY",
            "MISTRAL_API_KEY",
            "COHERE_API_KEY",
            "GEMINI_API_KEY",
            "TOGETHER_API_KEY",
            "PERPLEXITY_API_KEY",
        ] {
            if let Ok(val) = env::var(provider_key) {
                // Only set as the default api_key if no explicit key is configured yet.
                // Use a flag to avoid overriding an already-set key.
                builder = builder.set_default(
                    &format!("llm._provider_keys.{}", provider_key.to_lowercase()),
                    val,
                )?;
            }
        }

        let cfg = builder.build()?;
        let mut deserialized: Self = cfg.try_deserialize()?;

        // Backwards compatibility: legacy timeout toggle still maps to "no timeout".
        if deserialized.resilience.timeout_disabled {
            deserialized.resilience.request_timeout_ms = u64::MAX;
        }

        if deserialized.resilience.retryable_http_statuses.is_empty() {
            deserialized.resilience.retryable_http_statuses =
                ResilienceConfig::default_retryable_http_statuses();
        }

        Ok(deserialized)
    }

    /// Load config by parsing args from the environment (convenience wrapper).
    pub fn load() -> Result<Self, config::ConfigError> {
        Self::load_with_cli(Cli::parse())
    }

    /// Load config from a custom arg iterator — used in tests.
    pub fn load_from_args<I, T>(args: I) -> Result<Self, config::ConfigError>
    where
        I: IntoIterator<Item = T>,
        T: Into<std::ffi::OsString> + Clone,
    {
        let cli =
            Cli::try_parse_from(args).map_err(|e| config::ConfigError::Message(e.to_string()))?;
        Self::load_with_cli(cli)
    }
}

/// Load just the merged [`LlmConfig`] from all layered configuration sources.
///
/// Applies layers in ascending priority:
/// compiled defaults → `config.yaml` `llm:` section → `UAR_LLM__*` env vars →
/// `LLM_*` legacy env vars → `cli` arguments.
///
/// Takes ownership of `cli` (a parsed [`Cli`] struct). To load without CLI
/// arguments use `Cli::try_parse()` or construct a default `Cli` with
/// `Cli::parse_from(std::iter::empty::<&str>())`.
///
/// # Errors
///
/// Returns an error if any configuration source cannot be read or parsed.
pub fn load_llm_config(cli: Cli) -> anyhow::Result<LlmConfig> {
    AppConfig::load_with_cli(cli)
        .map(|cfg| cfg.llm)
        .map_err(|e| anyhow::anyhow!("Failed to load LLM configuration: {e}"))
}

/// Build a `liter_llm::ClientConfig` from the merged `LlmConfig`.
pub fn build_client_config(llm: &LlmConfig) -> liter_llm::ClientConfig {
    let api_key = llm
        .api_key
        .clone()
        .or_else(|| std::env::var("LLM_API_KEY").ok())
        .unwrap_or_default();

    let mut builder = liter_llm::ClientConfigBuilder::new(api_key)
        .timeout(Duration::from_secs(llm.timeout_secs))
        .max_retries(llm.max_retries);

    if let Some(ref url) = llm.base_url {
        builder = builder.base_url(url.clone());
    }

    builder.build()
}

// =============================================================================
// LLM CONFIGURATION
// =============================================================================

/// Unified LLM configuration for the liter-llm client.
///
/// Settings are loaded from all layers via [`load_llm_config`]:
/// compiled defaults → `config.yaml` `llm:` section → `UAR_LLM__*` env vars →
/// `LLM_*` legacy env vars → CLI arguments.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    /// Default model in `provider/model` format (e.g., `"openai/gpt-4o"`).
    #[serde(default = "LlmConfig::default_model")]
    pub model: String,
    /// API key for the default provider.
    #[serde(default)]
    pub api_key: Option<String>,
    /// Base URL override (bypasses provider auto-detection).
    #[serde(default)]
    pub base_url: Option<String>,
    /// Protocol hint: `"auto"` | `"chat"` | `"responses"`.
    #[serde(default = "LlmConfig::default_protocol")]
    pub protocol: String,
    /// Whether to enable parallel tool calls.
    #[serde(default)]
    pub parallel_tool_calls: Option<bool>,
    /// Request timeout in seconds.
    #[serde(default = "LlmConfig::default_timeout_secs")]
    pub timeout_secs: u64,
    /// Maximum retries on 429/5xx.
    #[serde(default = "LlmConfig::default_max_retries")]
    pub max_retries: u32,
    /// Cache configuration for Tower middleware.
    #[serde(default)]
    pub cache: Option<LlmCacheConfig>,
    /// Budget enforcement configuration.
    #[serde(default)]
    pub budget: Option<LlmBudgetConfig>,
    /// Rate limiting configuration.
    #[serde(default)]
    pub rate_limit: Option<LlmRateLimitConfig>,
    /// Enable per-request cost tracking.
    #[serde(default)]
    pub cost_tracking: bool,
    /// Enable OpenTelemetry tracing spans.
    #[serde(default = "LlmConfig::default_tracing")]
    pub tracing: bool,
    /// Cooldown after transient errors (seconds).
    #[serde(default)]
    pub cooldown_secs: Option<u64>,
    /// Health check interval (seconds).
    #[serde(default)]
    pub health_check_secs: Option<u64>,
}

impl LlmConfig {
    fn default_model() -> String {
        "openai/gpt-4o".to_string()
    }
    fn default_protocol() -> String {
        "auto".to_string()
    }
    fn default_timeout_secs() -> u64 {
        60
    }
    fn default_max_retries() -> u32 {
        3
    }
    fn default_tracing() -> bool {
        true
    }
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            model: Self::default_model(),
            api_key: None,
            base_url: None,
            protocol: Self::default_protocol(),
            parallel_tool_calls: None,
            timeout_secs: Self::default_timeout_secs(),
            max_retries: Self::default_max_retries(),
            cache: None,
            budget: None,
            rate_limit: None,
            cost_tracking: false,
            tracing: Self::default_tracing(),
            cooldown_secs: None,
            health_check_secs: None,
        }
    }
}

/// Cache configuration for the liter-llm Tower middleware.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmCacheConfig {
    /// Maximum number of cached entries.
    #[serde(default = "LlmCacheConfig::default_max_entries")]
    pub max_entries: usize,
    /// TTL for cached entries (seconds).
    #[serde(default = "LlmCacheConfig::default_ttl_secs")]
    pub ttl_secs: u64,
}

impl LlmCacheConfig {
    fn default_max_entries() -> usize {
        1000
    }
    fn default_ttl_secs() -> u64 {
        3600
    }
}

/// Budget enforcement configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmBudgetConfig {
    /// Global spending limit in USD.
    pub global_limit: f64,
    /// Per-model spending limits.
    #[serde(default)]
    pub model_limits: HashMap<String, f64>,
    /// Enforcement mode: `"hard"` (reject) or `"soft"` (warn).
    #[serde(default = "LlmBudgetConfig::default_enforcement")]
    pub enforcement: String,
}

impl LlmBudgetConfig {
    fn default_enforcement() -> String {
        "hard".to_string()
    }
}

/// Rate limiting configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmRateLimitConfig {
    /// Requests per minute.
    #[serde(default)]
    pub rpm: Option<u32>,
    /// Tokens per minute.
    #[serde(default)]
    pub tpm: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn base_cli() -> (Cli, PathBuf) {
        let cfg_path = write_test_config_file();
        (
            Cli {
                config: Some(cfg_path.to_string_lossy().to_string()),
                port: None,
                jwt_required: None,
                rate_limit_enabled: None,
                timeout_disabled: None,
                external_cache_enabled: None,
                llm_model: None,
                llm_api_key: None,
                llm_base_url: None,
                llm_protocol: None,
                llm_budget_limit: None,
            },
            cfg_path,
        )
    }

    fn write_test_config_file() -> PathBuf {
        let mut path = std::env::temp_dir();
        path.push(format!("uar-config-test-{}.yaml", uuid::Uuid::new_v4()));
        let yaml = r#"
security:
  jwt_secret: "test-secret"
persistence:
  provider: "surreal"
  database_url: "rocksdb://./data/test-config"
"#;
        fs::write(&path, yaml).expect("test config should be writable");
        path
    }

    #[test]
    fn resilience_defaults_include_retry_and_timeout_fields() {
        let (cli, cfg_path) = base_cli();
        let cfg = AppConfig::load_with_cli(cli).expect("config should load");
        assert_eq!(cfg.resilience.request_timeout_ms, 30_000);
        assert_eq!(cfg.resilience.stream_start_timeout_ms, 15_000);
        assert!(cfg.resilience.retries_enabled);
        assert_eq!(cfg.resilience.retry_max_attempts, 3);
        assert_eq!(
            cfg.resilience.retryable_http_statuses,
            vec![408, 425, 429, 500, 502, 503, 504]
        );
        let _ = fs::remove_file(cfg_path);
    }

    #[test]
    fn legacy_timeout_disabled_maps_to_no_timeout_budget() {
        let (mut cli, cfg_path) = base_cli();
        cli.timeout_disabled = Some(true);
        let cfg = AppConfig::load_with_cli(cli).expect("config should load");
        assert!(cfg.resilience.timeout_disabled);
        assert_eq!(cfg.resilience.request_timeout_ms, u64::MAX);
        let _ = fs::remove_file(cfg_path);
    }
}
