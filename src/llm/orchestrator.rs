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

use backon::Retryable;
use futures::{Stream, StreamExt};
use uuid::Uuid;

use crate::config::{FailoverConfig, FallbackModel, LlmConfig};
use crate::mcp::registry::McpRegistry;
use crate::normalized::{NormalizedEvent, RuntimeStepKind};
use crate::uar::runtime::native_skill::NativeSkillRegistry;
use crate::uar::tools::descriptor::{
    ApprovalClass, Exposure, ToolCollision, ToolDescriptor, ToolEffect,
};
use crate::uar::tools::validate;

use super::{
    LlmDriver, LlmRequest, Message, MessageContent, MessageRole, ToolCall, ToolCallFunction,
    anthropic_cache::CacheStrategy,
};

/// Build the protocol driver for a resolved LLM configuration.
///
/// Anthropic models use the native Messages API when its runtime gate is on;
/// all other configurations retain liter-llm's compatible provider routing.
pub fn build_driver(llm_config: &LlmConfig) -> anyhow::Result<Arc<dyn LlmDriver>> {
    // ADR-010 §1a (b)+(c). Every inference path -- primary, failover, server,
    // turn bindings -- constructs its driver here, so this is the one place
    // that can make "local-only" a guarantee rather than a configuration
    // preference. Fails closed: no driver is returned, so no request is sent.
    super::local_only::check_base_url(llm_config.base_url.as_deref())?;

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

type DriverEventStream = Pin<Box<dyn Stream<Item = anyhow::Result<NormalizedEvent>> + Send>>;

async fn open_driver_stream(
    driver: &dyn LlmDriver,
    request: LlmRequest,
    start_timeout: std::time::Duration,
    idle_timeout: std::time::Duration,
) -> anyhow::Result<DriverEventStream> {
    let mut stream = tokio::time::timeout(start_timeout, driver.stream(request))
        .await
        .map_err(|_| {
            super::ProviderError::timeout(format!(
                "LLM stream start timed out after {} ms",
                start_timeout.as_millis()
            ))
        })??;

    // Provider lifecycle/usage metadata is not committed assistant output.
    // Keep this prelude inside the retry boundary; otherwise Anthropic's
    // message_start makes a subsequent content-free stall non-retryable.
    // Coalesce the two metadata kinds so an untrusted prelude cannot grow
    // without bound. Metadata does not reset the first-output deadline.
    let mut prelude = Vec::new();
    let first = tokio::time::timeout(idle_timeout, async {
        loop {
            let Some(event) = stream.next().await else {
                return Ok::<_, anyhow::Error>(None);
            };
            let event = event?;
            match &event {
                NormalizedEvent::StreamStart { .. } | NormalizedEvent::Usage { .. } => {
                    prelude.retain(|previous| {
                        std::mem::discriminant(previous) != std::mem::discriminant(&event)
                    });
                    prelude.push(event);
                }
                NormalizedEvent::MessageDelta { text }
                | NormalizedEvent::ThinkingDelta { text }
                | NormalizedEvent::ReasoningDelta { text }
                    if text.is_empty() => {}
                _ => return Ok(Some(event)),
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .map_err(|_| {
        super::ProviderError::timeout(format!(
            "LLM stream idle timed out after {} ms before semantic output",
            idle_timeout.as_millis()
        ))
    })??;
    prelude.extend(first);
    Ok(Box::pin(
        futures::stream::iter(prelude.into_iter().map(Ok::<_, anyhow::Error>)).chain(stream),
    ))
}

fn prepend_attempt_manifest(
    stream: DriverEventStream,
    manifest: serde_json::Value,
) -> DriverEventStream {
    Box::pin(
        futures::stream::once(async move {
            Ok(NormalizedEvent::Custom {
                source: "uar.request".to_string(),
                event_name: "attempt_manifest".to_string(),
                payload: manifest,
            })
        })
        .chain(stream),
    )
}

/// Result of a tool approval gate check.
#[derive(Debug, Clone)]
pub enum ToolApprovalResult {
    /// Tool is approved, but may have required user confirmation. Execute serially.
    Approved,
    /// Host policy permits execution without user confirmation.
    Allowed,
    /// Governance was intentionally bypassed for a verified local-only process.
    GovernanceBypassed,
    /// Tool execution was rejected by the user or timed out.
    Rejected { reason: String },
}

/// A callback invoked before each tool call execution to allow approval/rejection.
/// Returns an admitted outcome to proceed or `Rejected` to skip the tool call.
pub type ToolApprovalGate = Arc<
    dyn Fn(
            String, // tool_call_id
            String, // tool_name
            crate::uar::tools::descriptor::ApprovalClass,
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
    /// Ordered fallback drivers activated when the primary driver errors.
    fallback_drivers: Vec<FailoverTarget>,
    /// Controls when/how to switch to the fallback driver.
    failover_config: FailoverConfig,
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
    sandbox_scope: Option<crate::sandbox::execution::SandboxRun>,
    terminal_scope: Option<crate::uar::tools::terminal_process::TerminalRun>,
    artifact_collector: Option<crate::uar::runtime::thread::artifacts::RunArtifactCollector>,
    thread_policy: Option<Arc<crate::uar::runtime::thread::policy_intersection::ThreadPolicy>>,
    /// Controls which tool calls are routed to the sandbox runner.
    tool_execution_mode: crate::uar::domain::artifact::ToolExecutionMode,
    resilience_policy: crate::uar::settings::resilience_policy::ResiliencePolicy,
    /// Bound applied once to every tool result when it is recorded into the
    /// model-visible history (MCP, native, and terminal results alike).
    tool_output_policy: crate::uar::runtime::context::truncate::TruncationPolicy,
    /// Per-run cache strategy copied into every policy-bearing tool-loop request.
    cache_strategy: Option<CacheStrategy>,
    /// Host-resolved final-wire budget contract copied into every request.
    request_budget_contract: Option<crate::uar::runtime::context::budget::RequestBudgetContract>,
    /// Exact model/template/settings contracts used to rebuild every provider
    /// attempt from canonical history.
    destination_preparations: Option<Arc<DestinationRequestPreparations>>,
    /// Unrendered, unreduced messages captured by the trusted host. Completed
    /// tool records are appended to this stream, never to a prepared view only.
    canonical_history: Option<Vec<Message>>,
    canonical_fragments: Option<Vec<crate::uar::runtime::prompt::PromptFragment>>,
    protected_continuity: Option<ProtectedContinuity>,
    skill_activation: Option<SkillActivationRuntime>,
    resolved_turn: Option<Arc<crate::uar::runtime::turn::ResolvedTurn>>,
    mcp_preflight: Option<Arc<crate::mcp::preflight::McpPreflight>>,
    shadow_turn: Option<(Arc<crate::uar::runtime::turn::ResolvedTurn>, Vec<Message>)>,
    world_state: Option<Arc<crate::uar::runtime::world_state::runtime::WorldStateRuntime>>,
    canonical_receipt_store: Option<Arc<dyn crate::uar::persistence::PersistenceLayer>>,
}

#[derive(Clone)]
struct SkillActivationRuntime {
    context: Arc<tokio::sync::Mutex<crate::uar::runtime::skills::activation::ActivationContext>>,
    strategy: crate::uar::context::ContextStrategy,
    model: String,
    context_limit: usize,
    budget: crate::config::SkillReattachmentBudget,
}

#[derive(Clone)]
struct FailoverTarget {
    model: String,
    provider_id: String,
    driver: Arc<dyn LlmDriver>,
}

/// Opaque provider state that must survive only through an explicitly
/// compatible destination. The body remains in canonical host storage; this
/// identity prevents transparent failover from rewriting or inventing it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtectedContinuity {
    pub provider_id: String,
    pub endpoint_kind: String,
    pub protocol_revision: String,
}

/// Exact host-reviewed preparation contract for one destination model.
#[derive(Debug, Clone)]
pub struct DestinationRequestPreparation {
    pub destination: super::prompt_dialect::TemplateDestination,
    pub template: crate::uar::runtime::prompt::PromptTemplateProfile,
    pub request_profile: super::EndpointRequestProfile,
    pub budget_contract: crate::uar::runtime::context::budget::RequestBudgetContract,
    pub exact_settings: Option<serde_json::Value>,
    pub compatible_continuity: Vec<ProtectedContinuity>,
}

impl DestinationRequestPreparation {
    fn qualified_model(&self) -> String {
        format!(
            "{}/{}",
            self.destination.provider_id, self.destination.model_id
        )
    }

    fn validate(&self) -> anyhow::Result<()> {
        let qualified_model = self.qualified_model();
        anyhow::ensure!(
            self.request_profile.qualified_model == qualified_model,
            "Destination preparation request profile belongs to another model"
        );
        anyhow::ensure!(
            self.request_profile.provider_id == self.destination.provider_id
                && self.request_profile.endpoint_kind == self.destination.endpoint_kind
                && self.request_profile.model_revision == self.destination.model_revision,
            "Destination preparation identities do not match the request profile"
        );
        anyhow::ensure!(
            self.budget_contract.destination_model == qualified_model,
            "Destination preparation budget belongs to another model"
        );
        anyhow::ensure!(
            self.budget_contract.destination_endpoint_fingerprint
                == self.request_profile.endpoint_fingerprint,
            "Destination preparation budget belongs to another endpoint"
        );
        anyhow::ensure!(
            self.request_profile.output_ceiling == Some(self.budget_contract.output_ceiling),
            "Destination preparation output ceiling does not match its budget"
        );
        super::prompt_dialect::PromptTemplateResolver::new(vec![self.template.clone()]).resolve(
            &self.destination,
            None,
            &super::prompt_dialect::TemplateOverridePolicy::default(),
        )?;
        Ok(())
    }
}

/// Immutable destination contracts captured by the trusted run host.
#[derive(Debug, Clone, Default)]
pub struct DestinationRequestPreparations {
    profiles: BTreeMap<String, DestinationRequestPreparation>,
}

impl DestinationRequestPreparations {
    /// Validate and index exact destination profiles.
    pub fn new(profiles: Vec<DestinationRequestPreparation>) -> anyhow::Result<Self> {
        let mut indexed = BTreeMap::new();
        for profile in profiles {
            profile.validate()?;
            let model = profile.qualified_model();
            anyhow::ensure!(
                indexed.insert(model.clone(), profile).is_none(),
                "Duplicate destination preparation for model `{model}`"
            );
        }
        Ok(Self { profiles: indexed })
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.profiles.is_empty()
    }

    fn prepare(
        &self,
        model: &str,
        mut request: LlmRequest,
        canonical_messages: &[serde_json::Value],
        fragments: &[crate::uar::runtime::prompt::PromptFragment],
        continuity: Option<&ProtectedContinuity>,
    ) -> anyhow::Result<LlmRequest> {
        let profile = self
            .profiles
            .get(model)
            .ok_or_else(|| anyhow::anyhow!("No exact request preparation for `{model}`"))?;
        if let Some(continuity) = continuity {
            anyhow::ensure!(
                profile.compatible_continuity.contains(continuity),
                "Destination `{model}` is incompatible with protected continuity `{}/{}/{}`",
                continuity.provider_id,
                continuity.endpoint_kind,
                continuity.protocol_revision
            );
        }
        let rendered =
            crate::uar::runtime::prompt::render_with_template(fragments, &profile.template)?;
        let system = serde_json::json!({"role": "system", "content": rendered});
        let mut messages = canonical_messages.to_vec();
        if messages
            .first()
            .and_then(|message| message.get("role"))
            .and_then(serde_json::Value::as_str)
            == Some("system")
        {
            messages[0] = system;
        } else {
            messages.insert(0, system);
        }
        request.messages = messages;
        request.extra_params.clone_from(&profile.exact_settings);
        request.budget_contract = Some(profile.budget_contract.clone());
        Ok(request)
    }
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
    async fn execute_direct_tool(
        &self,
        sequence: u64,
        call_id: &str,
        provider_name: &str,
        arguments: &serde_json::Value,
        output_policy: crate::uar::runtime::context::truncate::TruncationPolicy,
    ) -> anyhow::Result<(String, String, bool)> {
        if let Some(native) = self.native_skills.get(provider_name).await {
            let execution = match crate::uar::runtime::native_skill::execute_native(
                native.as_ref(),
                arguments.clone(),
                &self.native_execution_context(call_id),
            )
            .await
            {
                Ok(execution) => execution,
                Err(error) => {
                    let content = self
                        .preserve_terminal_tool_failure(
                            sequence,
                            call_id,
                            provider_name,
                            crate::uar::persistence::agent_threads::CanonicalReceiptSource::Native,
                            error.to_string(),
                        )
                        .await?;
                    return Ok((content.clone(), content, false));
                }
            };
            let source = if provider_name == "terminal_exec" {
                crate::uar::persistence::agent_threads::CanonicalReceiptSource::Terminal
            } else {
                crate::uar::persistence::agent_threads::CanonicalReceiptSource::Native
            };
            let value = self
                .preserve_canonical_tool_result(
                    sequence,
                    call_id,
                    provider_name,
                    source,
                    execution.value,
                    execution.canonical.raw_segments,
                    execution.canonical.acquisition_complete,
                    execution.canonical.observed_bytes,
                )
                .await?;
            let canonical = serde_json::to_string(&value)?;
            let display = native.format_result(&value, output_policy, &self.llm_config.model);
            Ok((canonical, display, true))
        } else {
            let source = if self.mcp.is_native_tool(provider_name) {
                crate::uar::persistence::agent_threads::CanonicalReceiptSource::Native
            } else {
                crate::uar::persistence::agent_threads::CanonicalReceiptSource::Mcp
            };
            let value = match self
                .call_mcp_tool(call_id, provider_name, arguments.clone())
                .await
            {
                Ok(value) => value,
                Err(error) => {
                    let content = self
                        .preserve_terminal_tool_failure(
                            sequence,
                            call_id,
                            provider_name,
                            source,
                            error.to_string(),
                        )
                        .await?;
                    return Ok((content.clone(), content, false));
                }
            };
            let value = self
                .preserve_canonical_tool_result(
                    sequence,
                    call_id,
                    provider_name,
                    source,
                    value,
                    Vec::new(),
                    true,
                    0,
                )
                .await?;
            let canonical = serde_json::to_string(&value)?;
            let display = crate::uar::runtime::context::truncate::formatted_truncate_for_model(
                &canonical,
                output_policy,
                &self.llm_config.model,
            );
            Ok((canonical, display, true))
        }
    }

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
            fallback_drivers: Vec::new(),
            failover_config: FailoverConfig::default(),
            health_monitor: None,
            native_skills,
            tool_approval_gate: None,
            sandbox_runner: None,
            sandbox_scope: None,
            terminal_scope: None,
            artifact_collector: None,
            thread_policy: None,
            tool_execution_mode: crate::uar::domain::artifact::ToolExecutionMode::default(),
            resilience_policy: crate::uar::settings::resilience_policy::ResiliencePolicy::default(),
            tool_output_policy: crate::uar::runtime::context::truncate::TruncationPolicy::default(),
            cache_strategy: None,
            request_budget_contract: None,
            destination_preparations: None,
            canonical_history: None,
            canonical_fragments: None,
            protected_continuity: None,
            skill_activation: None,
            resolved_turn: None,
            mcp_preflight: None,
            shadow_turn: None,
            world_state: None,
            canonical_receipt_store: None,
        }
    }

    /// Share host-owned activations without persisting their reclaimable bodies
    /// in the history that compaction summarizes.
    #[must_use]
    pub fn with_skill_activation(
        mut self,
        context: Arc<
            tokio::sync::Mutex<crate::uar::runtime::skills::activation::ActivationContext>,
        >,
        strategy: crate::uar::context::ContextStrategy,
        model: String,
        context_limit: usize,
        budget: crate::config::SkillReattachmentBudget,
    ) -> Self {
        self.skill_activation = Some(SkillActivationRuntime {
            context,
            strategy,
            model,
            context_limit,
            budget,
        });
        self
    }

    /// Attach a trusted-host-resolved final-wire budget contract to every
    /// request preparation performed by this orchestrator.
    #[must_use]
    pub fn with_request_budget_contract(
        mut self,
        contract: crate::uar::runtime::context::budget::RequestBudgetContract,
    ) -> Self {
        self.request_budget_contract = Some(contract);
        self
    }

    /// Attach exact destination contracts and the immutable canonical history
    /// from which every initial, retry, failover, graph, resume and loop request
    /// is prepared.
    #[must_use]
    pub fn with_destination_preparations(
        mut self,
        preparations: Arc<DestinationRequestPreparations>,
        canonical_history: Vec<Message>,
        canonical_fragments: Vec<crate::uar::runtime::prompt::PromptFragment>,
    ) -> Self {
        self.destination_preparations = Some(preparations);
        self.canonical_history = Some(canonical_history);
        self.canonical_fragments = Some(canonical_fragments);
        self
    }

    /// Bind opaque provider continuity retained by the trusted host.
    #[must_use]
    pub fn with_protected_continuity(mut self, continuity: ProtectedContinuity) -> Self {
        self.protected_continuity = Some(continuity);
        self
    }

    fn prepare_attempt(
        &self,
        model: &str,
        request: LlmRequest,
        canonical_messages: &[serde_json::Value],
        fragments: &[crate::uar::runtime::prompt::PromptFragment],
    ) -> anyhow::Result<LlmRequest> {
        match &self.destination_preparations {
            Some(preparations) => preparations.prepare(
                model,
                request,
                canonical_messages,
                fragments,
                self.protected_continuity.as_ref(),
            ),
            None => Ok(request),
        }
    }

    fn attempt_manifest(&self, model: &str, request: &LlmRequest) -> serde_json::Value {
        match &self.destination_preparations {
            Some(preparations) => {
                let preparation = preparations.profiles.get(model);
                serde_json::json!({
                    "schema_version": "uar.attempt-manifest.v1",
                    "destination_model": model,
                    "budgeting": {
                        "label": "budgeted",
                        "contract_present": request.budget_contract.is_some(),
                    },
                    "template": preparation.map(|profile| serde_json::json!({
                        "id": profile.template.id,
                        "revision": profile.template.revision,
                        "wire_contract_id": profile.template.wire_contract_id,
                    })),
                    "request_profile_revision": preparation
                        .map(|profile| profile.request_profile.profile_revision.clone()),
                })
            }
            None => serde_json::json!({
                "schema_version": "uar.attempt-manifest.v1",
                "destination_model": model,
                "budgeting": {
                    "label": "legacy_unbudgeted",
                    "contract_present": false,
                    "fit_guarantee": false,
                },
                "warning": "Provider dispatch is using the legacy unbudgeted path",
            }),
        }
    }

    #[must_use]
    pub fn with_resolved_turn(
        mut self,
        turn: Arc<crate::uar::runtime::turn::ResolvedTurn>,
    ) -> Self {
        self.resolved_turn = Some(turn);
        self
    }

    /// Persist pre-format tool results through the trusted host's run store.
    #[must_use]
    pub fn with_canonical_receipt_store(
        mut self,
        store: Option<Arc<dyn crate::uar::persistence::PersistenceLayer>>,
    ) -> Self {
        self.canonical_receipt_store = store;
        self
    }

    async fn preserve_canonical_tool_result(
        &self,
        sequence: u64,
        call_id: &str,
        tool: &str,
        source: crate::uar::persistence::agent_threads::CanonicalReceiptSource,
        value: serde_json::Value,
        raw_segments: Vec<crate::uar::persistence::agent_threads::CanonicalRawSegment>,
        acquisition_complete: bool,
        observed_bytes: u64,
    ) -> anyhow::Result<serde_json::Value> {
        let Some(store) = &self.canonical_receipt_store else {
            return Ok(value);
        };
        let turn = self
            .resolved_turn
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Canonical receipt store has no resolved turn"))?;
        let receipt =
            crate::uar::persistence::agent_threads::CanonicalToolReceipt::acquire_with_segments(
                turn.environment().owner_id.clone(),
                turn.environment().run_id.clone(),
                sequence,
                call_id,
                tool,
                source,
                value,
                raw_segments,
                acquisition_complete,
                observed_bytes,
            )?;
        let stored = store.save_canonical_tool_receipt(&receipt).await?;
        stored.typed_value.ok_or_else(|| {
            anyhow::anyhow!(
                "Canonical tool result was not retained: {:?}",
                stored.completeness
            )
        })
    }

    async fn preserve_terminal_tool_failure(
        &self,
        sequence: u64,
        call_id: &str,
        tool: &str,
        source: crate::uar::persistence::agent_threads::CanonicalReceiptSource,
        error: String,
    ) -> anyhow::Result<String> {
        let value = serde_json::json!({
            "status": "error",
            "tool_call_id": call_id,
            "tool": tool,
            "provenance": {
                "source": source,
                "terminal_state": "failed",
                "observed_by": "trusted_host"
            },
            "message": error,
        });
        let stored = self
            .preserve_canonical_tool_result(
                sequence,
                call_id,
                tool,
                source,
                value,
                Vec::new(),
                true,
                0,
            )
            .await?;
        serde_json::to_string(&stored).map_err(Into::into)
    }

    fn canonical_source_for(
        tool: &str,
        descriptor: &crate::uar::tools::descriptor::ToolDescriptor,
    ) -> crate::uar::persistence::agent_threads::CanonicalReceiptSource {
        if tool == "terminal_exec" {
            crate::uar::persistence::agent_threads::CanonicalReceiptSource::Terminal
        } else if descriptor.source == crate::uar::tools::descriptor::ToolSource::Mcp {
            crate::uar::persistence::agent_threads::CanonicalReceiptSource::Mcp
        } else {
            crate::uar::persistence::agent_threads::CanonicalReceiptSource::Native
        }
    }

    /// Use the host-prepared projection for all MCP advertisement and execution.
    /// Native implementations remain in their existing governed registries.
    #[must_use]
    pub fn with_mcp_preflight(
        mut self,
        preflight: Arc<crate::mcp::preflight::McpPreflight>,
    ) -> Self {
        self.mcp_preflight = Some(preflight);
        self
    }

    async fn call_mcp_tool(
        &self,
        call_id: &str,
        name: &str,
        arguments: serde_json::Value,
    ) -> anyhow::Result<serde_json::Value> {
        if self.mcp.is_native_tool(name) {
            return self
                .mcp
                .call_native_with_context(name, arguments, &self.native_execution_context(call_id))
                .await;
        }
        if let Some(preflight) = &self.mcp_preflight
            && !self.mcp.is_native_tool(name)
        {
            return preflight.call_tool(name, arguments).await;
        }
        self.mcp.call_namespaced_tool(name, arguments).await
    }

    /// Execute a graph's explicit MCP operation at the same trusted boundary
    /// as a model tool call. Graph state supplies arguments, never authority.
    #[allow(clippy::too_many_arguments)]
    pub(crate) async fn execute_graph_mcp_tool(
        &self,
        run_id: &str,
        step: u32,
        name: &str,
        arguments: serde_json::Value,
        events: &dyn crate::uar::domain::events::RuntimeEventSink,
        cancellation: &tokio_util::sync::CancellationToken,
    ) -> anyhow::Result<String> {
        use crate::uar::domain::events::NormalizedEvent as RuntimeEvent;
        anyhow::ensure!(!cancellation.is_cancelled(), "Graph tool host is closed");
        let turn = self
            .resolved_turn
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Graph tool host has no resolved turn"))?;
        anyhow::ensure!(
            turn.environment().run_id == run_id,
            "Graph tool belongs to another run"
        );
        let mut host = self.clone();
        if let Some(activation) = &self.skill_activation {
            let context = activation.context.lock().await;
            host.mcp = Arc::new(context.mcp().clone());
            host.mcp_preflight = context.mcp_preflight().cloned();
        }
        if let Some(preflight) = &host.mcp_preflight {
            anyhow::ensure!(
                turn.verified_owner() == Some(preflight.owner()),
                "Graph MCP binding belongs to another owner"
            );
        }
        let descriptors = host.assembled_descriptors().await?;
        let descriptor = descriptors
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("Graph tool is absent from the captured projection"))?;
        anyhow::ensure!(
            descriptor.source == crate::uar::tools::descriptor::ToolSource::Mcp
                || host.mcp.is_native_tool(name),
            "Graph tool is not an MCP registry operation"
        );
        anyhow::ensure!(
            !matches!(descriptor.exposure, Exposure::Hidden | Exposure::ModelOnly),
            "Graph tool is not exposed to host callers"
        );
        anyhow::ensure!(
            !host.requires_sandbox(descriptor),
            "Graph MCP operation has no sandbox execution adapter"
        );
        let arguments_json = serde_json::to_string(&arguments)?;
        let arguments = validate::validate(&descriptor.validator, &arguments_json)?;
        let gate = host
            .tool_approval_gate
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Graph tool host has no approval gate"))?;
        let call_id = Uuid::new_v4().to_string();
        if let ToolApprovalResult::Rejected { reason } = gate(
            call_id.clone(),
            name.to_owned(),
            descriptor.approval_class,
            arguments_json,
            step as usize,
        )
        .await
        {
            anyhow::bail!("Graph tool rejected: {reason}");
        }
        events
            .emit(RuntimeEvent::ToolStart {
                run_id: run_id.to_owned(),
                call_index: step as usize,
                tool_call_id: call_id.clone(),
                tool: name.to_owned(),
                input: arguments.clone(),
            })
            .await;
        // ToolStart publication awaits the event sink. Revocation during that
        // await must not turn shutdown's drain into a new external operation.
        let result = if cancellation.is_cancelled() {
            Err(anyhow::anyhow!("Graph tool cancelled before dispatch"))
        } else {
            host.call_mcp_tool(&call_id, name, arguments).await
        };
        let (canonical_content, success) = match result {
            Ok(result) => match host
                .preserve_canonical_tool_result(
                    u64::from(step) << 32,
                    &call_id,
                    name,
                    crate::uar::persistence::agent_threads::CanonicalReceiptSource::Graph,
                    result,
                    Vec::new(),
                    true,
                    0,
                )
                .await
            {
                Ok(result) => (serde_json::to_string(&result)?, true),
                Err(error) => (format!("Canonical receipt error: {error}"), false),
            },
            Err(_) => ("Graph MCP tool execution failed".to_owned(), false),
        };
        let display_content = crate::uar::runtime::context::truncate::formatted_truncate_for_model(
            &canonical_content,
            descriptor.output_limit.unwrap_or(host.tool_output_policy),
            &host.llm_config.model,
        );
        events
            .emit(RuntimeEvent::ToolEnd {
                run_id: run_id.to_owned(),
                call_index: step as usize,
                tool_call_id: call_id,
                tool: name.to_owned(),
                output: serde_json::Value::String(display_content.clone()),
                ok: success,
            })
            .await;
        anyhow::ensure!(success, "{display_content}");
        Ok(canonical_content)
    }

    #[must_use]
    pub fn with_shadow_turn(
        mut self,
        turn: Arc<crate::uar::runtime::turn::ResolvedTurn>,
        history: Vec<Message>,
    ) -> Self {
        self.shadow_turn = Some((turn, history));
        self
    }

    /// Attach the host-owned world state for reduction and governed file reads.
    pub fn with_world_state(
        mut self,
        state: Arc<crate::uar::runtime::world_state::runtime::WorldStateRuntime>,
    ) -> Self {
        self.world_state = Some(state);
        self
    }

    fn native_execution_context(
        &self,
        call_id: &str,
    ) -> crate::uar::runtime::native_skill::NativeExecutionContext {
        crate::uar::runtime::native_skill::NativeExecutionContext {
            presentations: self
                .resolved_turn
                .as_ref()
                .and_then(|turn| turn.presentations().cloned())
                .map(|snapshot| {
                    crate::uar::runtime::native_skill::PresentationExecutionContext::new(
                        snapshot, call_id,
                    )
                }),
            verified_owner: self
                .resolved_turn
                .as_ref()
                .and_then(|turn| turn.verified_owner().cloned()),
            session_id: self
                .resolved_turn
                .as_ref()
                .map(|turn| turn.environment().session_id.clone()),
            thread_policy: self.thread_policy.clone(),
            terminal_scope: self.terminal_scope.clone(),
            artifact_collector: self.artifact_collector.clone(),
            project_instructions: self
                .world_state
                .as_ref()
                .map(|state| Arc::clone(&state.instructions)),
        }
    }

    async fn assembled_descriptors(&self) -> anyhow::Result<BTreeMap<String, Arc<ToolDescriptor>>> {
        let mut descriptors = BTreeMap::<String, Arc<ToolDescriptor>>::new();
        let mut mcp_descriptors = self.mcp.descriptors();
        if let Some(preflight) = &self.mcp_preflight {
            mcp_descriptors
                .retain(|tool| tool.source != crate::uar::tools::descriptor::ToolSource::Mcp);
            mcp_descriptors.extend(
                preflight
                    .projection()
                    .tools()
                    .values()
                    .map(|tool| Arc::clone(tool.descriptor())),
            );
        }
        for descriptor in mcp_descriptors
            .into_iter()
            .chain(self.native_skills.descriptors().await)
        {
            if let Some(existing) = descriptors.get(&descriptor.provider_name) {
                if !existing.equivalent_to(&descriptor) {
                    return Err(ToolCollision {
                        provider_name: descriptor.provider_name.clone(),
                    }
                    .into());
                }
            } else {
                descriptors.insert(descriptor.provider_name.clone(), descriptor);
            }
        }
        Ok(descriptors)
    }

    /// Attach one fallback driver and failover configuration.
    ///
    /// When `failover_config.enabled` is `true` and the primary driver fails,
    /// the orchestrator will re-try the same request against the fallback.
    /// This compatibility helper uses the first configured fallback model.
    #[must_use]
    pub fn with_failover(
        self,
        fallback_driver: Arc<dyn LlmDriver>,
        failover_config: FailoverConfig,
    ) -> Self {
        let model = failover_config.fallback_models.first().map_or_else(
            || "external/fallback".to_string(),
            |fallback| fallback.model.clone(),
        );
        self.with_failovers(vec![(model, fallback_driver)], failover_config)
    }

    /// Attach an ordered set of fallback model drivers.
    #[must_use]
    pub fn with_failovers(
        mut self,
        fallback_drivers: Vec<(String, Arc<dyn LlmDriver>)>,
        failover_config: FailoverConfig,
    ) -> Self {
        self.fallback_drivers = fallback_drivers
            .into_iter()
            .map(|(model, driver)| FailoverTarget {
                provider_id: super::registry::split_model_string_pub(&model).0,
                model,
                driver,
            })
            .collect();
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
        config.api_key.clone_from(&fallback.api_key);
        config.base_url.clone_from(&fallback.base_url);
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
    /// Sandboxed mode requires isolation for every tool; Auto requires it for
    /// code-execution tools. A descriptor's sandbox requirement always applies.
    /// Missing isolation or an unsupported tool adapter returns a failed result
    /// and never falls through to direct native/MCP execution.
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

    /// Apply the artifact's execution mode even when no sandbox is available.
    /// Absence of a runner is not permission to discard an isolation policy.
    #[must_use]
    pub fn with_tool_execution_mode(
        mut self,
        mode: crate::uar::domain::artifact::ToolExecutionMode,
    ) -> Self {
        self.tool_execution_mode = mode;
        self
    }

    /// Bind the host-owned operation scope. The host must retain its lease and
    /// drain it before terminal completion; an unowned runner cannot execute.
    #[must_use]
    pub fn with_sandbox_scope(mut self, scope: crate::sandbox::execution::SandboxRun) -> Self {
        self.sandbox_scope = Some(scope);
        self
    }

    /// Attach the trusted host's terminal lifetime scope after run admission.
    pub(crate) fn with_terminal_scope(
        mut self,
        scope: crate::uar::tools::terminal_process::TerminalRun,
    ) -> Self {
        self.terminal_scope = Some(scope);
        self
    }

    /// Retain the exact actor invocation's structured-output receipt.
    pub(crate) fn with_artifact_collector(
        mut self,
        collector: Option<crate::uar::runtime::thread::artifacts::RunArtifactCollector>,
    ) -> Self {
        self.artifact_collector = collector;
        self
    }

    /// Retain the host-intersected authority for direct native tool calls.
    pub(crate) fn with_thread_policy(
        mut self,
        policy: Arc<crate::uar::runtime::thread::policy_intersection::ThreadPolicy>,
    ) -> Self {
        self.thread_policy = Some(policy);
        self
    }

    fn requires_sandbox(&self, descriptor: &crate::uar::tools::descriptor::ToolDescriptor) -> bool {
        use crate::uar::domain::artifact::ToolExecutionMode;
        descriptor.sandbox_required
            || match self.tool_execution_mode {
                ToolExecutionMode::Direct => false,
                ToolExecutionMode::Sandboxed => true,
                ToolExecutionMode::Auto => descriptor.effect == ToolEffect::CodeExecution,
            }
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
    pub(crate) async fn graph_chat_with_history(
        &self,
        run_id: &str,
        messages: Vec<Message>,
        cancellation: &tokio_util::sync::CancellationToken,
    ) -> anyhow::Result<impl Stream<Item = NormalizedEvent> + Send + 'static> {
        anyhow::ensure!(
            self.resolved_turn
                .as_ref()
                .is_some_and(|turn| { turn.environment().run_id == run_id }),
            "Graph model turn belongs to another run"
        );
        anyhow::ensure!(
            !cancellation.is_cancelled(),
            "Graph model turn is cancelled"
        );
        let gate = self
            .tool_approval_gate
            .clone()
            .ok_or_else(|| anyhow::anyhow!("Graph model turn has no host approval gate"))?;
        let cancellation = cancellation.clone();
        let mut host = self.clone();
        host.tool_approval_gate = Some(Arc::new(move |id, name, class, arguments, index| {
            let gate = Arc::clone(&gate);
            let cancellation = cancellation.clone();
            Box::pin(async move {
                if cancellation.is_cancelled() {
                    return ToolApprovalResult::Rejected {
                        reason: "Graph model turn is cancelled".into(),
                    };
                }
                let decision = gate(id, name, class, arguments, index).await;
                if cancellation.is_cancelled() {
                    ToolApprovalResult::Rejected {
                        reason: "Graph model turn is cancelled".into(),
                    }
                } else {
                    decision
                }
            })
        }));
        host.chat_with_history_mode(messages, true).await
    }

    /// Start a chat interaction with existing message history.
    pub async fn chat_with_history(
        &self,
        messages: Vec<Message>,
    ) -> anyhow::Result<impl Stream<Item = NormalizedEvent> + Send + 'static + use<>> {
        self.chat_with_history_mode(messages, false).await
    }

    /// Captured graph capabilities cannot enter the legacy ungoverned remote
    /// adapter. This is not an authorization handshake with an A2A peer.
    pub(crate) fn check_graph_remote_compatibility(&self) -> anyhow::Result<()> {
        anyhow::ensure!(
            self.mcp_preflight.is_none() && self.thread_policy.is_none(),
            "Captured or delegated graph runs require a policy-bound remote delegation host"
        );
        Ok(())
    }

    #[allow(clippy::too_many_lines)]
    async fn chat_with_history_mode(
        &self,
        messages: Vec<Message>,
        require_terminal: bool,
    ) -> anyhow::Result<impl Stream<Item = NormalizedEvent> + Send + 'static + use<>> {
        let request_id = Uuid::new_v4().to_string();
        let exposure = crate::mcp::exposure::McpToolExposure::default();
        let initial_exposure = exposure.project(&self.assembled_descriptors().await?);

        tracing::info!(
            request_id = %request_id,
            message_count = messages.len(),
            tool_count = initial_exposure.visible().len() + usize::from(initial_exposure.has_deferred()),
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

        let mut orchestrator = self.clone();
        // Discovery controls and selections belong to this stream, never the
        // shared host registry or another child using the same native handlers.
        orchestrator.native_skills = Arc::new(self.native_skills.filtered(None).await);
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
            let canonical_messages = if require_terminal {
                &messages
            } else {
                orchestrator.canonical_history.as_ref().unwrap_or(&messages)
            };
            let mut canonical_message_json: Vec<serde_json::Value> = canonical_messages
                .iter()
                .map(|message| serde_json::to_value(message).unwrap_or_default())
                .collect();

            tracing::debug!(
                request_id = %request_id,
                "Converted messages to JSON for driver"
            );

            let mut iteration = 0;
            let mut search_registered = false;

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

                let mut request_messages = message_json.clone();
                let mut active_bodies = Vec::new();
                let mut step_fragments = orchestrator.canonical_fragments.as_deref()
                    .or_else(|| orchestrator.resolved_turn.as_ref().map(|turn| turn.fragments()))
                    .map(|fragments| {
                    fragments.iter().filter(|fragment| {
                        fragment.retention != crate::uar::runtime::prompt::Retention::Reclaimable
                            && (iteration == 1 || fragment.section != crate::uar::runtime::prompt::PromptSection::WorldState)
                    }).cloned().collect::<Vec<_>>()
                }).unwrap_or_default();
                let mut skill_usage = crate::uar::runtime::skills::usage::SkillRequestUsage::new(
                    orchestrator.llm_config.model.clone(), Vec::new(), orchestrator.llm_config.cost_tracking,
                );
                if let Some(activation) = &orchestrator.skill_activation {
                    let (mcp, preflight, active) = {
                        let context = activation.context.lock().await;
                        (context.mcp().clone(), context.mcp_preflight().cloned(), context.active())
                    };
                    skill_usage.skills = active.iter().map(|entry| entry.skill.skill_id.clone()).collect();
                    active_bodies = active.iter().map(|entry| entry.fragment()).collect();
                    orchestrator.mcp = Arc::new(mcp);
                    if let Some(preflight) = preflight { orchestrator.mcp_preflight = Some(preflight); }
                    let history = match serde_json::from_value::<Vec<Message>>(serde_json::Value::Array(message_json.clone())) {
                        Ok(history) => history,
                        Err(error) => {
                            yield NormalizedEvent::Error {
                                message: error.to_string(),
                                code: Some("HISTORY_NORMALIZATION_FAILED".to_string()),
                            };
                            break;
                        }
                    };
                    // Initial history was already reduced by the run manager.
                    // Later steps reduce body-free history before reattachment.
                    let world_contributor = if iteration > 1 {
                        match &orchestrator.world_state {
                            Some(world_state) => Some(world_state.contributor(false).await),
                            None => None,
                        }
                    } else { None };
                    let world_reserved_tokens = if let Some(contributor) = &world_contributor {
                        match contributor.reserved_tokens(&history, &activation.model) {
                            Ok(tokens) if tokens.saturating_add(1_000) < activation.context_limit => tokens,
                            result => {
                                let message = match result {
                                    Ok(tokens) => format!("World state requires {tokens} tokens and does not fit the model context budget"),
                                    Err(error) => error.to_string(),
                                };
                                yield NormalizedEvent::Error { message, code: Some("WORLD_STATE_BUDGET_EXCEEDED".into()) };
                                break;
                            }
                        }
                    } else { 0 };
                    let (mut history, rewritten) = if iteration > 1 {
                        match crate::uar::runtime::context::reduce::reduce_history(
                            history, &activation.strategy, &activation.model,
                            activation.context_limit - world_reserved_tokens, Some(orchestrator.driver.as_ref()),
                        ).await {
                            Ok((history, report)) => (history, report.history_rewritten),
                            Err(error) => {
                                yield NormalizedEvent::Error {
                                    message: error.to_string(),
                                    code: Some(error.code().to_ascii_uppercase()),
                                };
                                break;
                            }
                        }
                    } else {
                        (history, false)
                    };
                    if let (Some(world_state), Some(contributor)) = (&orchestrator.world_state, &world_contributor) {
                        match contributor.baseline.prepare(&contributor.snapshot, &history, rewritten) {
                            Ok(update) => {
                                history.extend(update.messages.iter().cloned());
                                step_fragments.extend(update.fragments.iter().cloned());
                                world_state.commit(&update).await;
                            }
                            Err(error) => {
                                yield NormalizedEvent::Error {
                                    message: error.to_string(), code: Some("WORLD_STATE_ASSEMBLY_FAILED".into()),
                                };
                                break;
                            }
                        }
                    }
                    let dialect = super::prompt_dialect::PromptDialect::detect(&orchestrator.llm_config.model);
                    let (attached, skill_fragments) = crate::uar::runtime::skills::retention::reattach_skills(
                        &history, &active, &activation.model, activation.context_limit,
                        activation.budget,
                        crate::uar::runtime::prompt::RenderOptions {
                            prefers_xml_envelope: dialect.prefers_xml_envelope(),
                            markdown_averse: dialect.markdown_averse(),
                        },
                    );
                    step_fragments.extend(skill_fragments);
                    message_json = history.iter().map(|message| serde_json::json!(message)).collect();
                    request_messages = attached.iter().map(|message| serde_json::json!(message)).collect();
                }

                let projected = async {
                    use crate::uar::runtime::native_skills::search_tools::{SEARCH_TOOLS_NAME, SearchToolsTool};
                    let mut current = orchestrator.assembled_descriptors().await?;
                    if search_registered { current.remove(SEARCH_TOOLS_NAME); }
                    let snapshot = exposure.project(&current);
                    let mut visible = snapshot.visible().clone();
                    if snapshot.has_deferred() {
                        if !search_registered {
                            if current.contains_key(SEARCH_TOOLS_NAME) {
                                return Err(ToolCollision { provider_name: SEARCH_TOOLS_NAME.to_owned() }.into());
                            }
                            orchestrator.native_skills.register(SearchToolsTool::new(exposure.clone())
                                .with_thread_policy(orchestrator.thread_policy.clone())).await?;
                            search_registered = true;
                        }
                        let descriptor = orchestrator.native_skills.descriptor(SEARCH_TOOLS_NAME).await
                            .ok_or_else(|| anyhow::anyhow!("registered MCP discovery control is missing"))?;
                        visible.insert(SEARCH_TOOLS_NAME.to_owned(), descriptor);
                    }
                    Ok::<_, anyhow::Error>(visible)
                }.await;
                let mut descriptors = match projected {
                    Ok(visible) => Arc::new(visible),
                    Err(error) => {
                        yield NormalizedEvent::Error {
                            message: error.to_string(), code: Some("TOOL_ASSEMBLY_FAILED".to_string()),
                        };
                        break;
                    }
                };
                // Execution consults this same frozen visible map, so a search
                // cannot authorize another call in the current model batch.
                let tools = descriptors.values().map(|tool| tool.openai_tool_json()).collect::<Vec<_>>();

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

                let normalize_report = match crate::uar::runtime::context::normalize::normalize_provider_messages(&mut request_messages) {
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
                            code: Some("INVALID_HISTORY".to_string()),
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
                let base_req = LlmRequest {
                    messages: request_messages,
                    tools: tools.clone(),
                    cache_strategy: orchestrator.cache_strategy.clone(),
                    thinking_config: None,
                    anthropic_system: None,
                    extra_params: dialect_params
                        .as_object()
                        .filter(|o| !o.is_empty())
                        .map(|_| dialect_params.clone()),
                    budget_contract: orchestrator.request_budget_contract.clone(),
                };
                let primary_model = orchestrator.llm_config.model.clone();
                let req = match orchestrator.prepare_attempt(
                    &primary_model,
                    base_req.clone(),
                    &canonical_message_json,
                    &step_fragments,
                ) {
                    Ok(request) => request,
                    Err(error) => {
                        yield NormalizedEvent::Error {
                            message: error.to_string(),
                            code: Some("DESTINATION_PREPARATION_FAILED".into()),
                        };
                        break;
                    }
                };

                let resolved_step = if let Some(turn) = &orchestrator.resolved_turn {
                    let rendered = req.messages.iter().filter_map(|message| message["content"].as_str())
                        .collect::<Vec<_>>().join("\n");
                    let mut budgets = crate::uar::runtime::prompt::PromptBudgets::for_rendered(&rendered);
                    budgets.context_window_tokens = orchestrator.skill_activation.as_ref().map(|activation| activation.context_limit);
                    match crate::uar::runtime::turn::ResolvedStep::new(
                        Arc::clone(turn), step, req.clone(), (*descriptors).clone(),
                        Arc::clone(&orchestrator.mcp), skill_usage.skills.clone(), budgets,
                        step_fragments.clone(),
                    ).and_then(|snapshot| match &orchestrator.mcp_preflight {
                        Some(preflight) => snapshot.with_mcp_preflight(Arc::clone(preflight)),
                        None => Ok(snapshot),
                    }) {
                        Ok(snapshot) => Some(snapshot),
                        Err(error) => {
                            yield NormalizedEvent::Error { message: error.to_string(), code: Some("TURN_ASSEMBLY_REJECTED".to_string()) };
                            break;
                        }
                    }
                } else { None };
                let resolved_step = if let (Some(snapshot), Some((typed_turn, initial_history))) = (resolved_step.as_ref(), &orchestrator.shadow_turn) {
                    let comparison = (|| -> anyhow::Result<_> {
                        let history = if iteration == 1 {
                            initial_history.clone()
                        } else {
                            serde_json::from_value::<Vec<Message>>(serde_json::Value::Array(message_json.clone()))?
                        };
                        let dialect = super::prompt_dialect::PromptDialect::detect(&orchestrator.llm_config.model);
                        crate::uar::runtime::turn::shadow::compare_step(
                            snapshot, typed_turn, &history, &active_bodies,
                            orchestrator.skill_activation.as_ref().map(|activation| activation.budget).unwrap_or_default(),
                            crate::uar::runtime::prompt::RenderOptions {
                                prefers_xml_envelope: dialect.prefers_xml_envelope(),
                                markdown_averse: dialect.markdown_averse(),
                            },
                        )
                    })();
                    match comparison {
                        Ok(report) => Some(snapshot.clone().with_shadow(report)),
                        Err(error) => {
                            yield NormalizedEvent::Error { message: error.to_string(), code: Some("SHADOW_ASSEMBLY_FAILED".into()) };
                            break;
                        }
                    }
                } else { resolved_step };
                let req = if let Some(snapshot) = &resolved_step {
                    descriptors = Arc::new(snapshot.tools().clone());
                    orchestrator.mcp = Arc::clone(snapshot.mcp());
                    orchestrator.mcp_preflight = snapshot.mcp_preflight().cloned();
                    yield NormalizedEvent::Custom {
                        source: "uar.turn".into(),
                        event_name: "resolved_step".into(),
                        payload: serde_json::json!({
                            "step": snapshot.index(),
                            "model": snapshot.turn().credentials().model,
                            "mcp_catalog": snapshot.mcp_catalog(),
                            "manifest": snapshot.manifest(),
                        }),
                    };
                    snapshot.request().clone()
                } else { req };
                let mut attempt_source = base_req;
                attempt_source.tools.clone_from(&req.tools);

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
                let primary_provider_id = orchestrator
                    .llm_config
                    .resolved_provider_id
                    .clone()
                    .unwrap_or_else(|| {
                        super::registry::split_model_string_pub(&orchestrator.llm_config.model).0
                    });
                let driver_stream = {
                    let policy = &orchestrator.resilience_policy;
                    let max_attempts = if policy.retries_enabled {
                        policy.retry_max_attempts.max(1)
                    } else {
                        1
                    };
                    let retry_driver = Arc::clone(&orchestrator.driver);
                    let retry_orchestrator = orchestrator.clone();
                    let retry_source = attempt_source.clone();
                    let retry_canonical = canonical_message_json.clone();
                    let retry_fragments = step_fragments.clone();
                    let retry_model = primary_model.clone();
                    let stream_start_timeout =
                        std::time::Duration::from_millis(policy.stream_start_timeout_ms);
                    let stream_idle_timeout =
                        std::time::Duration::from_millis(policy.stream_idle_timeout_ms);
                    let retry_budget = std::time::Duration::from_millis(policy.retry_budget_ms);
                    let mut adjusted_delay_spent = std::time::Duration::ZERO;
                    let mut attempt = 1_u32;
                    let primary = (|| {
                        let driver = Arc::clone(&retry_driver);
                        let orchestrator = retry_orchestrator.clone();
                        let source = retry_source.clone();
                        let canonical = retry_canonical.clone();
                        let fragments = retry_fragments.clone();
                        let model = retry_model.clone();
                        async move {
                            let request = orchestrator.prepare_attempt(
                                &model,
                                source,
                                &canonical,
                                &fragments,
                            )?;
                            let manifest = orchestrator.attempt_manifest(&model, &request);
                            let stream = open_driver_stream(
                                driver.as_ref(),
                                request,
                                stream_start_timeout,
                                stream_idle_timeout,
                            )
                            .await?;
                            Ok(prepend_attempt_manifest(stream, manifest))
                        }
                    })
                    .retry(policy.retry_backoff_builder())
                    .when(|error| {
                        policy.retries_enabled
                            && super::ProviderError::from_anyhow(error)
                                .is_some_and(|error| error.is_retryable(policy))
                    })
                    .adjust(|error, proposed_delay| {
                        let proposed_delay = proposed_delay?;
                        let delay = if policy.retry_respect_retry_after {
                            super::ProviderError::from_anyhow(error)
                                .and_then(|error| error.retry_after)
                                .unwrap_or(proposed_delay)
                        } else {
                            proposed_delay
                        };
                        if adjusted_delay_spent.saturating_add(delay) > retry_budget {
                            return None;
                        }
                        adjusted_delay_spent = adjusted_delay_spent.saturating_add(delay);
                        Some(delay)
                    })
                    .notify(|error, delay| {
                        tracing::warn!(
                            request_id = %request_id,
                            iteration,
                            attempt,
                            max_attempts,
                            delay_ms = delay.as_millis(),
                            error = %error,
                            "LLM stream creation failed; retrying before semantic events"
                        );
                        attempt = attempt.saturating_add(1);
                    })
                    .await;
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
                            let mut fallback_stream = None;
                            let mut fallback_errors = Vec::new();
                            for fallback in &orchestrator.fallback_drivers {
                                if let Some(health) = &orchestrator.health_monitor
                                    && !health.is_available(&fallback.provider_id).await
                                {
                                    tracing::info!(
                                        request_id = %request_id,
                                        iteration,
                                        fallback_model = %fallback.model,
                                        fallback_provider = %fallback.provider_id,
                                        "Skipping fallback provider in cooldown"
                                    );
                                    continue;
                                }
                                tracing::warn!(
                                    request_id = %request_id,
                                    iteration,
                                    primary_error = %e,
                                    fallback_model = %fallback.model,
                                    "Primary LLM driver failed; attempting fallback",
                                );
                                let fallback_result = match orchestrator.prepare_attempt(
                                    &fallback.model,
                                    attempt_source.clone(),
                                    &canonical_message_json,
                                    &step_fragments,
                                ) {
                                    Ok(request) => {
                                        let manifest = orchestrator
                                            .attempt_manifest(&fallback.model, &request);
                                        open_driver_stream(
                                            fallback.driver.as_ref(),
                                            request,
                                            stream_start_timeout,
                                            stream_idle_timeout,
                                        )
                                        .await
                                        .map(|stream| prepend_attempt_manifest(stream, manifest))
                                    }
                                    Err(error) => Err(error),
                                };
                                match fallback_result
                                {
                                    Ok(s) => {
                                        if let Some(health) = &orchestrator.health_monitor {
                                            health.record_success(&fallback.provider_id).await;
                                        }
                                        skill_usage.model = fallback.model.clone();
                                        fallback_stream = Some(s);
                                        break;
                                    }
                                    Err(fe) => {
                                        if let Some(health) = &orchestrator.health_monitor {
                                            health
                                                .record_failure(
                                                    &fallback.provider_id,
                                                    orchestrator.failover_config.error_threshold,
                                                    orchestrator.failover_config.cooldown_secs,
                                                )
                                                .await;
                                        }
                                        tracing::warn!(
                                            request_id = %request_id,
                                            primary_error = %e,
                                            fallback_error = %fe,
                                            fallback_model = %fallback.model,
                                            "Fallback driver failed; trying the next candidate",
                                        );
                                        fallback_errors.push(format!("{}: {fe}", fallback.model));
                                    }
                                }
                            }
                            if let Some(stream) = fallback_stream {
                                stream
                            } else {
                                tracing::error!(
                                    request_id = %request_id,
                                    iteration = iteration,
                                    error = %e,
                                    fallback_errors = ?fallback_errors,
                                    "Primary driver failed; no healthy fallback succeeded",
                                );
                                let fallback_detail = if fallback_errors.is_empty() {
                                    "no healthy fallback available".to_string()
                                } else {
                                    fallback_errors.join("; ")
                                };
                                yield NormalizedEvent::Error {
                                    message: format!("primary: {e}; fallbacks: {fallback_detail}"),
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
                let mut saw_terminal = false;

                futures::pin_mut!(driver_stream);

                loop {
                    let result = match tokio::time::timeout(
                        std::time::Duration::from_millis(
                            orchestrator.resilience_policy.stream_idle_timeout_ms,
                        ),
                        driver_stream.next(),
                    )
                    .await
                    {
                        Ok(Some(result)) => result,
                        Ok(None) => {
                            if require_terminal && !saw_terminal {
                                yield NormalizedEvent::Error {
                                    message: "Graph provider stream ended without a terminal event".into(),
                                    code: Some("provider_stream_incomplete".into()),
                                };
                                return;
                            }
                            break;
                        },
                        Err(_) => Err(super::ProviderError::timeout(format!(
                            "LLM stream idle timed out after {} ms",
                            orchestrator.resilience_policy.stream_idle_timeout_ms
                        ))
                        .into()),
                    };
                    match result {
                        Ok(event) => {
                            skill_usage.observe(&event);
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
                                    saw_terminal = true;
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
                canonical_message_json.push(
                    message_json
                        .last()
                        .cloned()
                        .expect("assistant tool-call record was just appended"),
                );

                tracing::debug!(
                    request_id = %request_id,
                    iteration = iteration,
                    "Added assistant message with tool calls to history"
                );

                let batch_descriptors = tool_calls
                    .iter()
                    .map(|call| descriptors.get(&call.function.name).cloned())
                    .collect::<Option<Vec<_>>>();
                let schedulable = batch_descriptors.as_ref().is_some_and(|descriptors| {
                    descriptors.iter().all(|descriptor| {
                        descriptor.approval_class == ApprovalClass::NotRequired
                            && !orchestrator.requires_sandbox(descriptor)
                    })
                });
                let mut admitted_calls = std::collections::VecDeque::new();
                if orchestrator.llm_config.parallel_tool_calls == Some(true)
                    && tool_calls.len() > 1
                    && schedulable
                {
                    let batch_descriptors = Arc::new(
                        batch_descriptors.expect("schedulable batches have one descriptor per call"),
                    );
                    // Admit in call order before dispatch, so a policy-driven
                    // confirmation never races another tool's execution. The
                    // host gate includes budget admission and runs exactly once.
                    let mut confirmed = false;
                    for (index, (call, descriptor)) in tool_calls.iter()
                        .zip(batch_descriptors.iter()).enumerate()
                    {
                        let arguments = match validate::validate(
                            &descriptor.validator,
                            &call.function.arguments,
                        ) {
                            Ok(arguments) => arguments,
                            Err(error) => {
                                admitted_calls.push_back(Err(error.model_result().to_string()));
                                continue;
                            }
                        };
                        if let Some(gate) = &orchestrator.tool_approval_gate {
                            match gate(
                                call.id.clone(),
                                call.function.name.clone(),
                                descriptor.approval_class,
                                call.function.arguments.clone(),
                                index,
                            ).await {
                                ToolApprovalResult::Rejected { reason } => {
                                    admitted_calls.push_back(Err(format!("Tool call rejected: {reason}")));
                                    confirmed = true;
                                    break;
                                }
                                ToolApprovalResult::Approved => confirmed = true,
                                ToolApprovalResult::Allowed
                                | ToolApprovalResult::GovernanceBypassed => {}
                            }
                        }
                        admitted_calls.push_back(Ok(arguments));
                        if confirmed {
                            // Resume this approved call before asking about the
                            // next. The sequential path consumes these receipts
                            // without repeating approval or charging the budget.
                            break;
                        }
                    }
                    if !confirmed {
                    // Read-only descriptors share the global read lock. Every
                    // mutating, code-executing, or unknown descriptor takes the
                    // write lock and therefore runs alone. Equal read keys also
                    // share a FIFO mutex; distinct or absent keys may overlap.
                    let execution_gate = Arc::new(tokio::sync::RwLock::new(()));
                    let key_gates = Arc::new(
                        batch_descriptors
                            .iter()
                            .filter(|descriptor| descriptor.effect == ToolEffect::ReadOnly)
                            .filter_map(|descriptor| descriptor.concurrency_key.clone())
                            .map(|key| (key, Arc::new(tokio::sync::Mutex::new(()))))
                            .collect::<BTreeMap<_, _>>(),
                    );
                    let executions = futures::stream::iter(tool_calls.iter().cloned().zip(
                        batch_descriptors.iter().cloned(),
                    ).zip(admitted_calls.drain(..)).enumerate().map(|(index, ((call, descriptor), arguments))| {
                        let orchestrator = orchestrator.clone();
                        let execution_gate = Arc::clone(&execution_gate);
                        let key_gates = Arc::clone(&key_gates);
                        let sequence = (u64::from(step) << 32) | index as u64;
                        async move {
                            let arguments = match arguments {
                                Ok(arguments) => arguments,
                                Err(error) => {
                                    let source = Self::canonical_source_for(
                                        &call.function.name,
                                        &descriptor,
                                    );
                                    let outcome = orchestrator
                                        .preserve_terminal_tool_failure(
                                            sequence,
                                            &call.id,
                                            &call.function.name,
                                            source,
                                            error,
                                        )
                                        .await
                                        .map(|content| (content.clone(), content, false));
                                    return (call, outcome);
                                }
                            };
                            let output_policy = descriptor
                                .output_limit
                                .unwrap_or(orchestrator.tool_output_policy);
                            let outcome = if descriptor.effect == ToolEffect::ReadOnly {
                                let _read = execution_gate.read().await;
                                if let Some(key_gate) = descriptor
                                    .concurrency_key
                                    .as_ref()
                                    .and_then(|key| key_gates.get(key))
                                {
                                    let _key = key_gate.lock().await;
                                    orchestrator
                                        .execute_direct_tool(
                                            sequence,
                                            &call.id,
                                            &call.function.name,
                                            &arguments,
                                            output_policy,
                                        )
                                        .await
                                } else {
                                    orchestrator
                                        .execute_direct_tool(
                                            sequence,
                                            &call.id,
                                            &call.function.name,
                                            &arguments,
                                            output_policy,
                                        )
                                        .await
                                }
                            } else {
                                let _write = execution_gate.write().await;
                                orchestrator
                                    .execute_direct_tool(
                                        sequence,
                                        &call.id,
                                        &call.function.name,
                                        &arguments,
                                        output_policy,
                                    )
                                    .await
                            };
                            (call, outcome)
                        }
                    }))
                    .buffered(8)
                    .collect::<Vec<_>>()
                    .await;

                    for (call, outcome) in executions {
                        let (canonical_content, content, success) = match outcome {
                            Ok(outcome) => outcome,
                            Err(error) => {
                                yield NormalizedEvent::Error {
                                    message: error.to_string(),
                                    code: Some("TERMINAL_RESULT_PERSISTENCE_FAILED".to_string()),
                                };
                                return;
                            }
                        };
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
                        canonical_message_json.push(serde_json::json!({
                            "role": "tool",
                            "tool_call_id": call.id,
                            "content": canonical_content
                        }));
                    }
                    yield NormalizedEvent::RuntimeStep {
                        step,
                        kind: RuntimeStepKind::Finished,
                    };
                    continue;
                    }
                }

                // Execute mutating, approval-gated, sandboxed, or unknown calls
                // sequentially to preserve policy and side-effect ordering.
                for (idx, tool_call) in tool_calls.iter().enumerate() {
                    let tool_name = &tool_call.function.name;
                    let sequence = (u64::from(step) << 32) | idx as u64;
                    let Some(descriptor) = descriptors.get(tool_name) else {
                        yield NormalizedEvent::Error {
                            message: format!(
                                "No descriptor exists for requested tool '{tool_name}'"
                            ),
                            code: Some("UNKNOWN_TOOL_STATE_UNRECOVERED".to_string()),
                        };
                        return;
                    };
                    let preadmitted = admitted_calls.pop_front();
                    let admitted_by_host = preadmitted.is_some();
                    let arguments = match preadmitted.unwrap_or_else(|| {
                        validate::validate(&descriptor.validator, &tool_call.function.arguments)
                            .map_err(|error| error.model_result().to_string())
                    }) {
                        Ok(arguments) => arguments,
                        Err(error) => {
                            let source = Self::canonical_source_for(tool_name, descriptor);
                            let content = match orchestrator
                                .preserve_terminal_tool_failure(
                                    sequence,
                                    &tool_call.id,
                                    tool_name,
                                    source,
                                    error,
                                )
                                .await
                            {
                                Ok(content) => content,
                                Err(error) => {
                                    yield NormalizedEvent::Error {
                                        message: error.to_string(),
                                        code: Some(
                                            "TERMINAL_RESULT_PERSISTENCE_FAILED".to_string(),
                                        ),
                                    };
                                    return;
                                }
                            };
                            yield NormalizedEvent::ToolResult {
                                id: tool_call.id.clone(),
                                name: tool_name.clone(),
                                content: content.clone(),
                                success: false,
                            };
                            message_json.push(serde_json::json!({
                                "role": "tool",
                                "tool_call_id": tool_call.id,
                                "content": content
                            }));
                            canonical_message_json.push(
                                message_json
                                    .last()
                                    .cloned()
                                    .expect("invalid tool result was just appended"),
                            );
                            continue;
                        }
                    };
                    let output_policy = descriptor
                        .output_limit
                        .unwrap_or(orchestrator.tool_output_policy);

                    tracing::info!(
                        request_id = %request_id,
                        iteration = iteration,
                        tool_index = idx,
                        tool_id = %tool_call.id,
                        tool_name = %tool_name,
                        "Executing tool call"
                    );

                    // Check tool approval gate if configured
                    if !admitted_by_host && let Some(ref gate) = orchestrator.tool_approval_gate {
                        let result = gate(
                            tool_call.id.clone(),
                            tool_name.clone(),
                            descriptor.approval_class,
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
                            let rejection = format!("Tool call rejected: {reason}");
                            let source = Self::canonical_source_for(tool_name, descriptor);
                            let rejection_content = match orchestrator
                                .preserve_terminal_tool_failure(
                                    sequence,
                                    &tool_call.id,
                                    tool_name,
                                    source,
                                    rejection,
                                )
                                .await
                            {
                                Ok(content) => content,
                                Err(error) => {
                                    yield NormalizedEvent::Error {
                                        message: error.to_string(),
                                        code: Some(
                                            "TERMINAL_RESULT_PERSISTENCE_FAILED".to_string(),
                                        ),
                                    };
                                    return;
                                }
                            };
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
                            canonical_message_json.push(
                                message_json
                                    .last()
                                    .cloned()
                                    .expect("rejected tool result was just appended"),
                            );
                            continue;
                        }
                    }

                    // Determine sandbox routing
                    let sandbox_required = orchestrator.requires_sandbox(&descriptor);
                    let sandbox_attempt = sandbox_required.then(|| orchestrator.sandbox_runner.clone()).flatten()
                        .filter(|runner| runner.enforces_isolation());

                    let outcome: anyhow::Result<(String, String, bool)> = if sandbox_required && sandbox_attempt.is_none() {
                        let error = "Tool execution rejected: required sandbox isolation is unavailable";
                        orchestrator
                            .preserve_terminal_tool_failure(
                                sequence,
                                &tool_call.id,
                                tool_name,
                                crate::uar::persistence::agent_threads::CanonicalReceiptSource::Sandbox,
                                error.to_string(),
                            )
                            .await
                            .map(|content| (content.clone(), content, false))
                    } else if let Some(runner) = sandbox_attempt {
                        let request = match orchestrator.native_skills.get(tool_name).await {
                            Some(tool) => tool.sandbox_request(arguments.clone()),
                            None => Err(anyhow::anyhow!("MCP tool binding has no sandbox execution adapter")),
                        };
                        if let Ok(exec_req) = request {
                            let lang = exec_req.language.clone();
                            tracing::info!(
                                request_id = %request_id,
                                iteration = iteration,
                                tool_id = %tool_call.id,
                                tool_name = %tool_name,
                                language = ?lang,
                                "Executing tool in the bound sandbox"
                            );
                            let outcome = match &orchestrator.sandbox_scope {
                                Some(scope) => scope.execute(runner, exec_req).await,
                                None => Err(crate::sandbox::execution::SandboxExecutionError::Unavailable),
                            };
                            match outcome {
                                Ok(result) => {
                                    let value = serde_json::json!({
                                        "exit_code": result.exit_code,
                                        "stdout": result.stdout.clone(),
                                        "stderr": result.stderr.clone(),
                                    });
                                    orchestrator.preserve_canonical_tool_result(
                                        sequence,
                                        &tool_call.id,
                                        tool_name,
                                        crate::uar::persistence::agent_threads::CanonicalReceiptSource::Sandbox,
                                        value,
                                        Vec::new(),
                                        true,
                                        0,
                                    ).await.and_then(|stored| {
                                        let canonical = serde_json::to_string(&stored)?;
                                        if result.exit_code == 0 {
                                            Ok((canonical, format!("exit_code: {}\nstdout:\n{}\nstderr:\n{}",
                                                result.exit_code, result.stdout, result.stderr), true))
                                        } else {
                                            Ok((canonical, serde_json::json!({
                                                "status": "error",
                                                "tool_call_id": tool_call.id,
                                                "tool": tool_name,
                                                "provenance": {
                                                    "source": "sandbox",
                                                    "terminal_state": "failed",
                                                    "observed_by": "trusted_host"
                                                },
                                                "result": stored,
                                            }).to_string(), false))
                                        }
                                    })
                                }
                                Err(error) => orchestrator
                                    .preserve_terminal_tool_failure(
                                        sequence,
                                        &tool_call.id,
                                        tool_name,
                                        crate::uar::persistence::agent_threads::CanonicalReceiptSource::Sandbox,
                                        error.to_string(),
                                    )
                                    .await
                                    .map(|content| (content.clone(), content, false)),
                            }
                        } else {
                            let error = "Tool execution rejected: no sandbox adapter for this tool call";
                            orchestrator
                                .preserve_terminal_tool_failure(
                                    sequence,
                                    &tool_call.id,
                                    tool_name,
                                    crate::uar::persistence::agent_threads::CanonicalReceiptSource::Sandbox,
                                    error.to_string(),
                                )
                                .await
                                .map(|content| (content.clone(), content, false))
                        }
                    } else {
                        orchestrator
                            .execute_direct_tool(
                                sequence,
                                &tool_call.id,
                                tool_name,
                                &arguments,
                                output_policy,
                            )
                            .await
                    };
                    let (canonical_content, content, success) = match outcome {
                        Ok(outcome) => outcome,
                        Err(error) => {
                            yield NormalizedEvent::Error {
                                message: error.to_string(),
                                code: Some("TERMINAL_RESULT_PERSISTENCE_FAILED".to_string()),
                            };
                            return;
                        }
                    };
                    let content =
                        crate::uar::runtime::context::truncate::formatted_truncate_for_model(
                            &content,
                            output_policy,
                            &orchestrator.llm_config.model,
                        );

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
                    canonical_message_json.push(serde_json::json!({
                        "role": "tool",
                        "tool_call_id": tool_call.id,
                        "content": canonical_content
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
            budget_contract: self.request_budget_contract.clone(),
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

    struct LargeResultSkill;

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

    #[async_trait::async_trait]
    impl NativeSkill for LargeResultSkill {
        fn name(&self) -> &str {
            "large_result"
        }

        fn description(&self) -> &str {
            "Returns a large deterministic result"
        }

        fn parameters_schema(&self) -> serde_json::Value {
            serde_json::json!({"type": "object"})
        }

        async fn execute(&self, _: serde_json::Value) -> anyhow::Result<serde_json::Value> {
            Ok(serde_json::json!({"payload": "x".repeat(20_000)}))
        }
    }

    #[tokio::test]
    async fn direct_tool_preserves_canonical_value_before_display_truncation() {
        use crate::uar::domain::policy::{PolicyResolutionInput, resolve_run_policy};
        use crate::uar::persistence::PersistenceLayer;
        use crate::uar::persistence::providers::memory::InMemoryProvider;
        use crate::uar::runtime::context::truncate::TruncationPolicy;
        use crate::uar::runtime::turn::{ResolvedTurn, TurnEnvironment};

        let native_skills = Arc::new(NativeSkillRegistry::new());
        native_skills.register(LargeResultSkill).await.unwrap();
        let store: Arc<dyn PersistenceLayer> = Arc::new(InMemoryProvider::new());
        let turn = Arc::new(ResolvedTurn::new(
            crate::uar::defaults::default_agent(),
            resolve_run_policy(PolicyResolutionInput::default()),
            TurnEnvironment {
                run_id: "receipt-run".to_owned(),
                owner_id: "receipt-owner".to_owned(),
                session_id: "receipt-session".to_owned(),
            },
            LlmConfig::default(),
            Vec::new(),
        ));
        let orchestrator = Orchestrator::from_driver(
            LlmConfig::default(),
            Arc::new(McpRegistry::empty()),
            native_skills,
            Arc::new(MockLlmDriver::echo()),
        )
        .with_resolved_turn(turn)
        .with_canonical_receipt_store(Some(Arc::clone(&store)));

        let (canonical, displayed, success) = orchestrator
            .execute_direct_tool(
                1,
                "large-call",
                "large_result",
                &serde_json::json!({}),
                TruncationPolicy::Bytes(256),
            )
            .await
            .unwrap();
        assert!(success);
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&canonical).unwrap(),
            serde_json::json!({"payload": "x".repeat(20_000)})
        );
        assert!(displayed.len() <= 256);
        assert!(
            displayed.starts_with(crate::uar::runtime::context::truncate::WARNING_HEADER_PREFIX)
        );
        let receipts = store
            .list_canonical_tool_receipts("receipt-owner", "receipt-run")
            .await
            .unwrap();
        assert_eq!(receipts.len(), 1);
        assert_eq!(
            receipts[0].typed_value,
            Some(serde_json::json!({"payload": "x".repeat(20_000)}))
        );
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
        native_skills
            .register(RenderSkill)
            .await
            .expect("render descriptor registers");
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
        native_skills
            .register(RenderSkill)
            .await
            .expect("render descriptor registers");
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
            .await
            .expect("search descriptor registers");
        let gate: ToolApprovalGate =
            Arc::new(|_, _, _, _, _| Box::pin(async { ToolApprovalResult::GovernanceBypassed }));
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
