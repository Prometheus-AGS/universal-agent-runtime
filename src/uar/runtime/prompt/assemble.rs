//! Deterministic ordering and rendering for typed prompt fragments.

use serde::{Deserialize, Serialize};

use super::fragment::PromptFragment;

/// Stable top-level prompt order. Declaration order is the rendering order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PromptSection {
    AgentIdentity,
    EnforcedPolicy,
    HostInstructions,
    SkillCatalog,
    ActiveSkills,
    WorldState,
    MemoryAndRetrieval,
    ConversationHistory,
    CurrentInput,
}

impl PromptSection {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::AgentIdentity => "agent_identity",
            Self::EnforcedPolicy => "enforced_policy",
            Self::HostInstructions => "host_instructions",
            Self::SkillCatalog => "skill_catalog",
            Self::ActiveSkills => "active_skills",
            Self::WorldState => "world_state",
            Self::MemoryAndRetrieval => "memory_and_retrieval",
            Self::ConversationHistory => "conversation_history",
            Self::CurrentInput => "current_input",
        }
    }
}

/// Model-family rendering preferences resolved before request dispatch.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RenderOptions {
    pub prefers_xml_envelope: bool,
    pub markdown_averse: bool,
}

impl RenderOptions {
    const fn uses_structured_envelope(self) -> bool {
        self.prefers_xml_envelope || self.markdown_averse
    }
}

/// Render fragments in the fixed section order and then by stable fragment id.
#[must_use]
pub fn render(fragments: &[PromptFragment]) -> String {
    render_with_options(fragments, RenderOptions::default())
}

/// Render fragments with stable structure adapted to the resolved model family.
#[must_use]
pub fn render_with_options(fragments: &[PromptFragment], options: RenderOptions) -> String {
    let mut ordered = fragments.iter().collect::<Vec<_>>();
    ordered.sort_by(|left, right| {
        left.section
            .cmp(&right.section)
            .then_with(|| left.id.cmp(&right.id))
    });

    if options.uses_structured_envelope() {
        ordered
            .into_iter()
            .filter(|fragment| !fragment.content.is_empty())
            .map(|fragment| {
                format!(
                    "<uar-fragment section=\"{}\" authority=\"{}\">\n{}\n</uar-fragment>",
                    fragment.section.as_str(),
                    fragment.authority.as_str(),
                    fragment.marked_content()
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        ordered
            .into_iter()
            .map(PromptFragment::marked_content)
            .filter(|content| !content.is_empty())
            .collect::<Vec<_>>()
            .join("\n\n")
    }
}
