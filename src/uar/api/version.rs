//! `GET /api/uar/version`: the build identity of the running binary.
//!
//! Installers, upgrade scripts and supervising hosts call this to confirm that
//! the process they reached is the build they expect. Every value is fixed at
//! compile time: no configuration value, path, host name or credential
//! appears in the response.
//!
//! On a standalone server the route needs no credentials, like `/health`,
//! because an installer has none to present. In supervised mode the
//! launch-token guard covers it like every other route.

use axum::Json;
use serde::Serialize;

/// A2A protocol version served by the JSON-RPC and gRPC bindings in
/// `crate::uar::api::a2a::v1`.
const A2A_PROTOCOL_VERSION: &str = "1.0";

/// Wire protocol versions this build speaks.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProtocolVersions {
    /// A2A protocol version, or `null` when this build has no A2A transport
    /// (the `a2a-transport` Cargo feature is off).
    pub a2a: Option<&'static str>,
    /// AG-UI protocol version of the runs stream.
    pub ag_ui: &'static str,
    /// AG-UI profile id of the runs stream.
    pub ag_ui_profile: &'static str,
    /// A2UI protocol version.
    pub a2ui: &'static str,
    /// MCP protocol version the run-scoped MCP client negotiates.
    pub mcp: String,
}

/// Body of `GET /api/uar/version`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VersionResponse {
    /// Cargo package name.
    pub name: &'static str,
    /// Cargo package version.
    pub version: &'static str,
    /// Commit the binary was built from, as a lowercase hexadecimal object
    /// id. `null` when the build did not record one. A build records the
    /// `UAR_GIT_SHA` build variable when it is set; otherwise a release
    /// build records the checked-out commit and a debug build records
    /// nothing. It does not say whether the working tree was clean.
    pub git_commit: Option<&'static str>,
    /// Cargo build profile: `release` or `debug`.
    pub build_profile: &'static str,
    /// Target triple the binary was compiled for.
    pub target: &'static str,
    /// Enabled Cargo features, sorted, in their hyphenated spelling.
    pub features: Vec<&'static str>,
    /// Wire protocol versions this build speaks.
    pub protocols: ProtocolVersions,
    /// Schema revision of the administration manifest that
    /// `GET /api/uar/capabilities` returns.
    pub administration_schema: u32,
}

/// Return the build identity of this binary.
///
/// # Examples
///
/// ```
/// let info = universal_agent_runtime::uar::api::version::version_info();
/// assert_eq!(info.version, env!("CARGO_PKG_VERSION"));
/// assert!(info.features.contains(&"server"));
/// ```
#[must_use]
pub fn version_info() -> VersionResponse {
    let git_commit = env!("UAR_BUILD_GIT_COMMIT");
    VersionResponse {
        name: env!("CARGO_PKG_NAME"),
        version: env!("CARGO_PKG_VERSION"),
        git_commit: (!git_commit.is_empty()).then_some(git_commit),
        build_profile: env!("UAR_BUILD_PROFILE"),
        target: env!("UAR_BUILD_TARGET"),
        features: env!("UAR_BUILD_FEATURES")
            .split(',')
            .filter(|feature| !feature.is_empty())
            .collect(),
        protocols: ProtocolVersions {
            a2a: cfg!(feature = "a2a-transport").then_some(A2A_PROTOCOL_VERSION),
            ag_ui: super::agui::PROTOCOL_VERSION,
            ag_ui_profile: super::capabilities::AGUI_PROFILE,
            a2ui: crate::uar::a2ui::protocol::VERSION,
            mcp: rmcp::model::ProtocolVersion::V_2025_11_25.to_string(),
        },
        administration_schema: super::administration_capabilities::ADMINISTRATION_SCHEMA_VERSION,
    }
}

/// `GET /api/uar/version`
pub async fn version_handler() -> Json<VersionResponse> {
    Json(version_info())
}

#[cfg(test)]
mod tests {
    use super::{version_handler, version_info};

    #[test]
    fn reports_the_package_identity_and_build_facts() {
        let info = version_info();
        assert_eq!(info.name, "universal-agent-runtime");
        assert_eq!(info.version, env!("CARGO_PKG_VERSION"));
        assert!(matches!(info.build_profile, "debug" | "release"));
        assert!(!info.target.is_empty());
        assert!(info.features.contains(&"server"));
        assert!(info.features.is_sorted());
        assert_eq!(info.protocols.ag_ui_profile, "uar.agui/1");
        assert_eq!(info.protocols.mcp, "2025-11-25");
        assert_eq!(
            info.protocols.a2a.is_some(),
            cfg!(feature = "a2a-transport")
        );
    }

    #[test]
    fn a_recorded_commit_is_a_full_lowercase_object_id() {
        if let Some(commit) = version_info().git_commit {
            assert!(matches!(commit.len(), 40 | 64), "{commit}");
            assert!(
                commit
                    .bytes()
                    .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f')),
                "{commit}"
            );
        }
    }

    #[tokio::test]
    async fn serializes_every_documented_field() {
        let body = serde_json::to_value(version_handler().await.0).unwrap_or_default();
        for field in [
            "name",
            "version",
            "git_commit",
            "build_profile",
            "target",
            "features",
            "protocols",
            "administration_schema",
        ] {
            assert!(body.get(field).is_some(), "missing {field}: {body}");
        }
        for protocol in ["a2a", "ag_ui", "ag_ui_profile", "a2ui", "mcp"] {
            assert!(
                body["protocols"].get(protocol).is_some(),
                "missing protocols.{protocol}: {body}"
            );
        }
    }
}
