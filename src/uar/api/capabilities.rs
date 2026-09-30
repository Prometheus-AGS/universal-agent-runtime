//! `GET /api/uar/capabilities`: the runtime version, AG-UI profile, and the
//! named behaviours this binary implements, for a host's startup version gate.
//!
//! The response carries configured endpoint roles and opaque references but no
//! credential value or secret.
//! The route has no authentication or rate-limit exemption: in sidecar mode
//! the launch-token guard covers it like every other route.

use std::sync::Arc;

use axum::{Json, extract::State};
use serde::Serialize;

use crate::uar::service_instance::{
    CompatibilityResponse, PlacementSupport, ServiceEndpointRoles, ServiceInstanceAuthority,
    ServiceInstanceIdentity, ServiceInstanceReferences, ServicePlacementExpectation,
};

/// AG-UI profile id of the runs stream (`docs/protocols/ag-ui-profile.md`).
pub const AGUI_PROFILE: &str = "uar.agui/1";

/// AG-UI profile revision of the runs stream.
pub const AGUI_PROFILE_REVISION: u32 = 1;
pub const TEAM_EXECUTION_PROFILE: &str = "urn:prometheus:uar:team-execution:0.1.0";
pub const TEAM_EXECUTION_B_CAPABILITIES: [&str; 3] = [
    "team_execution_peer_tools_v1",
    "team_execution_shared_instructions_v1",
    "team_execution_continuations_v1",
];
/// Promote only after the operated-source/binary Gate B receipt is recorded.
pub const TEAM_EXECUTION_B_QUALIFIED: bool = false;
pub const TEAM_EXECUTION_B_QUALIFICATION_RECEIPT: Option<&str> = None;
pub fn team_execution_profile_stage() -> &'static str {
    match std::env::var("UAR_TEAM_EXECUTION_PROFILE_STAGE").as_deref() {
        Ok("operation") => "operation",
        _ if TEAM_EXECUTION_B_QUALIFIED => "qualified",
        _ => "unqualified",
    }
}
pub fn team_execution_b_enabled() -> bool {
    team_execution_profile_stage() != "unqualified"
}

/// Closed capability vocabulary (design Decision 12 of
/// `sidecar-launch-security`). A name may be advertised only once the owning
/// change lands its behaviour.
pub const CAPABILITY_VOCABULARY: [&str; 25] = [
    "agui_stream_fidelity",
    "approval_lifecycle_v1",
    "collaboration_definition_packages_v1",
    "collaboration_definition_packages_v2",
    "collaboration_deployment_bindings_v1",
    "collaboration_deployment_bindings_v2",
    "collaboration_conversion_reports_v1",
    "collaboration_representation_grant_refs_v1",
    "collaboration_team_planning_v1",
    "collaboration_team_execution_v1",
    "durable_agent_instances_v1",
    "full_harness_delegation_v1",
    "host_history",
    "ingest_scoped_credentials",
    "local_scoped_observers_v1",
    "reasoning_effort",
    "run_scoped_credentials",
    "run_scoped_mcp_servers",
    "secrets_at_rest",
    "session_principal",
    "service_instance_placement_v1",
    "working_directory",
    "team_execution_peer_tools_v1",
    "team_execution_shared_instructions_v1",
    "team_execution_continuations_v1",
];

/// Capabilities this binary implements. Each owning change adds its name in
/// the same commit as the behaviour.
pub const IMPLEMENTED_CAPABILITIES: [&str; 16] = [
    "approval_lifecycle_v1",
    "collaboration_definition_packages_v1",
    "collaboration_definition_packages_v2",
    "collaboration_deployment_bindings_v1",
    "collaboration_deployment_bindings_v2",
    "collaboration_conversion_reports_v1",
    "collaboration_representation_grant_refs_v1",
    "collaboration_team_planning_v1",
    "full_harness_delegation_v1",
    "host_history",
    "reasoning_effort",
    "run_scoped_credentials",
    "run_scoped_mcp_servers",
    "session_principal",
    "service_instance_placement_v1",
    "working_directory",
];

#[derive(Debug, Clone)]
pub struct CapabilitiesApiState {
    pub service_instance: Arc<ServiceInstanceAuthority>,
    pub collaboration_catalog:
        Arc<crate::uar::compiler::collaboration::CollaborationCatalogService>,
}

/// AG-UI profile section of [`CapabilitiesResponse`].
#[derive(Debug, Serialize)]
pub struct AguiProfile {
    pub profile: &'static str,
    pub profile_revision: u32,
}

/// Body of `GET /api/uar/capabilities`.
#[derive(Debug, Serialize)]
pub struct CapabilitiesResponse {
    #[serde(rename = "executionProfile")]
    pub execution_profile: &'static str,
    #[serde(rename = "executionProfileStage")]
    pub execution_profile_stage: &'static str,
    pub uar_version: &'static str,
    pub agui: AguiProfile,
    pub instance: ServiceInstanceIdentity,
    pub endpoints: ServiceEndpointRoles,
    pub ownership: crate::config::ServiceOwnership,
    pub references: ServiceInstanceReferences,
    pub placement: PlacementSupport,
    /// Sorted, duplicate-free names from [`CAPABILITY_VOCABULARY`].
    pub capabilities: Vec<String>,
    pub collaboration: CollaborationCapabilities,
    pub administration: super::administration_capabilities::AdministrationCapabilities,
}

/// Runtime collaboration contract shared by REST and MCP discovery.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CollaborationCapabilities {
    pub accepted_profiles: [&'static str; 2],
    pub document_kinds: [&'static str; 9],
    pub schema_ids: [&'static str; 11],
    pub export_classes: [&'static str; 5],
    pub activation: CollaborationActivationLimits,
}

/// Explicit activation limits for the collaboration profile.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CollaborationActivationLimits {
    pub ordinary_agent: bool,
    /// Persistent team/member/task administration only; no team execution.
    pub team_planning: bool,
    pub task_ownership: bool,
    pub team_mailbox: bool,
    pub team_instance: bool,
    pub team_execution: bool,
    pub workflow: bool,
    pub human_representation: bool,
}

/// Return the exact collaboration profiles and schema family implemented here.
pub fn collaboration_capabilities() -> CollaborationCapabilities {
    CollaborationCapabilities {
        accepted_profiles: [
            crate::uar::domain::collaboration::COLLABORATION_PROFILE_DRAFT_1,
            crate::uar::domain::collaboration::COLLABORATION_PROFILE_DRAFT_2,
        ],
        document_kinds: [
            "AgentDefinition",
            "TeamDefinition",
            "WorkflowDefinition",
            "PackageManifest",
            "DeploymentBinding",
            "RepresentationGrant",
            "ConversionReport",
            "EffectiveBindingReceipt",
            "DeploymentBindingTemplate",
        ],
        schema_ids: [
            "https://schemas.prometheus-ags.dev/uar/collaboration/0.1.0-draft.2/common.schema.json",
            "https://schemas.prometheus-ags.dev/uar/collaboration/0.1.0-draft.2/collaboration-document.schema.json",
            "https://schemas.prometheus-ags.dev/uar/collaboration/0.1.0-draft.2/agent-definition.schema.json",
            "https://schemas.prometheus-ags.dev/uar/collaboration/0.1.0-draft.2/team-definition.schema.json",
            "https://schemas.prometheus-ags.dev/uar/collaboration/0.1.0-draft.2/workflow-definition.schema.json",
            "https://schemas.prometheus-ags.dev/uar/collaboration/0.1.0-draft.2/package-manifest.schema.json",
            "https://schemas.prometheus-ags.dev/uar/collaboration/0.1.0-draft.2/deployment-binding.schema.json",
            "https://schemas.prometheus-ags.dev/uar/collaboration/0.1.0-draft.2/representation-grant.schema.json",
            "https://schemas.prometheus-ags.dev/uar/collaboration/0.1.0-draft.2/conversion-report.schema.json",
            "https://schemas.prometheus-ags.dev/uar/collaboration/0.1.0-draft.2/effective-binding-receipt.schema.json",
            "https://schemas.prometheus-ags.dev/uar/collaboration/0.1.0-draft.2/deployment-binding-template.schema.json",
        ],
        export_classes: [
            "portable-evidence",
            "non-executable-template",
            "private-installed-state",
            "private-binding-evidence",
            "private-authority-state",
        ],
        activation: CollaborationActivationLimits {
            ordinary_agent: true,
            team_planning: true,
            task_ownership: true,
            team_mailbox: true,
            team_instance: false,
            team_execution: false,
            workflow: false,
            human_representation: false,
        },
    }
}

/// `GET /api/uar/capabilities`
pub fn capabilities_response(service_instance: &ServiceInstanceAuthority) -> CapabilitiesResponse {
    let descriptor = service_instance.descriptor();
    let mut collaboration = collaboration_capabilities();
    let team_execution_available = descriptor
        .capabilities
        .iter()
        .any(|capability| capability == "collaboration_team_execution_v1");
    collaboration.activation.team_instance = team_execution_available;
    collaboration.activation.team_execution = team_execution_available;
    CapabilitiesResponse {
        execution_profile: TEAM_EXECUTION_PROFILE,
        execution_profile_stage: team_execution_profile_stage(),
        uar_version: env!("CARGO_PKG_VERSION"),
        agui: AguiProfile {
            profile: AGUI_PROFILE,
            profile_revision: AGUI_PROFILE_REVISION,
        },
        instance: descriptor.instance.clone(),
        endpoints: descriptor.endpoints.clone(),
        ownership: descriptor.ownership,
        references: descriptor.references.clone(),
        placement: descriptor.placement.clone(),
        capabilities: descriptor.capabilities.clone(),
        collaboration,
        administration: super::administration_capabilities::administration_capabilities(
            descriptor
                .capabilities
                .iter()
                .any(|capability| capability == "durable_agent_instances_v1"),
            descriptor
                .capabilities
                .iter()
                .any(|capability| capability == "local_scoped_observers_v1"),
        ),
    }
}

/// `GET /api/uar/capabilities`
pub async fn capabilities_handler(
    State(state): State<Arc<CapabilitiesApiState>>,
) -> Json<CapabilitiesResponse> {
    let mut response = capabilities_response(&state.service_instance);
    if !state
        .collaboration_catalog
        .execution_ownership_view()
        .await
        .is_ok_and(|v| v.owns_execution)
    {
        response.capabilities.retain(|c| {
            c != "collaboration_team_execution_v1"
                && !TEAM_EXECUTION_B_CAPABILITIES.contains(&c.as_str())
        });
        response.collaboration.activation.team_execution = false;
    }
    Json(response)
}

/// `POST /api/uar/compatibility`
pub async fn compatibility_handler(
    State(state): State<Arc<CapabilitiesApiState>>,
    Json(expectation): Json<ServicePlacementExpectation>,
) -> Json<CompatibilityResponse> {
    if expectation.required_capabilities.iter().any(|c| {
        c == "collaboration_team_execution_v1"
            || TEAM_EXECUTION_B_CAPABILITIES.contains(&c.as_str())
    }) && !state
        .collaboration_catalog
        .execution_ownership_view()
        .await
        .is_ok_and(|v| v.owns_execution)
    {
        return Json(CompatibilityResponse {
            compatible: false,
            diagnostics: vec![crate::uar::service_instance::CompatibilityDiagnostic {
                field: "requiredCapabilities",
                code: "TEAM_EXECUTION_OWNER_CONFLICT",
                message: "This executor does not hold the catalog execution authority.".into(),
            }],
            effective_binding: None,
        });
    }
    Json(
        state
            .service_instance
            .evaluate(&expectation, expectation.intent),
    )
}
