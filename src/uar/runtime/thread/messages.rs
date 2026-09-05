//! Typed inter-agent envelopes. Identity is metadata, never a prompt prefix.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::llm::{Message, MessageContent, MessageRole};

use super::{AgentThread, ThreadRecordError};

/// A host-authenticated message in one root tree, ordered by a host-assigned
/// recipient sequence. `trigger_turn: false` is mailbox input only.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InterAgentMessage {
    pub message_id: String,
    pub owner_id: String,
    pub root_thread_id: String,
    pub root_run_id: String,
    pub sender_thread_id: String,
    pub sender_artifact_id: String,
    pub recipient_thread_id: String,
    pub sequence: u64,
    pub content: String,
    pub trigger_turn: bool,
    pub created_at: DateTime<Utc>,
}

impl std::fmt::Debug for InterAgentMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InterAgentMessage")
            .field("message_id", &self.message_id)
            .field("owner_id", &self.owner_id)
            .field("root_thread_id", &self.root_thread_id)
            .field("sender_thread_id", &self.sender_thread_id)
            .field("recipient_thread_id", &self.recipient_thread_id)
            .field("sequence", &self.sequence)
            .field("trigger_turn", &self.trigger_turn)
            .field("content", &"<redacted>")
            .finish()
    }
}

impl InterAgentMessage {
    /// Bind identity from host-resolved records, not fields supplied by a model.
    ///
    /// # Errors
    /// Rejects invalid thread records and communication across owners or root trees.
    pub fn between(
        sender: &AgentThread,
        recipient: &AgentThread,
        sequence: u64,
        content: String,
        trigger_turn: bool,
    ) -> Result<Self, ThreadRecordError> {
        sender.validate()?;
        recipient.validate()?;
        if sender.owner_id != recipient.owner_id
            || sender.root_thread_id != recipient.root_thread_id
            || sender.root_run_id != recipient.root_run_id
        {
            return Err(ThreadRecordError::LineageMismatch);
        }
        Ok(Self {
            message_id: Uuid::new_v4().to_string(),
            owner_id: sender.owner_id.clone(),
            root_thread_id: sender.root_thread_id.clone(),
            root_run_id: sender.root_run_id.clone(),
            sender_thread_id: sender.thread_id.clone(),
            sender_artifact_id: sender.artifact_id.clone(),
            recipient_thread_id: recipient.thread_id.clone(),
            sequence,
            content,
            trigger_turn,
            created_at: Utc::now(),
        })
    }

    /// Provider-independent mailbox ordering, with an identity tie-breaker.
    pub fn order_key(&self) -> (u64, &str) {
        (self.sequence, &self.message_id)
    }

    /// Convert only the body into a user message. Scheduling must inspect
    /// `trigger_turn` separately; this pure conversion never starts a run.
    pub fn user_message(&self) -> Message {
        Message {
            role: MessageRole::User,
            content: MessageContent::text(self.content.clone()),
            tool_call_id: None,
            tool_calls: None,
        }
    }
}
