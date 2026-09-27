use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicUsize, Ordering},
};

use async_trait::async_trait;
use serde_json::json;
use universal_agent_runtime::uar::{
    governance::engine::GovernanceEngine,
    runtime::tool_admission::{
        AdmissionCancellationOutcome, AdmissionCancellationReason, AdmissionTerminalOutcome,
        AdmittedToolInvocation, HostAdmissionBinding, HostAdmissionDisposition,
        HostAdmissionPreparation, HostAdmissionReceipt, HostToolAdmissionPort,
        LocalAdmissionDisposition, PreparedToolInvocation, ToolAdmissionContext,
        ToolAdmissionRuntime, TOOL_ADMISSION_PROTOCOL_VERSION,
    },
    tools::descriptor::{ApprovalClass, Exposure, ToolDescriptor, ToolEffect, ToolSource},
};

#[derive(Debug)]
struct RecordingManagedHost {
    binding: HostAdmissionBinding,
    claim_revalidations: AtomicUsize,
    finished: AtomicUsize,
    deny_claim: AtomicBool,
}

impl RecordingManagedHost {
    fn new(runtime_epoch: &str) -> Self {
        Self {
            binding: HostAdmissionBinding {
                version: TOOL_ADMISSION_PROTOCOL_VERSION,
                host_epoch: format!("managed:{runtime_epoch}"),
            },
            claim_revalidations: AtomicUsize::new(0),
            finished: AtomicUsize::new(0),
            deny_claim: AtomicBool::new(false),
        }
    }
}

#[async_trait]
impl HostToolAdmissionPort for RecordingManagedHost {
    fn binding(&self) -> HostAdmissionBinding {
        self.binding.clone()
    }

    async fn prepare(
        &self,
        invocation: Arc<PreparedToolInvocation>,
    ) -> anyhow::Result<HostAdmissionPreparation> {
        Ok(HostAdmissionPreparation {
            version: invocation.version,
            admission_id: format!("managed-admission:{}", invocation.invocation_id),
            invocation_id: invocation.invocation_id.clone(),
            runtime_epoch: invocation.runtime_epoch.clone(),
            host_epoch: invocation.host_epoch.clone(),
            authority_revision: invocation.authority_revision.clone(),
            host_disposition: HostAdmissionDisposition::Auto,
            action_display: json!({"operation": invocation.provider_tool_name}),
            managed_mcp_metadata: true,
        })
    }

    async fn resolve(
        &self,
        invocation: Arc<PreparedToolInvocation>,
        preparation: HostAdmissionPreparation,
        local_disposition: LocalAdmissionDisposition,
        approved: bool,
    ) -> anyhow::Result<Option<AdmittedToolInvocation>> {
        if !approved {
            return Ok(None);
        }
        Ok(Some(AdmittedToolInvocation {
            prepared: invocation,
            local_disposition,
            host_receipt: HostAdmissionReceipt {
                version: preparation.version,
                admission_id: preparation.admission_id,
                invocation_id: preparation.invocation_id,
                runtime_epoch: preparation.runtime_epoch,
                host_epoch: preparation.host_epoch,
                authority_revision: preparation.authority_revision,
                managed_mcp_metadata: preparation.managed_mcp_metadata,
            },
        }))
    }

    async fn revalidate_claim(
        &self,
        admitted: &AdmittedToolInvocation,
    ) -> anyhow::Result<HostAdmissionReceipt> {
        self.claim_revalidations.fetch_add(1, Ordering::SeqCst);
        anyhow::ensure!(
            !self.deny_claim.load(Ordering::SeqCst),
            "managed authority denied the claim"
        );
        Ok(admitted.host_receipt.clone())
    }

    async fn cancel(
        &self,
        _invocation: &PreparedToolInvocation,
        _admission_id: &str,
        _reason: AdmissionCancellationReason,
    ) -> anyhow::Result<AdmissionCancellationOutcome> {
        Ok(AdmissionCancellationOutcome::Cancelled)
    }

    async fn finish(
        &self,
        _invocation: &PreparedToolInvocation,
        _admission_id: &str,
        _outcome: AdmissionTerminalOutcome,
    ) -> anyhow::Result<()> {
        self.finished.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

fn descriptor() -> ToolDescriptor {
    let schema = json!({
        "type": "object",
        "properties": {"path": {"type": "string"}},
        "required": ["path"],
        "additionalProperties": false
    });
    ToolDescriptor {
        id: "filesystem::write".to_owned(),
        provider_name: "filesystem_write".to_owned(),
        description: "Write one workspace file".to_owned(),
        source: ToolSource::Mcp,
        server: Some("filesystem".to_owned()),
        validator: Arc::new(jsonschema::validator_for(&schema).expect("tool schema compiles")),
        input_schema: schema,
        effect: ToolEffect::ExternalMutation,
        approval_class: ApprovalClass::Required,
        sandbox_required: false,
        concurrency_key: Some("workspace".to_owned()),
        exposure: Exposure::Eager,
        output_limit: None,
    }
}

async fn managed_runtime() -> (
    ToolAdmissionRuntime,
    Arc<RecordingManagedHost>,
    Arc<GovernanceEngine>,
) {
    let runtime_epoch = uuid::Uuid::new_v4().to_string();
    let host = Arc::new(RecordingManagedHost::new(&runtime_epoch));
    let governance = Arc::new(
        GovernanceEngine::with_default_permit().expect("explicit integration policy compiles"),
    );
    let context = ToolAdmissionContext::direct(
        "managed-owner".to_owned(),
        "authenticated-user".to_owned(),
        "/workspace/project".to_owned(),
        runtime_epoch,
        "catalog-revision-1".to_owned(),
        governance.policy_revision().await,
        host.binding(),
    );
    let host_port: Arc<dyn HostToolAdmissionPort> = host.clone();
    let runtime = ToolAdmissionRuntime::new(
        context,
        host_port,
        None,
        tokio_util::sync::CancellationToken::new(),
        Some(Arc::clone(&governance)),
        None,
    )
    .expect("managed admission runtime builds");
    (runtime, host, governance)
}

async fn admitted(runtime: &ToolAdmissionRuntime) -> AdmittedToolInvocation {
    let invocation = runtime.prepare(
        "model-call-1".to_owned(),
        &descriptor(),
        json!({"path": "output.txt"}),
        0,
    );
    let preparation = runtime
        .prepare_host(Arc::clone(&invocation))
        .await
        .expect("managed host prepares the exact effect");
    runtime
        .resolve(
            invocation,
            preparation,
            LocalAdmissionDisposition::Approved,
            true,
        )
        .await
        .expect("managed host resolves the exact effect")
        .expect("approved effect has an admission receipt")
}

#[tokio::test]
async fn managed_claim_is_persisted_before_the_effect_dispatches() {
    let (runtime, host, _) = managed_runtime().await;
    let admitted = admitted(&runtime).await;
    let dispatches = AtomicUsize::new(0);

    runtime
        .claim(&admitted)
        .await
        .expect("current authority admits the exact effect");
    assert_eq!(host.claim_revalidations.load(Ordering::SeqCst), 1);
    assert_eq!(dispatches.load(Ordering::SeqCst), 0);

    dispatches.fetch_add(1, Ordering::SeqCst);
    runtime
        .finish(&admitted, true)
        .await
        .expect("successful effect records its terminal receipt");
    assert_eq!(dispatches.load(Ordering::SeqCst), 1);
    assert_eq!(host.finished.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn managed_denial_revalidates_after_the_wait_and_dispatches_nothing() {
    let (runtime, host, _) = managed_runtime().await;
    let admitted = admitted(&runtime).await;
    let dispatches = AtomicUsize::new(0);
    host.deny_claim.store(true, Ordering::SeqCst);

    assert!(runtime.claim(&admitted).await.is_err());
    assert_eq!(host.claim_revalidations.load(Ordering::SeqCst), 1);
    assert_eq!(dispatches.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn changed_payload_or_authority_revision_is_rejected_before_dispatch() {
    let (runtime, _, _) = managed_runtime().await;
    let original = admitted(&runtime).await;
    let dispatches = AtomicUsize::new(0);

    let mut changed_payload = original.clone();
    let mut payload = (*changed_payload.prepared).clone();
    payload.validated_arguments = json!({"path": "different.txt"});
    changed_payload.prepared = Arc::new(payload);
    assert!(runtime.claim(&changed_payload).await.is_err());

    let mut changed_authority = original.clone();
    let mut authority = (*changed_authority.prepared).clone();
    authority.authority_revision = "sha256:forged".to_owned();
    changed_authority.prepared = Arc::new(authority);
    assert!(runtime.claim(&changed_authority).await.is_err());
    assert_eq!(dispatches.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn expired_lease_or_budget_is_rejected_before_dispatch() {
    let (runtime, _, _) = managed_runtime().await;
    let original = admitted(&runtime).await;
    let dispatches = AtomicUsize::new(0);

    let mut expired_lease = original.clone();
    let mut lease = (*expired_lease.prepared).clone();
    lease.lease.expires_at = 0;
    expired_lease.prepared = Arc::new(lease);
    assert!(runtime.claim(&expired_lease).await.is_err());

    let mut expired_budget = original.clone();
    let mut budget = (*expired_budget.prepared).clone();
    budget.budget_reservation.expires_at = 0;
    expired_budget.prepared = Arc::new(budget);
    assert!(runtime.claim(&expired_budget).await.is_err());
    assert_eq!(dispatches.load(Ordering::SeqCst), 0);
}

#[cfg(feature = "cedar-governance")]
#[tokio::test]
async fn governed_policy_loading_fails_closed_for_missing_empty_and_invalid_sources() {
    let root = tempfile::tempdir().expect("policy fixture directory");
    assert!(
        GovernanceEngine::load_from_dir(root.path().join("missing"))
            .await
            .is_err()
    );
    assert!(GovernanceEngine::load_from_dir(root.path()).await.is_err());

    std::fs::write(root.path().join("invalid.cedar"), "permit (")
        .expect("invalid Cedar fixture writes");
    assert!(GovernanceEngine::load_from_dir(root.path()).await.is_err());
}
