//! Exact, host-bound admission for one prepared tool invocation.
//!
//! The model's tool-call ID is retained only for event correlation. Execution
//! authority is the independently generated invocation ID plus the host's
//! admission receipt for the frozen call.

use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::uar::domain::artifact::AgentArtifact;
use crate::uar::domain::policy::EffectiveRunPolicy;
use crate::uar::tools::descriptor::{ApprovalClass, ToolDescriptor, ToolEffect, ToolSource};

mod approval_class_wire;
mod http;
mod lifecycle;
mod standalone;

pub use http::{HttpHostToolAdmissionPort, RunToolAdmissionInput};
pub use lifecycle::{
    AdmissionCancellationOutcome, AdmissionCancellationReason, AdmissionTerminalOutcome,
};
pub use standalone::StandaloneToolAdmissionPort;

pub const TOOL_ADMISSION_PROTOCOL_VERSION: u32 = 2;
pub const TOOL_ADMISSION_META_KEY: &str = "tools.know-me.the-boss/admission";
pub const DEFAULT_EFFECT_AUTHORITY_TTL_SECONDS: u64 = 300;

/// Executor ownership derived from the resolved tool source, never model input.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolExecutionKind {
    RuntimeNative,
    HostMcp,
}

impl ToolExecutionKind {
    pub const fn from_source(source: ToolSource) -> Self {
        match source {
            ToolSource::NativeSkill | ToolSource::BuiltIn => Self::RuntimeNative,
            ToolSource::Mcp => Self::HostMcp,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HostAdmissionBinding {
    pub version: u32,
    pub host_epoch: String,
}

/// Run-owned identities and revisions copied into every prepared invocation.
#[derive(Debug, Clone)]
pub struct ToolAdmissionContext {
    pub root_run_id: String,
    pub executing_run_id: String,
    pub owner_id: String,
    pub principal_id: String,
    pub workspace: String,
    pub runtime_epoch: String,
    pub catalog_revision: String,
    pub run_policy_revision: String,
    pub governance_policy_revision: String,
    pub grant_revision: String,
    pub lease_revision: String,
    pub budget_revision: String,
    pub lease_id: String,
    pub lease_expires_at: u64,
    pub budget_id: String,
    pub budget_expires_at: u64,
    pub host: HostAdmissionBinding,
}

impl ToolAdmissionContext {
    /// Capture one immutable run context from host-resolved inputs.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        root_run_id: String,
        executing_run_id: String,
        owner_id: String,
        principal_id: String,
        workspace: String,
        runtime_epoch: String,
        artifact: &AgentArtifact,
        policy: &EffectiveRunPolicy,
        governance_policy_revision: String,
        host: HostAdmissionBinding,
    ) -> anyhow::Result<Self> {
        let catalog_revision = artifact.catalog_metadata().map_or_else(
            || artifact.definition_revision(),
            |metadata| metadata.revision,
        );
        let run_policy_revision = digest_json(&serde_json::to_value(policy)?);
        let grant_revision = digest_json(&serde_json::json!({
            "tools": &policy.tools,
            "mcpServers": &policy.mcp_servers,
            "toolApproval": policy.tool_approval,
        }));
        let lease_revision = digest_json(&serde_json::json!({
            "rootRunId": &root_run_id,
            "executingRunId": &executing_run_id,
            "ownerId": &owner_id,
            "workspace": &workspace,
            "runtimeEpoch": &runtime_epoch,
            "hostEpoch": &host.host_epoch,
        }));
        let budget_revision =
            digest_json(artifact.extensions.get("budgets").unwrap_or(&Value::Null));
        let authority_ttl = artifact
            .extensions
            .get("budgets")
            .and_then(|value| value.get("timeout_seconds"))
            .and_then(Value::as_u64)
            .unwrap_or(DEFAULT_EFFECT_AUTHORITY_TTL_SECONDS);
        let budget_expires_at = expiry_from_now(authority_ttl)?;
        let lease_id = format!("uar-lease:{}", Uuid::new_v4());
        let budget_id = format!("uar-budget:{root_run_id}");
        Ok(Self {
            root_run_id,
            executing_run_id,
            owner_id,
            principal_id,
            workspace,
            runtime_epoch,
            catalog_revision,
            run_policy_revision,
            governance_policy_revision,
            grant_revision,
            lease_revision,
            budget_revision,
            lease_id,
            lease_expires_at: budget_expires_at,
            budget_id,
            budget_expires_at,
            host,
        })
    }

    /// Build the explicit authority envelope for an authenticated direct host
    /// call. This path has no agent artifact, delegated grant, or inherited
    /// budget; those absences are bound as named revisions rather than implied.
    pub fn direct(
        owner_id: String,
        principal_id: String,
        workspace: String,
        runtime_epoch: String,
        catalog_revision: String,
        governance_policy_revision: String,
        host: HostAdmissionBinding,
    ) -> Self {
        let run_id = format!("direct:{}", Uuid::new_v4());
        let lease_revision = digest_json(&serde_json::json!({
            "runId": &run_id,
            "ownerId": &owner_id,
            "workspace": &workspace,
            "runtimeEpoch": &runtime_epoch,
            "hostEpoch": &host.host_epoch,
        }));
        let grant_revision = digest_json(&serde_json::json!({
            "kind": "authenticated_direct_host",
            "ownerId": &owner_id,
            "principalId": &principal_id,
        }));
        let lease_id = format!("uar-lease:direct:{}", Uuid::new_v4());
        let budget_id = format!("uar-budget:{run_id}");
        let expires_at = default_effect_authority_expiry();
        Self {
            root_run_id: run_id.clone(),
            executing_run_id: run_id,
            owner_id,
            principal_id,
            workspace,
            runtime_epoch,
            catalog_revision,
            run_policy_revision: "direct:host".to_string(),
            governance_policy_revision,
            grant_revision,
            lease_revision,
            budget_revision: digest_json(&serde_json::json!({"kind": "direct:no_budget"})),
            lease_id,
            lease_expires_at: expires_at,
            budget_id,
            budget_expires_at: expires_at,
            host,
        }
    }

    /// Freeze a validated call after catalog resolution and before governance.
    pub fn prepare(
        &self,
        model_tool_call_id: String,
        descriptor: &ToolDescriptor,
        validated_arguments: Value,
        call_index: usize,
    ) -> PreparedToolInvocation {
        let invocation_id = Uuid::new_v4().to_string();
        let execution_kind = ToolExecutionKind::from_source(descriptor.source);
        let mounted_server_id = descriptor
            .server
            .clone()
            .unwrap_or_else(|| source_identity(descriptor.source).to_string());
        let native_tool_name = descriptor
            .server
            .as_deref()
            .and_then(|server| descriptor.id.strip_prefix(&format!("{server}::")))
            .unwrap_or(&descriptor.id)
            .to_string();
        let tool_policy_revision = digest_json(&serde_json::json!({
            "descriptorId": &descriptor.id,
            "providerName": &descriptor.provider_name,
            "source": source_identity(descriptor.source),
            "server": &descriptor.server,
            "effect": effect_identity(descriptor.effect),
            "approval": approval_identity(descriptor.approval_class),
            "sandboxRequired": descriptor.sandbox_required,
            "inputSchema": &descriptor.input_schema,
            "concurrencyKey": &descriptor.concurrency_key,
            "exposure": format!("{:?}", descriptor.exposure),
        }));
        let resource_revision = digest_json(&serde_json::json!({
            "mountedServerId": &mounted_server_id,
            "nativeToolName": &native_tool_name,
            "providerToolName": &descriptor.provider_name,
            "catalogRevision": &self.catalog_revision,
            "toolPolicyRevision": &tool_policy_revision,
        }));
        let payload_revision = digest_json(&validated_arguments);
        let lease = EffectLeaseFacts {
            lease_id: self.lease_id.clone(),
            task: descriptor.id.clone(),
            attempt: 1,
            epoch: self.runtime_epoch.clone(),
            holder: self.principal_id.clone(),
            expires_at: self.lease_expires_at,
            active: true,
        };
        let budget_reservation = EffectBudgetReservationFacts {
            reservation_id: format!("uar-budget-reservation:{invocation_id}"),
            budget_id: self.budget_id.clone(),
            revision: self.budget_revision.clone(),
            amount: 1,
            unit: "tool_call".to_string(),
            expires_at: self.budget_expires_at,
            active: true,
        };
        let authority_revision = digest_json(&serde_json::json!({
            "executionKind": execution_kind,
            "principalId": &self.principal_id,
            "ownerId": &self.owner_id,
            "rootRunId": &self.root_run_id,
            "executingRunId": &self.executing_run_id,
            "runtimeEpoch": &self.runtime_epoch,
            "hostEpoch": &self.host.host_epoch,
            "catalogRevision": &self.catalog_revision,
            "runPolicyRevision": &self.run_policy_revision,
            "expectedGovernancePolicyRevision": &self.governance_policy_revision,
            "toolPolicyRevision": &tool_policy_revision,
            "resourceRevision": &resource_revision,
            "payloadRevision": &payload_revision,
            "grantRevision": &self.grant_revision,
            "leaseRevision": &self.lease_revision,
            "budgetRevision": &self.budget_revision,
            "lease": &lease,
            "budgetReservation": &budget_reservation,
        }));
        PreparedToolInvocation {
            version: TOOL_ADMISSION_PROTOCOL_VERSION,
            execution_kind,
            invocation_id,
            model_tool_call_id,
            attempt: 1,
            root_run_id: self.root_run_id.clone(),
            executing_run_id: self.executing_run_id.clone(),
            owner_id: self.owner_id.clone(),
            principal_id: self.principal_id.clone(),
            workspace: self.workspace.clone(),
            runtime_epoch: self.runtime_epoch.clone(),
            host_epoch: self.host.host_epoch.clone(),
            catalog_revision: self.catalog_revision.clone(),
            mounted_server_id,
            native_tool_name,
            provider_tool_name: descriptor.provider_name.clone(),
            run_policy_revision: self.run_policy_revision.clone(),
            governance_policy_revision: self.governance_policy_revision.clone(),
            tool_policy_revision,
            resource_revision,
            payload_revision,
            grant_revision: self.grant_revision.clone(),
            lease_revision: self.lease_revision.clone(),
            budget_revision: self.budget_revision.clone(),
            lease,
            budget_reservation,
            authority_revision,
            approval_class: descriptor.approval_class,
            call_index,
            validated_arguments,
        }
    }
}

/// UAR-owned execution lease forwarded by the paired host to the governed
/// effect authority. Gate evaluates it but does not mint or mutate it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EffectLeaseFacts {
    pub lease_id: String,
    pub task: String,
    pub attempt: u32,
    pub epoch: String,
    pub holder: String,
    pub expires_at: u64,
    pub active: bool,
}

/// UAR-owned reservation for the exact effect attempt. One admitted tool call
/// consumes one tool-call unit under the bound UAR budget revision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EffectBudgetReservationFacts {
    pub reservation_id: String,
    pub budget_id: String,
    pub revision: String,
    pub amount: u64,
    pub unit: String,
    pub expires_at: u64,
    pub active: bool,
}

/// One exact call after resolution and validation, before any authority admits it.
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreparedToolInvocation {
    pub version: u32,
    pub execution_kind: ToolExecutionKind,
    pub invocation_id: String,
    pub model_tool_call_id: String,
    pub attempt: u32,
    pub root_run_id: String,
    pub executing_run_id: String,
    pub owner_id: String,
    pub principal_id: String,
    pub workspace: String,
    pub runtime_epoch: String,
    pub host_epoch: String,
    pub catalog_revision: String,
    pub mounted_server_id: String,
    pub native_tool_name: String,
    pub provider_tool_name: String,
    pub run_policy_revision: String,
    #[serde(rename = "expectedGovernancePolicyRevision")]
    pub governance_policy_revision: String,
    pub tool_policy_revision: String,
    pub resource_revision: String,
    pub payload_revision: String,
    pub grant_revision: String,
    pub lease_revision: String,
    pub budget_revision: String,
    pub lease: EffectLeaseFacts,
    pub budget_reservation: EffectBudgetReservationFacts,
    pub authority_revision: String,
    #[serde(with = "approval_class_wire")]
    pub approval_class: ApprovalClass,
    pub call_index: usize,
    pub validated_arguments: Value,
}

impl std::fmt::Debug for PreparedToolInvocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PreparedToolInvocation")
            .field("version", &self.version)
            .field("execution_kind", &self.execution_kind)
            .field("invocation_id", &self.invocation_id)
            .field("model_tool_call_id", &self.model_tool_call_id)
            .field("root_run_id", &self.root_run_id)
            .field("executing_run_id", &self.executing_run_id)
            .field("owner_id", &self.owner_id)
            .field("principal_id", &self.principal_id)
            .field("workspace", &self.workspace)
            .field("runtime_epoch", &self.runtime_epoch)
            .field("host_epoch", &self.host_epoch)
            .field("catalog_revision", &self.catalog_revision)
            .field("mounted_server_id", &self.mounted_server_id)
            .field("native_tool_name", &self.native_tool_name)
            .field("provider_tool_name", &self.provider_tool_name)
            .field("run_policy_revision", &self.run_policy_revision)
            .field(
                "governance_policy_revision",
                &self.governance_policy_revision,
            )
            .field("tool_policy_revision", &self.tool_policy_revision)
            .field("resource_revision", &self.resource_revision)
            .field("payload_revision", &self.payload_revision)
            .field("grant_revision", &self.grant_revision)
            .field("lease_revision", &self.lease_revision)
            .field("budget_revision", &self.budget_revision)
            .field("lease", &self.lease)
            .field("budget_reservation", &self.budget_reservation)
            .field("authority_revision", &self.authority_revision)
            .field("approval_class", &self.approval_class)
            .field("call_index", &self.call_index)
            .field("validated_arguments", &"[protected]")
            .finish()
    }
}

impl PreparedToolInvocation {
    pub fn validate_authority_envelope(&self) -> anyhow::Result<()> {
        anyhow::ensure!(
            self.version == TOOL_ADMISSION_PROTOCOL_VERSION,
            "Unsupported tool admission protocol version"
        );
        for value in [
            &self.invocation_id,
            &self.root_run_id,
            &self.executing_run_id,
            &self.owner_id,
            &self.principal_id,
            &self.workspace,
            &self.runtime_epoch,
            &self.host_epoch,
            &self.catalog_revision,
            &self.mounted_server_id,
            &self.native_tool_name,
            &self.provider_tool_name,
            &self.run_policy_revision,
            &self.governance_policy_revision,
            &self.tool_policy_revision,
            &self.resource_revision,
            &self.payload_revision,
            &self.grant_revision,
            &self.lease_revision,
            &self.budget_revision,
            &self.lease.lease_id,
            &self.lease.task,
            &self.lease.epoch,
            &self.lease.holder,
            &self.budget_reservation.reservation_id,
            &self.budget_reservation.budget_id,
            &self.budget_reservation.revision,
            &self.budget_reservation.unit,
            &self.authority_revision,
        ] {
            anyhow::ensure!(
                !value.trim().is_empty(),
                "Tool authority envelope is incomplete"
            );
        }
        anyhow::ensure!(
            digest_json(&self.validated_arguments) == self.payload_revision,
            "Tool payload no longer matches its authority binding"
        );
        anyhow::ensure!(
            self.lease.active
                && self.lease.attempt == self.attempt
                && self.lease.epoch == self.runtime_epoch
                && self.lease.holder == self.principal_id
                && self.budget_reservation.active
                && self.budget_reservation.amount == 1
                && self.budget_reservation.unit == "tool_call"
                && self.budget_reservation.revision == self.budget_revision,
            "Tool lease or budget reservation does not match its authority binding"
        );
        let now = unix_time()?;
        anyhow::ensure!(
            now < self.lease.expires_at && now < self.budget_reservation.expires_at,
            "Tool lease or budget reservation expired before claim"
        );
        let expected_authority = digest_json(&serde_json::json!({
            "executionKind": self.execution_kind,
            "principalId": &self.principal_id,
            "ownerId": &self.owner_id,
            "rootRunId": &self.root_run_id,
            "executingRunId": &self.executing_run_id,
            "runtimeEpoch": &self.runtime_epoch,
            "hostEpoch": &self.host_epoch,
            "catalogRevision": &self.catalog_revision,
            "runPolicyRevision": &self.run_policy_revision,
            "expectedGovernancePolicyRevision": &self.governance_policy_revision,
            "toolPolicyRevision": &self.tool_policy_revision,
            "resourceRevision": &self.resource_revision,
            "payloadRevision": &self.payload_revision,
            "grantRevision": &self.grant_revision,
            "leaseRevision": &self.lease_revision,
            "budgetRevision": &self.budget_revision,
            "lease": &self.lease,
            "budgetReservation": &self.budget_reservation,
        }));
        anyhow::ensure!(
            expected_authority == self.authority_revision,
            "Tool authority envelope digest does not match its fields"
        );
        Ok(())
    }
}

/// Result of UAR's local governance and, when required, human decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LocalAdmissionDisposition {
    Allowed,
    Approved,
    GovernanceBypassed,
    Denied,
}

/// Restriction contributed by the paired host for one prepared invocation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HostAdmissionDisposition {
    Auto,
    Ask,
    Deny,
}

/// Host correlation allocated before local governance can ask a human.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HostAdmissionPreparation {
    pub version: u32,
    pub execution_kind: ToolExecutionKind,
    pub admission_id: String,
    pub invocation_id: String,
    pub runtime_epoch: String,
    pub host_epoch: String,
    #[serde(default)]
    pub authority_revision: String,
    pub host_disposition: HostAdmissionDisposition,
    /// Host-created safe renderer projection. It is never execution input.
    pub action_display: Value,
    #[serde(default)]
    pub managed_mcp_metadata: bool,
}

/// Input to the local approval gate after the host has allocated exact
/// correlation. The opaque admission ID is display/decision correlation only.
#[derive(Debug, Clone)]
pub struct ToolApprovalRequest {
    pub invocation: Arc<PreparedToolInvocation>,
    pub admission_id: String,
    pub host_requires_approval: bool,
    pub action_display: Value,
}

/// Exact host receipt carried to dispatch with the prepared invocation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HostAdmissionReceipt {
    pub version: u32,
    pub execution_kind: ToolExecutionKind,
    pub admission_id: String,
    pub invocation_id: String,
    pub runtime_epoch: String,
    pub host_epoch: String,
    #[serde(default)]
    pub authority_revision: String,
    #[serde(default)]
    pub managed_mcp_metadata: bool,
}

impl HostAdmissionReceipt {
    /// MCP request metadata for the private managed bridge. Independent MCP
    /// servers and standalone UAR receive no Boss-specific extension.
    pub fn mcp_request_meta(&self) -> Option<rmcp::model::RequestMetaObject> {
        self.managed_mcp_metadata.then(|| {
            let mut values = serde_json::Map::new();
            values.insert(
                TOOL_ADMISSION_META_KEY.to_string(),
                serde_json::json!({
                    "version": self.version,
                    "executionKind": self.execution_kind,
                    "admissionId": self.admission_id,
                    "invocationId": self.invocation_id,
                    "runtimeEpoch": self.runtime_epoch,
                    "hostEpoch": self.host_epoch,
                    "authorityRevision": self.authority_revision,
                }),
            );
            rmcp::model::RequestMetaObject::from(values)
        })
    }
}

/// A prepared invocation admitted by both the local runtime and its host port.
#[derive(Clone)]
pub struct AdmittedToolInvocation {
    pub prepared: Arc<PreparedToolInvocation>,
    pub local_disposition: LocalAdmissionDisposition,
    pub host_receipt: HostAdmissionReceipt,
}

impl std::fmt::Debug for AdmittedToolInvocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AdmittedToolInvocation")
            .field("prepared", &self.prepared)
            .field("local_disposition", &self.local_disposition)
            .field("host_receipt", &self.host_receipt)
            .finish()
    }
}

#[async_trait]
pub trait HostToolAdmissionPort: Send + Sync + std::fmt::Debug {
    fn binding(&self) -> HostAdmissionBinding;

    async fn prepare(
        &self,
        invocation: Arc<PreparedToolInvocation>,
    ) -> anyhow::Result<HostAdmissionPreparation>;

    async fn resolve(
        &self,
        invocation: Arc<PreparedToolInvocation>,
        preparation: HostAdmissionPreparation,
        local_disposition: LocalAdmissionDisposition,
        approved: bool,
    ) -> anyhow::Result<Option<AdmittedToolInvocation>>;

    /// Re-evaluate host policy, grants, lease, budget, approval, and the exact
    /// payload immediately before UAR persists claim intent.
    async fn revalidate_claim(
        &self,
        admitted: &AdmittedToolInvocation,
    ) -> anyhow::Result<HostAdmissionReceipt>;

    /// Consume native authority after UAR persists claim intent. A repeated
    /// claim cannot acknowledge another execution of the same invocation.
    async fn consume_native(
        &self,
        admitted: &AdmittedToolInvocation,
    ) -> anyhow::Result<HostAdmissionReceipt>;

    async fn cancel(
        &self,
        invocation: &PreparedToolInvocation,
        admission_id: &str,
        reason: AdmissionCancellationReason,
    ) -> anyhow::Result<AdmissionCancellationOutcome>;

    async fn finish(
        &self,
        invocation: &PreparedToolInvocation,
        admission_id: &str,
        outcome: AdmissionTerminalOutcome,
    ) -> anyhow::Result<()>;
}

#[async_trait]
pub trait ClaimRevalidator: Send + Sync + std::fmt::Debug {
    async fn revalidate(&self) -> anyhow::Result<()>;
}

pub struct ToolAdmissionRuntime {
    context: Arc<ToolAdmissionContext>,
    host: Arc<dyn HostToolAdmissionPort>,
    lifecycle: Arc<lifecycle::AdmissionLifecycle>,
    cancellation: tokio_util::sync::CancellationToken,
    governance_engine: Option<Arc<crate::uar::governance::engine::GovernanceEngine>>,
    governance_gate: Option<crate::uar::governance::runtime_control::GovernanceGateHandle>,
    claim_revalidators: Vec<Arc<dyn ClaimRevalidator>>,
}

impl std::fmt::Debug for ToolAdmissionRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ToolAdmissionRuntime")
            .field("context", &self.context)
            .field("host", &self.host.binding())
            .finish()
    }
}

impl ToolAdmissionRuntime {
    /// Observe the existing run cancellation at the final executor boundary.
    pub fn is_cancelled(&self) -> bool {
        self.cancellation.is_cancelled()
    }

    #[must_use]
    pub fn standalone_ephemeral() -> Self {
        let runtime_epoch = Uuid::new_v4().to_string();
        let host: Arc<dyn HostToolAdmissionPort> =
            Arc::new(StandaloneToolAdmissionPort::new(&runtime_epoch));
        let binding = host.binding();
        let lease_id = format!("uar-lease:standalone:{runtime_epoch}");
        let budget_id = format!("uar-budget:standalone:{runtime_epoch}");
        let expires_at = default_effect_authority_expiry();
        Self {
            context: Arc::new(ToolAdmissionContext {
                root_run_id: format!("standalone:{runtime_epoch}"),
                executing_run_id: format!("standalone:{runtime_epoch}"),
                owner_id: "standalone".to_string(),
                principal_id: "standalone".to_string(),
                workspace: std::env::current_dir().map_or_else(
                    |_| "standalone".to_string(),
                    |path| path.display().to_string(),
                ),
                runtime_epoch,
                catalog_revision: "standalone".to_string(),
                run_policy_revision: "standalone".to_string(),
                governance_policy_revision: "standalone".to_string(),
                grant_revision: "standalone".to_string(),
                lease_revision: "standalone".to_string(),
                budget_revision: "standalone".to_string(),
                lease_id,
                lease_expires_at: expires_at,
                budget_id,
                budget_expires_at: expires_at,
                host: binding,
            }),
            host,
            lifecycle: Arc::new(lifecycle::AdmissionLifecycle::ephemeral()),
            cancellation: tokio_util::sync::CancellationToken::new(),
            governance_engine: None,
            governance_gate: None,
            claim_revalidators: Vec::new(),
        }
    }

    pub fn new(
        context: ToolAdmissionContext,
        host: Arc<dyn HostToolAdmissionPort>,
        persistence: Option<Arc<dyn crate::uar::persistence::PersistenceLayer>>,
        cancellation: tokio_util::sync::CancellationToken,
        governance_engine: Option<Arc<crate::uar::governance::engine::GovernanceEngine>>,
        governance_gate: Option<crate::uar::governance::runtime_control::GovernanceGateHandle>,
    ) -> anyhow::Result<Self> {
        anyhow::ensure!(
            context.host.version == TOOL_ADMISSION_PROTOCOL_VERSION
                && context.host == host.binding(),
            "Tool admission context belongs to another host binding"
        );
        Ok(Self {
            context: Arc::new(context),
            host,
            lifecycle: Arc::new(lifecycle::AdmissionLifecycle::new(persistence)),
            cancellation,
            governance_engine,
            governance_gate,
            claim_revalidators: Vec::new(),
        })
    }

    #[must_use]
    pub fn with_claim_revalidator(mut self, claim_revalidator: Arc<dyn ClaimRevalidator>) -> Self {
        self.claim_revalidators.push(claim_revalidator);
        self
    }

    /// Freeze a validated invocation under this run's captured identities.
    pub fn prepare(
        &self,
        model_tool_call_id: String,
        descriptor: &ToolDescriptor,
        validated_arguments: Value,
        call_index: usize,
    ) -> Arc<PreparedToolInvocation> {
        Arc::new(self.context.prepare(
            model_tool_call_id,
            descriptor,
            validated_arguments,
            call_index,
        ))
    }

    /// Allocate host correlation before local governance may publish a prompt.
    pub async fn prepare_host(
        &self,
        invocation: Arc<PreparedToolInvocation>,
    ) -> anyhow::Result<HostAdmissionPreparation> {
        invocation.validate_authority_envelope()?;
        self.lifecycle.reconcile(&self.context).await?;
        let prepared = self.host.prepare(invocation.clone()).await?;
        anyhow::ensure!(
            prepared.version == invocation.version
                && prepared.execution_kind == invocation.execution_kind
                && prepared.invocation_id == invocation.invocation_id
                && prepared.runtime_epoch == invocation.runtime_epoch
                && prepared.host_epoch == invocation.host_epoch
                && prepared.authority_revision == invocation.authority_revision,
            "Host preparation does not match the prepared invocation"
        );
        if let Err(error) = self
            .lifecycle
            .record_prepared(invocation.as_ref(), &prepared)
            .await
        {
            let cancellation = self
                .host
                .cancel(
                    invocation.as_ref(),
                    &prepared.admission_id,
                    AdmissionCancellationReason::Invalidated,
                )
                .await;
            return Err(match cancellation {
                Ok(_) => error.context("Tool admission preparation was not persisted"),
                Err(cancel_error) => error.context(format!(
                    "Tool admission preparation was not persisted and host invalidation failed: {cancel_error}"
                )),
            });
        }
        self.lifecycle.watch_cancellation(
            Arc::clone(&self.host),
            invocation,
            prepared.admission_id.clone(),
            self.cancellation.clone(),
        );
        Ok(prepared)
    }

    /// Resolve local governance against the exact host preparation.
    pub async fn resolve(
        &self,
        invocation: Arc<PreparedToolInvocation>,
        preparation: HostAdmissionPreparation,
        local_disposition: LocalAdmissionDisposition,
        approved: bool,
    ) -> anyhow::Result<Option<AdmittedToolInvocation>> {
        invocation.validate_authority_envelope()?;
        let admission_id = preparation.admission_id.clone();
        let admitted = self
            .host
            .resolve(invocation.clone(), preparation, local_disposition, approved)
            .await?;
        let Some(admitted) = admitted else {
            anyhow::ensure!(!approved, "Host omitted an approved admission receipt");
            self.reject(invocation.as_ref(), &admission_id).await?;
            return Ok(None);
        };
        let receipt = &admitted.host_receipt;
        anyhow::ensure!(
            Arc::ptr_eq(&admitted.prepared, &invocation)
                && receipt.version == invocation.version
                && receipt.execution_kind == invocation.execution_kind
                && receipt.admission_id == admission_id
                && receipt.invocation_id == invocation.invocation_id
                && receipt.runtime_epoch == invocation.runtime_epoch
                && receipt.host_epoch == invocation.host_epoch
                && receipt.authority_revision == invocation.authority_revision,
            "Host admission receipt does not match the prepared invocation"
        );
        Ok(Some(admitted))
    }
}

fn digest_json(value: &Value) -> String {
    let digest = Sha256::digest(canonical_json(value).as_bytes());
    let encoded = digest
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    format!("sha256:{encoded}")
}

fn unix_time() -> anyhow::Result<u64> {
    Ok(std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|_| anyhow::anyhow!("Runtime clock is before the Unix epoch"))?
        .as_secs())
}

fn expiry_from_now(seconds: u64) -> anyhow::Result<u64> {
    unix_time()?
        .checked_add(seconds)
        .ok_or_else(|| anyhow::anyhow!("Tool authority expiry exceeds the runtime clock range"))
}

fn default_effect_authority_expiry() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(DEFAULT_EFFECT_AUTHORITY_TTL_SECONDS, |now| {
            now.as_secs()
                .saturating_add(DEFAULT_EFFECT_AUTHORITY_TTL_SECONDS)
        })
}

fn canonical_json(value: &Value) -> String {
    match value {
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => value.to_string(),
        Value::Array(values) => format!(
            "[{}]",
            values
                .iter()
                .map(canonical_json)
                .collect::<Vec<_>>()
                .join(",")
        ),
        Value::Object(values) => {
            let mut entries = values.iter().collect::<Vec<_>>();
            entries.sort_unstable_by(|(left, _), (right, _)| left.cmp(right));
            format!(
                "{{{}}}",
                entries
                    .into_iter()
                    .map(|(key, value)| {
                        format!("{}:{}", Value::String(key.clone()), canonical_json(value))
                    })
                    .collect::<Vec<_>>()
                    .join(",")
            )
        }
    }
}

const fn source_identity(source: ToolSource) -> &'static str {
    match source {
        ToolSource::NativeSkill => "native_skill",
        ToolSource::Mcp => "mcp",
        ToolSource::BuiltIn => "builtin",
    }
}

const fn effect_identity(effect: ToolEffect) -> &'static str {
    match effect {
        ToolEffect::ReadOnly => "read_only",
        ToolEffect::ExternalMutation => "external_mutation",
        ToolEffect::CodeExecution => "code_execution",
        ToolEffect::Unknown => "unknown",
    }
}

const fn approval_identity(approval: ApprovalClass) -> &'static str {
    match approval {
        ApprovalClass::NotRequired => "not_required",
        ApprovalClass::Required => "required",
    }
}
