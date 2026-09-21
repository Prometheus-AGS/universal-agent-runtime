//! Deterministic ordering and rendering for typed prompt fragments.

use std::fmt;

use serde::{Deserialize, Serialize};

use super::fragment::{PromptFragment, PromptRole};

/// Stable top-level prompt order. Declaration order is the rendering order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PromptSection {
    AgentIdentity,
    EnforcedPolicy,
    HostInstructions,
    SkillCatalog,
    ActiveSkills,
    GovernedTools,
    WorldState,
    MemoryAndRetrieval,
    RequiredEvidence,
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
            Self::GovernedTools => "governed_tools",
            Self::WorldState => "world_state",
            Self::MemoryAndRetrieval => "memory_and_retrieval",
            Self::RequiredEvidence => "required_evidence",
            Self::ConversationHistory => "conversation_history",
            Self::CurrentInput => "current_input",
        }
    }
}

/// Scope that makes a template eligible for one resolved destination.
///
/// Family compatibility is an explicit reviewed identity supplied by the
/// trusted host. The resolver never derives it from a model-name substring.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "scope", rename_all = "snake_case")]
pub enum PromptTemplateSelector {
    Exact {
        provider_id: String,
        endpoint_kind: String,
        model_id: String,
        model_revision: String,
    },
    VerifiedFamily {
        provider_id: String,
        endpoint_kind: String,
        family_revision: String,
    },
    Generic {
        endpoint_kind: String,
    },
}

/// Data-only prompt layout. Endpoint serialization and provider settings are
/// separate contracts applied after this renderer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PromptTemplateLayout {
    Plain,
    StructuredXml,
}

/// Versioned prompt layout contract selected before request dispatch.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromptTemplateProfile {
    pub id: String,
    pub revision: String,
    pub selector: PromptTemplateSelector,
    pub layout: PromptTemplateLayout,
    pub required_slots: Vec<PromptSection>,
    pub supported_roles: Vec<PromptRole>,
    /// Identity only. Serialization remains owned by the driver boundary.
    pub wire_contract_id: String,
}

/// Explicit template validation failure. Callers must stop before dispatch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PromptTemplateError {
    MissingRequiredSlot {
        profile_id: String,
        slot: PromptSection,
    },
    UnsupportedRole {
        profile_id: String,
        fragment_id: String,
        role: PromptRole,
    },
}

impl fmt::Display for PromptTemplateError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingRequiredSlot { profile_id, slot } => write!(
                formatter,
                "prompt template `{profile_id}` is missing required slot `{}`",
                slot.as_str()
            ),
            Self::UnsupportedRole {
                profile_id,
                fragment_id,
                role,
            } => write!(
                formatter,
                "prompt template `{profile_id}` cannot represent role `{}` for fragment `{fragment_id}`",
                role.as_str()
            ),
        }
    }
}

impl std::error::Error for PromptTemplateError {}

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
    let ordered = ordered_fragments(fragments);

    if options.uses_structured_envelope() {
        ordered
            .into_iter()
            .filter(|fragment| !fragment.content.is_empty())
            .map(|fragment| {
                format!(
                    "<uar-fragment section=\"{}\" authority=\"{}\">\n{}\n</uar-fragment>",
                    fragment.section.as_str(),
                    fragment.authority.as_str(),
                    fragment.structured_content()
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

/// Validate and render through one resolved, versioned template profile.
pub fn render_with_template(
    fragments: &[PromptFragment],
    profile: &PromptTemplateProfile,
) -> Result<String, PromptTemplateError> {
    for slot in &profile.required_slots {
        if !fragments
            .iter()
            .any(|fragment| fragment.section == *slot && !fragment.content.is_empty())
        {
            return Err(PromptTemplateError::MissingRequiredSlot {
                profile_id: profile.id.clone(),
                slot: *slot,
            });
        }
    }
    for fragment in fragments {
        if !profile.supported_roles.contains(&fragment.role) {
            return Err(PromptTemplateError::UnsupportedRole {
                profile_id: profile.id.clone(),
                fragment_id: fragment.id.clone(),
                role: fragment.role,
            });
        }
    }

    let options = match profile.layout {
        PromptTemplateLayout::Plain => RenderOptions::default(),
        PromptTemplateLayout::StructuredXml => RenderOptions {
            prefers_xml_envelope: true,
            markdown_averse: false,
        },
    };
    Ok(render_with_options(fragments, options))
}

fn ordered_fragments(fragments: &[PromptFragment]) -> Vec<&PromptFragment> {
    let mut ordered = fragments.iter().collect::<Vec<_>>();
    ordered.sort_by(|left, right| {
        left.section
            .cmp(&right.section)
            .then_with(|| left.id.cmp(&right.id))
    });
    ordered
}
