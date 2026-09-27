//! Immutable collaboration definitions and private deployment bindings.
//!
//! This module deliberately models the I1 catalog boundary only. A stored
//! `TeamDefinition` is a reusable document, not a running team instance.

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::artifact::AgentArtifact;

pub const COLLABORATION_PROFILE_DRAFT_1: &str = "urn:prometheus:uar:collaboration:0.1.0-draft.1";
pub const COLLABORATION_PROFILE_DRAFT_2: &str = "urn:prometheus:uar:collaboration:0.1.0-draft.2";
pub const COLLABORATION_PROFILE: &str = COLLABORATION_PROFILE_DRAFT_2;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "PascalCase")]
pub enum CollaborationKind {
    AgentDefinition,
    TeamDefinition,
    WorkflowDefinition,
    PackageManifest,
    DeploymentBinding,
    RepresentationGrant,
    ConversionReport,
    EffectiveBindingReceipt,
    DeploymentBindingTemplate,
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
            Self::RepresentationGrant => "RepresentationGrant",
            Self::ConversionReport => "ConversionReport",
            Self::EffectiveBindingReceipt => "EffectiveBindingReceipt",
            Self::DeploymentBindingTemplate => "DeploymentBindingTemplate",
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

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CollaborationDocumentIdentity {
    pub profile: String,
    pub kind: CollaborationKind,
    pub id: String,
    pub version: String,
    pub content_digest: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SourceIdentity {
    pub profile: String,
    pub id: String,
    pub version: String,
    pub digest: String,
    pub revision: Option<u64>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SkillRef {
    pub id: String,
    pub version: String,
    pub digest: String,
    pub required: bool,
    pub config: Value,
    pub entrypoint: Option<String>,
    pub required_tools: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PrivateRevisionRef {
    pub id: String,
    pub revision: u64,
    pub digest: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RepresentationGrantRef {
    pub grant_id: String,
    pub revision: u64,
    pub constraint_digest: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum RepresentationGrantStatus {
    Pending,
    Active,
    Suspended,
    Revoked,
    Expired,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GrantRevocation {
    pub revision: u64,
    pub revoked_at: DateTime<Utc>,
    pub reason: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GrantRetention {
    pub policy: String,
    pub delete_after: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GrantOffboarding {
    pub mode: String,
    pub required_actions: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GrantRestrictions {
    pub forbidden_claims: Vec<String>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RepresentationGrant {
    pub profile: String,
    pub kind: CollaborationKind,
    pub export_class: String,
    pub grant_id: String,
    pub issuer_principal_id: String,
    pub subject_principal_id: String,
    pub grantee_agent_instance_id: String,
    pub organization_id: String,
    pub office: String,
    pub purpose: String,
    pub audience_scopes: Vec<String>,
    pub action_scopes: Vec<String>,
    pub resource_scopes: Vec<String>,
    pub data_scopes: Vec<String>,
    pub approval_requirements: Vec<String>,
    pub disclosure_requirements: Vec<String>,
    pub consent_evidence_ref: String,
    pub organizational_authority_evidence_ref: String,
    pub revision: u64,
    pub status: RepresentationGrantStatus,
    pub not_before: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub constraint_digest: String,
    pub revocation: Option<GrantRevocation>,
    pub retention: GrantRetention,
    pub offboarding: GrantOffboarding,
    pub restrictions: GrantRestrictions,
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
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FieldDiagnostic {
    /// RFC 6901 JSON Pointer into the source document.
    pub pointer: String,
    pub disposition: ConversionDisposition,
    pub reason_code: String,
    /// Safe operator-facing text. Source values never belong in this field.
    pub message: String,
    pub effective_binding_ref: Option<PrivateRevisionRef>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ConversionTarget {
    pub profile: Option<String>,
    pub harness: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ConversionReport {
    pub profile: String,
    pub kind: CollaborationKind,
    pub id: String,
    pub version: String,
    pub content_digest: String,
    pub provenance: Value,
    pub required_capabilities: Vec<String>,
    pub extensions: BTreeMap<String, Value>,
    pub export_class: String,
    pub source: SourceIdentity,
    pub target: ConversionTarget,
    pub diagnostics: Vec<FieldDiagnostic>,
    pub activation_blocked: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ResolvedSkill {
    pub skill: SkillRef,
    pub installed_location: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EffectiveBindingReceipt {
    pub profile: String,
    pub kind: CollaborationKind,
    pub export_class: String,
    pub id: String,
    pub revision: u64,
    pub content_digest: String,
    pub binding_ref: PrivateRevisionRef,
    pub package: ImmutableDefinitionRef,
    pub requested: Value,
    pub effective: Value,
    pub resolved_skills: Vec<ResolvedSkill>,
    pub resolved_models: Vec<Value>,
    pub policy_revision: String,
    pub representation_grants: Vec<RepresentationGrantRef>,
    pub runtime_capabilities: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub service_binding: Option<crate::uar::service_instance::EffectiveServiceBinding>,
    pub diagnostics: Vec<FieldDiagnostic>,
    pub admitted: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RequestedModelBinding {
    pub requested_alias: String,
    pub model_requirements: Value,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StorageRequirements {
    pub backend_class: String,
    pub durable_transactions: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DeploymentBindingTemplate {
    pub profile: String,
    pub kind: CollaborationKind,
    pub id: String,
    pub version: String,
    pub content_digest: String,
    pub provenance: Value,
    pub required_capabilities: Vec<String>,
    pub extensions: BTreeMap<String, Value>,
    pub export_class: String,
    pub package: ImmutableDefinitionRef,
    pub requested_model_bindings: Vec<RequestedModelBinding>,
    pub skill_requirements: Vec<SkillRef>,
    pub storage_requirements: StorageRequirements,
    pub effective_limits: Value,
    pub effective_budget: Value,
    pub rebind_fields: Vec<String>,
    pub status: String,
}

/// Immutable evidence relating source bytes to their normalized draft.2 identity.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MigrationReceipt {
    pub source: SourceIdentity,
    pub target: CollaborationDocumentIdentity,
    pub diagnostics: Vec<FieldDiagnostic>,
    pub migrated_at: DateTime<Utc>,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub conversion_report: Option<ConversionReport>,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effective_binding_receipt: Option<EffectiveBindingReceipt>,
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
    #[serde(default)]
    pub representation_grants: BTreeMap<String, RepresentationGrant>,
    #[serde(default)]
    pub representation_grant_history: BTreeMap<String, RepresentationGrant>,
    #[serde(default)]
    pub effective_binding_receipts: BTreeMap<String, EffectiveBindingReceipt>,
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
