use std::collections::BTreeMap;
use std::sync::Arc;

use crate::config::LlmConfig;
use crate::llm::LlmRequest;
use crate::mcp::registry::McpRegistry;
use crate::uar::domain::{artifact::AgentArtifact, policy::EffectiveRunPolicy};
use crate::uar::runtime::prompt::{PromptBudgets, PromptFragment};
use crate::uar::tools::descriptor::{Exposure, ToolDescriptor};

/// Only the execution identity, never the process environment or secrets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TurnEnvironment {
    pub run_id: String,
    pub owner_id: String,
    pub session_id: String,
}

/// Frozen turn inputs. No serialization or derived Debug can expose credentials.
#[derive(Clone)]
pub struct ResolvedTurn {
    artifact: AgentArtifact,
    policy: EffectiveRunPolicy,
    environment: TurnEnvironment,
    verified_owner: Option<crate::uar::runtime::actor::messages::ActorOwner>,
    presentations: Option<Arc<super::super::presentations::RunPresentationSnapshot>>,
    credentials: LlmConfig,
    fragments: Vec<PromptFragment>,
    shadow: Option<super::shadow::ShadowReport>,
}

impl ResolvedTurn {
    pub fn new(
        artifact: AgentArtifact,
        policy: EffectiveRunPolicy,
        environment: TurnEnvironment,
        credentials: LlmConfig,
        fragments: Vec<PromptFragment>,
    ) -> Self {
        Self {
            artifact,
            policy,
            environment,
            verified_owner: None,
            presentations: None,
            credentials,
            fragments,
            shadow: None,
        }
    }

    /// Preserve the ingress identity after the manager's admission check.
    /// This identity is host-only and never enters a serialized turn manifest.
    pub(crate) fn with_verified_owner(
        mut self,
        owner: Option<crate::uar::runtime::actor::messages::ActorOwner>,
    ) -> Self {
        self.verified_owner = owner;
        self
    }

    /// Exact host principal, including verified tenant, for resource binding.
    pub fn verified_owner(&self) -> Option<&crate::uar::runtime::actor::messages::ActorOwner> {
        self.verified_owner.as_ref()
    }

    pub(crate) fn with_presentations(
        mut self,
        snapshot: Arc<super::super::presentations::RunPresentationSnapshot>,
    ) -> Self {
        self.presentations = Some(snapshot);
        self
    }

    pub(crate) fn presentations(
        &self,
    ) -> Option<&Arc<super::super::presentations::RunPresentationSnapshot>> {
        self.presentations.as_ref()
    }

    pub fn artifact(&self) -> &AgentArtifact {
        &self.artifact
    }
    pub fn policy(&self) -> &EffectiveRunPolicy {
        &self.policy
    }
    pub fn environment(&self) -> &TurnEnvironment {
        &self.environment
    }
    pub fn credentials(&self) -> &LlmConfig {
        &self.credentials
    }
    pub fn fragments(&self) -> &[PromptFragment] {
        &self.fragments
    }
}

impl std::fmt::Debug for ResolvedTurn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResolvedTurn")
            .field("run_id", &self.environment.run_id)
            .field("agent_id", &self.artifact.id)
            .field("fragment_count", &self.fragments.len())
            .finish_non_exhaustive()
    }
}

/// The advertised descriptors, executable registry, settings and request all
/// belong to the same model-call snapshot. Later activations cannot mutate it.
#[derive(Clone)]
pub struct ResolvedStep {
    turn: Arc<ResolvedTurn>,
    index: u32,
    request: LlmRequest,
    tools: BTreeMap<String, Arc<ToolDescriptor>>,
    mcp: Arc<McpRegistry>,
    mcp_preflight: Option<Arc<crate::mcp::preflight::McpPreflight>>,
    mcp_catalog: Vec<String>,
    active_skills: Vec<String>,
    budgets: PromptBudgets,
    fragments: Vec<PromptFragment>,
}

impl std::fmt::Debug for ResolvedStep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResolvedStep")
            .field("turn", &self.turn)
            .field("index", &self.index)
            .field("tool_count", &self.tools.len())
            .finish_non_exhaustive()
    }
}

impl ResolvedStep {
    /// Freeze the host's already exposure-projected callable descriptors.
    /// Deferred declarations may be visible after host discovery; preserve their
    /// original metadata so binding/schema comparisons remain exact.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        turn: Arc<ResolvedTurn>,
        index: u32,
        request: LlmRequest,
        tools: BTreeMap<String, Arc<ToolDescriptor>>,
        mcp: Arc<McpRegistry>,
        active_skills: Vec<String>,
        budgets: PromptBudgets,
        fragments: Vec<PromptFragment>,
    ) -> Result<Self, super::contributors::AssemblyError> {
        super::contributors::validate_projection(turn.policy(), &tools, &tools, &active_skills)?;
        let tools = tools
            .into_iter()
            .filter(|(_, descriptor)| descriptor.exposure != Exposure::Hidden)
            .collect::<BTreeMap<_, _>>();
        let mut request = request;
        request.tools = tools
            .values()
            .map(|descriptor| descriptor.openai_tool_json())
            .collect();
        let mut mcp_catalog = mcp.server_names();
        mcp_catalog.sort();
        Ok(Self {
            turn,
            index,
            request,
            tools,
            mcp,
            mcp_preflight: None,
            mcp_catalog,
            active_skills,
            budgets,
            fragments,
        })
    }

    /// Retain exact generation-pinned execution bindings with this step.
    ///
    /// # Errors
    /// Rejects a missing/foreign verified owner or an advertised MCP descriptor
    /// missing or changed in preflight. Owner equality includes tenant identity.
    pub fn with_mcp_preflight(
        mut self,
        preflight: Arc<crate::mcp::preflight::McpPreflight>,
    ) -> Result<Self, super::contributors::AssemblyError> {
        if self.turn.verified_owner() != Some(preflight.owner()) {
            return Err(super::contributors::AssemblyError::McpOwnerMismatch);
        }
        for (name, descriptor) in &self.tools {
            if descriptor.source == crate::uar::tools::descriptor::ToolSource::Mcp
                && !preflight
                    .projection()
                    .tools()
                    .get(name)
                    .is_some_and(|tool| tool.descriptor().equivalent_to(descriptor))
            {
                return Err(super::contributors::AssemblyError::ToolChanged { name: name.clone() });
            }
        }
        self.mcp_catalog = preflight.projection().servers().keys().cloned().collect();
        self.mcp_preflight = Some(preflight);
        Ok(self)
    }

    /// Exact prepared execution set, when the host uses catalog-based MCP.
    pub fn mcp_preflight(&self) -> Option<&Arc<crate::mcp::preflight::McpPreflight>> {
        self.mcp_preflight.as_ref()
    }

    pub fn turn(&self) -> &ResolvedTurn {
        &self.turn
    }
    pub const fn index(&self) -> u32 {
        self.index
    }
    pub fn request(&self) -> &LlmRequest {
        &self.request
    }
    pub fn tools(&self) -> &BTreeMap<String, Arc<ToolDescriptor>> {
        &self.tools
    }
    pub fn mcp(&self) -> &Arc<McpRegistry> {
        &self.mcp
    }
    pub fn mcp_catalog(&self) -> &[String] {
        &self.mcp_catalog
    }
    pub fn active_skills(&self) -> &[String] {
        &self.active_skills
    }
    pub fn budgets(&self) -> &PromptBudgets {
        &self.budgets
    }
    pub fn fragments(&self) -> &[PromptFragment] {
        &self.fragments
    }

    pub fn manifest(&self) -> crate::uar::runtime::prompt::TurnManifest {
        let manifest = crate::uar::runtime::prompt::TurnManifest::from_fragments(
            &self.fragments,
            self.budgets.clone(),
            self.active_skills.clone(),
            self.tools.keys().cloned(),
            self.turn.policy().warnings.clone(),
        );
        match &self.turn.shadow {
            Some(report) => manifest.with_shadow(report.clone()),
            None => manifest,
        }
    }

    /// Attach comparison metadata to a new snapshot, leaving earlier steps intact.
    pub fn with_shadow(mut self, report: super::shadow::ShadowReport) -> Self {
        let mut turn = (*self.turn).clone();
        turn.shadow = Some(report);
        self.turn = Arc::new(turn);
        self
    }
}
