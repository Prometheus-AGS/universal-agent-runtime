use crate::uar::runtime::actor::messages::ActorOwner;

use super::{ApiError, FullHarnessTaskAuthority, TaskReceipt, project_a2a};

impl FullHarnessTaskAuthority {
    pub(super) fn require_revision(
        &self,
        owner: &ActorOwner,
        workspace_id: &str,
        task_id: &str,
        expected_revision: u64,
    ) -> Result<TaskReceipt, ApiError> {
        let receipt = self.owned(owner, workspace_id, task_id)?;
        if receipt.revision != expected_revision {
            return Err(revision_conflict(task_id, expected_revision, &receipt));
        }
        Ok(receipt)
    }

    fn mutate_at_revision(
        &self,
        owner: &ActorOwner,
        workspace_id: &str,
        task_id: &str,
        expected_revision: u64,
        mutation: impl FnOnce(&mut TaskReceipt),
    ) -> Result<TaskReceipt, ApiError> {
        self.require_revision(owner, workspace_id, task_id, expected_revision)?;
        let mut records = self
            .records
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let record = records
            .tasks
            .get_mut(task_id)
            .filter(|record| &record.owner == owner && record.workspace_id == workspace_id)
            .ok_or_else(|| ApiError::not_found("task_not_found", "task was not found"))?;
        if record.receipt.revision != expected_revision {
            return Err(revision_conflict(
                task_id,
                expected_revision,
                &record.receipt,
            ));
        }
        let previous = record.receipt.clone();
        mutation(&mut record.receipt);
        if record.receipt != previous {
            record.receipt.revision = record.receipt.revision.saturating_add(1);
        }
        Ok(record.receipt.clone())
    }

    pub(super) async fn cancel_at_revision(
        &self,
        owner: &ActorOwner,
        workspace_id: &str,
        task_id: &str,
        expected_revision: u64,
    ) -> Result<TaskReceipt, ApiError> {
        let receipt =
            self.mutate_at_revision(owner, workspace_id, task_id, expected_revision, |receipt| {
                receipt.cancellation.requested = true
            })?;
        let acknowledged = self
            .manager
            .cancel_run_for_owner(owner, &receipt.run_id)
            .await;
        self.update(task_id, |record| {
            record.receipt.cancellation.acknowledged |= acknowledged;
        });
        self.owned(owner, workspace_id, task_id)
    }

    pub(super) fn detach_at_revision(
        &self,
        owner: &ActorOwner,
        workspace_id: &str,
        task_id: &str,
        expected_revision: u64,
        observer_id: &str,
    ) -> Result<TaskReceipt, ApiError> {
        let observer_id = observer_id.trim();
        if observer_id.is_empty() {
            return Err(ApiError::bad_request(
                "detach_invalid",
                "observer_id must not be empty",
            ));
        }
        self.mutate_at_revision(owner, workspace_id, task_id, expected_revision, |receipt| {
            if !receipt
                .detached_observers
                .iter()
                .any(|existing| existing == observer_id)
            {
                receipt.detached_observers.push(observer_id.to_owned());
                receipt.detached_observers.sort();
            }
            receipt.detached = !receipt.detached_observers.is_empty();
        })
    }

    pub(crate) async fn a2a_cancel(
        &self,
        owner: &ActorOwner,
        workspace_id: &str,
        agent_id: &str,
        task_id: &str,
        expected_revision: u64,
    ) -> Result<crate::uar::api::a2a::types::Task, ApiError> {
        let receipt = self.require_revision(owner, workspace_id, task_id, expected_revision)?;
        if receipt.agent_id.as_deref() != Some(agent_id) {
            return Err(ApiError::not_found("task_not_found", "task was not found"));
        }
        let receipt = self
            .cancel_at_revision(owner, workspace_id, task_id, expected_revision)
            .await?;
        Ok(project_a2a(agent_id, receipt))
    }
}

fn revision_conflict(task_id: &str, expected_revision: u64, receipt: &TaskReceipt) -> ApiError {
    ApiError::conflict(
        "revision_conflict",
        format!(
            "expected revision {expected_revision} does not match current revision {}",
            receipt.revision
        ),
        Some(task_id.to_owned()),
        Some(receipt.admission_id.clone()),
    )
}
