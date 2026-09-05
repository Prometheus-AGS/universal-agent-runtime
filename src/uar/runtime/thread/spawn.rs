//! Delegation input and history forking. Parent system prompts, tool traces, and
//! intermediate assistant messages never become child conversational authority.

use serde::{Deserialize, Serialize};

use crate::llm::{Message, MessageContent, MessageRole};

use super::{ThreadRecordError, validate_task_name};

/// Explicit opt-in to parent dialogue. Even `Full` excludes tool traffic and
/// system context; the child kernel assembles its own artifact and policy.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "mode",
    content = "turns",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum HistoryForkMode {
    #[default]
    None,
    Full,
    LastTurns(u32),
}

/// Model-facing intent only. Parent/owner/root identities and delegation
/// authorization are supplied separately by the trusted host, never this input.
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentSpawnRequest {
    pub artifact_id: String,
    pub delegated_prompt: String,
    #[serde(default)]
    pub task_name: Option<String>,
    #[serde(default)]
    pub history_fork: HistoryForkMode,
}

/// Host-adapter intent for an endpoint already declared by the parent artifact.
/// The trusted host resolves its agent identity and credential binding.
#[derive(Debug, Clone)]
pub struct RemoteAgentSpawnRequest {
    pub(crate) endpoint: String,
    pub(crate) delegated_prompt: String,
    pub(crate) task_name: Option<String>,
}

impl RemoteAgentSpawnRequest {
    pub(crate) fn validate(&self) -> Result<(), ThreadRecordError> {
        if self.endpoint.trim().is_empty() {
            return Err(ThreadRecordError::EmptyField { field: "endpoint" });
        }
        if self.delegated_prompt.trim().is_empty() {
            return Err(ThreadRecordError::EmptyField {
                field: "delegated_prompt",
            });
        }
        if let Some(name) = &self.task_name {
            validate_task_name(name)?;
        }
        Ok(())
    }
}

impl std::fmt::Debug for AgentSpawnRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AgentSpawnRequest")
            .field("artifact_id", &self.artifact_id)
            .field("task_name", &self.task_name)
            .field("history_fork", &self.history_fork)
            .field("delegated_prompt", &"<redacted>")
            .finish()
    }
}

impl AgentSpawnRequest {
    /// Validate required delegation inputs without granting delegation authority.
    ///
    /// # Errors
    /// Rejects empty artifact/prompt fields and path-bearing task names.
    pub fn validate(&self) -> Result<(), ThreadRecordError> {
        if self.artifact_id.trim().is_empty() {
            return Err(ThreadRecordError::EmptyField {
                field: "artifact_id",
            });
        }
        if self.delegated_prompt.trim().is_empty() {
            return Err(ThreadRecordError::EmptyField {
                field: "delegated_prompt",
            });
        }
        if let Some(name) = &self.task_name {
            validate_task_name(name)?;
        }
        Ok(())
    }

    /// Select parent dialogue and append the delegated prompt exactly once.
    ///
    /// # Errors
    /// Returns the same invalid-input errors as [`Self::validate`].
    pub fn initial_messages(
        &self,
        parent_history: &[Message],
    ) -> Result<Vec<Message>, ThreadRecordError> {
        self.validate()?;
        let mut messages = fork_history(parent_history, self.history_fork);
        messages.push(Message {
            role: MessageRole::User,
            content: MessageContent::text(self.delegated_prompt.clone()),
            tool_call_id: None,
            tool_calls: None,
        });
        Ok(messages)
    }
}

/// Fork user-delimited turns, retaining each user message and only the final
/// assistant response. `LastTurns(2)` counts turns, not two individual messages.
/// A current user turn without a final response is retained as a user turn.
///
/// Tool-call assistants clear any earlier candidate response in that turn, so
/// a partial pre-tool answer cannot be mistaken for its final answer. The parent
/// must supply canonical messages; malformed user messages carrying tool metadata
/// are excluded rather than laundering tool traffic into a user role.
pub fn fork_history(history: &[Message], mode: HistoryForkMode) -> Vec<Message> {
    if matches!(mode, HistoryForkMode::None | HistoryForkMode::LastTurns(0)) {
        return Vec::new();
    }
    let mut turns: Vec<(Message, Option<Message>)> = Vec::new();
    let mut active_turn = false;
    for message in history {
        let has_tool_metadata = message.tool_call_id.is_some()
            || message
                .tool_calls
                .as_ref()
                .is_some_and(|calls| !calls.is_empty());
        match message.role {
            MessageRole::User if !has_tool_metadata => {
                turns.push((dialogue_message(message), None));
                active_turn = true;
            }
            MessageRole::Assistant if active_turn => {
                if let Some((_, final_response)) = turns.last_mut() {
                    *final_response = (!has_tool_metadata).then(|| dialogue_message(message));
                }
            }
            MessageRole::Tool if active_turn => {
                if let Some((_, final_response)) = turns.last_mut() {
                    *final_response = None;
                }
            }
            MessageRole::User => active_turn = false,
            MessageRole::System | MessageRole::Assistant | MessageRole::Tool => {}
        }
    }
    let keep = match mode {
        HistoryForkMode::None => 0,
        HistoryForkMode::Full => turns.len(),
        HistoryForkMode::LastTurns(count) => usize::try_from(count).unwrap_or(usize::MAX),
    };
    let skip = turns.len().saturating_sub(keep);
    turns
        .into_iter()
        .skip(skip)
        .flat_map(|(user, assistant)| std::iter::once(user).chain(assistant))
        .collect()
}

fn dialogue_message(message: &Message) -> Message {
    Message {
        role: message.role,
        content: message.content.clone(),
        tool_call_id: None,
        tool_calls: None,
    }
}
