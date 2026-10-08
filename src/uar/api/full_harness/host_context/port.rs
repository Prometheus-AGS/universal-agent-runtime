//! Original grant/context lease checked at the existing effect claim boundary.

use std::sync::Arc;
use async_trait::async_trait;
use crate::uar::runtime::tool_admission::{AdmissionCancellationOutcome, AdmissionCancellationReason,
    AdmissionTerminalOutcome, AdmittedToolInvocation, HostAdmissionBinding, HostAdmissionPreparation,
    HostAdmissionReceipt, HostToolAdmissionPort, LocalAdmissionDisposition, PreparedToolInvocation};
use super::DelegatedHostContext;

pub(super) struct DelegatedAdmissionPort {
    context: Arc<DelegatedHostContext>,
    run_id: String,
}
impl std::fmt::Debug for DelegatedAdmissionPort {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("DelegatedAdmissionPort([private])")
    }
}
impl DelegatedAdmissionPort {
    pub(super) fn new(context: Arc<DelegatedHostContext>, run_id: String) -> Self {
        Self { context, run_id }
    }
    fn require_authority(&self, invocation: &PreparedToolInvocation) -> anyhow::Result<()> {
        self.context.require_live().map_err(anyhow::Error::msg)?;
        self.require_identity(invocation)
    }
    fn require_identity(&self, invocation: &PreparedToolInvocation) -> anyhow::Result<()> {
        anyhow::ensure!(invocation.root_run_id == self.run_id
            && invocation.owner_id == self.context.principal.user_id
            && invocation.workspace.as_str() == self.context.working_directory.to_string_lossy().as_ref(),
            "delegated_host_context_mismatch: original run, owner and workspace must match");
        Ok(())
    }
}

#[async_trait]
impl HostToolAdmissionPort for DelegatedAdmissionPort {
    fn binding(&self) -> HostAdmissionBinding { self.context.admission.binding() }

    async fn prepare(&self, invocation: Arc<PreparedToolInvocation>) -> anyhow::Result<HostAdmissionPreparation> {
        self.require_authority(&invocation)?;
        let result = self.context.admission.prepare(Arc::clone(&invocation)).await
            .map_err(|error| safe_failure("prepare", error))?;
        self.require_authority(&invocation)?;
        Ok(result)
    }
    async fn resolve(&self, invocation: Arc<PreparedToolInvocation>, preparation: HostAdmissionPreparation,
        disposition: LocalAdmissionDisposition, approved: bool) -> anyhow::Result<Option<AdmittedToolInvocation>> {
        self.require_authority(&invocation)?;
        let result = self.context.admission.resolve(Arc::clone(&invocation), preparation, disposition, approved).await
            .map_err(|error| safe_failure("resolve", error))?;
        self.require_authority(&invocation)?;
        Ok(result)
    }
    async fn revalidate_claim(&self, admitted: &AdmittedToolInvocation) -> anyhow::Result<HostAdmissionReceipt> {
        self.require_authority(&admitted.prepared)?;
        let receipt = self.context.admission.revalidate_claim(admitted).await
            .map_err(|error| safe_failure("claim", error))?;
        self.require_authority(&admitted.prepared)?;
        Ok(receipt)
    }
    async fn cancel(&self, invocation: &PreparedToolInvocation, admission_id: &str,
        reason: AdmissionCancellationReason) -> anyhow::Result<AdmissionCancellationOutcome> {
        // Revocation forbids effects, not truthful cleanup of original admissions.
        self.require_identity(invocation)?;
        self.context.admission.cancel(invocation, admission_id, reason).await
            .map_err(|error| safe_failure("cancel", error))
    }
    async fn finish(&self, invocation: &PreparedToolInvocation, admission_id: &str,
        outcome: AdmissionTerminalOutcome) -> anyhow::Result<()> {
        self.require_identity(invocation)?;
        self.context.admission.finish(invocation, admission_id, outcome).await
            .map_err(|error| safe_failure("finish", error))
    }
}

fn safe_failure(operation: &str, error: anyhow::Error) -> anyhow::Error {
    // The existing adapter's HTTP rejection message is controlled, while
    // reqwest transport errors may contain a private URL. Retain status only.
    let status = error.chain().find_map(|cause| cause.downcast_ref::<reqwest::Error>()
        .and_then(reqwest::Error::status).map(|status| status.as_u16()))
        .or_else(|| error.to_string().rsplit_once("(HTTP ")
            .and_then(|(_, suffix)| suffix.strip_suffix(')')?.parse::<u16>().ok())
            .filter(|status| (100..600).contains(status)));
    match status {
        Some(status) => anyhow::anyhow!("delegated_host_admission_failed: POST {operation} HTTP {status}"),
        None => anyhow::anyhow!("delegated_host_admission_failed: POST {operation} transport or protocol failed"),
    }
}
