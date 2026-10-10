//! Admission routing from trusted registered runtime-control provenance.

use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

use crate::uar::persistence::tool_admission::AdmissionOwner;

use super::{
    AdmissionCancellationOutcome, AdmissionCancellationReason, AdmissionTerminalOutcome,
    AdmittedToolInvocation, HostAdmissionBinding, HostAdmissionDisposition,
    HostAdmissionPreparation, HostAdmissionReceipt, HostToolAdmissionPort,
    LocalAdmissionDisposition, PreparedToolInvocation, ToolAdmissionContext,
};

#[derive(Debug)]
pub(super) struct OwnedToolAdmissionPort {
    context: Arc<ToolAdmissionContext>,
    paired_host: Arc<dyn HostToolAdmissionPort>,
}

impl OwnedToolAdmissionPort {
    pub(super) fn new(
        context: Arc<ToolAdmissionContext>,
        paired_host: Arc<dyn HostToolAdmissionPort>,
    ) -> Self {
        Self {
            context,
            paired_host,
        }
    }

    fn validate_local(&self, invocation: &PreparedToolInvocation) -> anyhow::Result<()> {
        anyhow::ensure!(
            invocation.admission_owner == AdmissionOwner::UarRuntime
                && invocation.version == self.context.host.version
                && invocation.root_run_id == self.context.root_run_id
                && invocation.executing_run_id == self.context.executing_run_id
                && invocation.owner_id == self.context.owner_id
                && invocation.principal_id == self.context.principal_id
                && invocation.workspace == self.context.workspace
                && invocation.runtime_epoch == self.context.runtime_epoch
                && invocation.host_epoch == self.context.host.host_epoch
                && invocation.catalog_revision == self.context.catalog_revision
                && invocation.run_policy_revision == self.context.run_policy_revision
                && invocation.governance_policy_revision == self.context.governance_policy_revision
                && invocation.grant_revision == self.context.grant_revision
                && invocation.lease_revision == self.context.lease_revision
                && invocation.budget_revision == self.context.budget_revision,
            "Runtime-control admission belongs to another captured run context"
        );
        Ok(())
    }
}

#[async_trait]
impl HostToolAdmissionPort for OwnedToolAdmissionPort {
    fn binding(&self) -> HostAdmissionBinding {
        self.paired_host.binding()
    }

    async fn prepare(
        &self,
        invocation: Arc<PreparedToolInvocation>,
    ) -> anyhow::Result<HostAdmissionPreparation> {
        if invocation.admission_owner != AdmissionOwner::UarRuntime {
            return self.paired_host.prepare(invocation).await;
        }
        self.validate_local(&invocation)?;
        invocation.validate_authority_envelope()?;
        Ok(HostAdmissionPreparation {
            version: invocation.version,
            execution_kind: invocation.execution_kind,
            admission_id: Uuid::new_v4().to_string(),
            invocation_id: invocation.invocation_id.clone(),
            runtime_epoch: invocation.runtime_epoch.clone(),
            host_epoch: invocation.host_epoch.clone(),
            authority_revision: invocation.authority_revision.clone(),
            host_disposition: HostAdmissionDisposition::Auto,
            action_display: serde_json::json!({
                "operation": invocation.provider_tool_name,
                "arguments": invocation.validated_arguments,
            }),
            managed_mcp_metadata: false,
        })
    }

    async fn resolve(
        &self,
        invocation: Arc<PreparedToolInvocation>,
        preparation: HostAdmissionPreparation,
        local_disposition: LocalAdmissionDisposition,
        approved: bool,
    ) -> anyhow::Result<Option<AdmittedToolInvocation>> {
        if invocation.admission_owner != AdmissionOwner::UarRuntime {
            return self
                .paired_host
                .resolve(invocation, preparation, local_disposition, approved)
                .await;
        }
        self.validate_local(&invocation)?;
        invocation.validate_authority_envelope()?;
        anyhow::ensure!(
            preparation.version == invocation.version
                && preparation.execution_kind == invocation.execution_kind
                && preparation.invocation_id == invocation.invocation_id
                && preparation.runtime_epoch == invocation.runtime_epoch
                && preparation.host_epoch == invocation.host_epoch
                && preparation.authority_revision == invocation.authority_revision
                && matches!(
                    preparation.host_disposition,
                    HostAdmissionDisposition::Auto | HostAdmissionDisposition::Ask
                )
                && !preparation.managed_mcp_metadata,
            "Runtime-control preparation does not match the exact invocation"
        );
        if !approved {
            return Ok(None);
        }
        Ok(Some(AdmittedToolInvocation {
            prepared: invocation,
            local_disposition,
            host_receipt: HostAdmissionReceipt {
                version: preparation.version,
                execution_kind: preparation.execution_kind,
                admission_id: preparation.admission_id,
                invocation_id: preparation.invocation_id,
                runtime_epoch: preparation.runtime_epoch,
                host_epoch: preparation.host_epoch,
                authority_revision: preparation.authority_revision,
                managed_mcp_metadata: false,
            },
        }))
    }

    async fn revalidate_claim(
        &self,
        admitted: &AdmittedToolInvocation,
    ) -> anyhow::Result<HostAdmissionReceipt> {
        if admitted.prepared.admission_owner != AdmissionOwner::UarRuntime {
            return self.paired_host.revalidate_claim(admitted).await;
        }
        self.validate_local(&admitted.prepared)?;
        admitted.prepared.validate_authority_envelope()?;
        anyhow::ensure!(
            admitted.host_receipt.version == admitted.prepared.version
                && admitted.host_receipt.execution_kind == admitted.prepared.execution_kind
                && admitted.host_receipt.invocation_id == admitted.prepared.invocation_id
                && admitted.host_receipt.runtime_epoch == admitted.prepared.runtime_epoch
                && admitted.host_receipt.host_epoch == admitted.prepared.host_epoch
                && admitted.host_receipt.authority_revision == admitted.prepared.authority_revision
                && !admitted.host_receipt.managed_mcp_metadata,
            "Runtime-control claim does not match the exact invocation"
        );
        Ok(admitted.host_receipt.clone())
    }

    async fn consume_native(
        &self,
        admitted: &AdmittedToolInvocation,
    ) -> anyhow::Result<HostAdmissionReceipt> {
        if admitted.prepared.admission_owner != AdmissionOwner::UarRuntime {
            return self.paired_host.consume_native(admitted).await;
        }
        anyhow::ensure!(
            admitted.prepared.execution_kind == super::ToolExecutionKind::RuntimeNative,
            "Runtime-control claim requires native execution authority"
        );
        // AdmissionLifecycle persists and consumes this runtime-owned claim once.
        self.revalidate_claim(admitted).await
    }

    async fn cancel(
        &self,
        invocation: &PreparedToolInvocation,
        admission_id: &str,
        reason: AdmissionCancellationReason,
    ) -> anyhow::Result<AdmissionCancellationOutcome> {
        if invocation.admission_owner != AdmissionOwner::UarRuntime {
            return self
                .paired_host
                .cancel(invocation, admission_id, reason)
                .await;
        }
        self.validate_local(invocation)?;
        // AdmissionLifecycle handles already-claimed/terminal controls before
        // this port; only an unclaimed invocation can be cancelled here.
        Ok(AdmissionCancellationOutcome::Cancelled)
    }

    async fn finish(
        &self,
        invocation: &PreparedToolInvocation,
        admission_id: &str,
        outcome: AdmissionTerminalOutcome,
    ) -> anyhow::Result<()> {
        if invocation.admission_owner != AdmissionOwner::UarRuntime {
            return self
                .paired_host
                .finish(invocation, admission_id, outcome)
                .await;
        }
        self.validate_local(invocation)
    }
}
