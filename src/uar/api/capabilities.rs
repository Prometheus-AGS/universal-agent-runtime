//! `GET /api/uar/capabilities`: the runtime version, AG-UI profile, and the
//! named behaviours this binary implements, for a host's startup version gate.
//!
//! The response carries no configuration value, path, key source or secret.
//! The route has no authentication or rate-limit exemption: in sidecar mode
//! the launch-token guard covers it like every other route.

use axum::Json;
use serde::Serialize;

/// AG-UI profile id of the runs stream (`docs/protocols/ag-ui-profile.md`).
pub const AGUI_PROFILE: &str = "uar.agui/1";

/// AG-UI profile revision of the runs stream.
pub const AGUI_PROFILE_REVISION: u32 = 1;

/// Closed capability vocabulary (design Decision 12 of
/// `sidecar-launch-security`). A name may be advertised only once the owning
/// change lands its behaviour.
pub const CAPABILITY_VOCABULARY: [&str; 12] = [
    "agui_stream_fidelity",
    "approval_lifecycle_v1",
    "collaboration_definition_packages_v1",
    "collaboration_deployment_bindings_v1",
    "host_history",
    "ingest_scoped_credentials",
    "reasoning_effort",
    "run_scoped_credentials",
    "run_scoped_mcp_servers",
    "secrets_at_rest",
    "session_principal",
    "working_directory",
];

/// Capabilities this binary implements. Each owning change adds its name in
/// the same commit as the behaviour; none has landed yet.
pub const IMPLEMENTED_CAPABILITIES: [&str; 9] = [
    "approval_lifecycle_v1",
    "collaboration_definition_packages_v1",
    "collaboration_deployment_bindings_v1",
    "host_history",
    "reasoning_effort",
    "run_scoped_credentials",
    "run_scoped_mcp_servers",
    "session_principal",
    "working_directory",
];

/// AG-UI profile section of [`CapabilitiesResponse`].
#[derive(Debug, Serialize)]
pub struct AguiProfile {
    pub profile: &'static str,
    pub profile_revision: u32,
}

/// Body of `GET /api/uar/capabilities`.
#[derive(Debug, Serialize)]
pub struct CapabilitiesResponse {
    pub uar_version: &'static str,
    pub agui: AguiProfile,
    /// Sorted, duplicate-free names from [`CAPABILITY_VOCABULARY`].
    pub capabilities: Vec<&'static str>,
    pub administration: super::administration_capabilities::AdministrationCapabilities,
}

/// `GET /api/uar/capabilities`
pub async fn capabilities_handler() -> Json<CapabilitiesResponse> {
    let mut capabilities = IMPLEMENTED_CAPABILITIES.to_vec();
    capabilities.sort_unstable();
    capabilities.dedup();
    Json(CapabilitiesResponse {
        uar_version: env!("CARGO_PKG_VERSION"),
        agui: AguiProfile {
            profile: AGUI_PROFILE,
            profile_revision: AGUI_PROFILE_REVISION,
        },
        capabilities,
        administration: super::administration_capabilities::administration_capabilities(),
    })
}
