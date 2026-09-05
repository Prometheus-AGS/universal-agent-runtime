//! Fixed-stage, owned-data contributors. Cedar and execution stay in the host.

use std::collections::BTreeMap;
use std::sync::Arc;

use crate::llm::Message;
use crate::uar::domain::{
    artifact::AgentArtifact,
    events::MemoryItem,
    policy::{EffectiveRunPolicy, SelectionMode},
};
use crate::uar::runtime::prompt::{PromptBudgets, PromptFragment};
use crate::uar::tools::descriptor::{Exposure, ToolDescriptor, ToolSource};

#[derive(Debug, Clone, thiserror::Error)]
pub enum AssemblyError {
    #[error("contributor attempted to broaden {resource}: '{id}'")]
    OutsidePolicy { resource: String, id: String },
    #[error("contributor altered a host-authorized tool descriptor: '{name}'")]
    ToolChanged { name: String },
    #[error("prepared MCP bindings do not belong to the turn's verified owner")]
    McpOwnerMismatch,
    #[error("contributor attempted to broaden or replace immutable policy settings")]
    PolicyWidening,
    #[error("conflicting prompt fragment id '{id}'")]
    FragmentCollision { id: String },
    #[error("contributor '{name}' failed: {message}")]
    ContributorFailed { name: String, message: String },
}

/// A read-only view of host-prepared resources. Deliberately has no credentials,
/// native implementations, MCP clients, governance gates, or mutation handles.
pub struct AssemblyInputs {
    pub artifact: AgentArtifact,
    pub policy: EffectiveRunPolicy,
    pub memory_hits: Vec<MemoryItem>,
    pub prepared_fragments: Vec<PromptFragment>,
    pub history: Vec<Message>,
    /// Shadow may reuse the legacy reduction result, avoiding a second paid
    /// summarization call while independently rendering typed prompt sections.
    pub prepared_history: Option<Vec<Message>>,
    pub authorized_tools: BTreeMap<String, Arc<ToolDescriptor>>,
    pub active_skills: Vec<String>,
    pub budgets: PromptBudgets,
}

impl std::fmt::Debug for AssemblyInputs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AssemblyInputs")
            .field("artifact_id", &self.artifact.id)
            .field("history_count", &self.history.len())
            .finish_non_exhaustive()
    }
}

#[derive(Clone)]
pub struct AssemblyState {
    pub policy: EffectiveRunPolicy,
    pub fragments: Vec<PromptFragment>,
    pub history: Vec<Message>,
    pub tools: BTreeMap<String, Arc<ToolDescriptor>>,
    pub active_skills: Vec<String>,
    pub budgets: PromptBudgets,
    pub reduce_report: Option<crate::uar::runtime::context::reduce::ReduceReport>,
    pub world_state: Option<crate::uar::runtime::world_state::contributor::WorldStateUpdate>,
}

impl std::fmt::Debug for AssemblyState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AssemblyState")
            .field("history_count", &self.history.len())
            .field("fragment_count", &self.fragments.len())
            .finish_non_exhaustive()
    }
}

#[derive(Default)]
pub struct Contribution {
    pub fragments: Vec<PromptFragment>,
    pub tools: Vec<Arc<ToolDescriptor>>,
    pub active_skills: Vec<String>,
    pub narrowed_policy: Option<EffectiveRunPolicy>,
    pub history: Option<Vec<Message>>,
    pub budgets: Option<PromptBudgets>,
    pub reduce_report: Option<crate::uar::runtime::context::reduce::ReduceReport>,
    pub world_state: Option<crate::uar::runtime::world_state::contributor::WorldStateUpdate>,
}

impl std::fmt::Debug for Contribution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Contribution")
            .field("fragment_count", &self.fragments.len())
            .field("tool_count", &self.tools.len())
            .finish_non_exhaustive()
    }
}

macro_rules! contributor_trait {
    ($name:ident) => {
        #[async_trait::async_trait]
        pub trait $name: Send + Sync {
            fn name(&self) -> &str;
            async fn contribute(
                &self,
                inputs: &AssemblyInputs,
                state: &AssemblyState,
            ) -> Result<Contribution, AssemblyError>;
        }
    };
}

contributor_trait!(ArtifactInstructionsContributor);
contributor_trait!(EffectivePolicyContributor);
contributor_trait!(MemoryRetrievalContributor);
contributor_trait!(SkillsContributor);
contributor_trait!(McpToolsContributor);
contributor_trait!(ContextContributor);
contributor_trait!(LifecycleContributor);

/// Each stage is ordered by registration. Stages themselves cannot be reordered.
#[derive(Default)]
pub struct ContributorRegistry {
    pub artifact_instructions: Vec<Arc<dyn ArtifactInstructionsContributor>>,
    pub effective_policy: Vec<Arc<dyn EffectivePolicyContributor>>,
    pub memory_retrieval: Vec<Arc<dyn MemoryRetrievalContributor>>,
    pub skills: Vec<Arc<dyn SkillsContributor>>,
    pub mcp_tools: Vec<Arc<dyn McpToolsContributor>>,
    pub context: Vec<Arc<dyn ContextContributor>>,
    pub lifecycle: Vec<Arc<dyn LifecycleContributor>>,
}

impl std::fmt::Debug for ContributorRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ContributorRegistry")
            .field("artifact_instructions", &self.artifact_instructions.len())
            .field("effective_policy", &self.effective_policy.len())
            .field("memory_retrieval", &self.memory_retrieval.len())
            .field("skills", &self.skills.len())
            .field("mcp_tools", &self.mcp_tools.len())
            .field("context", &self.context.len())
            .field("lifecycle", &self.lifecycle.len())
            .finish()
    }
}

impl ContributorRegistry {
    pub async fn assemble(&self, inputs: &AssemblyInputs) -> Result<AssemblyState, AssemblyError> {
        let mut state = AssemblyState {
            policy: inputs.policy.clone(),
            fragments: Vec::new(),
            history: inputs.history.clone(),
            tools: BTreeMap::new(),
            active_skills: Vec::new(),
            budgets: inputs.budgets.clone(),
            reduce_report: None,
            world_state: None,
        };
        macro_rules! stage {
            ($field:ident) => {
                for contributor in &self.$field {
                    let contribution = contributor.contribute(inputs, &state).await?;
                    apply_contribution(inputs, &mut state, contribution)?;
                }
            };
        }
        stage!(artifact_instructions);
        stage!(effective_policy);
        stage!(memory_retrieval);
        stage!(skills);
        stage!(mcp_tools);
        stage!(context);
        stage!(lifecycle);
        validate_projection(
            &state.policy,
            &inputs.authorized_tools,
            &state.tools,
            &state.active_skills,
        )?;
        Ok(state)
    }
}

/// Validate every contribution against both the effective policy and the
/// descriptor snapshot admitted by the trusted host before assembly.
pub fn validate_projection(
    policy: &EffectiveRunPolicy,
    authorized: &BTreeMap<String, Arc<ToolDescriptor>>,
    tools: &BTreeMap<String, Arc<ToolDescriptor>>,
    active_skills: &[String],
) -> Result<(), AssemblyError> {
    for skill in active_skills {
        if !policy.skills.ids.contains(skill) {
            return Err(AssemblyError::OutsidePolicy {
                resource: "skills".into(),
                id: skill.clone(),
            });
        }
    }
    for (name, descriptor) in tools {
        let Some(original) = authorized.get(name) else {
            return Err(AssemblyError::OutsidePolicy {
                resource: "tools".into(),
                id: name.clone(),
            });
        };
        if !original.equivalent_to(descriptor) {
            return Err(AssemblyError::ToolChanged { name: name.clone() });
        }
        let model_control =
            descriptor.source == ToolSource::BuiltIn && descriptor.exposure == Exposure::ModelOnly;
        if !model_control
            && matches!(
                policy.tools.mode,
                SelectionMode::None | SelectionMode::Selected
            )
            && !policy.tools.ids.contains(name)
            && !policy.tools.ids.contains(&descriptor.id)
        {
            return Err(AssemblyError::OutsidePolicy {
                resource: "tools".into(),
                id: name.clone(),
            });
        }
        if let Some(server) = &descriptor.server
            && matches!(
                policy.mcp_servers.mode,
                SelectionMode::None | SelectionMode::Selected
            )
            && !policy.mcp_servers.ids.contains(server)
        {
            return Err(AssemblyError::OutsidePolicy {
                resource: "mcp_servers".into(),
                id: server.clone(),
            });
        }
    }
    Ok(())
}

/// Preserve collision rejection when combining host-native and MCP snapshots.
pub fn collect_authorized_tools(
    tools: impl IntoIterator<Item = Arc<ToolDescriptor>>,
) -> Result<BTreeMap<String, Arc<ToolDescriptor>>, AssemblyError> {
    let mut collected = BTreeMap::<String, Arc<ToolDescriptor>>::new();
    for tool in tools {
        if let Some(existing) = collected.get(&tool.provider_name) {
            if !existing.equivalent_to(&tool) {
                return Err(AssemblyError::ToolChanged {
                    name: tool.provider_name.clone(),
                });
            }
        } else {
            collected.insert(tool.provider_name.clone(), tool);
        }
    }
    Ok(collected)
}

fn apply_contribution(
    inputs: &AssemblyInputs,
    state: &mut AssemblyState,
    contribution: Contribution,
) -> Result<(), AssemblyError> {
    if let Some(policy) = contribution.narrowed_policy {
        validate_narrowing(&state.policy, &policy)?;
        state.policy = policy;
    }
    for fragment in contribution.fragments {
        if let Some(existing) = state
            .fragments
            .iter()
            .find(|existing| existing.id == fragment.id)
        {
            if existing != &fragment {
                return Err(AssemblyError::FragmentCollision { id: fragment.id });
            }
        } else {
            state.fragments.push(fragment);
        }
    }
    let proposed = contribution
        .tools
        .into_iter()
        .map(|tool| (tool.provider_name.clone(), tool))
        .collect();
    validate_projection(
        &state.policy,
        &inputs.authorized_tools,
        &proposed,
        &contribution.active_skills,
    )?;
    state.tools.extend(proposed);
    for skill in contribution.active_skills {
        if !state.active_skills.contains(&skill) {
            state.active_skills.push(skill);
        }
    }
    if let Some(history) = contribution.history {
        state.history = history;
    }
    if let Some(budgets) = contribution.budgets {
        state.budgets = budgets;
    }
    if let Some(report) = contribution.reduce_report {
        state.reduce_report = Some(report);
    }
    if let Some(world_state) = contribution.world_state {
        state.world_state = Some(world_state);
    }
    Ok(())
}

fn validate_narrowing(
    base: &EffectiveRunPolicy,
    candidate: &EffectiveRunPolicy,
) -> Result<(), AssemblyError> {
    let mut permitted = base.clone();
    macro_rules! narrow {
        ($field:ident) => {
            if candidate
                .$field
                .ids
                .iter()
                .any(|id| !base.$field.ids.contains(id))
            {
                return Err(AssemblyError::PolicyWidening);
            }
            if candidate.$field != base.$field
                && !matches!(
                    candidate.$field.mode,
                    SelectionMode::None | SelectionMode::Selected
                )
            {
                return Err(AssemblyError::PolicyWidening);
            }
            permitted.$field = candidate.$field.clone();
        };
    }
    narrow!(skills);
    narrow!(tools);
    narrow!(mcp_servers);
    narrow!(knowledge_bases);
    permitted.memory_enabled = base.memory_enabled && candidate.memory_enabled;
    permitted.warnings.clone_from(&candidate.warnings);
    if &permitted != candidate {
        return Err(AssemblyError::PolicyWidening);
    }
    Ok(())
}
