//! Protected checkpoint metadata and single-use continuation ownership.
//! Persisted identity is evidence, never a credential or executable capability.

use serde::{Deserialize, Serialize};

use crate::uar::domain::runs::Run;
use crate::uar::runtime::checkpoint::{Checkpoint, raw_digest};

/// Explicit resume failures. Backend outages remain errors as well.
#[derive(Debug, thiserror::Error)]
pub enum CheckpointResumeError {
    #[error("durable checkpoint resume is unsupported by this backend")]
    Unsupported,
    #[error("protected checkpoint resume metadata is invalid")]
    Invalid,
    #[error("checkpoint continuation already has an executor")]
    Conflict,
    #[error("predecessor execution or effects have not been safely excluded")]
    PredecessorUncertain,
    #[error("checkpoint executor is closed")]
    ExecutorClosed,
    #[error("current checkpoint admission denied execution")]
    AdmissionDenied,
    #[error("the captured host execution definition cannot be reconstructed")]
    ExecutionUnavailable,
}

/// The actual host execution path, separate from the agent artifact identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckpointExecutionKind { ToolLoop, A2aToolLoop, HostGraph, HostDriver, HostResources }

/// Opaque graph closures are inspectable checkpoints, never restorable authority.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckpointExecutionDefinition {
    pub schema_version: u32,
    pub kind: CheckpointExecutionKind,
}

/// Complete immutable resume evidence, created only from admitted host state.
/// Raw JSON preserves exact artifact/history bytes across database normalization.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckpointResumeEnvelope {
    pub schema_version: u32,
    pub owner_key: String,
    pub source_run_id: String,
    pub checkpoint_id: String,
    pub source_runtime_epoch: String,
    pub checkpoint_revision: String,
    pub source_json: String,
    pub checkpoint_json: String,
}

impl CheckpointResumeEnvelope {
    pub(crate) fn new(
        owner_key: String,
        source_runtime_epoch: String,
        source: &Run,
        checkpoint: &Checkpoint,
    ) -> anyhow::Result<Self> {
        checkpoint.validate_integrity()?;
        let source_json = serde_json::to_string(source)?;
        let checkpoint_json = serde_json::to_string(checkpoint)?;
        let mut envelope = Self {
            schema_version: 1,
            owner_key,
            source_run_id: source.run_id.clone(),
            checkpoint_id: checkpoint.id.clone(),
            source_runtime_epoch,
            checkpoint_revision: String::new(),
            source_json,
            checkpoint_json,
        };
        envelope.checkpoint_revision = envelope.revision();
        envelope.validate()?;
        Ok(envelope)
    }

    fn revision(&self) -> String {
        raw_digest(&(
            self.schema_version,
            &self.owner_key,
            &self.source_run_id,
            &self.checkpoint_id,
            &self.source_runtime_epoch,
            &self.source_json,
            &self.checkpoint_json,
        ))
    }

    /// Validate complete owner, artifact and exact checkpoint/history identity.
    /// # Errors
    /// Refuses incomplete legacy metadata or changed protected bytes.
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.schema_version != 1
            || self.owner_key.is_empty()
            || self.source_runtime_epoch.is_empty()
            || self.checkpoint_revision != self.revision()
        {
            return Err(CheckpointResumeError::Invalid.into());
        }
        let source = self.source_run()?;
        let checkpoint = self.checkpoint()?;
        if source.run_id != self.source_run_id
            || checkpoint.run_id != self.source_run_id
            || checkpoint.id != self.checkpoint_id
            || source.conversation_id.as_deref() != Some(checkpoint.thread_id.as_str())
            || source.user_id.as_deref().is_none_or(str::is_empty)
        {
            return Err(CheckpointResumeError::Invalid.into());
        }
        crate::uar::domain::artifact::AgentArtifactSnapshot::from_run_context(&source.context)
            .map_err(|_| CheckpointResumeError::Invalid)?;
        serde_json::from_value::<crate::uar::runtime::turn::host::HostResourcesMarker>(
            source.context.get("host_resources").cloned().ok_or(CheckpointResumeError::Invalid)?)
            .map_err(|_| CheckpointResumeError::Invalid)?;
        self.execution_definition()?;
        let authorization = source.context.get("checkpoint_authorization_digest")
            .and_then(serde_json::Value::as_str)
            .ok_or(CheckpointResumeError::Invalid)?;
        if checkpoint.validate_integrity()?.authorization_sha256.as_deref()
            != Some(authorization)
        {
            return Err(CheckpointResumeError::Invalid.into());
        }
        Ok(())
    }

    /// Read protected source metadata without reconstructing authentication.
    pub fn source_run(&self) -> anyhow::Result<Run> {
        Ok(serde_json::from_str(&self.source_json)?)
    }

    /// Inspect the admitted host path; an artifact alone does not identify it.
    pub fn execution_definition(&self) -> anyhow::Result<CheckpointExecutionDefinition> {
        let source = self.source_run()?;
        let definition: CheckpointExecutionDefinition = serde_json::from_value(
            source.context.get("checkpoint_execution").cloned().ok_or(CheckpointResumeError::Invalid)?)
            .map_err(|_| CheckpointResumeError::Invalid)?;
        if definition.schema_version != 1 { return Err(CheckpointResumeError::Invalid.into()); }
        Ok(definition)
    }

    /// Refuse an opaque host graph before any reconstruction or durable claim.
    pub fn require_restorable_execution(&self) -> anyhow::Result<()> {
        if self.execution_definition()?.kind != CheckpointExecutionKind::ToolLoop {
            return Err(CheckpointResumeError::ExecutionUnavailable.into());
        }
        Ok(())
    }

    /// Storage admission accepts only explicit host-restorable execution kinds.
    /// The caller must separately renew the execution's current authority.
    pub(crate) fn require_claimable_execution(&self) -> anyhow::Result<()> {
        if !matches!(self.execution_definition()?.kind,
            CheckpointExecutionKind::ToolLoop | CheckpointExecutionKind::A2aToolLoop) {
            return Err(CheckpointResumeError::ExecutionUnavailable.into());
        }
        Ok(())
    }

    /// A verified peer must renew the exact contract through current host
    /// admission. Protected provenance alone cannot call this execution path.
    pub(crate) fn require_restorable_a2a(
        &self, fresh: &crate::uar::api::a2a::contract::UarDelegationContract,
    ) -> anyhow::Result<()> {
        fresh.validate()?;
        let source = self.source_run()?;
        let saved: crate::uar::api::a2a::contract::UarDelegationContract = serde_json::from_value(
            source.context.get("a2a_delegation").cloned().ok_or(CheckpointResumeError::Invalid)?)?;
        if self.execution_definition()?.kind != CheckpointExecutionKind::A2aToolLoop
            || &saved != fresh || source.agent_id != fresh.target_agent_id
            || source.user_id.as_deref() != Some(fresh.owner_id.as_str()) {
            return Err(CheckpointResumeError::ExecutionUnavailable.into());
        }
        Ok(())
    }

    /// Read the exact immutable checkpoint and validate its protected history.
    pub fn checkpoint(&self) -> anyhow::Result<Checkpoint> {
        let checkpoint: Checkpoint = serde_json::from_str(&self.checkpoint_json)?;
        checkpoint.validate_integrity()?;
        Ok(checkpoint)
    }
}

/// Durable single continuation identity. No expiry or implicit takeover exists.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckpointContinuationClaim {
    pub schema_version: u32,
    pub owner_key: String,
    pub source_run_id: String,
    pub checkpoint_id: String,
    pub checkpoint_revision: String,
    pub source_runtime_epoch: String,
    pub executor_runtime_epoch: String,
    pub continuation_run_id: String,
}

impl CheckpointContinuationClaim {
    /// Validate the source revision and host-reserved executor identity.
    pub fn validate(&self, envelope: &CheckpointResumeEnvelope) -> anyhow::Result<()> {
        envelope.validate()?;
        if self.schema_version != 1
            || self.owner_key != envelope.owner_key
            || self.source_run_id != envelope.source_run_id
            || self.checkpoint_id != envelope.checkpoint_id
            || self.checkpoint_revision != envelope.checkpoint_revision
            || self.source_runtime_epoch != envelope.source_runtime_epoch
            || self.executor_runtime_epoch.is_empty()
            || self.executor_runtime_epoch == self.source_runtime_epoch
            || uuid::Uuid::parse_str(&self.continuation_run_id).is_err()
        {
            return Err(CheckpointResumeError::Invalid.into());
        }
        Ok(())
    }
}

/// A successful CAS owns execution; an existing claim is inspection only.
#[derive(Debug)]
pub enum CheckpointClaimOutcome {
    Acquired(CheckpointContinuationClaim),
    Existing(CheckpointContinuationClaim),
}
