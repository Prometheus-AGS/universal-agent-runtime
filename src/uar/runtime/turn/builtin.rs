//! The seven built-in stages compose the primitives extracted by changes 1–5.

use std::sync::Arc;

use crate::llm::{LlmDriver, Message, MessageContent, MessageRole};
use crate::uar::domain::{artifact::AgentArtifact, events::MemoryItem, policy::EffectiveRunPolicy};
use crate::uar::runtime::prompt::{
    Authority, PromptFragment, PromptRole, PromptSection, RenderOptions, Retention,
    render_with_options,
};

use super::contributors::*;

pub fn artifact_fragments(artifact: &AgentArtifact) -> Vec<PromptFragment> {
    let mut fragments = vec![PromptFragment::new(
        "agent.identity",
        PromptSection::AgentIdentity,
        format!("artifact:{}", artifact.id),
        Authority::System,
        PromptRole::System,
        Retention::Session,
        artifact.prompt.system.clone(),
    )];
    fragments.extend(artifact.prompt.instructions.iter().enumerate().map(
        |(index, instruction)| {
            PromptFragment::new(
                format!("host.instruction.{index:04}"),
                PromptSection::HostInstructions,
                format!("artifact:{}", artifact.id),
                Authority::Host,
                PromptRole::System,
                Retention::Session,
                instruction.clone(),
            )
        },
    ));
    fragments
}

pub fn policy_fragment(policy: &EffectiveRunPolicy) -> PromptFragment {
    PromptFragment::new(
        "policy.effective",
        PromptSection::EnforcedPolicy,
        "effective_run_policy",
        Authority::Policy,
        PromptRole::System,
        Retention::Turn,
        format!("[EFFECTIVE RUN POLICY]\n{}", serde_json::json!(policy)),
    )
}

pub fn memory_fragment(policy: &EffectiveRunPolicy, hits: &[MemoryItem]) -> Option<PromptFragment> {
    if !policy.memory_enabled || hits.is_empty() {
        return None;
    }
    let content = hits
        .iter()
        .map(|hit| {
            format!(
                "[{}/{}] {}",
                hit.scope.as_deref().unwrap_or("memory"),
                hit.memory_type.as_deref().unwrap_or("context"),
                hit.value
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    Some(PromptFragment::new(
        "retrieved.memory",
        PromptSection::MemoryAndRetrieval,
        "run.memory_hits",
        Authority::Retrieved,
        PromptRole::System,
        Retention::Turn,
        format!("[MEMORY CONTEXT]\n{content}"),
    ))
}

struct ArtifactStage;
#[async_trait::async_trait]
impl ArtifactInstructionsContributor for ArtifactStage {
    fn name(&self) -> &str {
        "artifact_instructions"
    }
    async fn contribute(
        &self,
        inputs: &AssemblyInputs,
        _: &AssemblyState,
    ) -> Result<Contribution, AssemblyError> {
        let mut fragments = artifact_fragments(&inputs.artifact);
        fragments.extend(
            inputs
                .prepared_fragments
                .iter()
                .filter(|fragment| {
                    fragment.id == "presentation.output"
                        && fragment.section == PromptSection::HostInstructions
                })
                .cloned(),
        );
        Ok(Contribution {
            fragments,
            ..Contribution::default()
        })
    }
}

struct PolicyStage;
#[async_trait::async_trait]
impl EffectivePolicyContributor for PolicyStage {
    fn name(&self) -> &str {
        "effective_policy"
    }
    async fn contribute(
        &self,
        _: &AssemblyInputs,
        state: &AssemblyState,
    ) -> Result<Contribution, AssemblyError> {
        Ok(Contribution {
            fragments: vec![policy_fragment(&state.policy)],
            ..Contribution::default()
        })
    }
}

struct MemoryStage;
#[async_trait::async_trait]
impl MemoryRetrievalContributor for MemoryStage {
    fn name(&self) -> &str {
        "memory_and_retrieval"
    }
    async fn contribute(
        &self,
        inputs: &AssemblyInputs,
        state: &AssemblyState,
    ) -> Result<Contribution, AssemblyError> {
        let mut fragments = inputs
            .prepared_fragments
            .iter()
            .filter(|fragment| {
                fragment.section == PromptSection::MemoryAndRetrieval
                    && fragment.id != "retrieved.memory"
            })
            .cloned()
            .collect::<Vec<_>>();
        fragments.extend(memory_fragment(&state.policy, &inputs.memory_hits));
        Ok(Contribution {
            fragments,
            ..Contribution::default()
        })
    }
}

struct SkillsStage;
#[async_trait::async_trait]
impl SkillsContributor for SkillsStage {
    fn name(&self) -> &str {
        "skills"
    }
    async fn contribute(
        &self,
        inputs: &AssemblyInputs,
        _: &AssemblyState,
    ) -> Result<Contribution, AssemblyError> {
        Ok(Contribution {
            fragments: inputs
                .prepared_fragments
                .iter()
                .filter(|fragment| fragment.section == PromptSection::SkillCatalog)
                .cloned()
                .collect(),
            active_skills: inputs.active_skills.clone(),
            ..Contribution::default()
        })
    }
}

struct ToolsStage;
#[async_trait::async_trait]
impl McpToolsContributor for ToolsStage {
    fn name(&self) -> &str {
        "mcp_and_tools"
    }
    async fn contribute(
        &self,
        inputs: &AssemblyInputs,
        _: &AssemblyState,
    ) -> Result<Contribution, AssemblyError> {
        Ok(Contribution {
            tools: inputs.authorized_tools.values().cloned().collect(),
            ..Contribution::default()
        })
    }
}

pub struct ContextStage {
    pub model: String,
    pub context_limit: usize,
    /// Host context reinserted after reduction, including a possible full world state.
    pub reserved_tokens: usize,
    pub strategy: crate::uar::context::ContextStrategy,
    pub options: RenderOptions,
    pub skill_budget: crate::config::SkillReattachmentBudget,
    pub driver: Option<Arc<dyn LlmDriver>>,
}

impl std::fmt::Debug for ContextStage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ContextStage")
            .field("model", &self.model)
            .field("context_limit", &self.context_limit)
            .finish_non_exhaustive()
    }
}

#[async_trait::async_trait]
impl ContextContributor for ContextStage {
    fn name(&self) -> &str {
        "context"
    }
    async fn contribute(
        &self,
        inputs: &AssemblyInputs,
        state: &AssemblyState,
    ) -> Result<Contribution, AssemblyError> {
        let system = Message {
            role: MessageRole::System,
            content: MessageContent::text(render_with_options(&state.fragments, self.options)),
            tool_call_id: None,
            tool_calls: None,
        };
        let (history, report) = if let Some(prepared) = &inputs.prepared_history {
            let mut history = prepared.clone();
            if history
                .first()
                .is_some_and(|message| message.role == MessageRole::System)
            {
                history[0] = system;
            } else {
                history.insert(0, system);
            }
            (history, None)
        } else {
            let mut history = vec![system];
            history.extend(inputs.history.clone());
            let (history, report) = crate::uar::runtime::context::reduce::reduce_history(
                history,
                &self.strategy,
                &self.model,
                self.context_limit.saturating_sub(self.reserved_tokens),
                self.driver.as_deref(),
            )
            .await;
            (history, Some(report))
        };
        let mut budgets = inputs.budgets.clone();
        budgets.context_window_tokens = Some(self.context_limit);
        Ok(Contribution {
            history: Some(history),
            budgets: Some(budgets),
            reduce_report: report,
            ..Contribution::default()
        })
    }
}

struct SkillBodiesStage {
    model: String,
    context_limit: usize,
    budget: crate::config::SkillReattachmentBudget,
    options: RenderOptions,
}

#[async_trait::async_trait]
impl ContextContributor for SkillBodiesStage {
    fn name(&self) -> &str {
        "active_skill_bodies"
    }
    async fn contribute(
        &self,
        inputs: &AssemblyInputs,
        state: &AssemblyState,
    ) -> Result<Contribution, AssemblyError> {
        let bodies = inputs
            .prepared_fragments
            .iter()
            .filter(|fragment| {
                fragment.section == PromptSection::ActiveSkills
                    && fragment.retention == Retention::Reclaimable
            })
            .cloned()
            .collect::<Vec<_>>();
        let (_, fragments) = crate::uar::runtime::skills::retention::reattach_fragments(
            &state.history,
            &bodies,
            &self.model,
            self.context_limit,
            self.budget,
            self.options,
        );
        Ok(Contribution {
            fragments,
            ..Contribution::default()
        })
    }
}

struct LifecycleStage;
#[async_trait::async_trait]
impl LifecycleContributor for LifecycleStage {
    fn name(&self) -> &str {
        "lifecycle"
    }
    async fn contribute(
        &self,
        _: &AssemblyInputs,
        state: &AssemblyState,
    ) -> Result<Contribution, AssemblyError> {
        tracing::debug!(
            fragments = state.fragments.len(),
            tools = state.tools.len(),
            "Typed assembly complete"
        );
        Ok(Contribution::default())
    }
}

pub fn registry(context: ContextStage) -> ContributorRegistry {
    let bodies = SkillBodiesStage {
        model: context.model.clone(),
        context_limit: context.context_limit,
        budget: context.skill_budget,
        options: context.options,
    };
    ContributorRegistry {
        artifact_instructions: vec![Arc::new(ArtifactStage)],
        effective_policy: vec![Arc::new(PolicyStage)],
        memory_retrieval: vec![Arc::new(MemoryStage)],
        skills: vec![Arc::new(SkillsStage)],
        mcp_tools: vec![Arc::new(ToolsStage)],
        context: vec![Arc::new(context), Arc::new(bodies)],
        lifecycle: vec![Arc::new(LifecycleStage)],
    }
}
