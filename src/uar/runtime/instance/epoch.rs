//! Trusted activation fence shared by root persistence and tool admission.

use std::sync::Arc;

use crate::uar::persistence::{
    PersistenceLayer,
    agent_instances::{InstanceCommandKind, InstanceCommandStatus, InstanceLifecycle},
};

/// Exact instance/attempt authority captured before a bounded root turn starts.
#[derive(Clone)]
pub struct InstanceEpochBinding {
    store: Arc<dyn PersistenceLayer>,
    owner_id: String,
    workspace_id: String,
    instance_id: String,
    epoch: u64,
    pub(crate) principal_id: String,
    pub(crate) representation_revision: u64,
    pub(crate) representation_grants: Vec<crate::uar::domain::collaboration::RepresentationGrantRef>,
    root_run_id: Option<String>,
}

impl std::fmt::Debug for InstanceEpochBinding {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("InstanceEpochBinding")
            .field("instance_id", &self.instance_id)
            .field("epoch", &self.epoch)
            .field("root_run_id", &self.root_run_id)
            .finish()
    }
}

impl InstanceEpochBinding {
    pub(crate) fn new(
        store: Arc<dyn PersistenceLayer>,
        owner_id: String,
        workspace_id: String,
        instance_id: String,
        epoch: u64,
        principal_id: String,
        representation_revision: u64,
        representation_grants: Vec<crate::uar::domain::collaboration::RepresentationGrantRef>,
    ) -> Self {
        Self {
            store,
            owner_id,
            workspace_id,
            instance_id,
            epoch,
            principal_id,
            representation_revision,
            representation_grants,
            root_run_id: None,
        }
    }

    pub(crate) fn for_run(&self, root_run_id: &str) -> Self {
        let mut bound = self.clone();
        bound.root_run_id = Some(root_run_id.to_owned());
        bound
    }

    pub(crate) fn instance_id(&self) -> &str {
        &self.instance_id
    }

    /// Reject a replaced activation or turn before it commits another effect.
    pub(crate) async fn revalidate(&self) -> anyhow::Result<()> {
        let current = self
            .store
            .load_agent_instance(&self.owner_id, &self.workspace_id, &self.instance_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Logical instance no longer exists"))?;
        current.validate(&self.owner_id, &self.workspace_id)?;
        anyhow::ensure!(
            current.epoch == self.epoch
                && current.representation_revision == self.representation_revision
                && current.representation_grants == self.representation_grants
                && matches!(
                    current.lifecycle,
                    InstanceLifecycle::Active | InstanceLifecycle::Draining
                ),
            "Logical instance activation was replaced"
        );
        if let Some(run_id) = &self.root_run_id {
            anyhow::ensure!(
                current.active_attempt.as_ref().is_some_and(|attempt| {
                    attempt.epoch == self.epoch
                        && &attempt.root_run_id == run_id
                        && current.inbox.iter().any(|command| {
                            command.command_id == attempt.command_id
                                && command.status == InstanceCommandStatus::Running
                        })
                }),
                "Logical instance turn is no longer current"
            );
        }
        Ok(())
    }

    /// A committed cancellation intent bars a later protected effect even
    /// before RunManager has registered the root's cancellation token.
    pub(crate) async fn revalidate_effect(&self) -> anyhow::Result<()> {
        self.revalidate().await?;
        let current = self
            .store
            .load_agent_instance(&self.owner_id, &self.workspace_id, &self.instance_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Logical instance no longer exists"))?;
        anyhow::ensure!(
            !current.inbox.iter().any(|command| {
                command.status == InstanceCommandStatus::Accepted
                    && matches!(
                        command.kind,
                        InstanceCommandKind::Cancel
                            | InstanceCommandKind::Passivate
                            | InstanceCommandKind::Disable
                            | InstanceCommandKind::Restart
                    )
            }),
            "Logical instance cancellation is pending"
        );
        Ok(())
    }
}

#[async_trait::async_trait]
impl crate::uar::runtime::tool_admission::ClaimRevalidator for InstanceEpochBinding {
    async fn revalidate(&self) -> anyhow::Result<()> {
        InstanceEpochBinding::revalidate_effect(self).await
    }
}
