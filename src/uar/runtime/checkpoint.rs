//! State checkpoint types for graph-based run persistence.
//!
//! A [`Checkpoint`] is a snapshot of [`GraphState`] at a named node boundary.
//! Checkpoints allow runs to be paused and resumed from any saved point.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt::Write as _;

/// Host-owned state key carrying the effective authorization identity that
/// applied when a graph checkpoint was written.
pub const CHECKPOINT_AUTHORIZATION_DIGEST_KEY: &str = "_checkpoint_authorization_digest";

/// Integrity and authorization metadata added to current checkpoints.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CheckpointProtection {
    pub schema_version: u8,
    pub raw_state_json: String,
    pub raw_history_json: String,
    pub raw_state_sha256: String,
    pub raw_history_sha256: String,
    /// Host-owned authorization identity in effect when the checkpoint was written.
    pub authorization_sha256: Option<String>,
    pub completeness: CheckpointCompleteness,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CheckpointCompleteness {
    Complete,
    IncompleteLegacy,
}

/// A persisted snapshot of graph execution state at a named boundary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    /// Unique checkpoint ID (UUID v4).
    pub id: String,
    /// The run that produced this checkpoint.
    pub run_id: String,
    /// The conversation/session thread ID.
    pub thread_id: String,
    /// The graph node that recorded this checkpoint.
    pub node_id: String,
    /// Graph iteration count at the time of the checkpoint.
    pub iteration: u32,
    /// Serialised `GraphState.data` bag.
    pub state: serde_json::Value,
    /// Serialised `GraphState.messages` at this point in execution.
    pub messages: Vec<serde_json::Value>,
    /// ISO 8601 UTC timestamp.
    pub created_at: String,
    /// Absent on legacy rows, which cannot prove their history was complete.
    #[serde(default)]
    pub protection: Option<CheckpointProtection>,
}

impl Checkpoint {
    /// Create a new checkpoint from the current graph state.
    pub fn new(
        run_id: impl Into<String>,
        thread_id: impl Into<String>,
        node_id: impl Into<String>,
        state: &crate::uar::runtime::graph::GraphState,
        authorization_sha256: &str,
    ) -> Self {
        assert!(
            valid_sha256(authorization_sha256),
            "checkpoint authorization identity must be a SHA-256 digest"
        );
        let mut protected_data = state.data.clone();
        protected_data.insert(
            CHECKPOINT_AUTHORIZATION_DIGEST_KEY.to_string(),
            serde_json::Value::String(authorization_sha256.to_string()),
        );
        let state_value = serde_json::to_value(protected_data).unwrap_or_default();
        let raw_state_json =
            serde_json::to_string(&state_value).expect("checkpoint state is JSON serializable");
        let raw_history_json = serde_json::to_string(&state.messages)
            .expect("checkpoint history is JSON serializable");
        let protection = CheckpointProtection {
            schema_version: 1,
            raw_state_sha256: bytes_digest(raw_state_json.as_bytes()),
            raw_history_sha256: bytes_digest(raw_history_json.as_bytes()),
            raw_state_json,
            raw_history_json,
            authorization_sha256: Some(authorization_sha256.to_string()),
            completeness: CheckpointCompleteness::Complete,
        };
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            run_id: run_id.into(),
            thread_id: thread_id.into(),
            node_id: node_id.into(),
            iteration: state.iteration,
            state: state_value,
            messages: state.messages.clone(),
            created_at: chrono::Utc::now().to_rfc3339(),
            protection: Some(protection),
        }
    }

    /// Prove that a checkpoint is current-format, complete, and byte-identical
    /// to the state/history values recorded when it was created.
    pub fn validate_integrity(&self) -> anyhow::Result<&CheckpointProtection> {
        let protection = self.protection.as_ref().ok_or_else(|| {
            anyhow::anyhow!(
                "checkpoint {} is legacy-incomplete and cannot prove protected history",
                self.id
            )
        })?;
        anyhow::ensure!(
            protection.schema_version == 1
                && protection.completeness == CheckpointCompleteness::Complete,
            "checkpoint {} has incomplete protection metadata",
            self.id
        );
        anyhow::ensure!(
            bytes_digest(protection.raw_state_json.as_bytes()) == protection.raw_state_sha256,
            "checkpoint {} state integrity check failed",
            self.id
        );
        anyhow::ensure!(
            bytes_digest(protection.raw_history_json.as_bytes()) == protection.raw_history_sha256,
            "checkpoint {} history integrity check failed",
            self.id
        );
        let (protected_state, protected_history) = self.protected_values(protection)?;
        anyhow::ensure!(
            protected_state == self.state,
            "checkpoint {} state value differs from protected bytes",
            self.id
        );
        anyhow::ensure!(
            protected_history == self.messages,
            "checkpoint {} history value differs from protected bytes",
            self.id
        );
        Ok(protection)
    }

    /// Replace database-normalized values with the exact protected JSON values.
    ///
    /// SurrealDB can normalize string-like values during a structured-value
    /// round trip. The raw envelope is authoritative once its byte digests and
    /// host-owned authorization binding have been verified.
    pub fn restore_protected_values(&mut self) -> anyhow::Result<()> {
        let Some(protection) = self.protection.as_ref() else {
            return Ok(());
        };
        anyhow::ensure!(
            protection.schema_version == 1
                && protection.completeness == CheckpointCompleteness::Complete,
            "checkpoint {} has incomplete protection metadata",
            self.id
        );
        anyhow::ensure!(
            bytes_digest(protection.raw_state_json.as_bytes()) == protection.raw_state_sha256,
            "checkpoint {} state integrity check failed",
            self.id
        );
        anyhow::ensure!(
            bytes_digest(protection.raw_history_json.as_bytes()) == protection.raw_history_sha256,
            "checkpoint {} history integrity check failed",
            self.id
        );
        let (state, history) = self.protected_values(protection)?;
        self.state = state;
        self.messages = history;
        Ok(())
    }

    fn protected_values(
        &self,
        protection: &CheckpointProtection,
    ) -> anyhow::Result<(serde_json::Value, Vec<serde_json::Value>)> {
        let authorization = protection.authorization_sha256.as_deref().ok_or_else(|| {
            anyhow::anyhow!("checkpoint {} has no authorization binding", self.id)
        })?;
        anyhow::ensure!(
            valid_sha256(authorization),
            "checkpoint {} has an invalid authorization binding",
            self.id
        );
        let protected_state: serde_json::Value = serde_json::from_str(&protection.raw_state_json)
            .map_err(|error| {
            anyhow::anyhow!("checkpoint {} raw state is unreadable: {error}", self.id)
        })?;
        anyhow::ensure!(
            protected_state
                .get(CHECKPOINT_AUTHORIZATION_DIGEST_KEY)
                .and_then(serde_json::Value::as_str)
                == Some(authorization),
            "checkpoint {} authorization binding differs from protected state",
            self.id
        );
        let protected_history: Vec<serde_json::Value> =
            serde_json::from_str(&protection.raw_history_json).map_err(|error| {
                anyhow::anyhow!("checkpoint {} raw history is unreadable: {error}", self.id)
            })?;
        Ok((protected_state, protected_history))
    }

    /// Restore a [`GraphState`] from this checkpoint, reporting a corrupt
    /// state bag rather than silently substituting an empty one.
    ///
    /// # Errors
    ///
    /// Returns an error when `state` cannot be deserialized into the graph's
    /// data bag. Resuming with an empty state would look like a successful
    /// resume while discarding everything the checkpoint recorded.
    pub fn try_restore_state(&self) -> anyhow::Result<crate::uar::runtime::graph::GraphState> {
        self.validate_integrity()?;
        let data = serde_json::from_value(self.state.clone()).map_err(|e| {
            anyhow::anyhow!("checkpoint {} has an unreadable state bag: {e}", self.id)
        })?;
        Ok(crate::uar::runtime::graph::GraphState {
            data,
            messages: self.messages.clone(),
            iteration: self.iteration,
        })
    }

    /// Restore a [`GraphState`], falling back to an empty data bag.
    ///
    /// Prefer [`Self::try_restore_state`]; this exists for callers that
    /// genuinely tolerate a partial restore.
    #[must_use]
    pub fn restore_state(&self) -> crate::uar::runtime::graph::GraphState {
        let data = serde_json::from_value(self.state.clone()).unwrap_or_default();
        crate::uar::runtime::graph::GraphState {
            data,
            messages: self.messages.clone(),
            iteration: self.iteration,
        }
    }
}

/// Convert a checkpoint's recorded messages into typed conversation history.
///
/// This is what seeds a resumed run, so a malformed entry is an error: a run
/// that silently starts with no history is indistinguishable from a fresh run.
///
/// # Errors
///
/// Returns an error when any recorded message cannot be deserialized.
pub fn history_from_checkpoint(
    checkpoint: &Checkpoint,
) -> anyhow::Result<Vec<crate::llm::Message>> {
    checkpoint.validate_integrity()?;
    checkpoint
        .messages
        .iter()
        .enumerate()
        .map(|(i, value)| {
            serde_json::from_value(value.clone()).map_err(|e| {
                anyhow::anyhow!(
                    "checkpoint {} message {i} is not a valid conversation message: {e}",
                    checkpoint.id
                )
            })
        })
        .collect()
}

/// Stable SHA-256 over the exact JSON bytes stored by the checkpoint contract.
#[must_use]
pub fn raw_digest<T: Serialize + ?Sized>(value: &T) -> String {
    let bytes = serde_json::to_vec(value).expect("checkpoint values are JSON serializable");
    bytes_digest(&bytes)
}

fn bytes_digest(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut encoded = String::with_capacity(digest.len() * 2);
    for byte in digest {
        write!(&mut encoded, "{byte:02x}").expect("writing to a String cannot fail");
    }
    encoded
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

/// Redacted identity for the complete effective authorization policy.
#[must_use]
pub fn authorization_digest<T: Serialize + ?Sized>(policy: &T) -> String {
    raw_digest(policy)
}
