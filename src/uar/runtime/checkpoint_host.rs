//! Trusted-host checkpoint writer. Graph nodes emit intent and never receive it.

use std::sync::Arc;

use crate::uar::domain::runs::Run;
use crate::uar::persistence::{PersistenceLayer, checkpoint_resume::CheckpointResumeEnvelope};
use crate::uar::runtime::{checkpoint::Checkpoint, graph::GraphState};

/// Host-only checkpoint capability bound to an admitted source run and epoch.
/// Its private constructor prevents graph input from minting runnable authority.
pub struct CheckpointHost {
    persistence: Option<Arc<dyn PersistenceLayer>>,
    source: Run,
    owner_key: Option<String>,
    runtime_epoch: String,
    authorization_digest: String,
    cancellation: tokio_util::sync::CancellationToken,
    scrubber: super::turn::host::RunSecretScrubber,
    binding: Option<super::turn::CollaborationRunBinding>,
}

impl std::fmt::Debug for CheckpointHost {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.debug_struct("CheckpointHost").finish_non_exhaustive()
    }
}

impl CheckpointHost {
    pub(crate) fn new(
        persistence: Option<Arc<dyn PersistenceLayer>>,
        mut source: Run,
        owner: Option<&super::actor::messages::ActorOwner>,
        runtime_epoch: String,
        authorization_digest: String,
        cancellation: tokio_util::sync::CancellationToken,
        scrubber: super::turn::host::RunSecretScrubber,
        binding: Option<super::turn::CollaborationRunBinding>,
    ) -> Self {
        // Display diagnostics and raw input do not become resume authority.
        let mut protected = serde_json::Map::new();
        for key in ["agent_snapshot", "effective_run_policy", "effective_target", "host_resources",
            "effective_service_binding", "effective_collaboration_binding", "host_context", "checkpoint_execution", "a2a_delegation"] {
            if let Some(value) = source.context.get(key) {
                protected.insert(key.to_owned(), value.clone());
            }
        }
        protected.insert("checkpoint_authorization_digest".into(), authorization_digest.clone().into());
        source.context = protected.into();
        Self {
            persistence, source,
            owner_key: owner.and_then(|owner| owner.credential_principal()).map(|principal| principal.storage_key()),
            runtime_epoch, authorization_digest, cancellation, scrubber, binding,
        }
    }

    /// Whether this host can mint resume authority at all. Anonymous runs have
    /// no verified checkpoint principal, so no envelope they produce could ever
    /// be resumed; callers must skip the write instead of failing the run.
    pub(crate) fn has_resume_authority(&self) -> bool {
        self.owner_key.is_some()
    }

    /// Handle a checkpoint intent on the host, preserving exact projected bytes.
    /// # Errors
    /// Refuses absent owner/durability, revoked bindings, cancellation, or a
    /// failed atomic save. None of those can claim a resumable checkpoint.
    pub(crate) async fn persist(&self, node_id: &str, state: &GraphState) -> anyhow::Result<()> {
        self.persist_envelope(node_id, state).await.map(|_| ())
    }

    /// Return the committed protected identity for a host-owned pending approval.
    pub(crate) async fn persist_envelope(&self, node_id: &str, state: &GraphState) -> anyhow::Result<CheckpointResumeEnvelope> {
        anyhow::ensure!(!self.cancellation.is_cancelled(), "checkpoint executor is cancelled");
        let persistence = self.persistence.as_ref()
            .ok_or(crate::uar::persistence::checkpoint_resume::CheckpointResumeError::Unsupported)?;
        let owner_key = self.owner_key.as_ref()
            .ok_or_else(|| anyhow::anyhow!("verified checkpoint principal required"))?;
        if let Some(binding) = &self.binding {
            let artifact = crate::uar::domain::artifact::AgentArtifactSnapshot::from_run_context(&self.source.context)
                .map_err(|_| crate::uar::persistence::checkpoint_resume::CheckpointResumeError::Invalid)?;
            binding.revalidate(&artifact.artifact).await?;
        }
        let projected = self.scrubber.project_value(serde_json::to_value(state)?);
        let state: GraphState = serde_json::from_value(projected)?;
        let checkpoint = Checkpoint::new(
            &self.source.run_id,
            self.source.conversation_id.as_deref().ok_or_else(|| anyhow::anyhow!("checkpoint session required"))?,
            node_id, &state, &self.authorization_digest,
        );
        let envelope = CheckpointResumeEnvelope::new(owner_key.clone(), self.runtime_epoch.clone(), &self.source, &checkpoint)
            .map_err(|error| {
                tracing::warn!(checkpoint_stage = "envelope_validation", checkpoint_error_class = "resume_metadata_error",
                    "Checkpoint persistence operation reported an error");
                error
            })?;
        persistence.save_resumable_checkpoint(&envelope).await.map_err(|error| {
            let class = match error.downcast_ref::<crate::uar::persistence::checkpoint_resume::CheckpointResumeError>() {
                Some(crate::uar::persistence::checkpoint_resume::CheckpointResumeError::Unsupported) => "unsupported",
                Some(crate::uar::persistence::checkpoint_resume::CheckpointResumeError::Invalid) => "protected_metadata_invalid",
                _ => "persistence_error",
            };
            tracing::warn!(checkpoint_stage = "host_save", checkpoint_error_class = class,
                "Checkpoint persistence operation reported an error");
            error
        })?;
        Ok(envelope)
    }
}
