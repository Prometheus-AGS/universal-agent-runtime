//! LLM orchestrator with tool loop execution.
//!
//! The orchestrator manages the complete lifecycle of an LLM interaction:
//! 1. Send user message to the LLM
//! 2. Stream the response, detecting tool calls
//! 3. Execute tool calls via MCP
//! 4. Feed tool results back to the LLM
//! 5. Repeat until the model produces a final response
//!
//! # Example
//!
//! ```rust,ignore
//! use universal_agent_runtime::config::LlmConfig;
//! use universal_agent_runtime::llm::Orchestrator;
//! use universal_agent_runtime::mcp::registry::McpRegistry;
//! use universal_agent_runtime::uar::runtime::native_skill::NativeSkillRegistry;
//!
//! let llm_config = LlmConfig::default();
//! let mcp = McpRegistry::load_from_file("mcp.json").await?;
//! let native_skills = Arc::new(NativeSkillRegistry::new());
//! let orchestrator = Orchestrator::new(llm_config, mcp, native_skills)?;
//!
//! let stream = orchestrator.chat("Hello, what time is it?").await?;
//! ```

use std::collections::BTreeMap;
use std::pin::Pin;
use std::sync::Arc;

use futures::{Stream, StreamExt};
use uuid::Uuid;

use crate::config::{FailoverConfig, FallbackModel, LlmConfig};
use crate::mcp::registry::McpRegistry;
use crate::normalized::{NormalizedEvent, RuntimeStepKind};
use crate::uar::runtime::native_skill::NativeSkillRegistry;

use super::{
    LlmDriver, LlmRequest, Message, MessageContent, MessageRole, ToolCall, ToolCallFunction,
    anthropic_cache::CacheStrategy,
};

/// Build the protocol driver for a resolved LLM configuration.
///
/// Anthropic models use the native Messages API when its runtime gate is on;
/// all other configurations retain liter-llm's compatible provider routing.
pub fn build_driver(llm_config: &LlmConfig) -> anyhow::Result<Arc<dyn LlmDriver>> {
    let (model_provider_id, model_id) = super::registry::split_model_string_pub(&llm_config.model);
    let provider_id = llm_config
        .resolved_provider_id
        .as_deref()
        .unwrap_or(&model_provider_id);
    if provider_id == "anthropic" && super::anthropic_native_driver_enabled() {
        let api_key = llm_config
            .api_key
            .clone()
            .or_else(|| llm_config.provider_keys.get("anthropic").cloned())
            .unwrap_or_default();
        return Ok(Arc::new(super::anthropic_driver::AnthropicDriver::new(
            api_key,
            model_id,
            llm_config.base_url.clone(),
            None,
            None,
            None,
        )));
    }

    let client_config = crate::config::build_client_config(llm_config);
    Ok(Arc::new(super::LiterLlmDriver::new(
        client_config,
        llm_config.model.clone(),
        llm_config.parallel_tool_calls,
    )?))
}

/// Result of a tool approval gate check.
#[derive(Debug, Clone)]
pub enum ToolApprovalResult {
    /// Tool is approved for execution.
    Approved,
    /// Governance was intentionally bypassed for a verified local-only process.
    GovernanceBypassed,
    /// Tool execution was rejected by the user or timed out.
    Rejected { reason: String },
}

/// A callback invoked before each tool call execution to allow approval/rejection.
/// Returns `Approved` to proceed or `Rejected` to skip the tool call.
pub type ToolApprovalGate = Arc<
    dyn Fn(
            String, // tool_call_id
            String, // tool_name
            String, // arguments_json
            usize,  // call_index
        ) -> Pin<Box<dyn Future<Output = ToolApprovalResult> + Send>>
        + Send
        + Sync,
>;

/// Maximum number of tool loop iterations to prevent infinite loops.
const MAX_TOOL_ITERATIONS: usize = 10;

/// Accumulated state for a streaming tool call.
#[derive(Debug, Default, Clone)]
struct ToolCallAccumulator {
    id: Option<String>,
    name: Option<String>,
    arguments: String,
}

/// Returns `true` if the tool name looks like a code-execution tool.
///
/// Used by the `Auto` execution mode to decide whether to sandbox a tool call.
fn is_code_execution_tool(name: &str) -> bool {
    let n = name.to_lowercase();
    // Strip namespace prefix (e.g. "mcp::execute_code" → "execute_code")
    let local = n.rsplit("::").next().unwrap_or(&n);
    matches!(
        local,
        "execute_code"
            | "run_code"
            | "eval_code"
            | "code_interpreter"
            | "python_repl"
            | "bash"
            | "shell"
            | "run_bash"
            | "run_python"
            | "run_script"
            | "computer"
    ) || local.starts_with("execute_")
        || local.ends_with("_repl")
}

/// Conservative allowlist for parallel execution. Unknown and mutating tools
/// remain sequential because their side effects may depend on call order.
fn is_parallel_safe_tool(name: &str) -> bool {
    let local = name
        .rsplit_once("__")
        .map_or(name, |(_, local)| local)
        .to_ascii_lowercase();
    [
        "get_", "list_", "read_", "search_", "query_", "fetch_", "lookup_", "status_", "health_",
    ]
    .iter()
    .any(|prefix| local.starts_with(prefix))
}

fn is_retryable_provider_error(
    error: &anyhow::Error,
    policy: &crate::uar::settings::resilience_policy::ResiliencePolicy,
) -> bool {
    let message = error.to_string().to_ascii_lowercase();
    let status_retryable = policy.retryable_http_statuses.iter().any(|status| {
        let status = status.to_string();
        message.contains(&format!("status {status}"))
            || message.contains(&format!("status: {status}"))
            || message.contains(&format!("http {status}"))
    });
    status_retryable
        || (policy.retryable_transport_errors
            && [
                "connection",
                "transport",
                "timed out",
                "timeout",
                "temporarily unavailable",
                "broken pipe",
                "reset by peer",
            ]
            .iter()
            .any(|needle| message.contains(needle)))
}

/// Extract (`code`, `language`) from a tool-call's argument JSON, if present.
fn extract_code_from_arguments(
    tool_name: &str,
    args: &serde_json::Value,
) -> Option<(String, crate::sandbox::Language)> {
    let code = args
        .get("code")
        .or_else(|| args.get("command"))
        .or_else(|| args.get("script"))
        .and_then(serde_json::Value::as_str)
        .map(String::from)?;

    let lang_str = args
        .get("language")
        .and_then(serde_json::Value::as_str)
        .map(str::to_lowercase);

    let language = match lang_str.as_deref() {
        Some("python" | "python3" | "py") => crate::sandbox::Language::Python,
        Some("node" | "nodejs" | "javascript" | "js") => crate::sandbox::Language::Node,
        _ => {
            // Infer from tool name
            let n = tool_name.to_lowercase();
            if n.contains("python") {
                crate::sandbox::Language::Python
            } else if n.contains("node") || n.contains("js") {
                crate::sandbox::Language::Node
            } else {
                crate::sandbox::Language::Bash
            }
        }
    };

    Some((code, language))
}

/// Lowercase metric label for a sandbox language.
fn sandbox_language_label(lang: &crate::sandbox::Language) -> &'static str {
    use crate::sandbox::Language;
    match lang {
        Language::Bash => "bash",
        Language::Python => "python",
        Language::Node => "node",
        Language::Rust => "rust",
    }
}

/// Metric label for a sandbox runner type.
fn sandbox_runner_type_label(rt: crate::sandbox::runner::RunnerType) -> &'static str {
    use crate::sandbox::runner::RunnerType;
    match rt {
        RunnerType::MicroVm => "microsandbox",
        RunnerType::Wasmtime => "wasmtime",
        RunnerType::Remote => "remote",
    }
}

/// Classify a sandbox error into a bounded metric label.
fn sandbox_error_type(e: &crate::sandbox::SandboxError) -> &'static str {
    use crate::sandbox::SandboxError;
    match e {
        SandboxError::CreationFailed(_) => "creation_failed",
        SandboxError::ExecutionFailed(_) => "execution_failed",
        SandboxError::FileError(_) => "file_error",
        SandboxError::NotFound(_) => "not_found",
        SandboxError::CapacityExceeded(_) => "capacity_exceeded",
        SandboxError::Timeout(_) => "timeout",
        SandboxError::RunnerUnavailable(_) => "runner_unavailable",
    }
}

/// Sandbox execution duration in seconds.
#[expect(clippy::cast_precision_loss, reason = "duration ms fits within f64")]
fn sandbox_duration_secs(ms: u64) -> f64 {
    ms as f64 / 1000.0
}

/// LLM orchestrator with tool loop execution.
///
/// The orchestrator wraps an [`LlmDriver`] and adds:
/// - Tool call detection and accumulation
/// - Tool execution via MCP
/// - Automatic tool result feeding
/// - Request ID tracking
#[derive(Clone)]
pub struct Orchestrator {
    llm_config: LlmConfig,
    mcp: Arc<McpRegistry>,
    driver: Arc<dyn LlmDriver>,
    /// Optional fallback driver activated when the primary driver errors.
    fallback_driver: Option<Arc<dyn LlmDriver>>,
    /// Controls when/how to switch to the fallback driver.
    failover_config: FailoverConfig,
    /// Provider id (e.g. `"openai"`) the fallback driver targets, derived from
    /// `failover_config.fallback_models.first()` at `with_failover` time — used
    /// to record health outcomes for the fallback attempt (CH-03).
    fallback_provider_id: Option<String>,
    /// Shared provider-health monitor (CH-03). When set, every driver
    /// success/failure updates `ProviderRegistry::health()` so subsequent
    /// routing decisions see the outcome immediately.
    health_monitor: Option<Arc<super::health::ProviderHealthMonitor>>,
    native_skills: Arc<NativeSkillRegistry>,
    /// Optional gate that is consulted before each tool call execution.
    /// If `None`, all tool calls are approved automatically.
    tool_approval_gate: Option<ToolApprovalGate>,
    /// Optional sandbox runner for isolated code execution.
    sandbox_runner: Option<Arc<dyn crate::sandbox::SandboxRunner>>,
    /// Controls which tool calls are routed to the sandbox runner.
    tool_execution_mode: crate::uar::domain::artifact::ToolExecutionMode,
    resilience_policy: crate::uar::settings::resilience_policy::ResiliencePolicy,
    /// Bound applied once to every tool result when it is recorded into the
    /// model-visible history (MCP, native, and terminal results alike).
    tool_output_policy: crate::uar::runtime::context::truncate::TruncationPolicy,
    /// Per-run cache strategy copied into every policy-bearing tool-loop request.
    cache_strategy: Option<CacheStrategy>,
}

#[allow(clippy::missing_fields_in_debug)]
impl std::fmt::Debug for Orchestrator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Orchestrator")
            .field("model", &self.llm_config.model)
            .field("mcp", &"McpRegistry")
            .finish()
    }
}

impl Orchestrator {
    /// Create a new orchestrator with the given LLM config, MCP registry, and native skill registry.
    ///
    /// Uses `LiterLlmDriver` backed by liter-llm's `DefaultClient` for all
    /// 142+ providers with unified tool-call normalization.
    /// # Errors
    ///
    /// Returns an error if the underlying LLM client cannot be constructed.
    #[allow(dead_code)]
    pub fn new(
        llm_config: LlmConfig,
        mcp: Arc<McpRegistry>,
        native_skills: Arc<NativeSkillRegistry>,
    ) -> anyhow::Result<Self> {
        let driver = build_driver(&llm_config)?;

        Ok(Self::from_driver(llm_config, mcp, native_skills, driver))
    }

    /// Create a new orchestrator with a host-supplied LLM driver.
    ///
    /// This is the embedding seam for environments that own a local model
    /// runtime outside UAR, such as KnowMe mobile. UAR still owns the agent
    /// loop, tool governance, skills, and normalized events; the host-supplied
    /// driver only provides model streaming.
    #[must_use]
    pub fn from_driver(
        llm_config: LlmConfig,
        mcp: Arc<McpRegistry>,
        native_skills: Arc<NativeSkillRegistry>,
        driver: Arc<dyn LlmDriver>,
    ) -> Self {
        Self {
            llm_config,
            mcp,
            driver,
            fallback_driver: None,
            failover_config: FailoverConfig::default(),
            fallback_provider_id: None,
            health_monitor: None,
            native_skills,
            tool_approval_gate: None,
            sandbox_runner: None,
            tool_execution_mode: crate::uar::domain::artifact::ToolExecutionMode::default(),
            resilience_policy: crate::uar::settings::resilience_policy::ResiliencePolicy::default(),
            tool_output_policy: crate::uar::runtime::context::truncate::TruncationPolicy::default(),
            cache_strategy: None,
        }
    }

    /// Attach a fallback driver and failover configuration.
    ///
    /// When `failover_config.enabled` is `true` and the primary driver fails,
    /// the orchestrator will re-try the same request against `fallback_driver`.
    /// The fallback's provider id (for health recording) is derived from the
    /// first entry in `failover_config.fallback_models` (`FailoverStrategy::Priority`).
    #[must_use]
    pub fn with_failover(
        mut self,
        fallback_driver: Arc<dyn LlmDriver>,
        failover_config: FailoverConfig,
    ) -> Self {
        self.fallback_provider_id = failover_config
            .fallback_models
            .first()
            .map(|f| super::registry::split_model_string_pub(&f.model).0);
        self.fallback_driver = Some(fallback_driver);
        self.failover_config = failover_config;
        self
    }

    /// Attach the shared provider-health monitor (CH-03). Driver
    /// successes/failures are recorded against it so `ModelRouter` and
    /// `ProviderRegistry::resolve_to_llm_config` see the outcome on the very
    /// next call.
    #[must_use]
    pub fn with_health_monitor(
        mut self,
        health_monitor: Arc<super::health::ProviderHealthMonitor>,
    ) -> Self {
        self.health_monitor = Some(health_monitor);
        self
    }

    /// Build a driver for one `FailoverConfig::fallback_models` entry, reusing
    /// `base_llm_config` for every field except `model`/`api_key`/`base_url`
    /// (CH-03). Used by callers wiring `with_failover` from app config.
    ///
    /// # Errors
    ///
    /// Returns an error if the underlying LLM client cannot be constructed.
    pub fn build_fallback_driver(
        base_llm_config: &LlmConfig,
        fallback: &FallbackModel,
    ) -> anyhow::Result<Arc<dyn LlmDriver>> {
        let fallback_llm_config = Self::fallback_llm_config(base_llm_config, fallback);
        build_driver(&fallback_llm_config)
    }

    fn fallback_llm_config(base: &LlmConfig, fallback: &FallbackModel) -> LlmConfig {
        let mut config = base.clone();
        config.model.clone_from(&fallback.model);
        config.resolved_provider_id = None;
        if fallback.api_key.is_some() {
            config.api_key.clone_from(&fallback.api_key);
        }
        if fallback.base_url.is_some() {
            config.base_url.clone_from(&fallback.base_url);
        }
        config
    }

    /// Get the LLM configuration.
    #[must_use]
    #[allow(dead_code)]
    pub fn llm_config(&self) -> &LlmConfig {
        &self.llm_config
    }

    /// Get the MCP registry.
    #[must_use]
    #[allow(dead_code)]
    pub fn mcp(&self) -> &McpRegistry {
        &self.mcp
    }

    /// Set a tool approval gate that will be consulted before each tool call.
    #[must_use]
    pub fn with_tool_approval_gate(mut self, gate: ToolApprovalGate) -> Self {
        self.tool_approval_gate = Some(gate);
        self
    }

    /// Attach a sandbox runner and execution mode for tool isolation.
    ///
    /// When `mode` is [`ToolExecutionMode::Sandboxed`] or [`ToolExecutionMode::Auto`],
    /// code-execution tool calls will be routed through the provided sandbox runner
    /// instead of being executed directly via MCP.
    #[must_use]
    pub fn with_sandbox(
        mut self,
        runner: Arc<dyn crate::sandbox::SandboxRunner>,
        mode: crate::uar::domain::artifact::ToolExecutionMode,
    ) -> Self {
        self.sandbox_runner = Some(runner);
        self.tool_execution_mode = mode;
        self
    }

    /// Apply bounded provider retry and stream-start policy to this run.
    #[must_use]
    pub fn with_resilience_policy(
        mut self,
        policy: crate::uar::settings::resilience_policy::ResiliencePolicy,
    ) -> Self {
        self.resilience_policy = policy;
        self
    }

    /// Set the bound applied to every recorded tool result.
    #[must_use]
    pub fn with_tool_output_policy(
        mut self,
        policy: crate::uar::runtime::context::truncate::TruncationPolicy,
    ) -> Self {
        self.tool_output_policy = policy;
        self
    }

    /// Apply the effective prompt-caching strategy to every policy-bearing
    /// request created by this orchestrator, including retries and failover.
    #[must_use]
    pub fn with_cache_strategy(mut self, strategy: Option<CacheStrategy>) -> Self {
        self.cache_strategy = strategy;
        self
    }

    /// Start a chat interaction with the given user message.
    ///
    /// Returns a stream of [`NormalizedEvent`]s that includes:
    /// - `StreamStart` with a unique request ID
    /// - `MessageDelta` for assistant text
    /// - `ToolCallDelta` and `ToolCallComplete` for tool calls
    /// - `ToolResult` after tool execution
    /// - `Done` when complete
    ///
    /// The orchestrator will automatically execute tool calls and feed
    /// results back to the LLM until a final response is produced.
    #[allow(dead_code)]
    pub async fn chat(
        &self,
        user_message: &str,
    ) -> anyhow::Result<impl Stream<Item = NormalizedEvent> + Send> {
        self.chat_with_history(vec![Message {
            role: MessageRole::User,
            content: MessageContent::text(user_message),
            tool_call_id: None,
            tool_calls: None,
        }])
        .await
    }

    /// Start a chat interaction with existing message history.
    #[allow(clippy::too_many_lines)]
    pub async fn chat_with_history(
        &self,
        messages: Vec<Message>,
    ) -> anyhow::Result<impl Stream<Item = NormalizedEvent> + Send + 'static> {
        let request_id = Uuid::new_v4().to_string();
        let mut tools = self.mcp.openai_tools_json();
        // Native skills execute in the same governed tool loop as MCP tools,
        // so they must also be declared to the model. Previously they were
        // executable only if a model somehow guessed their names, leaving
        // registered tools such as `a2ui_render` impossible to call.
        for native_tool in self.native_skills.openai_tools_json().await {
            let native_name = native_tool["function"]["name"].as_str();
            tools.retain(|tool| tool["function"]["name"].as_str() != native_name);
            tools.push(native_tool);
        }
        tools.sort_by(|left, right| {
            left["function"]["name"]
                .as_str()
                .cmp(&right["function"]["name"].as_str())
        });

        tracing::info!(
            request_id = %request_id,
            message_count = messages.len(),
            tool_count = tools.len(),
            "Starting orchestrator chat"
        );

        // Log initial message history
        for (idx, msg) in messages.iter().enumerate() {
            let content_preview = msg.content.as_text().map_or(0, str::len);
            tracing::debug!(
                request_id = %request_id,
                message_index = idx,
                role = ?msg.role,
                content_length = content_preview,
                has_tool_calls = msg.tool_calls.is_some(),
                "Initial message"
            );
        }

        let orchestrator = self.clone();
        let messages = messages.clone();

        let stream = async_stream::stream! {
            // Emit stream start
            yield NormalizedEvent::StreamStart {
                request_id: request_id.clone(),
            };

            // Convert messages to JSON for the driver
            let mut message_json: Vec<serde_json::Value> = messages
                .iter()
                .map(|m| serde_json::to_value(m).unwrap_or_default())
                .collect();

            tracing::debug!(
                request_id = %request_id,
                "Converted messages to JSON for driver"
            );

            let mut iteration = 0;

            loop {
                if iteration >= MAX_TOOL_ITERATIONS {
                    tracing::error!(
                        request_id = %request_id,
                        iteration = iteration,
                        max_iterations = MAX_TOOL_ITERATIONS,
                        "Maximum tool loop iterations exceeded"
                    );
                    yield NormalizedEvent::Error {
                        message: "Maximum tool loop iterations exceeded".to_string(),
                        code: Some("MAX_ITERATIONS".to_string()),
                    };
                    break;
                }
                iteration += 1;
                let step = u32::try_from(iteration).unwrap_or(u32::MAX);

                // Per-step run progress: this iteration is beginning.
                yield NormalizedEvent::RuntimeStep {
                    step,
                    kind: RuntimeStepKind::Started,
                };

                tracing::info!(
                    request_id = %request_id,
                    iteration = iteration,
                    message_count = message_json.len(),
                    "Starting tool loop iteration"
                );

                let normalize_report = match crate::uar::runtime::context::normalize::normalize_provider_messages(&mut message_json) {
                    Ok(report) => report,
                    Err(error) => {
                        tracing::error!(
                            request_id = %request_id,
                            iteration,
                            error = %error,
                            "Provider history normalization failed"
                        );
                        yield NormalizedEvent::Error {
                            message: error.to_string(),
                            code: Some("HISTORY_NORMALIZATION_FAILED".to_string()),
                        };
                        break;
                    }
                };
                let _ = normalize_report;

                // CH-04: per-model dialect params (extended-thinking budgets,
                // reasoning-persistence toggles) keyed off the model id.
                // `thinking_budget` doubles as the "this deployment wants
                // reasoning" signal — it was otherwise a dead config knob.
                let dialect_params = super::prompt_dialect::PromptDialectEngine::new()
                    .request_params(
                        &orchestrator.llm_config.model,
                        super::prompt_dialect::DialectRequest {
                            wants_reasoning: orchestrator.llm_config.thinking_budget.is_some(),
                            multi_turn: message_json.len() > 1,
                            hard: orchestrator.llm_config.thinking_budget.unwrap_or(0) > 4096,
                        },
                    );
                let req = LlmRequest {
                    messages: message_json.clone(),
                    tools: tools.clone(),
                    cache_strategy: orchestrator.cache_strategy.clone(),
                    thinking_config: None,
                    anthropic_system: None,
                    extra_params: dialect_params
                        .as_object()
                        .filter(|o| !o.is_empty())
                        .map(|_| dialect_params.clone()),
                };

                // Log the full request being sent to the LLM
                tracing::debug!(
                    request_id = %request_id,
                    iteration = iteration,
                    messages = ?req.messages,
                    tool_count = req.tools.len(),
                    "Sending request to LLM driver"
                );

                // Stream from the driver (with automatic failover if configured).
                // CH-03: every outcome is recorded against the shared health
                // monitor (when attached) so ModelRouter/ProviderRegistry see
                // it on the very next routing decision, independent of
                // whether failover itself is enabled.
                let primary_provider_id =
                    super::registry::split_model_string_pub(&orchestrator.llm_config.model).0;
                let driver_stream = {
                    let policy = &orchestrator.resilience_policy;
                    let max_attempts = if policy.retries_enabled {
                        policy.retry_max_attempts.max(1)
                    } else {
                        1
                    };
                    let retry_started = std::time::Instant::now();
                    let mut attempt = 1_u32;
                    let primary = loop {
                        match tokio::time::timeout(
                            std::time::Duration::from_millis(policy.stream_start_timeout_ms),
                            orchestrator.driver.stream(req.clone()),
                        )
                        .await
                        {
                            Ok(Ok(stream)) => break Ok(stream),
                            Ok(Err(error))
                                if attempt < max_attempts
                                    && is_retryable_provider_error(&error, policy) =>
                            {
                                let elapsed_ms = u64::try_from(retry_started.elapsed().as_millis())
                                    .unwrap_or(u64::MAX);
                                let exponent = i32::try_from(attempt.saturating_sub(1))
                                    .unwrap_or(i32::MAX);
                                let delay_ms = ((policy.retry_base_delay_ms as f64)
                                    * f64::from(policy.retry_backoff_multiplier).powi(exponent))
                                    .min(policy.retry_max_delay_ms as f64)
                                    as u64;
                                if elapsed_ms.saturating_add(delay_ms) > policy.retry_budget_ms {
                                    break Err(error);
                                }
                                tracing::warn!(
                                    request_id = %request_id,
                                    iteration,
                                    attempt,
                                    max_attempts,
                                    delay_ms,
                                    error = %error,
                                    "LLM stream creation failed; retrying before semantic events"
                                );
                                tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
                                attempt = attempt.saturating_add(1);
                            }
                            Ok(Err(error)) => break Err(error),
                            Err(_) if attempt < max_attempts => {
                                let elapsed_ms = u64::try_from(retry_started.elapsed().as_millis())
                                    .unwrap_or(u64::MAX);
                                let delay_ms = policy
                                    .retry_base_delay_ms
                                    .min(policy.retry_max_delay_ms);
                                if elapsed_ms.saturating_add(delay_ms) > policy.retry_budget_ms {
                                    break Err(anyhow::anyhow!(
                                        "LLM stream start timed out after {} ms",
                                        policy.stream_start_timeout_ms
                                    ));
                                }
                                tracing::warn!(
                                    request_id = %request_id,
                                    iteration,
                                    attempt,
                                    timeout_ms = policy.stream_start_timeout_ms,
                                    "LLM stream start timed out; retrying"
                                );
                                tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
                                attempt = attempt.saturating_add(1);
                            }
                            Err(_) => break Err(anyhow::anyhow!(
                                "LLM stream start timed out after {} ms",
                                policy.stream_start_timeout_ms
                            )),
                        }
                    };
                    match primary {
                        Ok(s) => {
                            tracing::debug!(
                                request_id = %request_id,
                                iteration = iteration,
                                "Driver stream created successfully"
                            );
                            if let Some(health) = &orchestrator.health_monitor {
                                health.record_success(&primary_provider_id).await;
                            }
                            s
                        }
                        Err(e) if orchestrator.failover_config.enabled => {
                            if let Some(health) = &orchestrator.health_monitor {
                                health
                                    .record_failure(
                                        &primary_provider_id,
                                        orchestrator.failover_config.error_threshold,
                                        orchestrator.failover_config.cooldown_secs,
                                    )
                                    .await;
                            }
                            if let Some(ref fallback) = orchestrator.fallback_driver {
                                tracing::warn!(
                                    request_id = %request_id,
                                    iteration = iteration,
                                    primary_error = %e,
                                    "Primary LLM driver failed; attempting failover",
                                );
                                match fallback.stream(req).await {
                                    Ok(s) => {
                                        if let (Some(health), Some(fallback_id)) = (
                                            &orchestrator.health_monitor,
                                            &orchestrator.fallback_provider_id,
                                        ) {
                                            health.record_success(fallback_id).await;
                                        }
                                        s
                                    }
                                    Err(fe) => {
                                        if let (Some(health), Some(fallback_id)) = (
                                            &orchestrator.health_monitor,
                                            &orchestrator.fallback_provider_id,
                                        ) {
                                            health
                                                .record_failure(
                                                    fallback_id,
                                                    orchestrator.failover_config.error_threshold,
                                                    orchestrator.failover_config.cooldown_secs,
                                                )
                                                .await;
                                        }
                                        tracing::error!(
                                            request_id = %request_id,
                                            primary_error = %e,
                                            fallback_error = %fe,
                                            "Fallback driver also failed",
                                        );
                                        yield NormalizedEvent::Error {
                                            message: format!(
                                                "primary: {e}; fallback: {fe}"
                                            ),
                                            code: None,
                                        };
                                        break;
                                    }
                                }
                            } else {
                                tracing::error!(
                                    request_id = %request_id,
                                    iteration = iteration,
                                    error = %e,
                                    "Primary driver failed; no fallback configured",
                                );
                                yield NormalizedEvent::Error {
                                    message: e.to_string(),
                                    code: None,
                                };
                                break;
                            }
                        }
                        Err(e) => {
                            if let Some(health) = &orchestrator.health_monitor {
                                health
                                    .record_failure(
                                        &primary_provider_id,
                                        orchestrator.failover_config.error_threshold,
                                        orchestrator.failover_config.cooldown_secs,
                                    )
                                    .await;
                            }
                            tracing::error!(
                                request_id = %request_id,
                                iteration = iteration,
                                error = %e,
                                "Failed to create driver stream"
                            );
                            yield NormalizedEvent::Error {
                                message: e.to_string(),
                                code: None,
                            };
                            break;
                        }
                    }
                };

                let mut tool_accumulators: BTreeMap<usize, ToolCallAccumulator> = BTreeMap::new();
                let mut assistant_text = String::new();
                let mut has_tool_calls = false;
                let mut finish_reason: Option<String> = None;

                futures::pin_mut!(driver_stream);

                while let Some(result) = driver_stream.next().await {
                    match result {
                        Ok(event) => {
                            match &event {
                                NormalizedEvent::MessageDelta { text } => {
                                    assistant_text.push_str(text);
                                }
                                NormalizedEvent::ToolCallDelta {
                                    call_index,
                                    id,
                                    name,
                                    arguments_delta,
                                } => {
                                    has_tool_calls = true;
                                    let acc = tool_accumulators.entry(*call_index).or_default();
                                    if acc.id.is_none() {
                                        acc.id = id.clone();
                                    }
                                    if acc.name.is_none() {
                                        acc.name = name.clone();
                                    }
                                    if let Some(delta) = arguments_delta {
                                        acc.arguments.push_str(delta);
                                    }
                                }
                                NormalizedEvent::ToolCallComplete { .. } => {
                                    has_tool_calls = true;
                                    finish_reason = Some("tool_calls".to_string());
                                }
                                NormalizedEvent::Done => {
                                    // Don't yield Done yet if we have tool calls to process
                                    if !has_tool_calls {
                                        yield event;
                                        return;
                                    }
                                    continue;
                                }
                                NormalizedEvent::Error { .. } => {
                                    yield event;
                                    return;
                                }
                                _ => {}
                            }
                            yield event;
                        }
                        Err(e) => {
                            yield NormalizedEvent::Error {
                                message: e.to_string(),
                                code: None,
                            };
                            return;
                        }
                    }
                }

                // If no tool calls, we're done
                if !has_tool_calls || finish_reason.as_deref() != Some("tool_calls") {
                    tracing::info!(
                        request_id = %request_id,
                        iteration = iteration,
                        has_tool_calls = has_tool_calls,
                        finish_reason = ?finish_reason,
                        "No tool calls to process, completing stream"
                    );
                    yield NormalizedEvent::RuntimeStep {
                        step,
                        kind: RuntimeStepKind::Finished,
                    };
                    yield NormalizedEvent::Done;
                    break;
                }

                tracing::info!(
                    request_id = %request_id,
                    iteration = iteration,
                    accumulator_count = tool_accumulators.len(),
                    "Building tool calls from accumulators"
                );

                // Build tool calls from accumulators
                let tool_calls: Vec<ToolCall> = tool_accumulators
                    .values()
                    .filter_map(|acc| {
                        let id = acc.id.clone()?;
                        let name = acc.name.clone()?;
                        Some(ToolCall {
                            id,
                            call_type: "function".to_string(),
                            function: ToolCallFunction {
                                name,
                                arguments: acc.arguments.clone(),
                            },
                        })
                    })
                    .collect();

                if tool_calls.is_empty() {
                    tracing::warn!(
                        request_id = %request_id,
                        iteration = iteration,
                        "No valid tool calls built from accumulators"
                    );
                    yield NormalizedEvent::RuntimeStep {
                        step,
                        kind: RuntimeStepKind::Finished,
                    };
                    yield NormalizedEvent::Done;
                    break;
                }

                tracing::info!(
                    request_id = %request_id,
                    iteration = iteration,
                    tool_call_count = tool_calls.len(),
                    "Built tool calls, adding to message history"
                );

                // Log each tool call
                for (idx, tc) in tool_calls.iter().enumerate() {
                    tracing::info!(
                        request_id = %request_id,
                        iteration = iteration,
                        tool_index = idx,
                        tool_id = %tc.id,
                        tool_name = %tc.function.name,
                        args_length = tc.function.arguments.len(),
                        "Tool call to execute"
                    );
                    tracing::debug!(
                        request_id = %request_id,
                        tool_id = %tc.id,
                        arguments = %tc.function.arguments,
                        "Tool call arguments"
                    );
                }

                // Add assistant message with tool calls to history
                message_json.push(serde_json::json!({
                    "role": "assistant",
                    "content": assistant_text,
                    "tool_calls": tool_calls.iter().map(|tc| {
                        serde_json::json!({
                            "id": tc.id,
                            "type": tc.call_type,
                            "function": {
                                "name": tc.function.name,
                                "arguments": tc.function.arguments
                            }
                        })
                    }).collect::<Vec<_>>()
                }));

                tracing::debug!(
                    request_id = %request_id,
                    iteration = iteration,
                    "Added assistant message with tool calls to history"
                );

                // Read-only calls can execute concurrently. `buffered` bounds
                // concurrency while preserving input order, which keeps tool
                // result message ordering deterministic for the next LLM turn.
                if orchestrator.llm_config.parallel_tool_calls == Some(true)
                    && tool_calls.len() > 1
                    && orchestrator.tool_approval_gate.is_none()
                    && orchestrator.sandbox_runner.is_none()
                    && tool_calls
                        .iter()
                        .all(|call| is_parallel_safe_tool(&call.function.name))
                {
                    let executions = futures::stream::iter(tool_calls.iter().cloned().map(|call| {
                        let orchestrator = orchestrator.clone();
                        async move {
                            let arguments = serde_json::from_str(&call.function.arguments)
                                .unwrap_or_else(|_| serde_json::Value::Object(serde_json::Map::new()));
                            let outcome = if let Some(native) =
                                orchestrator.native_skills.get(&call.function.name).await
                            {
                                native.execute(arguments).await.map(|value| {
                                    native.format_result(
                                        &value,
                                        orchestrator.tool_output_policy,
                                        &orchestrator.llm_config.model,
                                    )
                                })
                            } else {
                                orchestrator
                                    .mcp
                                    .call_namespaced_tool(&call.function.name, arguments)
                                    .await
                                    .map(|value| serde_json::to_string(&value).unwrap_or_default())
                                    .map(|content| {
                                        crate::uar::runtime::context::truncate::formatted_truncate_for_model(
                                            &content,
                                            orchestrator.tool_output_policy,
                                            &orchestrator.llm_config.model,
                                        )
                                    })
                            };
                            let (content, success) = match outcome {
                                Ok(content) => (content, true),
                                Err(error) => (
                                    crate::uar::runtime::context::truncate::formatted_truncate_for_model(
                                        &format!("Error: {error}"),
                                        orchestrator.tool_output_policy,
                                        &orchestrator.llm_config.model,
                                    ),
                                    false,
                                ),
                            };
                            (call, content, success)
                        }
                    }))
                    .buffered(8)
                    .collect::<Vec<_>>()
                    .await;

                    for (call, content, success) in executions {
                        yield NormalizedEvent::ToolResult {
                            id: call.id.clone(),
                            name: call.function.name.clone(),
                            content: content.clone(),
                            success,
                        };
                        message_json.push(serde_json::json!({
                            "role": "tool",
                            "tool_call_id": call.id,
                            "content": content
                        }));
                    }
                    yield NormalizedEvent::RuntimeStep {
                        step,
                        kind: RuntimeStepKind::Finished,
                    };
                    continue;
                }

                // Execute mutating, approval-gated, sandboxed, or unknown calls
                // sequentially to preserve policy and side-effect ordering.
                for (idx, tool_call) in tool_calls.iter().enumerate() {
                    let tool_name = &tool_call.function.name;
                    let arguments: serde_json::Value = serde_json::from_str(&tool_call.function.arguments)
                        .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));

                    tracing::info!(
                        request_id = %request_id,
                        iteration = iteration,
                        tool_index = idx,
                        tool_id = %tool_call.id,
                        tool_name = %tool_name,
                        "Executing tool call"
                    );

                    // Check tool approval gate if configured
                    if let Some(ref gate) = orchestrator.tool_approval_gate {
                        let result = gate(
                            tool_call.id.clone(),
                            tool_name.clone(),
                            tool_call.function.arguments.clone(),
                            idx,
                        ).await;
                        if let ToolApprovalResult::Rejected { reason } = result {
                            tracing::warn!(
                                request_id = %request_id,
                                tool_id = %tool_call.id,
                                tool_name = %tool_name,
                                reason = %reason,
                                "Tool call rejected by approval gate"
                            );
                            let rejection_content = format!("Tool call rejected: {reason}");
                            yield NormalizedEvent::ToolResult {
                                id: tool_call.id.clone(),
                                name: tool_name.clone(),
                                content: rejection_content.clone(),
                                success: false,
                            };
                            message_json.push(serde_json::json!({
                                "role": "tool",
                                "tool_call_id": tool_call.id,
                                "content": rejection_content
                            }));
                            continue;
                        }
                    }

                    // Determine sandbox routing
                    let sandbox_attempt: Option<Arc<dyn crate::sandbox::SandboxRunner>> = {
                        use crate::uar::domain::artifact::ToolExecutionMode;
                        let should_sandbox = match &orchestrator.tool_execution_mode {
                            ToolExecutionMode::Direct => false,
                            ToolExecutionMode::Sandboxed | ToolExecutionMode::Auto => {
                                is_code_execution_tool(tool_name)
                            }
                        };
                        if should_sandbox {
                            orchestrator.sandbox_runner.clone()
                        } else {
                            None
                        }
                    };

                    let (content, success) = if let Some(runner) = sandbox_attempt {
                        // Route to sandbox if we can extract code from the arguments
                        if let Some((code, lang)) = extract_code_from_arguments(tool_name, &arguments) {
                            tracing::info!(
                                request_id = %request_id,
                                iteration = iteration,
                                tool_id = %tool_call.id,
                                tool_name = %tool_name,
                                language = ?lang,
                                "Executing tool in microsandbox"
                            );
                            let sandbox_cfg = crate::sandbox::SandboxConfig::default();
                            let lang_label = sandbox_language_label(&lang);
                            match runner.create(sandbox_cfg).await {
                                Ok(handle) => {
                                    crate::uar::telemetry::metrics::record_sandbox_created(
                                        sandbox_runner_type_label(runner.capabilities().runner_type),
                                        lang_label,
                                    );
                                    crate::uar::telemetry::metrics::sandbox_active_inc();
                                    let exec_req = crate::sandbox::ExecutionRequest {
                                        language: lang,
                                        code,
                                        stdin: None,
                                        env: std::collections::HashMap::new(),
                                        cwd: None,
                                        timeout_seconds: Some(30),
                                        mode: crate::sandbox::ExecutionMode::Ephemeral,
                                    };
                                    match runner.execute(&handle, exec_req).await {
                                        Ok(result) => {
                                            let _ = runner.destroy(handle).await;
                                            crate::uar::telemetry::metrics::record_sandbox_execution(
                                                lang_label,
                                                if result.exit_code == 0 {
                                                    "success"
                                                } else {
                                                    "error"
                                                },
                                                sandbox_duration_secs(result.execution_time_ms),
                                            );
                                            crate::uar::telemetry::metrics::sandbox_active_dec();
                                            let out = format!(
                                                "exit_code: {}\nstdout:\n{}\nstderr:\n{}",
                                                result.exit_code, result.stdout, result.stderr
                                            );
                                            tracing::info!(
                                                request_id = %request_id,
                                                tool_name = %tool_name,
                                                exit_code = result.exit_code,
                                                "Sandbox execution completed"
                                            );
                                            (out, result.exit_code == 0)
                                        }
                                        Err(e) => {
                                            let _ = runner.destroy(handle).await;
                                            crate::uar::telemetry::metrics::record_sandbox_error(
                                                sandbox_error_type(&e),
                                            );
                                            crate::uar::telemetry::metrics::sandbox_active_dec();
                                            tracing::error!(
                                                request_id = %request_id,
                                                tool_name = %tool_name,
                                                error = %e,
                                                "Sandbox execution failed"
                                            );
                                            (format!("Sandbox execution error: {e}"), false)
                                        }
                                    }
                                }
                                Err(e) => {
                                    crate::uar::telemetry::metrics::record_sandbox_error(
                                        sandbox_error_type(&e),
                                    );
                                    tracing::error!(
                                        request_id = %request_id,
                                        tool_name = %tool_name,
                                        error = %e,
                                        "Sandbox creation failed"
                                    );
                                    (format!("Sandbox creation error: {e}"), false)
                                }
                            }
                        } else {
                            // No code to extract — fall through to native/MCP
                            if let Some(native_skill) = orchestrator.native_skills.get(tool_name).await {
                                match native_skill.execute(arguments.clone()).await {
                                    Ok(r) => (serde_json::to_string(&r).unwrap_or_default(), true),
                                    Err(e) => (format!("Native skill error: {e}"), false),
                                }
                            } else {
                                match orchestrator.mcp.call_namespaced_tool(tool_name, arguments.clone()).await {
                                    Ok(r) => (serde_json::to_string(&r).unwrap_or_default(), true),
                                    Err(e) => (format!("Error: {e}"), false),
                                }
                            }
                        }
                    } else {
                        // Priority: check native skills first, then fall back to MCP
                        if let Some(native_skill) = orchestrator.native_skills.get(tool_name).await {
                            tracing::info!(
                                request_id = %request_id,
                                iteration = iteration,
                                tool_id = %tool_call.id,
                                tool_name = %tool_name,
                                "Executing via native skill (bypassing MCP)"
                            );
                            match native_skill.execute(arguments.clone()).await {
                                Ok(result) => {
                                    let content = native_skill.format_result(
                                        &result,
                                        orchestrator.tool_output_policy,
                                        &orchestrator.llm_config.model,
                                    );
                                    tracing::info!(
                                        request_id = %request_id,
                                        tool_id = %tool_call.id,
                                        tool_name = %tool_name,
                                        result_length = content.len(),
                                        "Native skill execution succeeded"
                                    );
                                    (content, true)
                                }
                                Err(e) => {
                                    let error_msg = crate::uar::runtime::context::truncate::formatted_truncate_for_model(
                                        &format!("Native skill error: {e}"),
                                        orchestrator.tool_output_policy,
                                        &orchestrator.llm_config.model,
                                    );
                                    tracing::error!(
                                        request_id = %request_id,
                                        tool_id = %tool_call.id,
                                        tool_name = %tool_name,
                                        error = %e,
                                        "Native skill execution failed"
                                    );
                                    (error_msg, false)
                                }
                            }
                        } else {
                            match orchestrator.mcp.call_namespaced_tool(tool_name, arguments.clone()).await {
                                Ok(result) => {
                                    let content = serde_json::to_string(&result).unwrap_or_default();
                                    let content = crate::uar::runtime::context::truncate::formatted_truncate_for_model(
                                        &content,
                                        orchestrator.tool_output_policy,
                                        &orchestrator.llm_config.model,
                                    );
                                    tracing::info!(
                                        request_id = %request_id,
                                        iteration = iteration,
                                        tool_id = %tool_call.id,
                                        tool_name = %tool_name,
                                        result_length = content.len(),
                                        "Tool call succeeded"
                                    );
                                    tracing::debug!(
                                        request_id = %request_id,
                                        tool_id = %tool_call.id,
                                        result = %content,
                                        "Tool call result"
                                    );
                                    (content, true)
                                }
                                Err(e) => {
                                    let error_msg = crate::uar::runtime::context::truncate::formatted_truncate_for_model(
                                        &format!("Error: {e}"),
                                        orchestrator.tool_output_policy,
                                        &orchestrator.llm_config.model,
                                    );
                                    tracing::error!(
                                        request_id = %request_id,
                                        iteration = iteration,
                                        tool_id = %tool_call.id,
                                        tool_name = %tool_name,
                                        error = %e,
                                        "Tool call failed"
                                    );
                                    (error_msg, false)
                                }
                            }
                        }
                    };

                    // Emit tool result event
                    yield NormalizedEvent::ToolResult {
                        id: tool_call.id.clone(),
                        name: tool_name.clone(),
                        content: content.clone(),
                        success,
                    };

                    // Add tool result to message history
                    message_json.push(serde_json::json!({
                        "role": "tool",
                        "tool_call_id": tool_call.id,
                        "content": content
                    }));

                    tracing::debug!(
                        request_id = %request_id,
                        iteration = iteration,
                        tool_id = %tool_call.id,
                        "Added tool result to message history"
                    );
                }

                tracing::info!(
                    request_id = %request_id,
                    iteration = iteration,
                    "All tool calls executed, continuing to next iteration"
                );

                // This iteration's tool work is done; the loop cycles for the
                // next response.
                yield NormalizedEvent::RuntimeStep {
                    step,
                    kind: RuntimeStepKind::Finished,
                };

                // Continue the loop to get the next response
            }
        };

        Ok(stream)
    }

    /// Non-streaming chat for simple requests (e.g., title generation).
    ///
    /// This collects all message deltas into a single string response.
    pub async fn chat_non_streaming(&self, messages: Vec<Message>) -> anyhow::Result<String> {
        let request_id = Uuid::new_v4().to_string();
        let tools = Vec::new(); // No tools for simple requests

        tracing::debug!(
            request_id = %request_id,
            message_count = messages.len(),
            "Starting non-streaming chat"
        );

        let mut message_json: Vec<serde_json::Value> = messages
            .iter()
            .map(|m| serde_json::to_value(m).unwrap_or_default())
            .collect();
        crate::uar::runtime::context::normalize::normalize_provider_messages(&mut message_json)?;

        let req = LlmRequest {
            messages: message_json,
            tools,
            cache_strategy: None,
            thinking_config: None,
            anthropic_system: None,
            extra_params: None,
        };

        // Stream from the driver and collect message deltas
        let mut stream = self.driver.stream(req).await?;
        let mut content = String::new();

        while let Some(event_result) = stream.next().await {
            match event_result {
                Ok(NormalizedEvent::MessageDelta { text }) => {
                    content.push_str(&text);
                }
                Err(e) => {
                    tracing::error!(request_id = %request_id, error = %e, "Error in stream");
                    return Err(e);
                }
                _ => {} // Ignore other events
            }
        }

        tracing::debug!(
            request_id = %request_id,
            content_length = content.len(),
            "Non-streaming chat completed"
        );

        Ok(content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::mock_driver::MockLlmDriver;
    use crate::uar::runtime::native_skill::NativeSkill;
    use std::sync::{
        Mutex,
        atomic::{AtomicUsize, Ordering},
    };

    #[derive(Debug, Default)]
    struct FailingDriver {
        requests: Mutex<Vec<LlmRequest>>,
    }

    #[async_trait::async_trait]
    impl LlmDriver for FailingDriver {
        async fn stream(
            &self,
            req: LlmRequest,
        ) -> anyhow::Result<Pin<Box<dyn Stream<Item = anyhow::Result<NormalizedEvent>> + Send>>>
        {
            self.requests.lock().expect("requests lock").push(req);
            Err(anyhow::anyhow!("stub primary failure"))
        }
    }

    struct RenderSkill;

    struct SearchSkill {
        calls: Arc<AtomicUsize>,
    }

    #[test]
    fn resolved_provider_identity_survives_bare_model_for_explicit_base_url() {
        let config = LlmConfig {
            model: "custom-claude-alias".to_string(),
            resolved_provider_id: Some("anthropic".to_string()),
            base_url: Some("https://anthropic-proxy.example/v1/messages".to_string()),
            ..LlmConfig::default()
        };
        let (inferred, _) = super::super::registry::split_model_string_pub(&config.model);
        assert_ne!(inferred, "anthropic");
        assert_eq!(config.resolved_provider_id.as_deref(), Some("anthropic"));
    }

    #[test]
    fn cross_provider_fallback_does_not_reuse_primary_provider_identity() {
        let primary = LlmConfig {
            model: "custom-claude-alias".to_string(),
            resolved_provider_id: Some("anthropic".to_string()),
            base_url: Some("https://anthropic-proxy.example".to_string()),
            ..LlmConfig::default()
        };
        let fallback = FallbackModel {
            model: "openai/gpt-5.6".to_string(),
            api_key: Some("fallback-key".to_string()),
            base_url: Some("https://openai-proxy.example/v1".to_string()),
        };

        let resolved = Orchestrator::fallback_llm_config(&primary, &fallback);
        assert_eq!(resolved.model, "openai/gpt-5.6");
        assert_eq!(resolved.resolved_provider_id, None);
        assert_eq!(
            super::super::registry::split_model_string_pub(&resolved.model).0,
            "openai"
        );
    }

    #[async_trait::async_trait]
    impl NativeSkill for RenderSkill {
        fn name(&self) -> &str {
            "a2ui_render"
        }
        fn description(&self) -> &str {
            "Render an A2UI surface"
        }
        fn parameters_schema(&self) -> serde_json::Value {
            serde_json::json!({"type":"object","properties":{"messages":{"type":"array"}}})
        }
        async fn execute(&self, _: serde_json::Value) -> anyhow::Result<serde_json::Value> {
            Ok(serde_json::json!({"ok": true}))
        }
    }

    #[async_trait::async_trait]
    impl NativeSkill for SearchSkill {
        fn name(&self) -> &str {
            "search_web"
        }

        fn description(&self) -> &str {
            "Searches the web fixture"
        }

        fn parameters_schema(&self) -> serde_json::Value {
            serde_json::json!({
                "type": "object",
                "properties": { "query": { "type": "string" } }
            })
        }

        async fn execute(&self, args: serde_json::Value) -> anyhow::Result<serde_json::Value> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            Ok(serde_json::json!({"result": args["query"]}))
        }
    }

    #[tokio::test]
    async fn from_driver_uses_host_supplied_driver() {
        let driver = Arc::new(MockLlmDriver::echo());
        let orchestrator = Orchestrator::from_driver(
            LlmConfig::default(),
            Arc::new(McpRegistry::empty()),
            Arc::new(NativeSkillRegistry::new()),
            driver.clone(),
        );

        let response = orchestrator
            .chat_non_streaming(vec![Message {
                role: MessageRole::User,
                content: MessageContent::text("hello"),
                tool_call_id: None,
                tool_calls: None,
            }])
            .await
            .unwrap();

        assert_eq!(response, "Hello from mock!");
        assert_eq!(driver.call_count(), 1);
    }

    #[tokio::test]
    async fn declares_registered_native_skills_to_the_model() {
        let driver = Arc::new(MockLlmDriver::echo());
        let native_skills = Arc::new(NativeSkillRegistry::new());
        native_skills.register(RenderSkill).await;
        let orchestrator = Orchestrator::from_driver(
            LlmConfig::default(),
            Arc::new(McpRegistry::empty()),
            native_skills,
            driver.clone(),
        );

        let stream = orchestrator.chat("render a card").await.unwrap();
        futures::pin_mut!(stream);
        while stream.next().await.is_some() {}

        let requests = driver.requests();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].tools.len(), 1);
        assert_eq!(requests[0].tools[0]["function"]["name"], "a2ui_render");
    }

    #[tokio::test]
    async fn applies_cache_strategy_to_policy_bearing_requests_only_when_configured() {
        let enabled_driver = Arc::new(MockLlmDriver::echo());
        let enabled = Orchestrator::from_driver(
            LlmConfig::default(),
            Arc::new(McpRegistry::empty()),
            Arc::new(NativeSkillRegistry::new()),
            enabled_driver.clone(),
        )
        .with_cache_strategy(Some(CacheStrategy::default()));
        let stream = enabled.chat("cache this").await.unwrap();
        futures::pin_mut!(stream);
        while stream.next().await.is_some() {}

        let disabled_driver = Arc::new(MockLlmDriver::echo());
        let disabled = Orchestrator::from_driver(
            LlmConfig::default(),
            Arc::new(McpRegistry::empty()),
            Arc::new(NativeSkillRegistry::new()),
            disabled_driver.clone(),
        );
        let stream = disabled.chat("do not cache this").await.unwrap();
        futures::pin_mut!(stream);
        while stream.next().await.is_some() {}

        assert!(enabled_driver.requests()[0].cache_strategy.is_some());
        assert!(disabled_driver.requests()[0].cache_strategy.is_none());
    }

    #[tokio::test]
    async fn cache_strategy_is_preserved_across_tool_loop_iterations() {
        let driver = Arc::new(MockLlmDriver::new(vec![
            vec![
                NormalizedEvent::ToolCallDelta {
                    call_index: 0,
                    id: Some("call-1".into()),
                    name: Some("a2ui_render".into()),
                    arguments_delta: Some("{}".into()),
                },
                NormalizedEvent::ToolCallComplete {
                    call_index: 0,
                    id: "call-1".into(),
                    name: "a2ui_render".into(),
                    arguments_json: "{}".into(),
                },
                NormalizedEvent::Done,
            ],
            vec![
                NormalizedEvent::MessageDelta {
                    text: "complete".into(),
                },
                NormalizedEvent::Done,
            ],
        ]));
        let native_skills = Arc::new(NativeSkillRegistry::new());
        native_skills.register(RenderSkill).await;
        let orchestrator = Orchestrator::from_driver(
            LlmConfig::default(),
            Arc::new(McpRegistry::empty()),
            native_skills,
            driver.clone(),
        )
        .with_cache_strategy(Some(CacheStrategy::default()));

        let stream = orchestrator.chat("render").await.expect("chat stream");
        futures::pin_mut!(stream);
        while stream.next().await.is_some() {}

        let requests = driver.requests();
        assert_eq!(requests.len(), 2);
        assert!(
            requests
                .iter()
                .all(|request| request.cache_strategy.is_some())
        );
    }

    #[tokio::test]
    async fn governance_bypassed_executes_tool_without_approval_or_denial_events() {
        let driver = Arc::new(MockLlmDriver::new(vec![
            vec![
                NormalizedEvent::ToolCallDelta {
                    call_index: 0,
                    id: Some("search-call".into()),
                    name: Some("search_web".into()),
                    arguments_delta: Some(r#"{"query":"loopback governance"}"#.into()),
                },
                NormalizedEvent::ToolCallComplete {
                    call_index: 0,
                    id: "search-call".into(),
                    name: "search_web".into(),
                    arguments_json: r#"{"query":"loopback governance"}"#.into(),
                },
                NormalizedEvent::Done,
            ],
            vec![
                NormalizedEvent::MessageDelta {
                    text: "search complete".into(),
                },
                NormalizedEvent::Done,
            ],
        ]));
        let calls = Arc::new(AtomicUsize::new(0));
        let native_skills = Arc::new(NativeSkillRegistry::new());
        native_skills
            .register(SearchSkill {
                calls: Arc::clone(&calls),
            })
            .await;
        let gate: ToolApprovalGate =
            Arc::new(|_, _, _, _| Box::pin(async { ToolApprovalResult::GovernanceBypassed }));
        let orchestrator = Orchestrator::from_driver(
            LlmConfig::default(),
            Arc::new(McpRegistry::empty()),
            native_skills,
            driver,
        )
        .with_tool_approval_gate(gate);

        let stream = orchestrator.chat("search").await.expect("chat stream");
        let events = stream.collect::<Vec<_>>().await;

        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert!(events.iter().any(|event| matches!(
            event,
            NormalizedEvent::ToolResult {
                name,
                success: true,
                ..
            } if name == "search_web"
        )));
        assert!(
            events
                .iter()
                .all(|event| !matches!(event, NormalizedEvent::ToolResult { success: false, .. }))
        );
    }

    #[tokio::test]
    async fn cache_strategy_is_preserved_on_failover_request() {
        let primary = Arc::new(FailingDriver::default());
        let fallback = Arc::new(MockLlmDriver::echo());
        let mut failover = FailoverConfig::default();
        failover.enabled = true;
        let orchestrator = Orchestrator::from_driver(
            LlmConfig::default(),
            Arc::new(McpRegistry::empty()),
            Arc::new(NativeSkillRegistry::new()),
            primary.clone(),
        )
        .with_failover(fallback.clone(), failover)
        .with_cache_strategy(Some(CacheStrategy::default()));

        let stream = orchestrator.chat("fail over").await.expect("chat stream");
        futures::pin_mut!(stream);
        while stream.next().await.is_some() {}

        assert!(
            primary.requests.lock().expect("primary requests")[0]
                .cache_strategy
                .is_some()
        );
        assert!(fallback.requests()[0].cache_strategy.is_some());
    }
}
