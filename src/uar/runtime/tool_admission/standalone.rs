//! Standalone admission adapter for UAR deployments without a paired host.

use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

use super::{
    AdmissionCancellationOutcome, AdmissionCancellationReason, AdmissionTerminalOutcome,
    AdmittedToolInvocation, HostAdmissionBinding, HostAdmissionDisposition,
    HostAdmissionPreparation, HostAdmissionReceipt, HostToolAdmissionPort,
    LocalAdmissionDisposition, PreparedToolInvocation, TOOL_ADMISSION_PROTOCOL_VERSION,
};

/// UAR remains fully usable without The Boss while routing every execution
/// through the same exact-invocation seam.
#[derive(Debug)]
pub struct StandaloneToolAdmissionPort {
    binding: HostAdmissionBinding,
}

impl StandaloneToolAdmissionPort {
    #[must_use]
    pub fn new(runtime_epoch: &str) -> Self {
        Self {
            binding: HostAdmissionBinding {
                version: TOOL_ADMISSION_PROTOCOL_VERSION,
                host_epoch: format!("standalone:{runtime_epoch}"),
            },
        }
    }
}

#[async_trait]
impl HostToolAdmissionPort for StandaloneToolAdmissionPort {
    fn binding(&self) -> HostAdmissionBinding {
        self.binding.clone()
    }

    async fn prepare(
        &self,
        invocation: Arc<PreparedToolInvocation>,
    ) -> anyhow::Result<HostAdmissionPreparation> {
        anyhow::ensure!(
            invocation.version == self.binding.version
                && invocation.host_epoch == self.binding.host_epoch,
            "Tool invocation belongs to another host admission binding"
        );
        Ok(HostAdmissionPreparation {
            version: self.binding.version,
            admission_id: Uuid::new_v4().to_string(),
            invocation_id: invocation.invocation_id.clone(),
            runtime_epoch: invocation.runtime_epoch.clone(),
            host_epoch: self.binding.host_epoch.clone(),
            host_disposition: HostAdmissionDisposition::Auto,
            action_display: serde_json::json!({
                "operation": invocation.provider_tool_name,
                "detailsAvailable": false,
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
        anyhow::ensure!(
            preparation.invocation_id == invocation.invocation_id
                && preparation.host_epoch == self.binding.host_epoch,
            "Standalone admission preparation does not match the invocation"
        );
        if !approved {
            return Ok(None);
        }
        let receipt = HostAdmissionReceipt {
            version: preparation.version,
            admission_id: preparation.admission_id,
            invocation_id: preparation.invocation_id,
            runtime_epoch: preparation.runtime_epoch,
            host_epoch: preparation.host_epoch,
            managed_mcp_metadata: preparation.managed_mcp_metadata,
        };
        Ok(Some(AdmittedToolInvocation {
            prepared: invocation,
            local_disposition,
            host_receipt: receipt,
        }))
    }

    async fn cancel(
        &self,
        invocation: &PreparedToolInvocation,
        _admission_id: &str,
        _reason: AdmissionCancellationReason,
    ) -> anyhow::Result<AdmissionCancellationOutcome> {
        anyhow::ensure!(
            invocation.host_epoch == self.binding.host_epoch,
            "Standalone admission belongs to another host binding"
        );
        Ok(AdmissionCancellationOutcome::Cancelled)
    }

    async fn finish(
        &self,
        invocation: &PreparedToolInvocation,
        _admission_id: &str,
        _outcome: AdmissionTerminalOutcome,
    ) -> anyhow::Result<()> {
        anyhow::ensure!(
            invocation.host_epoch == self.binding.host_epoch,
            "Standalone admission belongs to another host binding"
        );
        Ok(())
    }
}
