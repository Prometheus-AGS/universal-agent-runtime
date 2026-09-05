//! Pure world-state contribution after history reduction. Baselines are host
//! records, never reconstructed from model-supplied marker text.

use crate::llm::{Message, MessageContent, MessageRole};
use crate::uar::runtime::prompt::{
    Authority, PromptFragment, PromptRole, PromptSection, Retention,
};
use crate::uar::runtime::turn::contributors::{
    AssemblyError, AssemblyInputs, AssemblyState, ContextContributor, Contribution,
};

use super::merge_patch::{self, MergePatchError};
use super::sections::WorldStateSnapshot;

/// Host-owned previous snapshot and exact messages needed to interpret its diff.
#[derive(Debug, Clone, Default)]
pub struct WorldStateBaseline {
    snapshot: Option<WorldStateSnapshot>,
    messages: Vec<Message>,
}

/// An immutable contribution that the host commits to the session once selected.
#[derive(Debug, Clone)]
pub struct WorldStateUpdate {
    pub fragments: Vec<PromptFragment>,
    pub messages: Vec<Message>,
    pub baseline: WorldStateBaseline,
}

impl WorldStateBaseline {
    /// Generate full sections after any rewrite or lost baseline message.
    ///
    /// # Errors
    /// Returns an unrepresentable merge-patch error for explicit null additions.
    pub fn prepare(
        &self,
        snapshot: &WorldStateSnapshot,
        history: &[Message],
        rewritten: bool,
    ) -> Result<WorldStateUpdate, MergePatchError> {
        let mut remaining = history.iter();
        let complete = self.messages.iter().all(|expected| {
            remaining.any(|message| {
                message.role == MessageRole::System
                    && serde_json::json!(message) == serde_json::json!(expected)
            })
        });
        let previous = self.snapshot.as_ref().filter(|_| !rewritten && complete);
        let mut fragments = Vec::new();
        for (id, value) in &snapshot.sections {
            let old = previous.and_then(|snapshot| snapshot.sections.get(id));
            let (mode, body) = match old {
                Some(old) => match merge_patch::generate(old, value)? {
                    Some(patch) => ("merge_patch", patch),
                    None => continue,
                },
                None => ("full", value.clone()),
            };
            fragments.push(PromptFragment::new(
                format!("world_state.{}", id.as_str()),
                PromptSection::WorldState,
                "host.world_state",
                Authority::Host,
                PromptRole::System,
                Retention::Session,
                format!(
                    "[WORLD STATE: {} / {mode}]\n{}\n{body}",
                    id.as_str(),
                    id.replacement_text()
                ),
            ));
        }
        if let Some(previous) = previous {
            for id in previous
                .sections
                .keys()
                .filter(|id| !snapshot.sections.contains_key(id))
            {
                fragments.push(PromptFragment::new(
                    format!("world_state.{}", id.as_str()),
                    PromptSection::WorldState,
                    "host.world_state",
                    Authority::Host,
                    PromptRole::System,
                    Retention::Session,
                    format!(
                        "[WORLD STATE: {} / removed]\n{}\nnull",
                        id.as_str(),
                        id.removal_text()
                    ),
                ));
            }
        }
        fragments.sort_by(|left, right| left.id.cmp(&right.id));
        let messages = fragments
            .iter()
            .map(|fragment| Message {
                role: MessageRole::System,
                content: MessageContent::text(fragment.marked_content()),
                tool_call_id: None,
                tool_calls: None,
            })
            .collect::<Vec<_>>();
        let mut baseline_messages = if previous.is_some() {
            self.messages.clone()
        } else {
            Vec::new()
        };
        baseline_messages.extend(messages.iter().cloned());
        Ok(WorldStateUpdate {
            fragments,
            messages,
            baseline: Self {
                snapshot: Some(snapshot.clone()),
                messages: baseline_messages,
            },
        })
    }
}

/// Registered after the reducer, still within the fixed context stage.
#[derive(Clone)]
pub struct WorldStateContributor {
    pub snapshot: WorldStateSnapshot,
    pub baseline: WorldStateBaseline,
    pub history_rewritten: bool,
}

impl std::fmt::Debug for WorldStateContributor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorldStateContributor")
            .field("history_rewritten", &self.history_rewritten)
            .finish_non_exhaustive()
    }
}

impl WorldStateContributor {
    /// Reserve enough tokens for either a diff or the full snapshot required if
    /// reduction invalidates the baseline. Capture once, before reduction.
    ///
    /// # Errors
    /// Returns an unrepresentable merge-patch error.
    pub fn reserved_tokens(
        &self,
        history: &[Message],
        model: &str,
    ) -> Result<usize, MergePatchError> {
        use crate::uar::runtime::context::token_service::TokenService;
        let full = WorldStateBaseline::default().prepare(&self.snapshot, history, true)?;
        let delta = self
            .baseline
            .prepare(&self.snapshot, history, self.history_rewritten)?;
        Ok(TokenService::count_messages(model, &full.messages)
            .max(TokenService::count_messages(model, &delta.messages)))
    }
}

#[async_trait::async_trait]
impl ContextContributor for WorldStateContributor {
    fn name(&self) -> &str {
        "world_state"
    }

    async fn contribute(
        &self,
        _: &AssemblyInputs,
        state: &AssemblyState,
    ) -> Result<Contribution, AssemblyError> {
        let rewritten = self.history_rewritten
            || state
                .reduce_report
                .as_ref()
                .is_some_and(|report| report.history_rewritten);
        let update = self
            .baseline
            .prepare(&self.snapshot, &state.history, rewritten)
            .map_err(|error| AssemblyError::ContributorFailed {
                name: self.name().into(),
                message: error.to_string(),
            })?;
        let mut history = state.history.clone();
        history.extend(update.messages.iter().cloned());
        Ok(Contribution {
            fragments: update.fragments.clone(),
            history: Some(history),
            world_state: Some(update),
            ..Contribution::default()
        })
    }
}
