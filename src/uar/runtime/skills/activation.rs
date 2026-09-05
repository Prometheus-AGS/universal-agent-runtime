//! Host-owned, run-scoped skill activation and dependency admission.

use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::mcp::binding_cache::McpBindingEnvironment;
use crate::mcp::catalog::McpCatalog;
use crate::mcp::preflight::{McpPreflight, McpPreflightError};
use crate::mcp::projection::{McpProjectionScope, McpServerProjection};
use crate::mcp::registry::McpRegistry;
use crate::mcp::runtime::McpRuntimeManager;
use crate::uar::domain::policy::EffectiveRunPolicy;
use crate::uar::domain::skills::Skill;
use crate::uar::runtime::actor::messages::ActorOwner;
use crate::uar::tools::descriptor::ToolDescriptor;

use super::registry::SkillRegistry;

/// The source of an activation, independent of the candidate matcher.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InvokeType {
    Attachment,
    Model,
    Implicit,
}

impl InvokeType {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Attachment => "attachment",
            Self::Model => "model",
            Self::Implicit => "implicit",
        }
    }
}

/// A body admitted by the host after all checks succeed.
#[derive(Debug, Clone)]
pub struct ActivatedSkill {
    pub skill: Skill,
    pub invoke_type: InvokeType,
    pub sequence: u64,
}

impl ActivatedSkill {
    pub fn fragment(&self) -> crate::uar::runtime::prompt::PromptFragment {
        use crate::uar::runtime::prompt::{
            Authority, PromptFragment, PromptRole, PromptSection, Retention,
        };
        PromptFragment::new(
            format!("skill.{}", self.skill.skill_id),
            PromptSection::ActiveSkills,
            format!("skill:{}", self.skill.skill_id),
            Authority::Skill,
            PromptRole::System,
            Retention::Reclaimable,
            format!(
                "[SKILL: {}]\n{}",
                self.skill.title, self.skill.prompt_overlay
            ),
        )
    }
}

/// A failure never carries skill content or changes the active set.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, thiserror::Error)]
#[serde(tag = "code", rename_all = "snake_case")]
pub enum ActivationFailure {
    #[error("skill '{skill_id}' is missing")]
    Missing { skill_id: String },
    #[error("skill '{skill_id}' is outside the effective eligible set")]
    Ineligible { skill_id: String },
    #[error("skill '{skill_id}' is disabled in this scope")]
    Disabled { skill_id: String },
    #[error("skill '{skill_id}' has an invalid dependency: {reason}")]
    DependencyInvalid { skill_id: String, reason: String },
    #[error("skill '{skill_id}' cannot activate: max_active is {limit}")]
    LimitReached { skill_id: String, limit: u32 },
}

/// Root-host capture reused by every activation in a run. Child activations
/// instead retain their existing frozen binding view and never receive this host.
#[derive(Debug, Clone)]
pub struct ProjectedActivationHost {
    runtime: McpRuntimeManager,
    catalog: Arc<McpCatalog>,
    policy: EffectiveRunPolicy,
    owner: ActorOwner,
    environment: Arc<McpBindingEnvironment>,
    cancellation: tokio_util::sync::CancellationToken,
    events: Option<Arc<crate::mcp::run_events::McpRunEvents>>,
}

impl ProjectedActivationHost {
    /// Capture already-resolved authority, owner and launch inputs. No I/O.
    pub fn new(
        runtime: McpRuntimeManager,
        catalog: Arc<McpCatalog>,
        policy: EffectiveRunPolicy,
        owner: ActorOwner,
        environment: Arc<McpBindingEnvironment>,
        cancellation: tokio_util::sync::CancellationToken,
    ) -> Self {
        Self {
            runtime,
            catalog,
            policy,
            owner,
            environment,
            cancellation,
            events: None,
        }
    }

    pub(crate) fn with_events(
        mut self,
        run_id: String,
        sink: Arc<dyn crate::uar::domain::events::RuntimeEventSink>,
    ) -> Self {
        self.events = Some(Arc::new(crate::mcp::run_events::McpRunEvents::new(
            run_id,
            self.owner.clone(),
            sink,
        )));
        self
    }

    async fn prepare(
        &self,
        active_skills: BTreeSet<String>,
    ) -> Result<McpPreflight, McpPreflightError> {
        let projection = McpServerProjection::resolve(
            &self.catalog,
            &self.policy,
            &McpProjectionScope { active_skills },
        )?;
        if let Some(events) = &self.events {
            return self
                .runtime
                .preflight_observed(
                    &projection,
                    &self.owner,
                    &self.environment,
                    events,
                    &self.cancellation,
                )
                .await;
        }
        // Applies to both initial globals and later model/attachment activation.
        // Dropping preflight leaves cache refresh cancellation and partial
        // transport cleanup with their existing owned guards/supervisor.
        tokio::select! {
            biased;
            () = self.cancellation.cancelled() => Err(McpPreflightError::Cancelled),
            result = self.runtime.preflight(&projection, &self.owner, &self.environment) => result,
        }
    }
}

/// Immutable access envelope plus mutable run-local activation state.
///
/// The manager creates this context from the effective policy. Model callers
/// can supply an id only; they cannot replace its registry, scope, or allowlists.
pub struct ActivationContext {
    registry: Arc<RwLock<SkillRegistry>>,
    eligible: HashSet<String>,
    agent_id: String,
    conversation_id: String,
    max_active: u32,
    allowed_servers: Option<HashSet<String>>,
    native_descriptors: Vec<Arc<ToolDescriptor>>,
    mcp: McpRegistry,
    projected_host: Option<ProjectedActivationHost>,
    mcp_preflight: Option<Arc<McpPreflight>>,
    active: BTreeMap<String, ActivatedSkill>,
    sequence: u64,
    shadow_candidates: Option<(String, HashSet<String>)>,
}

impl std::fmt::Debug for ActivationContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ActivationContext")
            .field("eligible_count", &self.eligible.len())
            .finish_non_exhaustive()
    }
}

impl ActivationContext {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        registry: Arc<RwLock<SkillRegistry>>,
        eligible: HashSet<String>,
        agent_id: String,
        conversation_id: String,
        max_active: u32,
        mcp: McpRegistry,
        allowed_servers: Option<HashSet<String>>,
        allowed_tools: Option<HashSet<String>>,
        native_descriptors: Vec<Arc<ToolDescriptor>>,
    ) -> Self {
        let mcp = mcp.filtered(allowed_servers.as_ref(), allowed_tools.as_ref());
        Self {
            registry,
            eligible,
            agent_id,
            conversation_id,
            max_active,
            allowed_servers,
            native_descriptors,
            mcp,
            projected_host: None,
            mcp_preflight: None,
            active: BTreeMap::new(),
            sequence: 0,
            shadow_candidates: None,
        }
    }

    /// Build a root activation context whose dependencies use the captured
    /// catalog/cache. Initial global preflight completes before publication.
    ///
    /// # Errors
    /// Rejects a child/frozen view, failed preflight, or native/MCP collisions.
    #[allow(clippy::too_many_arguments)]
    pub async fn new_projected(
        registry: Arc<RwLock<SkillRegistry>>,
        agent_id: String,
        conversation_id: String,
        max_active: u32,
        mcp: McpRegistry,
        native_descriptors: Vec<Arc<ToolDescriptor>>,
        host: ProjectedActivationHost,
    ) -> anyhow::Result<Self> {
        anyhow::ensure!(
            !mcp.has_frozen_bindings(),
            "A child must retain its inherited MCP bindings"
        );
        let preflight = Arc::new(host.prepare(BTreeSet::new()).await?);
        let mut context = Self::new(
            registry,
            host.policy.skills.ids.iter().cloned().collect(),
            agent_id,
            conversation_id,
            max_active,
            mcp,
            Some(host.policy.mcp_servers.ids.iter().cloned().collect()),
            Some(host.policy.tools.ids.iter().cloned().collect()),
            native_descriptors,
        );
        context.check_projected_collisions(&preflight)?;
        context.projected_host = Some(host);
        context.mcp_preflight = Some(preflight);
        Ok(context)
    }

    /// Exact current prepared dependencies, committed atomically with activation.
    pub fn mcp_preflight(&self) -> Option<&Arc<McpPreflight>> {
        self.mcp_preflight.as_ref()
    }

    /// Descriptor snapshot paired with the current prepared execution set.
    /// Legacy transport descriptors cannot re-enter a projected run's tool list.
    pub fn mcp_descriptors(&self) -> Vec<Arc<ToolDescriptor>> {
        let mut descriptors = self.mcp.descriptors();
        if let Some(preflight) = &self.mcp_preflight {
            descriptors
                .retain(|tool| tool.source != crate::uar::tools::descriptor::ToolSource::Mcp);
            descriptors.extend(
                preflight
                    .projection()
                    .tools()
                    .values()
                    .map(|tool| Arc::clone(tool.descriptor())),
            );
        }
        descriptors
    }

    /// Capture the current dependency snapshot for a host-authorized child.
    /// Projected roots never fall back to legacy MCP connection recipes.
    /// Already-frozen children can only retain their inherited connections.
    ///
    /// # Errors
    /// Rejects unavailable, changed or revoked bindings without publishing a
    /// partial grant or mutating this context's activation state.
    pub async fn freeze_mcp_bindings(&self) -> anyhow::Result<McpRegistry> {
        match &self.mcp_preflight {
            Some(preflight) => preflight.freeze_bindings(&self.mcp).await,
            None => self.mcp.freeze_bindings().await,
        }
    }

    fn check_projected_collisions(&self, preflight: &McpPreflight) -> anyhow::Result<()> {
        let legacy_native = self
            .mcp
            .descriptors()
            .into_iter()
            .filter(|tool| tool.source != crate::uar::tools::descriptor::ToolSource::Mcp);
        let reserved = self
            .native_descriptors
            .iter()
            .cloned()
            .chain(legacy_native)
            .collect::<Vec<_>>();
        for tool in preflight.projection().tools().values() {
            let descriptor = tool.descriptor();
            if reserved.iter().any(|native| {
                native.provider_name == descriptor.provider_name
                    && !native.equivalent_to(descriptor)
            }) {
                anyhow::bail!(
                    "tool {:?} collides with a native descriptor",
                    descriptor.provider_name
                );
            }
        }
        Ok(())
    }

    /// Latest activation per skill, in activation order rather than map order.
    pub fn active(&self) -> Vec<ActivatedSkill> {
        let mut active = self.active.values().cloned().collect::<Vec<_>>();
        active.sort_by_key(|entry| entry.sequence);
        active
    }

    pub fn mcp(&self) -> &McpRegistry {
        &self.mcp
    }

    /// Complete the reserved native set before any activation is attempted.
    pub fn set_native_descriptors(&mut self, descriptors: Vec<Arc<ToolDescriptor>>) {
        self.native_descriptors = descriptors;
    }

    /// The reducer is observational only: it cannot mutate eligibility.
    pub fn set_shadow_candidates(
        &mut self,
        backend: String,
        ranked_ids: impl IntoIterator<Item = String>,
    ) {
        self.shadow_candidates = Some((backend, ranked_ids.into_iter().take(10).collect()));
    }

    /// Graph executions report their terminal outcome for every active skill.
    pub fn record_outcomes(&self, success: bool) {
        for skill_id in self.active.keys() {
            crate::uar::telemetry::metrics::record_skill_activation_outcome(skill_id, success);
        }
    }
}

/// Admit one skill without widening the run's effective policy.
///
/// Dependency startup, filtering, and collision checks precede body admission.
/// The caller serializes this operation so simultaneous activations cannot race
/// the `max_active` limit.
pub async fn activate(
    skill_id: &str,
    ctx: &mut ActivationContext,
    invoke_type: InvokeType,
) -> Result<ActivatedSkill, ActivationFailure> {
    let skill = ctx
        .registry
        .read()
        .await
        .get(skill_id)
        .cloned()
        .ok_or_else(|| ActivationFailure::Missing {
            skill_id: skill_id.to_string(),
        })?;
    if !ctx.eligible.contains(skill_id) {
        return Err(ActivationFailure::Ineligible {
            skill_id: skill_id.to_string(),
        });
    }
    if !skill.enabled_for(Some(&ctx.agent_id), Some(&ctx.conversation_id)) {
        return Err(ActivationFailure::Disabled {
            skill_id: skill_id.to_string(),
        });
    }
    if !ctx.active.contains_key(skill_id) && ctx.active.len() >= ctx.max_active as usize {
        return Err(ActivationFailure::LimitReached {
            skill_id: skill_id.to_string(),
            limit: ctx.max_active,
        });
    }

    let dependency_error = |reason| ActivationFailure::DependencyInvalid {
        skill_id: skill_id.to_string(),
        reason,
    };
    let mcp = ctx.mcp.clone();
    let mut prepared = ctx.mcp_preflight.clone();
    if let Some(host) = &ctx.projected_host {
        if let Some(config) = &skill.mcp_config {
            for server in config.mcp_servers.keys() {
                if !host.policy.mcp_servers.ids.contains(server) {
                    return Err(dependency_error(format!(
                        "MCP server '{server}' is not eligible"
                    )));
                }
            }
        }
        let active_skills = ctx
            .active
            .keys()
            .cloned()
            .chain(std::iter::once(skill_id.to_owned()))
            .collect();
        let next = host
            .prepare(active_skills)
            .await
            .map_err(|error| dependency_error(error.to_string()))?;
        if let Some(config) = &skill.mcp_config {
            for server in config.mcp_servers.keys() {
                if !next.projection().servers().contains_key(server)
                    && !next
                        .warnings()
                        .iter()
                        .any(|warning| warning.server() == server)
                {
                    return Err(dependency_error(format!(
                        "MCP server '{server}' is absent from the captured catalog"
                    )));
                }
            }
        }
        ctx.check_projected_collisions(&next)
            .map_err(|error| dependency_error(error.to_string()))?;
        prepared = Some(Arc::new(next));
    } else {
        if mcp.has_frozen_bindings() {
            mcp.require_bound_servers(std::iter::empty::<&str>())
                .map_err(|_| dependency_error("Inherited MCP view is closed".to_string()))?;
        }
        if let Some(config) = &skill.mcp_config {
            for server in config.mcp_servers.keys() {
                if ctx
                    .allowed_servers
                    .as_ref()
                    .is_some_and(|allowed| !allowed.contains(server))
                {
                    return Err(dependency_error(format!(
                        "MCP server '{server}' is not eligible"
                    )));
                }
            }
            if mcp.has_frozen_bindings() {
                // Child artifacts declare required identities, not new connection
                // recipes. Never read their URL/auth/env or spawn their commands.
                mcp.require_bound_servers(config.mcp_servers.keys().map(String::as_str))
                    .map_err(|error| {
                        tracing::warn!(skill_id, %error, "Inherited skill dependency unavailable");
                        dependency_error("Inherited MCP binding is unavailable".to_string())
                    })?;
            } else if !ctx.active.contains_key(skill_id) {
                return Err(dependency_error(
                    "Root MCP dependencies require a verified host binding".to_string(),
                ));
            }
        }
    }
    for descriptor in mcp.descriptors().into_iter().filter(|tool| {
        prepared.is_none() || tool.source != crate::uar::tools::descriptor::ToolSource::Mcp
    }) {
        if ctx.native_descriptors.iter().any(|native| {
            native.provider_name == descriptor.provider_name && !native.equivalent_to(&descriptor)
        }) {
            return Err(dependency_error(format!(
                "tool '{}' collides with a native descriptor",
                descriptor.provider_name,
            )));
        }
    }

    ctx.sequence = ctx.sequence.saturating_add(1);
    let activated = ActivatedSkill {
        skill,
        invoke_type,
        sequence: ctx.sequence,
    };
    ctx.mcp = mcp;
    ctx.mcp_preflight = prepared;
    ctx.active.insert(skill_id.to_string(), activated.clone());
    crate::uar::telemetry::metrics::record_skill_invocation(skill_id, invoke_type.as_str());
    if matches!(invoke_type, InvokeType::Attachment | InvokeType::Model)
        && let Some((backend, top_ten)) = &ctx.shadow_candidates
    {
        let hit = top_ten.contains(skill_id);
        crate::uar::telemetry::metrics::record_skill_shadow_recall(backend, hit);
        tracing::debug!(
            skill_id,
            backend,
            hit,
            "Shadow skill reducer Recall@10 sample"
        );
    }
    Ok(activated)
}
