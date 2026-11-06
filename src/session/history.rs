//! Presentation metadata alongside canonical model history. It never authorizes replay.
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::llm::{MessageContent, MessageRole};

/// Actual host outcome, or explicit absence of terminal evidence.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionMessageStatus {
    #[default]
    Unknown,
    Pending,
    Running,
    Complete,
    Error,
    Cancelled,
    Interrupted,
}

/// Persisted identity allocated at append time, never synthesized on reload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMessageMetadata {
    pub message_id: String,
    pub run_id: Option<String>,
    pub client_turn_id: Option<String>,
    pub created_at: Option<String>,
    #[serde(default)]
    pub status: SessionMessageStatus,
}

impl SessionMessageMetadata {
    /// Construct metadata for a host-admitted message using its original identity.
    pub fn for_run(
        message_id: String,
        run_id: String,
        client_turn_id: Option<String>,
        status: SessionMessageStatus,
    ) -> Self {
        Self {
            message_id,
            run_id: Some(run_id),
            client_turn_id,
            created_at: Some(chrono::Utc::now().to_rfc3339()),
            status,
        }
    }
}

/// Public content projection omits tool arguments and host-only resources.
#[derive(Debug, Clone, Serialize)]
pub struct SessionHistoryMessage {
    pub message_id: String,
    pub role: MessageRole,
    #[serde(flatten)]
    pub content: MessageContent,
    pub run_id: Option<String>,
    pub client_turn_id: Option<String>,
    pub created_at: Option<String>,
    pub status: SessionMessageStatus,
    pub provenance: &'static str,
}

pub(super) fn legacy_message_id(owner_id: &str, session_id: &str, index: usize) -> String {
    let digest = Sha256::digest(
        format!(
            "{}:{owner_id}{}:{session_id}:{index}",
            owner_id.len(),
            session_id.len()
        )
        .as_bytes(),
    );
    format!(
        "legacy-{}",
        digest
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    )
}
