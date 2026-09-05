use crate::uar::runtime::matching::ClassifierConfig;
use clap::{Parser, Subcommand};
use config::{Config, Environment};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::time::Duration;

const FALLBACK_JWT_SECRET: &str = "fallback_secret_change_in_production";

#[cfg(feature = "vault")]
pub mod vault;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Environment file loaded before telemetry and application configuration.
    #[arg(long, env = "UAR_ENV_FILE")]
    pub env_file: Option<String>,

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

    // --- Model failover ---
    /// Enable runtime model failover
    #[arg(long, env = "UAR_FAILOVER__ENABLED")]
    pub failover_enabled: Option<bool>,

    // --- Native tools ---
    /// Enable file_read / file_write / file_patch native tools
    #[arg(long, env = "UAR_NATIVE_TOOLS__FILE_TOOLS_ENABLED")]
    pub native_file_tools: Option<bool>,

    /// Enable web_fetch native tool
    #[arg(long, env = "UAR_NATIVE_TOOLS__WEB_FETCH_ENABLED")]
    pub native_web_fetch: Option<bool>,

    /// Enable terminal_exec native tool
    #[arg(long, env = "UAR_NATIVE_TOOLS__TERMINAL_EXEC_ENABLED")]
    pub native_terminal_exec: Option<bool>,

    // --- Skill evolution ---
    /// Enable post-run skill evolution
    #[arg(long, env = "UAR_SKILL_EVOLUTION__ENABLED")]
    pub skill_evolution_enabled: Option<bool>,

    /// Model to use for skill evolution reflection (provider/model format)
    #[arg(long, env = "UAR_SKILL_EVOLUTION__TRIGGER_MODEL")]
    pub skill_evolution_model: Option<String>,

    // --- ACP server ---
    /// Enable ACP server endpoint
    #[arg(long, env = "UAR_ACP__ENABLED")]
    pub acp_enabled: Option<bool>,

    /// ACP server path prefix
    #[arg(long, env = "UAR_ACP__PATH")]
    pub acp_path: Option<String>,

    /// Optional subcommand. When omitted, the binary starts the server.
    #[command(subcommand)]
    pub command: Option<Command>,

    /// Treat config source conflicts (e.g., env var vs config file) as hard errors.
    #[arg(long, env = "UAR_STRICT_CONFIG")]
    pub strict_config: bool,
}

/// Top-level binary subcommands. Absent ⇒ run the server (default).
#[derive(Subcommand, Debug, Clone)]
pub enum Command {
    /// Run under the Windows Service Control Manager.
    #[cfg(windows)]
    Service,
    /// Run and inspect the LLM evaluation harness.
    Eval {
        #[command(subcommand)]
        action: EvalAction,
    },
    /// Compile a UAR-AGENT-MD document into a signed `CompiledDescriptor`
    /// (CH-15: used by CI to compile+sign `templates/*.agent.md` as release
    /// artifacts, and available standalone for any `.agent.md` document).
    Compile {
        /// Path to the `.agent.md` source document.
        path: String,
        /// Output path for the compiled+signed JSON. Prints to stdout when omitted.
        #[arg(long)]
        out: Option<String>,
    },
}

/// `eval` subcommands.
#[derive(Subcommand, Debug, Clone)]
pub enum EvalAction {
    /// Run a suite, score it, persist results, and gate on regression.
    Run {
        /// Suite name (resolved to `evals/<suite>.{yaml,yml,json}`) or a direct path.
        suite: String,
        /// Regression threshold: a scorer regresses if its mean drops below the
        /// baseline by more than this (delta-vs-baseline).
        #[arg(long, default_value_t = 0.05)]
        threshold: f32,
        /// Directory for result + baseline files.
        #[arg(long, default_value = "evals/results")]
        results_dir: String,
        /// Save this run's summary as the new baseline (no regression gating).
        #[arg(long)]
        update_baseline: bool,
        /// Strict gate: fail (non-zero exit) when no baseline exists, instead of
        /// passing silently. Off by default; the scheduled CI tier opts in.
        #[arg(long)]
        require_baseline: bool,
    },
    /// List stored result files (optionally filtered by suite).
    List {
        /// Only list results for this suite.
        #[arg(long)]
        suite: Option<String>,
        /// Directory holding result files.
        #[arg(long, default_value = "evals/results")]
        results_dir: String,
    },
    /// Print the stored baseline summary for a suite.
    Baseline {
        /// Suite name.
        suite: String,
        /// Directory holding the baseline file.
        #[arg(long, default_value = "evals/results")]
        results_dir: String,
    },
}

/// Skill matching suggests catalog entries or activates legacy overlays.
#[derive(
    Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq, schemars::JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum SkillActivationMode {
    #[default]
    LegacyOverlay,
    Catalog,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, schemars::JsonSchema)]
pub struct HarnessConfig {
    #[serde(default)]
    pub mode: HarnessMode,
    #[serde(default)]
    pub skill_activation_mode: SkillActivationMode,
    #[serde(default)]
    pub skill_reattachment: SkillReattachmentBudget,
}

/// Typed assembly is the default after the recorded parity-and-smoke gate.
#[derive(
    Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq, schemars::JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum HarnessMode {
    /// Deprecated rollback path, retained for one minor release after the flip.
    Legacy,
    Shadow,
    #[default]
    Typed,
}

/// Model-token ceilings for re-attaching active bodies after compaction.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(default)]
pub struct SkillReattachmentBudget {
    pub per_skill_tokens: usize,
    pub total_tokens: usize,
}

impl Default for SkillReattachmentBudget {
    fn default() -> Self {
        Self {
            per_skill_tokens: 5_000,
            total_tokens: 25_000,
        }
    }
}

#[derive(Debug, Deserialize, Clone, schemars::JsonSchema)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub security: SecurityConfig,
    /// Authenticated UAR peers eligible for governed outbound A2A delegation.
    #[serde(default)]
    pub a2a: A2aConfig,
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
    /// Progressive runtime migration controls.
    #[serde(default)]
    pub harness: HarnessConfig,
    /// Host-only project instruction discovery; no workspace is trusted by default.
    #[serde(default)]
    pub project_instructions: crate::uar::runtime::project_instructions::ProjectInstructionsConfig,
    /// World-state time precision; defaults to one minute.
    #[serde(default)]
    pub world_state: crate::uar::runtime::world_state::sections::WorldStateConfig,
    /// LLM configuration (liter-llm unified client)
    #[serde(default)]
    pub llm: LlmConfig,
    /// LLM provider configurations (multi-provider support)
    #[serde(default)]
    pub providers: Vec<crate::llm::registry::ProviderConfig>,
    /// Agent memory system configuration
    #[serde(default)]
    pub memory: MemoryConfig,
    /// Sandbox code-execution configuration
    #[serde(default)]
    pub sandbox: SandboxRuntimeConfig,
    /// Runtime model failover configuration.
    #[serde(default)]
    pub failover: FailoverConfig,
    /// Native (in-process) tool configuration.
    #[serde(default)]
    pub native_tools: NativeToolsConfig,
    /// Post-run skill evolution configuration.
    #[serde(default)]
    pub skill_evolution: SkillEvolutionConfig,
    /// Sycophancy detection configuration for LLM response quality.
    #[serde(default)]
    pub sycophancy: SycophancyConfig,
    /// Input guardrail (injection/PII screening) configuration.
    #[serde(default)]
    pub guardrails: GuardrailsConfig,
    /// ACP server endpoint configuration.
    #[serde(default)]
    pub acp: AcpConfig,
    /// Global default context strategy for trimming conversation history before LLM calls.
    /// Defaults to `SlidingWindow { max_messages: 20 }`.
    /// Can be overridden per-agent via agent policy extensions.
    #[serde(default)]
    pub context_strategy: crate::uar::context::ContextStrategy,
}

/// Host-owned A2A peer trust. Graphs and agent artifacts can select only an
/// exact endpoint listed here; credentials never enter graph or artifact data.
#[derive(Clone, Default, Deserialize, schemars::JsonSchema)]
pub struct A2aConfig {
    /// Stable identity this instance presents in the UAR delegation contract.
    #[serde(default)]
    pub instance_id: String,
    /// Exact authenticated agent endpoints accepted for governed delegation.
    #[serde(default)]
    pub trusted_peers: Vec<A2aPeerConfig>,
}

impl std::fmt::Debug for A2aConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("A2aConfig")
            .field("instance_id", &self.instance_id)
            .field("trusted_peers", &self.trusted_peers)
            .finish()
    }
}

/// One exact remote UAR agent binding. Reusing a token for several agents is
/// explicit: each endpoint still needs its own entry and artifact dependency.
#[derive(Clone, Deserialize, schemars::JsonSchema)]
pub struct A2aPeerConfig {
    pub instance_id: String,
    pub agent_id: String,
    pub endpoint: String,
    #[schemars(with = "String")]
    pub bearer_token: secrecy::SecretString,
}

impl std::fmt::Debug for A2aPeerConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("A2aPeerConfig")
            .field("instance_id", &self.instance_id)
            .field("agent_id", &self.agent_id)
            .field("endpoint", &self.endpoint)
            .field("bearer_token", &REDACTED)
            .finish()
    }
}

impl A2aConfig {
    fn validate(&self) -> Result<(), config::ConfigError> {
        use secrecy::ExposeSecret;
        use std::collections::BTreeSet;

        if self.trusted_peers.is_empty() {
            return Ok(());
        }
        if self.instance_id.trim().is_empty() || self.instance_id != self.instance_id.trim() {
            return Err(config::ConfigError::Message(
                "a2a.instance_id must be non-empty when trusted peers are configured".to_owned(),
            ));
        }
        let mut endpoints = BTreeSet::new();
        for peer in &self.trusted_peers {
            if peer.instance_id.trim().is_empty()
                || peer.agent_id.trim().is_empty()
                || peer.instance_id != peer.instance_id.trim()
                || peer.agent_id != peer.agent_id.trim()
                || peer.bearer_token.expose_secret().is_empty()
            {
                return Err(config::ConfigError::Message(
                    "each trusted A2A peer requires non-empty instance_id, agent_id, endpoint, and bearer_token"
                        .to_owned(),
                ));
            }
            let endpoint = reqwest::Url::parse(&peer.endpoint).map_err(|_| {
                config::ConfigError::Message(
                    "trusted A2A peer endpoint must be an absolute URL".to_owned(),
                )
            })?;
            let loopback_http = endpoint.scheme() == "http"
                && endpoint.host_str().is_some_and(|host| {
                    host.eq_ignore_ascii_case("localhost")
                        || host
                            .parse::<std::net::IpAddr>()
                            .is_ok_and(|address| address.is_loopback())
                });
            if endpoint.scheme() != "https" && !loopback_http {
                return Err(config::ConfigError::Message(
                    "trusted A2A peer credentials require HTTPS except on loopback".to_owned(),
                ));
            }
            if endpoint.username() != ""
                || endpoint.password().is_some()
                || endpoint.query().is_some()
                || endpoint.fragment().is_some()
            {
                return Err(config::ConfigError::Message(
                    "trusted A2A peer endpoints cannot contain userinfo, query, or fragment components".to_owned(),
                ));
            }
            if !endpoints.insert(peer.endpoint.clone()) {
                return Err(config::ConfigError::Message(
                    "trusted A2A peer endpoints must be unique".to_owned(),
                ));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize, Clone, schemars::JsonSchema)]
pub struct ServerConfig {
    pub port: u16,
    pub host: String,
    #[serde(default = "ServerConfig::default_shutdown_timeout_secs")]
    pub shutdown_timeout_secs: u64,
    #[serde(default = "ServerConfig::default_log_format")]
    pub log_format: LogFormat,
    /// Port for the A2A v0.3 gRPC transport. Defaults to 50051.
    /// Configurable via `UAR_SERVER__GRPC_PORT`.
    #[serde(default = "ServerConfig::default_grpc_port")]
    pub grpc_port: u16,
}

impl ServerConfig {
    fn default_shutdown_timeout_secs() -> u64 {
        30
    }

    fn default_log_format() -> LogFormat {
        LogFormat::Json
    }

    fn default_grpc_port() -> u16 {
        50051
    }
}

#[derive(Debug, Deserialize, Clone, Default, schemars::JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    #[default]
    Json,
    Compact,
    Pretty,
}

/// Placeholder rendered in `Debug` output in place of any secret value.
/// Keeps `tracing::info!("… {config:?}")` dumps free of plaintext credentials.
const REDACTED: &str = "***redacted***";

/// Mask an optional secret for `Debug`: a present value becomes
/// `Some("***redacted***")`, while `None` is preserved so "not set" stays
/// visible without leaking the value.
fn redact_opt<T>(value: &Option<T>) -> Option<&'static str> {
    value.as_ref().map(|_| REDACTED)
}

pub(crate) fn secret_value_matches(
    expected: &Option<secrecy::SecretString>,
    supplied: Option<&str>,
) -> bool {
    let (Some(expected), Some(supplied)) = (expected.as_ref(), supplied) else {
        return false;
    };
    let expected = secrecy::ExposeSecret::expose_secret(expected).as_bytes();
    let supplied = supplied.as_bytes();
    let mut difference = expected.len() ^ supplied.len();
    for index in 0..expected.len().max(supplied.len()) {
        difference |= usize::from(
            expected.get(index).copied().unwrap_or_default()
                ^ supplied.get(index).copied().unwrap_or_default(),
        );
    }
    difference == 0
}

#[derive(Deserialize, Clone, schemars::JsonSchema)]
pub struct SecurityConfig {
    pub jwt_required: bool,
    /// The JWT signing secret. Wrapped in `SecretString` (rather than a
    /// plain `String`) so the value cannot be accidentally logged or
    /// serialized — callers must explicitly `secrecy::ExposeSecret::expose_secret()`.
    /// The generated JSON Schema represents this as an opaque `string`
    /// (`#[schemars(with = "String")]`) since `SecretString` intentionally
    /// has no `JsonSchema`/`Serialize` impl of its own.
    #[schemars(with = "String")]
    pub jwt_secret: secrecy::SecretString,
    /// Optional JSON Web Key Set endpoint for asymmetric JWT verification.
    #[serde(default)]
    pub jwks_url: Option<String>,
    /// Required JWT issuer for either shared-secret or JWKS verification.
    #[serde(default)]
    pub jwt_issuer: Option<String>,
    /// Required JWT audience for either shared-secret or JWKS verification.
    #[serde(default)]
    pub jwt_audience: Option<String>,
    /// Validate a token's `nbf` claim when it is present.
    #[serde(default = "SecurityConfig::default_jwt_validate_nbf")]
    pub jwt_validate_nbf: bool,
    /// When true (default), protected `/api/uar/settings` endpoints require the
    /// configured `X-UAR-Admin-Key` value.
    /// Set to `false` for trusted local development only.
    #[serde(default = "SecurityConfig::default_settings_mutation_auth_required")]
    pub settings_mutation_auth_required: bool,
    /// Secret accepted by protected settings administration endpoints.
    #[serde(default)]
    #[schemars(with = "Option<String>")]
    pub settings_admin_key: Option<secrecy::SecretString>,
}

impl std::fmt::Debug for SecurityConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SecurityConfig")
            .field("jwt_required", &self.jwt_required)
            .field("jwt_secret", &REDACTED)
            .field("jwks_url", &self.jwks_url)
            .field("jwt_issuer", &self.jwt_issuer)
            .field("jwt_audience", &self.jwt_audience)
            .field("jwt_validate_nbf", &self.jwt_validate_nbf)
            .field(
                "settings_mutation_auth_required",
                &self.settings_mutation_auth_required,
            )
            .field("settings_admin_key", &redact_opt(&self.settings_admin_key))
            .finish()
    }
}

impl SecurityConfig {
    const fn default_jwt_validate_nbf() -> bool {
        true
    }

    fn default_settings_mutation_auth_required() -> bool {
        true
    }

    pub(crate) fn validate(&self) -> Result<(), config::ConfigError> {
        if self.jwt_required
            && secrecy::ExposeSecret::expose_secret(&self.jwt_secret) == FALLBACK_JWT_SECRET
        {
            return Err(config::ConfigError::Message(
                "security.jwt_secret uses the built-in fallback while security.jwt_required=true; set UAR_SECURITY__JWT_SECRET to a deliberate secret or explicitly disable JWT authentication"
                    .to_owned(),
                ));
        }
        if self.settings_mutation_auth_required
            && self
                .settings_admin_key
                .as_ref()
                .is_none_or(|key| secrecy::ExposeSecret::expose_secret(key).is_empty())
        {
            return Err(config::ConfigError::Message(
                "security.settings_admin_key must be configured when security.settings_mutation_auth_required=true"
                    .to_owned(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize, Clone, schemars::JsonSchema)]
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
    #[serde(default = "ResilienceConfig::default_stream_idle_timeout_ms")]
    pub stream_idle_timeout_ms: u64,
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

    fn default_stream_idle_timeout_ms() -> u64 {
        30_000
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

#[derive(Deserialize, Clone, schemars::JsonSchema)]
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
    /// Optional SurrealDB namespace for UAR persistence (default `uar`).
    /// Set this to share a namespace with another Surreal-backed service
    /// (e.g. surreal-memory-server uses `memory`).
    #[serde(default)]
    pub surreal_ns: Option<String>,
    /// Optional SurrealDB database name for UAR persistence (default `uar`).
    #[serde(default)]
    pub surreal_db: Option<String>,
}

impl std::fmt::Debug for PersistenceConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PersistenceConfig")
            .field("provider", &self.provider)
            .field("database_url", &self.database_url)
            .field("vector_dimension", &self.vector_dimension)
            .field("external_cache_enabled", &self.external_cache_enabled)
            .field("surreal_user", &self.surreal_user)
            .field("surreal_pass", &redact_opt(&self.surreal_pass))
            .field("surreal_ns", &self.surreal_ns)
            .field("surreal_db", &self.surreal_db)
            .finish()
    }
}

/// Configuration for file processing and uploads.
#[derive(Debug, Deserialize, Clone, schemars::JsonSchema)]
pub struct FileProcessingConfig {
    /// Provider to use: "kreuzberg" (local), "auto", "unstructured", "mistral", "local"
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
            provider: "kreuzberg".to_string(),
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
#[derive(Debug, Deserialize, Clone, schemars::JsonSchema)]
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
#[derive(Debug, Deserialize, Clone, schemars::JsonSchema)]
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

/// Kreuzberg text-chunking configuration for RAG pipelines.
/// Field names use Rust conventions (max_characters/overlap) — differs from the
/// Python SDK (max_chars/max_overlap). See kreuzberg SKILL.md pitfalls.
#[derive(Debug, Deserialize, Clone, schemars::JsonSchema)]
pub struct KreuzbergChunkingConfig {
    /// Maximum characters per chunk (default: 1000)
    #[serde(default = "KreuzbergChunkingConfig::default_max_characters")]
    pub max_characters: usize,
    /// Overlap between consecutive chunks in characters (default: 200)
    #[serde(default = "KreuzbergChunkingConfig::default_overlap")]
    pub overlap: usize,
}

impl KreuzbergChunkingConfig {
    fn default_max_characters() -> usize {
        1000
    }
    fn default_overlap() -> usize {
        200
    }
}

impl Default for KreuzbergChunkingConfig {
    fn default() -> Self {
        Self {
            max_characters: Self::default_max_characters(),
            overlap: Self::default_overlap(),
        }
    }
}

/// Kreuzberg local file processing configuration.
/// Kreuzberg is a high-performance document intelligence framework with a Rust core.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Deserialize, Clone, schemars::JsonSchema)]
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
    /// Optional chunking config for RAG pipelines (kreuzberg v4 feature)
    #[serde(default)]
    pub chunking: Option<KreuzbergChunkingConfig>,
    /// Maximum accepted input size in bytes before handing content to
    /// kreuzberg's native parsers. Compensating control for RUSTSEC-2026-0187
    /// (lopdf stack overflow) / RUSTSEC-2026-0194 (quick-xml quadratic
    /// attribute-check DoS): both are reachable through kreuzberg's
    /// document-parsing path with no consumer-side opt-out, so bounding the
    /// input size limits the CPU/stack cost a single crafted document can incur.
    #[serde(default = "KreuzbergConfig::default_max_input_bytes")]
    pub max_input_bytes: u64,
    /// Wall-clock timeout (seconds) for a single kreuzberg extraction call.
    /// Compensating control alongside `max_input_bytes` — bounds worst-case
    /// request latency if a crafted document still triggers pathological
    /// parsing cost within the size cap.
    #[serde(default = "KreuzbergConfig::default_extraction_timeout_secs")]
    pub extraction_timeout_secs: u64,
}

impl KreuzbergConfig {
    fn default_ocr_enabled() -> bool {
        false
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
    fn default_max_input_bytes() -> u64 {
        100 * 1024 * 1024 // 100 MiB
    }
    fn default_extraction_timeout_secs() -> u64 {
        60
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
            chunking: None,
            max_input_bytes: Self::default_max_input_bytes(),
            extraction_timeout_secs: Self::default_extraction_timeout_secs(),
        }
    }
}

/// Vision/Image processing configuration.
#[derive(Debug, Deserialize, Clone, schemars::JsonSchema)]
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
#[derive(Debug, Deserialize, Clone, schemars::JsonSchema)]
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
#[derive(Debug, Deserialize, Clone, Default, schemars::JsonSchema)]
pub struct KnowledgeBasesConfig {
    /// Default knowledge base configuration (always exists)
    #[serde(default)]
    pub default: Option<KnowledgeBaseConfig>,
    /// Additional named knowledge bases
    #[serde(default)]
    pub named: HashMap<String, KnowledgeBaseConfig>,
}

/// Configuration for a single knowledge base.
#[derive(Debug, Deserialize, Clone, schemars::JsonSchema)]
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
    /// File processor: "kreuzberg", "auto", "unstructured", "mistral", "local"
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
        "kreuzberg".to_string()
    }
}

/// Chunking strategy configuration.
#[derive(Debug, Deserialize, Clone, schemars::JsonSchema)]
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

/// Configuration for the in-process agent memory system, backed by surreal-memory + SurrealDB/SurrealKV.
#[derive(Deserialize, Clone, schemars::JsonSchema)]
pub struct MemoryConfig {
    /// Enable the memory system (default: false — opt-in).
    #[serde(default)]
    pub enabled: bool,
    /// Path to the SurrealKV data directory for embedded mode.
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

impl std::fmt::Debug for MemoryConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MemoryConfig")
            .field("enabled", &self.enabled)
            .field("db_path", &self.db_path)
            .field("embedding_provider", &self.embedding_provider)
            .field("embedding_model", &self.embedding_model)
            .field("openai_api_key", &redact_opt(&self.openai_api_key))
            .field("cohere_api_key", &redact_opt(&self.cohere_api_key))
            .field("auto_capture", &self.auto_capture)
            .field("vector_weight", &self.vector_weight)
            .field("bm25_weight", &self.bm25_weight)
            .field("inject_context", &self.inject_context)
            .field("max_context_tokens", &self.max_context_tokens)
            .field("mcp_http_enabled", &self.mcp_http_enabled)
            .field("mcp_http_path", &self.mcp_http_path)
            .field("namespace", &self.namespace)
            .field("database", &self.database)
            .field("surreal_endpoint", &self.surreal_endpoint)
            .field("surreal_user", &self.surreal_user)
            .field("surreal_pass", &redact_opt(&self.surreal_pass))
            .finish()
    }
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

// =============================================================================
// SANDBOX RUNTIME CONFIGURATION
// =============================================================================

/// Configuration for the sandboxed code-execution subsystem.
#[derive(Debug, Deserialize, Clone, schemars::JsonSchema)]
pub struct SandboxRuntimeConfig {
    /// Runner selection: "auto", "wasmtime", "remote".
    #[serde(default = "SandboxRuntimeConfig::default_runner")]
    pub runner: String,
    /// URL for the remote sandbox runner (required when runner = "remote").
    #[serde(default)]
    pub remote_url: Option<String>,
    /// Default memory limit per sandbox (MiB).
    #[serde(default = "SandboxRuntimeConfig::default_memory_mib")]
    pub default_memory_mib: u32,
    /// Default execution timeout (seconds).
    #[serde(default = "SandboxRuntimeConfig::default_timeout_secs")]
    pub default_timeout_secs: u64,
    /// Whether sandboxes have network access by default.
    #[serde(default)]
    pub network_enabled: bool,
    /// Maximum number of concurrent sandbox sessions.
    #[serde(default = "SandboxRuntimeConfig::default_max_concurrent")]
    pub max_concurrent: usize,
}

impl SandboxRuntimeConfig {
    fn default_runner() -> String {
        "auto".to_string()
    }
    fn default_memory_mib() -> u32 {
        512
    }
    fn default_timeout_secs() -> u64 {
        300
    }
    fn default_max_concurrent() -> usize {
        10
    }
}

impl Default for SandboxRuntimeConfig {
    fn default() -> Self {
        Self {
            runner: Self::default_runner(),
            remote_url: None,
            default_memory_mib: Self::default_memory_mib(),
            default_timeout_secs: Self::default_timeout_secs(),
            network_enabled: false,
            max_concurrent: Self::default_max_concurrent(),
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
            .set_default("server.port", 1906)?
            .set_default("server.host", "0.0.0.0")?
            .set_default("security.jwt_required", true)?
            .set_default("security.settings_mutation_auth_required", true)?
            .set_default("security.jwt_secret", FALLBACK_JWT_SECRET)?
            .set_default("security.jwt_validate_nbf", true)?
            .set_default("resilience.rate_limit_enabled", true)?
            .set_default("resilience.timeout_disabled", false)? // Default enabled (timeout_disabled=false)
            .set_default("resilience.requests_per_second", 10.0)?
            .set_default("resilience.burst_size", 20.0)?
            .set_default("resilience.request_timeout_ms", 30_000_i64)?
            .set_default("resilience.stream_start_timeout_ms", 15_000_i64)?
            .set_default("resilience.stream_idle_timeout_ms", 30_000_i64)?
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
            .set_default("persistence.provider", "surreal")?
            .set_default("persistence.database_url", "surrealkv://./data/uar.db")?
            .set_default("persistence.external_cache_enabled", false)?
            // 1536 = text-embedding-3-small; override via UAR_PERSISTENCE__VECTOR_DIMENSION
            .set_default("persistence.vector_dimension", 1536_i64)?
            // File processing defaults
            .set_default("file_processing.provider", "kreuzberg")?
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
            .set_default("llm.tracing", true)?
            .set_default("llm.embedding.backend", "fastembed")?
            .set_default("llm.embedding.model", "bge-small-en-v1.5")?
            .set_default("llm.embedding.vector_dimension", 384_i64)?
            .set_default("llm.embedding.batch_size", 32_i64)?
            .set_default(
                "llm.embedding.models_dir",
                "src/uar/runtime/matching/models",
            )?;

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
        if let Some(port) = cli.port {
            builder = builder.set_override("server.port", i64::from(port))?;
        }
        if let Some(jwt_required) = cli.jwt_required {
            builder = builder.set_override("security.jwt_required", jwt_required)?;
        }
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

        // Failover CLI overrides
        if let Some(fe) = cli.failover_enabled {
            builder = builder.set_override("failover.enabled", fe)?;
        }

        // Native tools CLI overrides
        if let Some(ft) = cli.native_file_tools {
            builder = builder.set_override("native_tools.file_tools_enabled", ft)?;
        }
        if let Some(wf) = cli.native_web_fetch {
            builder = builder.set_override("native_tools.web_fetch_enabled", wf)?;
        }
        if let Some(te) = cli.native_terminal_exec {
            builder = builder.set_override("native_tools.terminal_exec_enabled", te)?;
        }

        // Skill evolution CLI overrides
        if let Some(se) = cli.skill_evolution_enabled {
            builder = builder.set_override("skill_evolution.enabled", se)?;
        }
        if let Some(ref m) = cli.skill_evolution_model {
            builder = builder.set_override("skill_evolution.trigger_model", m.clone())?;
        }

        // ACP CLI overrides
        if let Some(ae) = cli.acp_enabled {
            builder = builder.set_override("acp.enabled", ae)?;
        }
        if let Some(ref ap) = cli.acp_path {
            builder = builder.set_override("acp.path", ap.clone())?;
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
        // Provider-specific API key shortcuts.
        //
        // Each well-known provider env var (OPENAI_API_KEY, ANTHROPIC_API_KEY, …)
        // is written to TWO places:
        //   1. `llm.api_key` (via `set_default`) — so if no explicit key was
        //      configured, the shortcut supplies one for `build_client_config`.
        //   2. `llm.provider_keys.<provider_id>` — so per-provider key lookup
        //      (used by the registry / `configured` status) can find it without
        //      knowing which provider is the active default.
        //
        // `set_default` means UAR_LLM__API_KEY and LLM_API_KEY still win — they
        // are applied earlier at higher priority tiers.
        let provider_key_map: &[(&str, &str)] = &[
            ("OPENAI_API_KEY", "openai"),
            ("ANTHROPIC_API_KEY", "anthropic"),
            ("GROQ_API_KEY", "groq"),
            ("MISTRAL_API_KEY", "mistral"),
            ("COHERE_API_KEY", "cohere"),
            ("GEMINI_API_KEY", "google"),
            ("TOGETHER_API_KEY", "together"),
            ("PERPLEXITY_API_KEY", "perplexity"),
        ];
        for (env_var, provider_id) in provider_key_map {
            if let Ok(val) = env::var(env_var) {
                // Fallback api_key for the default client (lower priority than
                // UAR_LLM__API_KEY / LLM_API_KEY which use set_override).
                builder = builder.set_default("llm.api_key", val.clone())?;
                // Per-provider key — read by registry configured-status check.
                builder = builder.set_default(&format!("llm.provider_keys.{provider_id}"), val)?;
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

        if let Some(api_key_env) = deserialized.llm.api_key_env.as_deref() {
            let api_key_env = api_key_env.trim();
            if !api_key_env.is_empty()
                && let Ok(api_key) = env::var(api_key_env)
            {
                deserialized.llm.api_key = Some(api_key);
            }
        }

        deserialized.security.validate()?;
        deserialized.a2a.validate()?;
        deserialized
            .project_instructions
            .validate()
            .map_err(|error| config::ConfigError::Message(error.to_string()))?;

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

    /// The canonical JSON Schema for `AppConfig`, generated at call time from
    /// the live struct definitions via `schemars`. Exposed at
    /// `GET /.well-known/uar-config`; codegen'd into SDK types via
    /// `pnpm generate-config-types`. Secret fields (e.g. `jwt_secret`) are
    /// represented as opaque `string`s in the schema — the schema describes
    /// shape, not values.
    #[must_use]
    pub fn json_schema() -> serde_json::Value {
        let schema = schemars::schema_for!(AppConfig);
        serde_json::to_value(schema).unwrap_or(serde_json::Value::Null)
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
    // Key resolution order (highest priority first):
    //   1. Explicit `llm.api_key` from config / UAR_LLM__API_KEY / LLM_API_KEY
    //   2. `api_key_env` indirection (e.g. `api_key_env: "MY_SECRET"`)
    //   3. `LLM_API_KEY` env (legacy)
    //   4. Per-provider shortcut stored in `provider_keys` (OPENAI_API_KEY etc.)
    let (provider_id, _) = crate::llm::registry::split_model_string_pub(&llm.model);
    let api_key = llm
        .api_key
        .clone()
        .or_else(|| {
            llm.api_key_env
                .as_deref()
                .and_then(|key| std::env::var(key).ok())
        })
        .or_else(|| std::env::var("LLM_API_KEY").ok())
        .or_else(|| llm.provider_keys.get(&provider_id).cloned())
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
#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct LlmConfig {
    /// Default model in `provider/model` format (e.g., `"openai/gpt-4o"`).
    #[serde(default = "LlmConfig::default_model")]
    pub model: String,
    /// Authoritative provider identity attached by runtime provider resolution.
    ///
    /// This is runtime metadata rather than a user-configured field. It keeps
    /// provider identity available when an explicit base URL requires a bare
    /// model name on the wire.
    #[serde(skip)]
    #[schemars(skip)]
    pub resolved_provider_id: Option<String>,
    /// API key for the default provider.
    #[serde(default)]
    pub api_key: Option<String>,
    /// Environment variable name that contains the API key for the default provider.
    ///
    /// This lets deployments keep provider-specific secrets in `.env` without
    /// copying the secret value into `config.yaml`.
    #[serde(default)]
    pub api_key_env: Option<String>,
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
    /// Extended thinking budget (tokens). When set, enables thinking for
    /// Anthropic models with the specified token budget.
    #[serde(default)]
    pub thinking_budget: Option<u32>,
    /// Embedding backend configuration for RAG and memory.
    #[serde(default)]
    pub embedding: EmbeddingBackendConfig,
    /// Per-provider API keys loaded from env shortcuts (OPENAI_API_KEY, etc.).
    ///
    /// Keyed by provider id (e.g. `"openai"`, `"anthropic"`). Populated at
    /// config-load time from well-known env vars. Used by the provider registry
    /// to compute `configured` status (key present → truly configured) and to
    /// pass the right key when a non-default provider is used.
    ///
    /// NOT read from `config.yaml` — always sourced from env at boot.
    #[serde(default)]
    pub provider_keys: std::collections::HashMap<String, String>,
}

impl std::fmt::Debug for LlmConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Show which providers have keys (provider ids are not secret) but mask
        // every value; sorted for deterministic output.
        let provider_keys: std::collections::BTreeMap<&str, &str> = self
            .provider_keys
            .keys()
            .map(|provider| (provider.as_str(), REDACTED))
            .collect();
        f.debug_struct("LlmConfig")
            .field("model", &self.model)
            .field("resolved_provider_id", &self.resolved_provider_id)
            .field("api_key", &redact_opt(&self.api_key))
            .field("api_key_env", &self.api_key_env)
            .field("base_url", &self.base_url)
            .field("protocol", &self.protocol)
            .field("parallel_tool_calls", &self.parallel_tool_calls)
            .field("timeout_secs", &self.timeout_secs)
            .field("max_retries", &self.max_retries)
            .field("cache", &self.cache)
            .field("budget", &self.budget)
            .field("rate_limit", &self.rate_limit)
            .field("cost_tracking", &self.cost_tracking)
            .field("tracing", &self.tracing)
            .field("cooldown_secs", &self.cooldown_secs)
            .field("health_check_secs", &self.health_check_secs)
            .field("thinking_budget", &self.thinking_budget)
            .field("embedding", &self.embedding)
            .field("provider_keys", &provider_keys)
            .finish()
    }
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
            resolved_provider_id: None,
            api_key: None,
            api_key_env: None,
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
            thinking_budget: None,
            embedding: EmbeddingBackendConfig::default(),
            provider_keys: std::collections::HashMap::new(),
        }
    }
}

/// Embedding backend configuration for RAG and memory.
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct EmbeddingBackendConfig {
    /// Backend identifier: `fastembed`, `candle`, `openai`, `voyage`, `cohere`.
    #[serde(default = "EmbeddingBackendConfig::default_backend")]
    pub backend: String,
    /// Model identifier for the selected backend.
    #[serde(default = "EmbeddingBackendConfig::default_model")]
    pub model: String,
    /// API key for hosted backends.
    #[serde(default)]
    pub api_key: Option<String>,
    /// Environment variable name that contains the API key.
    #[serde(default)]
    pub api_key_env: Option<String>,
    /// Base URL override for hosted backends.
    #[serde(default)]
    pub base_url: Option<String>,
    /// Output vector dimension.
    #[serde(default = "EmbeddingBackendConfig::default_vector_dimension")]
    pub vector_dimension: usize,
    /// Maximum number of texts sent per request.
    #[serde(default = "EmbeddingBackendConfig::default_batch_size")]
    pub batch_size: usize,
    /// Local model directory for FastEmbed/Candle.
    #[serde(default = "EmbeddingBackendConfig::default_models_dir")]
    pub models_dir: String,
}

impl EmbeddingBackendConfig {
    fn default_backend() -> String {
        "fastembed".to_string()
    }

    fn default_model() -> String {
        "bge-small-en-v1.5".to_string()
    }

    fn default_vector_dimension() -> usize {
        384
    }

    fn default_batch_size() -> usize {
        32
    }

    fn default_models_dir() -> String {
        "src/uar/runtime/matching/models".to_string()
    }
}

impl Default for EmbeddingBackendConfig {
    fn default() -> Self {
        Self {
            backend: Self::default_backend(),
            model: Self::default_model(),
            api_key: None,
            api_key_env: None,
            base_url: None,
            vector_dimension: Self::default_vector_dimension(),
            batch_size: Self::default_batch_size(),
            models_dir: Self::default_models_dir(),
        }
    }
}

/// Cache configuration for the liter-llm Tower middleware.
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
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
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
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
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
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

    #[test]
    fn harness_defaults_to_typed_and_retains_legacy_rollback() {
        assert_eq!(HarnessConfig::default().mode, HarnessMode::Typed);
        let omitted: HarnessConfig = serde_json::from_str("{}").unwrap();
        assert_eq!(omitted.mode, HarnessMode::Typed);
        let rollback: HarnessConfig = serde_json::from_str(r#"{"mode":"legacy"}"#).unwrap();
        assert_eq!(rollback.mode, HarnessMode::Legacy);
    }

    #[test]
    fn json_schema_describes_the_top_level_shape() {
        let schema = AppConfig::json_schema();
        let properties = schema
            .get("properties")
            .expect("schema must have a properties object");
        for field in ["server", "security", "llm", "context_strategy"] {
            assert!(
                properties.get(field).is_some(),
                "expected top-level field `{field}` in the generated schema"
            );
        }
    }

    #[test]
    fn json_schema_hides_the_jwt_secret_value_behind_an_opaque_string() {
        let schema = AppConfig::json_schema();
        let security = schema["properties"]["security"].clone();
        // `$ref`-based schemas point at `definitions`/`$defs`; resolve the
        // reference the same way a schema consumer would.
        let security_schema = if let Some(reference) = security.get("$ref") {
            let pointer = reference
                .as_str()
                .expect("$ref must be a string")
                .trim_start_matches('#');
            schema
                .pointer(pointer)
                .cloned()
                .expect("schema must resolve the $ref for SecurityConfig")
        } else {
            security
        };
        let jwt_secret_schema = &security_schema["properties"]["jwt_secret"];
        assert_eq!(
            jwt_secret_schema["type"], "string",
            "jwt_secret must be described as an opaque string, not the internal SecretString shape"
        );
    }

    fn base_cli() -> (Cli, PathBuf) {
        let cfg_path = write_test_config_file();
        (
            Cli {
                env_file: None,
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
                failover_enabled: None,
                native_file_tools: None,
                native_web_fetch: None,
                native_terminal_exec: None,
                skill_evolution_enabled: None,
                skill_evolution_model: None,
                acp_enabled: None,
                acp_path: None,
                command: None,
                strict_config: false,
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
  settings_mutation_auth_required: false
persistence:
  provider: "surreal"
  database_url: "surrealkv://./data/test-config"
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
    fn document_processing_defaults_to_kreuzberg_without_ocr() {
        let (cli, cfg_path) = base_cli();
        let cfg = AppConfig::load_with_cli(cli).expect("config should load");
        assert_eq!(cfg.file_processing.provider, "kreuzberg");
        assert!(!cfg.kreuzberg.unwrap_or_default().ocr_enabled);
        assert_eq!(
            KnowledgeBaseConfig::default_file_processor(),
            "kreuzberg".to_string()
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

    #[test]
    fn cli_port_overrides_server_port() {
        // Regression: `--port` / `PORT` was a dead passthrough — parsed into Cli
        // but never applied to the config builder.
        let (cli, cfg_path) = base_cli();
        let default_cfg = AppConfig::load_with_cli(cli).expect("config should load");
        assert_eq!(default_cfg.server.port, 1906, "default port");

        let (mut cli, _) = base_cli();
        cli.config = Some(cfg_path.to_string_lossy().to_string());
        cli.port = Some(8123);
        let cfg = AppConfig::load_with_cli(cli).expect("config should load");
        assert_eq!(cfg.server.port, 8123, "--port must override server.port");
        let _ = fs::remove_file(cfg_path);
    }

    #[test]
    fn cli_jwt_required_overrides_security_jwt_required() {
        // Regression: `--jwt-required` / `JWT_REQUIRED` was a dead passthrough —
        // a silently-ignored security flag.
        let (cli, cfg_path) = base_cli();
        let default_cfg = AppConfig::load_with_cli(cli).expect("config should load");
        assert!(
            default_cfg.security.jwt_required,
            "jwt_required defaults true"
        );

        let (mut cli, _) = base_cli();
        cli.config = Some(cfg_path.to_string_lossy().to_string());
        cli.jwt_required = Some(false);
        let cfg = AppConfig::load_with_cli(cli).expect("config should load");
        assert!(
            !cfg.security.jwt_required,
            "--jwt-required=false must override security.jwt_required"
        );
        let _ = fs::remove_file(cfg_path);
    }

    #[test]
    fn required_jwt_rejects_the_built_in_fallback_secret() {
        let mut path = std::env::temp_dir();
        path.push(format!(
            "uar-jwt-fallback-test-{}.yaml",
            uuid::Uuid::new_v4()
        ));
        fs::write(
            &path,
            format!("security:\n  jwt_required: true\n  jwt_secret: \"{FALLBACK_JWT_SECRET}\"\n"),
        )
        .expect("test config should be writable");
        let cli = Cli {
            config: Some(path.to_string_lossy().to_string()),
            ..base_cli().0
        };

        let error = AppConfig::load_with_cli(cli)
            .expect_err("required JWT auth must reject the published fallback secret");
        assert!(
            error.to_string().contains("built-in fallback"),
            "configuration error must identify the unsafe fallback: {error}"
        );
        let _ = fs::remove_file(path);
    }

    #[test]
    fn anonymous_mode_explicitly_allows_the_built_in_fallback_secret() {
        let mut path = std::env::temp_dir();
        path.push(format!(
            "uar-jwt-anonymous-test-{}.yaml",
            uuid::Uuid::new_v4()
        ));
        fs::write(
            &path,
            format!("security:\n  jwt_required: false\n  jwt_secret: \"{FALLBACK_JWT_SECRET}\"\n  settings_mutation_auth_required: false\n"),
        )
        .expect("test config should be writable");
        let cli = Cli {
            config: Some(path.to_string_lossy().to_string()),
            ..base_cli().0
        };

        let config = AppConfig::load_with_cli(cli)
            .expect("an explicitly anonymous deployment may retain the unused fallback secret");
        assert!(!config.security.jwt_required);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn protected_settings_require_a_configured_admin_key() {
        let config = SecurityConfig {
            jwt_required: false,
            jwt_secret: FALLBACK_JWT_SECRET.to_owned().into(),
            jwks_url: None,
            jwt_issuer: None,
            jwt_audience: None,
            jwt_validate_nbf: true,
            settings_mutation_auth_required: true,
            settings_admin_key: None,
        };

        let error = config
            .validate()
            .expect_err("protected settings require an actual shared secret");
        assert!(error.to_string().contains("settings_admin_key"));
    }

    #[test]
    fn protected_settings_accept_a_configured_admin_key() {
        let config = SecurityConfig {
            jwt_required: false,
            jwt_secret: FALLBACK_JWT_SECRET.to_owned().into(),
            jwks_url: None,
            jwt_issuer: None,
            jwt_audience: None,
            jwt_validate_nbf: true,
            settings_mutation_auth_required: true,
            settings_admin_key: Some("configured-admin-key".to_owned().into()),
        };

        config
            .validate()
            .expect("a configured settings admin key satisfies the boundary");
    }

    #[test]
    fn post_load_secret_resolution_is_revalidated() {
        let (cli, cfg_path) = base_cli();
        let mut config = AppConfig::load_with_cli(cli).expect("deliberate secret should load");
        config.security.jwt_secret = FALLBACK_JWT_SECRET.to_owned().into();

        let error = config
            .security
            .validate()
            .expect_err("a post-load resolver must not replace the secret with the fallback");
        assert!(error.to_string().contains("built-in fallback"));
        let _ = fs::remove_file(cfg_path);
    }
}

// =============================================================================
// MODEL FAILOVER CONFIGURATION
// =============================================================================

/// Selection strategy for picking the next provider when the primary fails.
#[derive(Debug, Deserialize, Clone, Default, Serialize, PartialEq, Eq, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FailoverStrategy {
    /// Try each fallback in declared priority order (default).
    #[default]
    Priority,
    /// Distribute load round-robin across providers not in cooldown.
    RoundRobin,
    /// Always pick the cheapest available provider that meets capability requirements.
    CostOptimized,
}

/// A single entry in the ordered fallback model list.
#[derive(Debug, Deserialize, Clone, Serialize, schemars::JsonSchema)]
pub struct FallbackModel {
    /// Model in `provider/model` format, e.g. `"openrouter/anthropic/claude-3.5-sonnet"`.
    pub model: String,
    /// Explicit API key for this provider (overrides env-based key resolution).
    #[serde(default)]
    pub api_key: Option<String>,
    /// Optional base URL override for this provider.
    #[serde(default)]
    pub base_url: Option<String>,
}

/// Runtime model failover configuration.
///
/// When the primary model returns a non-retryable error (hard 4xx, provider
/// outage), the orchestrator attempts each `fallback_models` entry in order.
/// Failed providers enter a cooldown window and are skipped until recovered.
#[derive(Debug, Deserialize, Clone, Serialize, schemars::JsonSchema)]
pub struct FailoverConfig {
    /// Enable runtime model failover (default: false — opt-in).
    #[serde(default)]
    pub enabled: bool,
    /// Provider selection strategy when multiple fallbacks are available.
    #[serde(default)]
    pub strategy: FailoverStrategy,
    /// Consecutive errors before a provider enters cooldown (default: 3).
    #[serde(default = "FailoverConfig::default_error_threshold")]
    pub error_threshold: u32,
    /// Cooldown duration in seconds after a provider failure (default: 60).
    #[serde(default = "FailoverConfig::default_cooldown_secs")]
    pub cooldown_secs: u64,
    /// Ordered fallback model list tried when the primary fails.
    #[serde(default)]
    pub fallback_models: Vec<FallbackModel>,
}

impl FailoverConfig {
    fn default_error_threshold() -> u32 {
        3
    }
    fn default_cooldown_secs() -> u64 {
        60
    }
}

impl Default for FailoverConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            strategy: FailoverStrategy::default(),
            error_threshold: Self::default_error_threshold(),
            cooldown_secs: Self::default_cooldown_secs(),
            fallback_models: Vec::new(),
        }
    }
}

// =============================================================================
// NATIVE TOOLS CONFIGURATION
// =============================================================================

/// Configuration for native (in-process) tool implementations.
///
/// Native tools bypass MCP serialization for maximum performance. Each category
/// can be independently enabled/disabled and security-scoped.
#[derive(Debug, Deserialize, Clone, Serialize, schemars::JsonSchema)]
pub struct NativeToolsConfig {
    // --- File system tools ---
    /// Enable file_read / file_write / file_patch (default: false — opt-in).
    #[serde(default)]
    pub file_tools_enabled: bool,
    /// Directory prefix allowlist for file tools. Empty = deny all.
    /// Use `["*"]` to allow all paths (not recommended in production).
    #[serde(default)]
    pub file_allowed_paths: Vec<String>,
    /// Maximum file size in KB that file_read may return (default: 1024).
    #[serde(default = "NativeToolsConfig::default_file_max_size_kb")]
    pub file_max_size_kb: u64,
    /// Maximum file size in KB that file_write may create/overwrite (default: 512).
    #[serde(default = "NativeToolsConfig::default_file_write_max_kb")]
    pub file_write_max_kb: u64,

    // --- Web fetch tool ---
    /// Enable the web_fetch native tool (default: false).
    #[serde(default)]
    pub web_fetch_enabled: bool,
    /// Request timeout for web_fetch in seconds (default: 30).
    #[serde(default = "NativeToolsConfig::default_web_fetch_timeout_secs")]
    pub web_fetch_timeout_secs: u64,
    /// Maximum response body size in KB (default: 512).
    #[serde(default = "NativeToolsConfig::default_web_fetch_max_size_kb")]
    pub web_fetch_max_size_kb: u64,
    /// Domain allowlist for web_fetch. Empty = allow all domains.
    #[serde(default)]
    pub web_fetch_allowed_domains: Vec<String>,

    // --- Terminal exec tool ---
    /// Enable terminal_exec (default: false; routes through sandbox when enabled).
    #[serde(default)]
    pub terminal_exec_enabled: bool,
    /// Shell binary for terminal_exec (default: "sh").
    #[serde(default = "NativeToolsConfig::default_terminal_shell")]
    pub terminal_shell: String,
    /// Per-command timeout in seconds for terminal_exec (default: 30).
    #[serde(default = "NativeToolsConfig::default_terminal_timeout_secs")]
    pub terminal_timeout_secs: u64,
    /// Route through the configured sandbox runner (default: true).
    #[serde(default = "NativeToolsConfig::default_terminal_use_sandbox")]
    pub terminal_use_sandbox: bool,

    // --- Session search tool ---
    /// Enable session_search (default: true when persistence is configured).
    #[serde(default = "NativeToolsConfig::default_session_search_enabled")]
    pub session_search_enabled: bool,
    /// Maximum results returned by session_search (default: 10).
    #[serde(default = "NativeToolsConfig::default_session_search_max_results")]
    pub session_search_max_results: usize,
}

impl NativeToolsConfig {
    fn default_file_max_size_kb() -> u64 {
        1024
    }
    fn default_file_write_max_kb() -> u64 {
        512
    }
    fn default_web_fetch_timeout_secs() -> u64 {
        30
    }
    fn default_web_fetch_max_size_kb() -> u64 {
        512
    }
    fn default_terminal_shell() -> String {
        "sh".to_string()
    }
    fn default_terminal_timeout_secs() -> u64 {
        30
    }
    fn default_terminal_use_sandbox() -> bool {
        true
    }
    fn default_session_search_enabled() -> bool {
        true
    }
    fn default_session_search_max_results() -> usize {
        10
    }
}

impl Default for NativeToolsConfig {
    fn default() -> Self {
        Self {
            file_tools_enabled: false,
            file_allowed_paths: Vec::new(),
            file_max_size_kb: Self::default_file_max_size_kb(),
            file_write_max_kb: Self::default_file_write_max_kb(),
            web_fetch_enabled: false,
            web_fetch_timeout_secs: Self::default_web_fetch_timeout_secs(),
            web_fetch_max_size_kb: Self::default_web_fetch_max_size_kb(),
            web_fetch_allowed_domains: Vec::new(),
            terminal_exec_enabled: false,
            terminal_shell: Self::default_terminal_shell(),
            terminal_timeout_secs: Self::default_terminal_timeout_secs(),
            terminal_use_sandbox: Self::default_terminal_use_sandbox(),
            session_search_enabled: Self::default_session_search_enabled(),
            session_search_max_results: Self::default_session_search_max_results(),
        }
    }
}

// =============================================================================
// SKILL EVOLUTION CONFIGURATION
// =============================================================================

/// Configuration for the post-run skill evolution system.
///
/// After each successful run, the orchestrator can prompt the agent to reflect
/// on whether a reusable skill should be extracted from the interaction. Agent
/// responses containing a `skill_create` tool call are automatically registered
/// in the `SkillService`.
#[derive(Debug, Deserialize, Clone, Serialize, schemars::JsonSchema)]
pub struct SkillEvolutionConfig {
    /// Enable the skill evolution loop (default: false — opt-in).
    #[serde(default)]
    pub enabled: bool,
    /// Model to use for the post-run reflection call (`provider/model` format).
    /// Falls back to the run's primary model when `None`.
    #[serde(default)]
    pub trigger_model: Option<String>,
    /// Minimum number of tool calls in a run before evolution is considered (default: 2).
    /// Prevents trivial single-turn interactions from spawning skills.
    #[serde(default = "SkillEvolutionConfig::default_min_tool_calls")]
    pub min_tool_calls: usize,
    /// Maximum new skills that can be created from a single run (default: 1).
    #[serde(default = "SkillEvolutionConfig::default_max_skills_per_run")]
    pub max_skills_per_run: usize,
    /// Allow the evolution system to update (patch) existing skills (default: false).
    #[serde(default)]
    pub allow_update: bool,
    /// Allow the evolution system to delete under-used skills (default: false).
    #[serde(default)]
    pub allow_deletion: bool,
    /// Minimum times a skill must have executed before deletion is permitted (default: 5).
    #[serde(default = "SkillEvolutionConfig::default_min_executions_before_delete")]
    pub min_executions_before_delete: u32,
    /// Custom reflection system-prompt override. Uses built-in prompt when `None`.
    #[serde(default)]
    pub reflection_prompt: Option<String>,
}

impl SkillEvolutionConfig {
    fn default_min_tool_calls() -> usize {
        2
    }
    fn default_max_skills_per_run() -> usize {
        1
    }
    fn default_min_executions_before_delete() -> u32 {
        5
    }
}

impl Default for SkillEvolutionConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            trigger_model: None,
            min_tool_calls: Self::default_min_tool_calls(),
            max_skills_per_run: Self::default_max_skills_per_run(),
            allow_update: false,
            allow_deletion: false,
            min_executions_before_delete: Self::default_min_executions_before_delete(),
            reflection_prompt: None,
        }
    }
}

// =============================================================================
// ACP SERVER CONFIGURATION
// =============================================================================

/// Configuration for the ACP (Agent Communication Protocol) server endpoint.
///
/// ACP is the BeeAI / IBM Research protocol that provides a JSON-RPC 2.0
/// Sycophancy detection configuration.
///
/// Controls whether LLM responses are checked for sycophantic patterns and
/// at what threshold corrections are applied.
#[derive(Debug, Deserialize, Clone, Serialize, schemars::JsonSchema)]
pub struct SycophancyConfig {
    /// Enable sycophancy detection on LLM responses (default: true).
    #[serde(default = "SycophancyConfig::default_enabled")]
    pub enabled: bool,
    /// Strictness level: "permissive", "standard", "strict", "adversarial".
    #[serde(default = "SycophancyConfig::default_strictness")]
    pub strictness: String,
    /// Score threshold for auto-correction (default: 0.5). Configurable per-context.
    #[serde(default = "SycophancyConfig::default_auto_correct_threshold")]
    pub auto_correct_threshold: f32,
    /// Score threshold for reflect-phase outputs (lower — more sensitive).
    #[serde(default = "SycophancyConfig::default_reflect_threshold")]
    pub reflect_threshold: f32,
    /// Log detections without blocking (default: false).
    #[serde(default)]
    pub log_only: bool,
    /// Run one corrective LLM pass on a flagged response and emit the rewrite as
    /// a follow-up (default: false — detection-only). Suppressed when `log_only`.
    #[serde(default)]
    pub auto_correct: bool,
}

impl SycophancyConfig {
    fn default_enabled() -> bool {
        true
    }
    fn default_strictness() -> String {
        "standard".to_string()
    }
    fn default_auto_correct_threshold() -> f32 {
        0.5
    }
    fn default_reflect_threshold() -> f32 {
        0.3
    }
}

impl Default for SycophancyConfig {
    fn default() -> Self {
        Self {
            enabled: Self::default_enabled(),
            strictness: Self::default_strictness(),
            auto_correct_threshold: Self::default_auto_correct_threshold(),
            reflect_threshold: Self::default_reflect_threshold(),
            log_only: false,
            auto_correct: false,
        }
    }
}

/// Input guardrail configuration: heuristic prompt-injection and PII/secret
/// screening of chat input before the LLM call.
#[derive(Debug, Deserialize, Clone, Serialize, schemars::JsonSchema)]
pub struct GuardrailsConfig {
    /// Screen chat input for injection/PII patterns (default: true). Screening
    /// is non-blocking by default, so enabling it only adds flag events/metrics.
    #[serde(default = "GuardrailsConfig::default_input_screening_enabled")]
    pub input_screening_enabled: bool,
    /// Reject (block before the LLM call) inputs flagged as prompt-injection
    /// (default: false — detect-only).
    #[serde(default)]
    pub block_on_injection: bool,
    /// Reject (block before the LLM call) inputs flagged as PII/secret
    /// (default: false — detect-only). Off by default because blocking a user's
    /// own data is user-hostile; operators opt in.
    #[serde(default)]
    pub block_on_pii: bool,
}

impl GuardrailsConfig {
    fn default_input_screening_enabled() -> bool {
        true
    }
}

impl Default for GuardrailsConfig {
    fn default() -> Self {
        Self {
            input_screening_enabled: Self::default_input_screening_enabled(),
            block_on_injection: false,
            block_on_pii: false,
        }
    }
}

/// interface for agent introspection, session management, and streaming run
/// execution — used by IDEs, debuggers, and the BeeAI platform.
#[derive(Debug, Deserialize, Clone, Serialize, schemars::JsonSchema)]
pub struct AcpConfig {
    /// Enable the ACP endpoint (default: false — opt-in).
    #[serde(default)]
    pub enabled: bool,
    /// HTTP path prefix for the ACP endpoint (default: "/acp").
    #[serde(default = "AcpConfig::default_path")]
    pub path: String,
    /// Require JWT authentication on ACP endpoints (default: true).
    #[serde(default = "AcpConfig::default_auth_required")]
    pub auth_required: bool,
    /// Session idle TTL in seconds before ACP sessions are evicted (default: 3600).
    #[serde(default = "AcpConfig::default_session_ttl_secs")]
    pub session_ttl_secs: u64,
}

impl AcpConfig {
    fn default_path() -> String {
        "/acp".to_string()
    }
    fn default_auth_required() -> bool {
        true
    }
    fn default_session_ttl_secs() -> u64 {
        3600
    }
}

impl Default for AcpConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            path: Self::default_path(),
            auth_required: Self::default_auth_required(),
            session_ttl_secs: Self::default_session_ttl_secs(),
        }
    }
}
