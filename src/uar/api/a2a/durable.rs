//! Durable transport correlation only. Execution and recovery remain host-owned.
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::{contract::UarDelegationContract, types::Task};

/// A reservation committed before dispatching an exact actor run. The process
/// identity never expires and is not permission to replay after a restart.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct A2aBinding {
    pub schema_version: u32,
    pub owner_key: String,
    pub agent_id: String,
    pub task_id: String,
    pub context_id: String,
    pub revision: u64,
    pub process_id: String,
    /// Trusted host epoch captured before dispatch; legacy absence is not proof.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runtime_epoch: Option<String>,
    pub run_id: String,
    pub request_digest: String,
    #[serde(default)]
    pub cancellation_requested: bool,
    /// Last transport projection; durable thread state takes precedence.
    pub task: Task,
    /// Provenance only. A fresh authenticated request must revalidate it.
    pub contract: Option<UarDelegationContract>,
}

impl A2aBinding {
    pub fn storage_key_for(owner: &str, agent: &str, task: &str) -> String {
        let encoded = format!("{}:{owner}{}:{agent}{}:{task}", owner.len(), agent.len(), task.len());
        Sha256::digest(encoded.as_bytes()).iter().map(|byte| format!("{byte:02x}")).collect::<String>()
    }

    pub fn storage_key(&self) -> String {
        Self::storage_key_for(&self.owner_key, &self.agent_id, &self.task_id)
    }

    /// Only a settled terminal task releases its context's submission slot.
    pub fn context_closed(&self) -> bool {
        matches!(self.task.status.state, super::types::TaskState::Completed
            | super::types::TaskState::Canceled | super::types::TaskState::Failed)
            && !self.task.cleanup_unconfirmed()
            && self.task.metadata.get("context_unconfirmed") != Some(&serde_json::Value::Bool(true))
            && self.task.metadata.get("execution_unconfirmed") != Some(&serde_json::Value::Bool(true))
    }

    pub fn validate(&self) -> anyhow::Result<()> {
        anyhow::ensure!(self.schema_version == 1 && self.revision > 0 && self.revision <= i64::MAX as u64,
            "Invalid A2A binding version or revision");
        for value in [&self.owner_key, &self.agent_id, &self.task_id, &self.context_id, &self.process_id, &self.run_id] {
            anyhow::ensure!(!value.is_empty() && value.trim() == value.as_str() && !value.chars().any(char::is_control),
                "Invalid A2A binding identity");
        }
        anyhow::ensure!(self.runtime_epoch.as_ref().is_none_or(|epoch| !epoch.is_empty()
            && epoch.trim() == epoch && !epoch.chars().any(char::is_control)), "Invalid A2A runtime epoch");
        anyhow::ensure!(self.task.id == self.task_id && self.task.context_id.as_deref() == Some(self.context_id.as_str()),
            "A2A task correlation mismatch");
        Ok(())
    }

    pub fn validate_transition(&self, next: &Self) -> anyhow::Result<()> {
        self.validate()?;
        next.validate()?;
        anyhow::ensure!(self.owner_key == next.owner_key && self.agent_id == next.agent_id
            && self.task_id == next.task_id && self.context_id == next.context_id
            && self.contract == next.contract && self.revision.checked_add(1) == Some(next.revision),
            "A2A binding identity or revision changed");
        anyhow::ensure!(self.run_id == next.run_id || !self.cancellation_requested,
            "A cancelled task cannot admit another continuation");
        anyhow::ensure!(self.run_id != next.run_id || self.runtime_epoch == next.runtime_epoch,
            "A2A run epoch is immutable");
        Ok(())
    }
}
