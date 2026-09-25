//! Exact, host-bound admission for one prepared tool invocation.
//!
//! The model's tool-call ID is retained only for event correlation. Execution
//! authority is the independently generated invocation ID plus the host's
//! admission receipt for the frozen call.

use std::sync::Arc;

use anyhow::Context;
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

pub const TOOL_ADMISSION_PROTOCOL_VERSION: u32 = 1;
pub const TOOL_ADMISSION_META_KEY: &str = "tools.know-me.the-boss/admission";

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
    pub workspace: String,
    pub runtime_epoch: String,
    pub catalog_revision: String,
    pub run_policy_revision: String,
    pub host: HostAdmissionBinding,
}

impl ToolAdmissionContext {
    /// Capture one immutable run context from host-resolved inputs.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        root_run_id: String,
        executing_run_id: String,
        owner_id: String,
        workspace: String,
        runtime_epoch: String,
        artifact: &AgentArtifact,
        policy: &EffectiveRunPolicy,
        host: HostAdmissionBinding,
    ) -> anyhow::Result<Self> {
        let catalog_revision = artifact
            .catalog_metadata()
            .map_or_else(|| artifact.definition_revision(), |metadata| metadata.revision);
        let run_policy_revision = digest_json(&serde_json::to_value(policy)?);
        Ok(Self {
            root_run_id,
            executing_run_id,
            owner_id,
            workspace,
            runtime_epoch,
            catalog_revision,
            run_policy_revision,
            host,
        })
    }

    /// Freeze a validated call after catalog resolution and before governance.
    pub fn prepare(
        &self,
        model_tool_call_id: String,
        descriptor: &ToolDescriptor,
        validated_arguments: Value,
        call_index: usize,
    ) -> PreparedToolInvocation {
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
            "descriptorId": descriptor.id,
            "providerName": descriptor.provider_name,
            "source": source_identity(descriptor.source),
            "server": descriptor.server,
            "effect": effect_identity(descriptor.effect),
            "approval": approval_identity(descriptor.approval_class),
            "sandboxRequired": descriptor.sandbox_required,
            "inputSchema": descriptor.input_schema,
        }));
        PreparedToolInvocation {
            version: TOOL_ADMISSION_PROTOCOL_VERSION,
            invocation_id: Uuid::new_v4().to_string(),
            model_tool_call_id,
            attempt: 1,
            root_run_id: self.root_run_id.clone(),
            executing_run_id: self.executing_run_id.clone(),
            owner_id: self.owner_id.clone(),
            workspace: self.workspace.clone(),
            runtime_epoch: self.runtime_epoch.clone(),
            host_epoch: self.host.host_epoch.clone(),
            catalog_revision: self.catalog_revision.clone(),
            mounted_server_id,
            native_tool_name,
            provider_tool_name: descriptor.provider_name.clone(),
            run_policy_revision: self.run_policy_revision.clone(),
            tool_policy_revision,
            approval_class: descriptor.approval_class,
            call_index,
            validated_arguments,
        }
    }
}

/// One exact call after resolution and validation, before any authority admits it.
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreparedToolInvocation {
    pub version: u32,
    pub invocation_id: String,
    pub model_tool_call_id: String,
    pub attempt: u32,
    pub root_run_id: String,
    pub executing_run_id: String,
    pub owner_id: String,
    pub workspace: String,
    pub runtime_epoch: String,
    pub host_epoch: String,
    pub catalog_revision: String,
    pub mounted_server_id: String,
    pub native_tool_name: String,
    pub provider_tool_name: String,
    pub run_policy_revision: String,
    pub tool_policy_revision: String,
    #[serde(with = "approval_class_wire")]
    pub approval_class: ApprovalClass,
    pub call_index: usize,
    pub validated_arguments: Value,
}

impl std::fmt::Debug for PreparedToolInvocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PreparedToolInvocation")
            .field("version", &self.version)
            .field("invocation_id", &self.invocation_id)
            .field("model_tool_call_id", &self.model_tool_call_id)
            .field("root_run_id", &self.root_run_id)
            .field("executing_run_id", &self.executing_run_id)
            .field("owner_id", &self.owner_id)
            .field("workspace", &self.workspace)
            .field("runtime_epoch", &self.runtime_epoch)
            .field("host_epoch", &self.host_epoch)
            .field("catalog_revision", &self.catalog_revision)
            .field("mounted_server_id", &self.mounted_server_id)
            .field("native_tool_name", &self.native_tool_name)
            .field("provider_tool_name", &self.provider_tool_name)
            .field("run_policy_revision", &self.run_policy_revision)
            .field("tool_policy_revision", &self.tool_policy_revision)
            .field("approval_class", &self.approval_class)
            .field("call_index", &self.call_index)
            .field("validated_arguments", &"[protected]")
            .finish()
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
    pub admission_id: String,
    pub invocation_id: String,
    pub runtime_epoch: String,
    pub host_epoch: String,
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
    pub admission_id: String,
    pub invocation_id: String,
    pub runtime_epoch: String,
    pub host_epoch: String,
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
                    "admissionId": self.admission_id,
                    "invocationId": self.invocation_id,
                    "runtimeEpoch": self.runtime_epoch,
                    "hostEpoch": self.host_epoch,
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

pub struct ToolAdmissionRuntime {
    context: Arc<ToolAdmissionContext>,
    host: Arc<dyn HostToolAdmissionPort>,
    lifecycle: Arc<lifecycle::AdmissionLifecycle>,
    cancellation: tokio_util::sync::CancellationToken,
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
    #[must_use]
    pub fn standalone_ephemeral() -> Self {
        let runtime_epoch = Uuid::new_v4().to_string();
        let host: Arc<dyn HostToolAdmissionPort> =
            Arc::new(StandaloneToolAdmissionPort::new(&runtime_epoch));
        let binding = host.binding();
        Self {
            context: Arc::new(ToolAdmissionContext {
                root_run_id: format!("standalone:{runtime_epoch}"),
                executing_run_id: format!("standalone:{runtime_epoch}"),
                owner_id: "standalone".to_string(),
                workspace: std::env::current_dir()
                    .map_or_else(|_| "standalone".to_string(), |path| path.display().to_string()),
                runtime_epoch,
                catalog_revision: "standalone".to_string(),
                run_policy_revision: "standalone".to_string(),
                host: binding,
            }),
            host,
            lifecycle: Arc::new(lifecycle::AdmissionLifecycle::ephemeral()),
            cancellation: tokio_util::sync::CancellationToken::new(),
        }
    }

    pub fn new(
        context: ToolAdmissionContext,
        host: Arc<dyn HostToolAdmissionPort>,
        persistence: Option<Arc<dyn crate::uar::persistence::PersistenceLayer>>,
        cancellation: tokio_util::sync::CancellationToken,
    ) -> anyhow::Result<Self> {
        anyhow::ensure!(
            context.host == host.binding(),
            "Tool admission context belongs to another host binding"
        );
        Ok(Self {
            context: Arc::new(context),
            host,
            lifecycle: Arc::new(lifecycle::AdmissionLifecycle::new(persistence)),
            cancellation,
        })
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
        self.lifecycle.reconcile(&self.context).await?;
        let prepared = self.host.prepare(invocation.clone()).await?;
        anyhow::ensure!(
            prepared.version == invocation.version
                && prepared.invocation_id == invocation.invocation_id
                && prepared.runtime_epoch == invocation.runtime_epoch
                && prepared.host_epoch == invocation.host_epoch,
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
        let admission_id = preparation.admission_id.clone();
        let admitted = self
            .host
            .resolve(
                invocation.clone(),
                preparation,
                local_disposition,
                approved,
            )
            .await?;
        let Some(admitted) = admitted else {
            anyhow::ensure!(!approved, "Host omitted an approved admission receipt");
            self.reject(invocation.as_ref(), &admission_id)
                .await?;
            return Ok(None);
        };
        let receipt = &admitted.host_receipt;
        anyhow::ensure!(
            Arc::ptr_eq(&admitted.prepared, &invocation)
                && receipt.version == invocation.version
                && receipt.invocation_id == invocation.invocation_id
                && receipt.runtime_epoch == invocation.runtime_epoch
                && receipt.host_epoch == invocation.host_epoch,
            "Host admission receipt does not match the prepared invocation"
        );
        Ok(Some(admitted))
}
}
fn digest_json(value: &Value) -> String {
    let digest = Sha256::digest(value.to_string().as_bytes());
    let encoded = digest
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    format!("sha256:{encoded}")
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
