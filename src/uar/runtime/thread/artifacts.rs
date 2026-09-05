//! Exact, host-owned artifact receipts for one actor invocation. Structured
//! output is retained before model-history truncation and is not parsed from
//! assistant prose or a later conversation's history.

use std::sync::{Arc, RwLock};

use crate::uar::runtime::actor::messages::ActorOwner;

/// Structured output declared by a successful native tool implementation.
#[derive(Clone)]
pub struct ToolOutputArtifact {
    pub artifact_id: String,
    pub name: String,
    pub description: String,
    pub data: serde_json::Value,
}

impl std::fmt::Debug for ToolOutputArtifact {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ToolOutputArtifact")
            .field("artifact_id", &self.artifact_id)
            .finish_non_exhaustive()
    }
}

#[derive(Default)]
struct Receipts {
    closed: bool,
    artifacts: Vec<ToolOutputArtifact>,
}

/// A capability minted by the actor host, not deserialized from a request.
/// The host closes it after execution and before sending the completion reply.
#[derive(Clone)]
pub struct RunArtifactCollector {
    owner: ActorOwner,
    run_id: String,
    receipts: Arc<RwLock<Receipts>>,
}

impl std::fmt::Debug for RunArtifactCollector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RunArtifactCollector")
            .field("run_id", &self.run_id)
            .finish_non_exhaustive()
    }
}

impl RunArtifactCollector {
    pub(crate) fn new(owner: ActorOwner, run_id: String) -> Self {
        Self {
            owner,
            run_id,
            receipts: Arc::new(RwLock::new(Receipts::default())),
        }
    }

    pub(crate) fn check_binding(&self, owner: &ActorOwner, run_id: &str) -> anyhow::Result<()> {
        anyhow::ensure!(
            owner == &self.owner && run_id == self.run_id,
            "Artifact receipt belongs to another owner or run"
        );
        Ok(())
    }

    pub(crate) fn publish(
        &self,
        owner: &ActorOwner,
        artifacts: Vec<ToolOutputArtifact>,
    ) -> anyhow::Result<()> {
        anyhow::ensure!(owner == &self.owner, "Foreign artifact receipt owner");
        let mut receipts = self
            .receipts
            .write()
            .map_err(|_| anyhow::anyhow!("Artifact receipts unavailable"))?;
        anyhow::ensure!(!receipts.closed, "Artifact receipt collection has closed");
        receipts.artifacts.extend(artifacts);
        Ok(())
    }

    pub(crate) fn close(&self) -> anyhow::Result<()> {
        self.receipts
            .write()
            .map_err(|_| anyhow::anyhow!("Artifact receipts unavailable"))?
            .closed = true;
        Ok(())
    }

    pub(crate) fn snapshot(&self) -> anyhow::Result<Vec<ToolOutputArtifact>> {
        let receipts = self
            .receipts
            .read()
            .map_err(|_| anyhow::anyhow!("Artifact receipts unavailable"))?;
        anyhow::ensure!(receipts.closed, "Artifact receipts are not final");
        Ok(receipts.artifacts.clone())
    }
}
