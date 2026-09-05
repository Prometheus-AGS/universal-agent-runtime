//! Native Skill Trait System.
//!
//! Provides a mechanism for embedding high-performance Rust tool implementations
//! directly into the runtime binary, bypassing MCP serialization overhead for
//! critical hot-path operations.
//!
//! # Architecture
//!
//! Native skills implement the [`NativeSkill`] trait and are registered in the
//! [`NativeSkillRegistry`]. When the orchestrator receives a tool call, it first
//! checks whether a native skill is registered for that tool name. If found, the
//! native skill is executed directly; otherwise, the call falls through to MCP.
//!
//! # Example
//!
//! ```rust,ignore
//! use universal_agent_runtime::uar::runtime::native_skill::{NativeSkill, NativeSkillRegistry};
//!
//! struct MyTool;
//!
//! #[async_trait::async_trait]
//! impl NativeSkill for MyTool {
//!     fn name(&self) -> &str { "my_tool" }
//!     fn description(&self) -> &str { "Does something useful" }
//!     fn parameters_schema(&self) -> serde_json::Value {
//!         serde_json::json!({ "type": "object", "properties": {} })
//!     }
//!     async fn execute(&self, _args: serde_json::Value) -> anyhow::Result<serde_json::Value> {
//!         Ok(serde_json::json!({ "result": "done" }))
//!     }
//! }
//!
//! let registry = NativeSkillRegistry::new();
//! registry.register(MyTool).await?;
//! ```

use std::collections::{BTreeMap, HashSet};
use std::sync::Arc;

use tokio::sync::RwLock;

use crate::uar::runtime::context::truncate::TruncationPolicy;
use crate::uar::tools::descriptor::{
    ApprovalClass, Exposure, ToolAssemblyError, ToolCollision, ToolDescriptor, ToolEffect,
    ToolSource,
};
use crate::uar::tools::validate::ValidatorCompiler;

/// Trait for embedding high-performance Rust tool implementations directly
/// into the runtime binary.
///
/// Native skills bypass MCP serialization and execute in-process, making them
/// ideal for latency-sensitive operations like system introspection, caching
/// lookups, or internal state queries.
#[async_trait::async_trait]
pub trait NativeSkill: Send + Sync {
    /// Unique tool name. Must be globally unique across both native and MCP tools.
    fn name(&self) -> &str;

    /// Human-readable description of what the tool does.
    fn description(&self) -> &str;

    /// JSON Schema describing the expected input parameters.
    fn parameters_schema(&self) -> serde_json::Value;

    /// Declared effect used for scheduling. Undeclared legacy skills fail
    /// closed to [`ToolEffect::Unknown`].
    fn effect(&self) -> ToolEffect {
        ToolEffect::Unknown
    }

    /// Descriptor-level approval classification.
    fn approval_class(&self) -> ApprovalClass {
        match self.effect() {
            ToolEffect::ReadOnly => ApprovalClass::NotRequired,
            ToolEffect::ExternalMutation | ToolEffect::CodeExecution | ToolEffect::Unknown => {
                ApprovalClass::Required
            }
        }
    }

    /// Whether this skill must execute in a sandbox.
    fn sandbox_required(&self) -> bool {
        false
    }

    /// Whether this implementation has an adapter for the host's sandbox
    /// protocol. A code-execution effect is not evidence of an adapter.
    fn supports_sandbox_execution(&self) -> bool {
        false
    }

    /// Admit direct execution under a captured child policy. Implementations
    /// must enforce any authority they consume, not merely declare ReadOnly.
    ///
    /// # Errors
    /// Unported tools cannot silently inherit an unrestricted host execution
    /// path. Sandbox execution uses the host's separate physical binding.
    fn check_thread_policy(
        &self,
        _policy: &super::thread::policy_intersection::ThreadPolicy,
    ) -> anyhow::Result<()> {
        anyhow::bail!("Native tool has no delegated permission enforcement")
    }

    /// Translate validated arguments into this tool's actual sandbox operation.
    /// The host must not guess executable code or language from field/tool names.
    ///
    /// # Errors
    /// The default rejects tools without an explicit sandbox adapter. Implementors
    /// must preserve argument semantics and reject unsupported execution modes.
    fn sandbox_request(
        &self,
        _args: serde_json::Value,
    ) -> anyhow::Result<crate::sandbox::ExecutionRequest> {
        anyhow::bail!("Tool has no sandbox execution adapter")
    }

    /// Optional key used to serialize conflicting read-only operations.
    fn concurrency_key(&self) -> Option<&str> {
        None
    }

    /// Model and host exposure class.
    fn exposure(&self) -> Exposure {
        Exposure::Eager
    }

    /// Descriptor-specific model-visible output bound.
    fn output_limit(&self) -> Option<TruncationPolicy> {
        None
    }

    /// Registration source. Built-in runtime tools override this value.
    fn source(&self) -> ToolSource {
        ToolSource::NativeSkill
    }

    /// Execute the tool with the given arguments and return the result.
    async fn execute(&self, args: serde_json::Value) -> anyhow::Result<serde_json::Value>;

    /// Execute with host context only after schema validation and governance.
    /// Implementations without contextual behavior retain their existing path.
    async fn execute_with_context(
        &self,
        args: serde_json::Value,
        _context: &NativeExecutionContext,
    ) -> anyhow::Result<serde_json::Value> {
        self.execute(args).await
    }

    /// Declare structured outputs from a successful result, before truncation.
    /// The trusted host owns publication; ordinary tools produce no artifacts.
    fn result_artifacts(
        &self,
        _result: &serde_json::Value,
    ) -> anyhow::Result<Vec<super::thread::artifacts::ToolOutputArtifact>> {
        Ok(Vec::new())
    }

    /// Format one successful result for model-visible history.
    ///
    /// The default preserves the existing JSON representation and applies the
    /// run's output policy at the single history-ingest boundary. Native tools
    /// whose raw output needs format-aware truncation may override this method.
    fn format_result(
        &self,
        result: &serde_json::Value,
        policy: crate::uar::runtime::context::truncate::TruncationPolicy,
        model: &str,
    ) -> String {
        let content = serde_json::to_string(result).unwrap_or_default();
        crate::uar::runtime::context::truncate::formatted_truncate_for_model(
            &content, policy, model,
        )
    }
}

/// Opaque per-call Presentation capability. Only the trusted run host can
/// construct it; public context struct-update construction remains supported.
#[derive(Clone, Debug)]
pub struct PresentationExecutionContext {
    snapshot: Arc<super::presentations::RunPresentationSnapshot>,
    call_id: String,
}

impl PresentationExecutionContext {
    pub(crate) fn new(
        snapshot: Arc<super::presentations::RunPresentationSnapshot>,
        call_id: &str,
    ) -> Self {
        Self {
            snapshot,
            call_id: call_id.to_owned(),
        }
    }

    pub(crate) fn snapshot(&self) -> &super::presentations::RunPresentationSnapshot {
        &self.snapshot
    }
}

/// Non-model-owned capability supplied by the governed host tool loop.
#[derive(Clone, Default)]
pub struct NativeExecutionContext {
    /// Private host capture. No tool argument can install another snapshot.
    pub presentations: Option<PresentationExecutionContext>,
    /// Principal retained by the admitted turn, never taken from tool arguments.
    pub verified_owner: Option<super::actor::messages::ActorOwner>,
    /// Host-resolved conversation (the thread ID for delegated turns). Tool
    /// arguments may select records inside it, never a replacement namespace.
    pub session_id: Option<String>,
    /// Present only for a host-admitted delegated turn.
    pub thread_policy: Option<Arc<super::thread::policy_intersection::ThreadPolicy>>,
    /// Run-owned lifetime scope for host terminal calls, not an execution grant.
    pub terminal_scope: Option<crate::uar::tools::terminal_process::TerminalRun>,
    /// Exact actor-turn output receipt, never supplied by model arguments.
    pub artifact_collector: Option<super::thread::artifacts::RunArtifactCollector>,
    pub project_instructions:
        Option<Arc<tokio::sync::Mutex<super::project_instructions::ProjectInstructions>>>,
}

impl std::fmt::Debug for NativeExecutionContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NativeExecutionContext")
            .field("has_verified_owner", &self.verified_owner.is_some())
            .field("has_session", &self.session_id.is_some())
            .field("has_thread_policy", &self.thread_policy.is_some())
            .field("has_terminal_scope", &self.terminal_scope.is_some())
            .field("has_artifact_collector", &self.artifact_collector.is_some())
            .field(
                "has_project_instructions",
                &self.project_instructions.is_some(),
            )
            .finish_non_exhaustive()
    }
}

/// Trusted direct-call boundary shared by sequential and parallel tool dispatch.
/// A tool's overridable contextual method cannot bypass delegated admission.
///
/// # Errors
/// Rejects a missing/foreign owner or unenforceable child policy before I/O.
pub(crate) async fn execute_native(
    skill: &dyn NativeSkill,
    args: serde_json::Value,
    context: &NativeExecutionContext,
) -> anyhow::Result<serde_json::Value> {
    if let Some(policy) = &context.thread_policy {
        anyhow::ensure!(
            context
                .verified_owner
                .as_ref()
                .is_some_and(|owner| owner.user_id() == policy.owner_id()),
            "Delegated native tool has no matching verified owner"
        );
        skill.check_thread_policy(policy)?;
    }
    let mut result = match skill.execute_with_context(args, context).await {
        Ok(result) => result,
        Err(error) => {
            if matches!(skill.name(), "a2ui_render" | "presentation_render")
                && let Some(presentations) = &context.presentations
            {
                presentations.snapshot.record_generation_failure();
            }
            return Err(error);
        }
    };
    if let Some(collector) = &context.artifact_collector {
        let owner = context
            .verified_owner
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Artifact output has no verified owner"))?;
        collector.publish(owner, skill.result_artifacts(&result)?)?;
    }
    if matches!(skill.name(), "a2ui_render" | "presentation_render")
        && let Some(presentations) = &context.presentations
    {
        presentations.snapshot.retain_preparation(
            &presentations.call_id,
            skill.name(),
            result.clone(),
        )?;
        // The full validated surface stays on the host. Model history and
        // ToolEnd carry a compact preparation receipt, never publication data.
        if let Some(fields) = result.as_object_mut() {
            fields.remove("a2uiMessages");
        }
    }
    Ok(result)
}

/// Registry holding all registered native skills, keyed by their name.
///
/// Thread-safe for concurrent reads and writes via [`tokio::sync::RwLock`].
pub struct NativeSkillRegistry {
    skills: RwLock<BTreeMap<String, RegisteredNativeSkill>>,
    validator_compiler: Arc<ValidatorCompiler>,
}

#[derive(Clone)]
struct RegisteredNativeSkill {
    implementation: Arc<dyn NativeSkill>,
    descriptor: Arc<ToolDescriptor>,
}

impl std::fmt::Debug for NativeSkillRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NativeSkillRegistry")
            .field("skills", &"<locked>")
            .finish()
    }
}

impl Default for NativeSkillRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl NativeSkillRegistry {
    /// Convert an internal native-skill identifier into the function-name
    /// alphabet accepted by OpenAI-compatible providers.
    fn provider_tool_name(name: &str) -> String {
        name.chars()
            .map(|character| {
                if character.is_ascii_alphanumeric() || matches!(character, '_' | '-') {
                    character
                } else {
                    '_'
                }
            })
            .collect()
    }

    /// Create an empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self::with_validator_compiler(Arc::new(ValidatorCompiler::default()))
    }

    /// Create an empty registry using an observable schema compiler.
    #[must_use]
    pub fn with_validator_compiler(validator_compiler: Arc<ValidatorCompiler>) -> Self {
        Self {
            skills: RwLock::new(BTreeMap::new()),
            validator_compiler,
        }
    }

    /// Register a native skill after compiling its descriptor.
    ///
    /// # Errors
    ///
    /// Returns an assembly error for an invalid schema or a conflicting
    /// provider-visible name. An identical descriptor is deduplicated.
    pub async fn register<S: NativeSkill + 'static>(
        &self,
        skill: S,
    ) -> Result<(), ToolAssemblyError> {
        self.register_arc(Arc::new(skill)).await
    }

    /// Register a native skill from an [`Arc`] after compiling its descriptor.
    ///
    /// # Errors
    ///
    /// Returns an assembly error for an invalid schema or a conflicting
    /// provider-visible name. An identical descriptor is deduplicated.
    pub async fn register_arc(&self, skill: Arc<dyn NativeSkill>) -> Result<(), ToolAssemblyError> {
        let id = skill.name().to_owned();
        let provider_name = Self::provider_tool_name(&id);
        let input_schema = skill.parameters_schema();
        let validator = self
            .validator_compiler
            .compile(&provider_name, &input_schema)?;
        let descriptor = Arc::new(ToolDescriptor {
            id,
            provider_name: provider_name.clone(),
            description: skill.description().to_owned(),
            source: skill.source(),
            server: None,
            input_schema,
            validator,
            effect: skill.effect(),
            approval_class: skill.approval_class(),
            sandbox_required: skill.sandbox_required(),
            concurrency_key: skill.concurrency_key().map(str::to_owned),
            exposure: skill.exposure(),
            output_limit: skill.output_limit(),
        });

        let mut skills = self.skills.write().await;
        if let Some(existing) = skills.get(&provider_name) {
            if existing.descriptor.equivalent_to(&descriptor) {
                return Ok(());
            }
            return Err(ToolCollision { provider_name }.into());
        }
        tracing::info!(skill_name = %descriptor.id, "Registering native skill");
        skills.insert(
            provider_name,
            RegisteredNativeSkill {
                implementation: skill,
                descriptor,
            },
        );
        Ok(())
    }

    /// Look up a native skill by name.
    pub async fn get(&self, name: &str) -> Option<Arc<dyn NativeSkill>> {
        let skills = self.skills.read().await;
        skills
            .get(name)
            .map(|registered| Arc::clone(&registered.implementation))
            .or_else(|| {
                skills
                    .values()
                    .find(|registered| registered.descriptor.id == name)
                    .map(|registered| Arc::clone(&registered.implementation))
            })
    }

    /// Check whether a native skill is registered for the given name.
    pub async fn contains(&self, name: &str) -> bool {
        let skills = self.skills.read().await;
        skills.contains_key(name)
            || skills
                .values()
                .any(|registered| registered.descriptor.id == name)
    }

    /// Return a snapshot of all registered skill names.
    pub async fn names(&self) -> Vec<String> {
        self.skills
            .read()
            .await
            .values()
            .map(|registered| registered.descriptor.id.clone())
            .collect()
    }

    /// Return the number of registered native skills.
    pub async fn len(&self) -> usize {
        self.skills.read().await.len()
    }

    /// Return whether the registry is empty.
    pub async fn is_empty(&self) -> bool {
        self.skills.read().await.is_empty()
    }

    /// Clone a policy-filtered registry snapshot.
    ///
    /// `None` preserves every registered native skill. `Some` keeps only the
    /// explicitly eligible names, matching the MCP registry's filtering
    /// contract so scoped run policies govern both tool backends identically.
    pub async fn filtered(&self, allowed: Option<&HashSet<String>>) -> Self {
        let skills = self.skills.read().await;
        let filtered = skills
            .iter()
            .filter(|(provider_name, registered)| {
                allowed.is_none_or(|names| {
                    names.contains(*provider_name) || names.contains(&registered.descriptor.id)
                })
            })
            .map(|(name, registered)| (name.clone(), registered.clone()))
            .collect();
        Self {
            skills: RwLock::new(filtered),
            validator_compiler: Arc::clone(&self.validator_compiler),
        }
    }

    /// Return the compiled descriptors in provider-name order.
    pub async fn descriptors(&self) -> Vec<Arc<ToolDescriptor>> {
        self.skills
            .read()
            .await
            .values()
            .map(|registered| Arc::clone(&registered.descriptor))
            .collect()
    }

    /// Look up one compiled descriptor by source-local or provider-visible name.
    pub async fn descriptor(&self, name: &str) -> Option<Arc<ToolDescriptor>> {
        let skills = self.skills.read().await;
        skills
            .get(name)
            .map(|registered| Arc::clone(&registered.descriptor))
            .or_else(|| {
                skills
                    .values()
                    .find(|registered| registered.descriptor.id == name)
                    .map(|registered| Arc::clone(&registered.descriptor))
            })
    }

    /// Generate OpenAI-compatible tool definitions for all registered native skills.
    ///
    /// This allows native skills to be announced to the LLM alongside MCP tools.
    pub async fn openai_tools_json(&self) -> Vec<serde_json::Value> {
        let skills = self.skills.read().await;
        skills
            .values()
            .filter(|registered| registered.descriptor.exposure == Exposure::Eager)
            .map(|registered| registered.descriptor.openai_tool_json())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;

    struct EchoSkill;

    #[async_trait::async_trait]
    impl NativeSkill for EchoSkill {
        fn name(&self) -> &str {
            "echo"
        }
        fn description(&self) -> &str {
            "Returns the input unchanged"
        }
        fn parameters_schema(&self) -> serde_json::Value {
            serde_json::json!({
                "type": "object",
                "properties": {
                    "message": { "type": "string" }
                }
            })
        }
        async fn execute(&self, args: serde_json::Value) -> anyhow::Result<serde_json::Value> {
            Ok(args)
        }
    }

    #[tokio::test]
    async fn test_register_and_get() {
        let registry = NativeSkillRegistry::new();
        registry.register(EchoSkill).await.unwrap();

        assert!(registry.contains("echo").await);
        assert!(!registry.contains("nonexistent").await);
        assert_eq!(registry.len().await, 1);

        let skill = registry.get("echo").await.unwrap();
        assert_eq!(skill.name(), "echo");

        let result = skill
            .execute(serde_json::json!({"message": "hello"}))
            .await
            .unwrap();
        assert_eq!(result, serde_json::json!({"message": "hello"}));
    }

    #[tokio::test]
    async fn test_names() {
        let registry = NativeSkillRegistry::new();
        registry.register(EchoSkill).await.unwrap();

        let names = registry.names().await;
        assert_eq!(names, vec!["echo".to_string()]);
    }

    #[tokio::test]
    async fn test_openai_tools_json() {
        let registry = NativeSkillRegistry::new();
        registry.register(EchoSkill).await.unwrap();

        let tools = registry.openai_tools_json().await;
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0]["type"], "function");
        assert_eq!(tools[0]["function"]["name"], "echo");
    }
}
