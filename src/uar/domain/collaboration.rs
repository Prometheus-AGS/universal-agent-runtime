//! Immutable collaboration definitions and private deployment bindings.
//!
//! This module deliberately models the I1 catalog boundary only. A stored
//! `TeamDefinition` is a reusable document, not a running team instance.

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::artifact::AgentArtifact;

pub const COLLABORATION_PROFILE: &str = "urn:prometheus:uar:collaboration:0.1.0-draft.1";

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "PascalCase")]
pub enum CollaborationKind {
    AgentDefinition,
    TeamDefinition,
    WorkflowDefinition,
    PackageManifest,
    DeploymentBinding,
}

impl CollaborationKind {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::AgentDefinition => "AgentDefinition",
            Self::TeamDefinition => "TeamDefinition",
            Self::WorkflowDefinition => "WorkflowDefinition",
            Self::PackageManifest => "PackageManifest",
            Self::DeploymentBinding => "DeploymentBinding",
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub struct ImmutableDefinitionRef {
    pub id: String,
    pub version: String,
    pub digest: String,
}

impl ImmutableDefinitionRef {
    #[must_use]
    pub fn storage_key(&self) -> String {
        format!("{}\u{1f}{}\u{1f}{}", self.id, self.version, self.digest)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackageFile {
    pub path: String,
    pub kind: CollaborationKind,
    pub definition: ImmutableDefinitionRef,
    pub byte_digest: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackageLockEntry {
    pub requested_by: String,
    pub reference: ImmutableDefinitionRef,
    pub resolved_path: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CapabilityDeclaration {
    pub capability: String,
    pub required: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackageManifest {
    pub profile: String,
    pub kind: CollaborationKind,
    pub id: String,
    pub version: String,
    pub content_digest: String,
    pub provenance: Value,
    pub required_capabilities: Vec<String>,
    pub extensions: BTreeMap<String, Value>,
    pub entrypoints: Vec<ImmutableDefinitionRef>,
    pub files: Vec<PackageFile>,
    pub lock: Vec<PackageLockEntry>,
    pub capability_declarations: Vec<CapabilityDeclaration>,
    pub resolution: String,
}

impl PackageManifest {
    #[must_use]
    pub fn definition_ref(&self) -> ImmutableDefinitionRef {
        ImmutableDefinitionRef {
            id: self.id.clone(),
            version: self.version.clone(),
            digest: self.content_digest.clone(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ConversionDisposition {
    Exact,
    Translated,
    OptionalUnsupported,
    RequiredUnsupported,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversionDiagnostic {
    pub field: String,
    pub disposition: ConversionDisposition,
    pub message: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CollaborationDefinitionRecord {
    pub identity: ImmutableDefinitionRef,
    pub kind: CollaborationKind,
    pub document: Value,
    pub source_json: String,
    pub byte_digest: String,
    pub package: ImmutableDefinitionRef,
    pub conversion_diagnostics: Vec<ConversionDiagnostic>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compatibility_agent: Option<AgentArtifact>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CollaborationPackageRecord {
    pub identity: ImmutableDefinitionRef,
    pub manifest: Value,
    pub source_json: String,
    pub definition_keys: Vec<String>,
    pub installed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeploymentBindingRecord {
    pub id: String,
    pub owner_id: String,
    pub workspace_id: String,
    pub revision: u64,
    pub document: Value,
    pub package: ImmutableDefinitionRef,
    pub activation_supported: bool,
    pub preflight_diagnostics: Vec<ConversionDiagnostic>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CollaborationCommandReceipt {
    pub command_id: String,
    pub owner_id: String,
    pub request_digest: String,
    pub operation: String,
    pub resource_id: String,
    pub catalog_revision: u64,
    pub binding_revision: Option<u64>,
    pub committed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CollaborationCatalogState {
    pub generation: u64,
    pub catalog_revision: u64,
    pub definitions: BTreeMap<String, CollaborationDefinitionRecord>,
    pub packages: BTreeMap<String, CollaborationPackageRecord>,
    pub bindings: BTreeMap<String, DeploymentBindingRecord>,
    #[serde(default)]
    pub binding_history: BTreeMap<String, DeploymentBindingRecord>,
    pub command_receipts: BTreeMap<String, CollaborationCommandReceipt>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackageSourceRequest {
    pub command_id: String,
    #[serde(default)]
    pub expected_catalog_revision: Option<u64>,
    /// Exact UTF-8 manifest bytes represented as a JSON string.
    pub manifest: String,
    /// Exact UTF-8 file bytes keyed by the manifest-relative path.
    pub files: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreparedPackage {
    pub manifest: PackageManifest,
    pub manifest_document: Value,
    pub manifest_json: String,
    pub definitions: Vec<CollaborationDefinitionRecord>,
    pub diagnostics: Vec<ConversionDiagnostic>,
    pub activation_supported: bool,
    pub request_digest: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackagePreflightResponse {
    pub package: ImmutableDefinitionRef,
    pub definitions: Vec<ImmutableDefinitionRef>,
    pub diagnostics: Vec<ConversionDiagnostic>,
    pub activation_supported: bool,
    pub request_digest: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageInstallResponse {
    pub preflight: PackagePreflightResponse,
    pub receipt: CollaborationCommandReceipt,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BindingInstallResponse {
    pub preflight: BindingPreflightResponse,
    pub binding: DeploymentBindingRecord,
    pub receipt: CollaborationCommandReceipt,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BindingCommandRequest {
    pub command_id: String,
    #[serde(default)]
    pub expected_revision: Option<u64>,
    pub binding: Value,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BindingPreflightResponse {
    pub binding_id: String,
    pub package: ImmutableDefinitionRef,
    pub diagnostics: Vec<ConversionDiagnostic>,
    pub activation_supported: bool,
    pub request_digest: String,
}
