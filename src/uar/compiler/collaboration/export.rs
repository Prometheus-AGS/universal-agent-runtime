use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::uar::domain::collaboration::{
    COLLABORATION_PROFILE_DRAFT_2, CollaborationKind, ConversionDisposition, ConversionReport,
    ConversionTarget, DeploymentBindingTemplate, FieldDiagnostic, ImmutableDefinitionRef,
    PackageSourceRequest, RequestedModelBinding, SkillRef, StorageRequirements,
};

use super::bindings::binding_key;
use super::service::{CollaborationCatalogService, CollaborationError};
use super::validation::canonical_digest;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CanonicalPackageExport {
    pub package: ImmutableDefinitionRef,
    pub manifest: String,
    pub files: BTreeMap<String, String>,
    pub conversion_reports: Vec<ConversionReport>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum PackageExportTarget {
    CanonicalDraft2,
    Compatibility {
        profile: Option<String>,
        harness: Option<String>,
        #[serde(default)]
        supported_semantics: Vec<String>,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PackageExportRequest {
    pub package: ImmutableDefinitionRef,
    pub target: PackageExportTarget,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum PackageExportOutcome {
    Exported { export: CanonicalPackageExport },
    Refused { reports: Vec<ConversionReport> },
}

impl CanonicalPackageExport {
    #[must_use]
    pub fn into_source_request(self, command_id: String) -> PackageSourceRequest {
        PackageSourceRequest {
            command_id,
            expected_catalog_revision: None,
            manifest: self.manifest,
            files: self.files,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeploymentBindingTemplateExport {
    pub template: DeploymentBindingTemplate,
}

impl CollaborationCatalogService {
    pub async fn export_package(
        &self,
        request: &PackageExportRequest,
    ) -> Result<PackageExportOutcome, CollaborationError> {
        let state = self.load_state().await?;
        let record = state
            .packages
            .get(&request.package.storage_key())
            .ok_or_else(|| CollaborationError::NotFound(request.package.id.clone()))?;
        let manifest_files = record
            .manifest
            .get("files")
            .and_then(Value::as_array)
            .ok_or_else(|| {
                CollaborationError::Storage("package file inventory is missing".to_owned())
            })?;
        let mut files = BTreeMap::new();
        let mut conversion_reports = Vec::new();
        let mut definitions = Vec::new();
        for file in manifest_files {
            let path = file
                .get("path")
                .and_then(Value::as_str)
                .ok_or_else(|| CollaborationError::Storage("package path is missing".to_owned()))?;
            let definition: ImmutableDefinitionRef =
                serde_json::from_value(file.get("definition").cloned().ok_or_else(|| {
                    CollaborationError::Storage("definition reference is missing".to_owned())
                })?)
                .map_err(|_| {
                    CollaborationError::Storage("definition reference is invalid".to_owned())
                })?;
            let stored = state
                .definitions
                .get(&definition.storage_key())
                .ok_or_else(|| {
                    CollaborationError::Storage("package definition is missing".to_owned())
                })?;
            definitions.push(stored);
            files.insert(path.to_owned(), stored.source_json.clone());
            if let Some(report) = &stored.conversion_report {
                conversion_reports.push(report.clone());
            }
        }
        if let PackageExportTarget::Compatibility {
            profile,
            harness,
            supported_semantics,
        } = &request.target
        {
            let reports = definitions
                .into_iter()
                .map(|definition| {
                    compatibility_export_refusal(
                        definition,
                        profile.clone(),
                        harness.clone(),
                        supported_semantics,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?;
            return Ok(PackageExportOutcome::Refused { reports });
        }
        Ok(PackageExportOutcome::Exported {
            export: CanonicalPackageExport {
                package: request.package.clone(),
                manifest: record.source_json.clone(),
                files,
                conversion_reports,
            },
        })
    }

    pub async fn export_binding_template(
        &self,
        owner_id: &str,
        workspace_id: &str,
        binding_id: &str,
    ) -> Result<DeploymentBindingTemplateExport, CollaborationError> {
        let state = self.load_state().await?;
        let binding = state
            .bindings
            .get(&binding_key(owner_id, workspace_id, binding_id))
            .ok_or_else(|| CollaborationError::NotFound(binding_id.to_owned()))?;
        let definition = super::bindings::bound_agent_definition(&state, &binding.package)?;
        let skill_requirements = definition
            .document
            .get("skills")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .cloned()
            .map(serde_json::from_value::<SkillRef>)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| CollaborationError::Storage("stored SkillRef is invalid".to_owned()))?;
        let model_requirements = definition
            .document
            .get("modelRequirements")
            .and_then(|value| value.get("value"))
            .cloned()
            .unwrap_or_else(|| json!({}));
        let requested_model_bindings = binding
            .document
            .get("modelBindings")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(|model| model.get("requestedAlias").and_then(Value::as_str))
            .map(|requested_alias| RequestedModelBinding {
                requested_alias: requested_alias.to_owned(),
                model_requirements: model_requirements.clone(),
            })
            .collect();
        let storage = binding
            .document
            .get("storage")
            .ok_or_else(|| CollaborationError::Storage("binding storage is missing".to_owned()))?;
        let mut template = DeploymentBindingTemplate {
            profile: COLLABORATION_PROFILE_DRAFT_2.to_owned(),
            kind: CollaborationKind::DeploymentBindingTemplate,
            id: format!("{binding_id}/template"),
            version: binding
                .document
                .get("version")
                .and_then(Value::as_str)
                .unwrap_or("1.0.0")
                .to_owned(),
            content_digest:
                "sha256:0000000000000000000000000000000000000000000000000000000000000000".to_owned(),
            provenance: json!({"source": "UAR sanitized binding template export", "authors": ["Prometheus-AGS"]}),
            required_capabilities: vec!["collaboration_deployment_bindings_v2".to_owned()],
            extensions: BTreeMap::new(),
            export_class: "non-executable-template".to_owned(),
            package: binding.package.clone(),
            requested_model_bindings,
            skill_requirements,
            storage_requirements: StorageRequirements {
                backend_class: storage
                    .get("backend")
                    .and_then(Value::as_str)
                    .unwrap_or("durable-transactional")
                    .to_owned(),
                durable_transactions: storage
                    .get("durableTransactions")
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
            },
            effective_limits: binding
                .document
                .get("effectiveLimits")
                .cloned()
                .unwrap_or_else(|| json!({})),
            effective_budget: binding
                .document
                .get("effectiveBudget")
                .cloned()
                .unwrap_or_else(|| json!({})),
            rebind_fields: vec![
                "/ownerId",
                "/workspaceId",
                "/runtimeInstanceId",
                "/modelBindings/0/credentialRef",
                "/storage/connectionRef",
                "/policyRevision",
                "/representationGrantRefs",
            ]
            .into_iter()
            .map(str::to_owned)
            .collect(),
            status: "needs-private-binding".to_owned(),
        };
        template.content_digest = canonical_digest(&serde_json::to_value(&template)?)?;
        Ok(DeploymentBindingTemplateExport { template })
    }
}

fn compatibility_export_refusal(
    definition: &crate::uar::domain::collaboration::CollaborationDefinitionRecord,
    profile: Option<String>,
    harness: Option<String>,
    supported_semantics: &[String],
) -> Result<ConversionReport, CollaborationError> {
    let mut diagnostics = Vec::new();
    for field in [
        "modelRequirements",
        "promptDialect",
        "ragConfiguration",
        "contextStrategy",
        "apiHarness",
    ] {
        let Some(value) = definition.document.get(field) else {
            continue;
        };
        let required = value
            .get("required")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let supported = supported_semantics
            .iter()
            .any(|candidate| candidate == field);
        diagnostics.push(FieldDiagnostic {
            pointer: format!("/{field}"),
            disposition: if supported {
                ConversionDisposition::Translated
            } else if required {
                ConversionDisposition::RequiredUnsupported
            } else {
                ConversionDisposition::OptionalUnsupported
            },
            reason_code: if supported {
                "export.target-declared-support"
            } else {
                "export.target-semantic-unsupported"
            }
            .to_owned(),
            message: if supported {
                "The target declares this semantic, but UAR has no canonical encoder for the requested target."
            } else {
                "The requested target does not declare support for this semantic."
            }
            .to_owned(),
            effective_binding_ref: None, source_kind: None, source_definition: None,
        });
    }
    for (index, skill) in definition
        .document
        .get("skills")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .enumerate()
    {
        let required = skill
            .get("required")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let supported = supported_semantics
            .iter()
            .any(|candidate| candidate == "skillRef");
        if required || !supported {
            diagnostics.push(FieldDiagnostic {
                pointer: format!("/skills/{index}"),
                disposition: if supported {
                    ConversionDisposition::Translated
                } else if required {
                    ConversionDisposition::RequiredUnsupported
                } else {
                    ConversionDisposition::OptionalUnsupported
                },
                reason_code: "export.target-skill-ref-unsupported".to_owned(),
                message: "The requested target has no canonical full SkillRef encoder.".to_owned(),
                effective_binding_ref: None, source_kind: None, source_definition: None,
            });
        }
    }
    diagnostics.push(FieldDiagnostic {
        pointer: String::new(),
        disposition: ConversionDisposition::RequiredUnsupported,
        reason_code: "export.target-encoder-unavailable".to_owned(),
        message: "UAR cannot encode the requested compatibility target without changing immutable semantics."
            .to_owned(),
        effective_binding_ref: None, source_kind: None, source_definition: None,
    });
    let source = definition
        .document
        .get("sourceIdentity")
        .cloned()
        .and_then(|value| serde_json::from_value(value).ok())
        .unwrap_or_else(|| crate::uar::domain::collaboration::SourceIdentity {
            profile: definition
                .document
                .get("profile")
                .and_then(Value::as_str)
                .unwrap_or(COLLABORATION_PROFILE_DRAFT_2)
                .to_owned(),
            id: definition.identity.id.clone(),
            version: definition.identity.version.clone(),
            digest: definition.identity.digest.clone(),
            revision: None,
        });
    let mut report = ConversionReport {
        profile: COLLABORATION_PROFILE_DRAFT_2.to_owned(),
        kind: CollaborationKind::ConversionReport,
        id: format!("{}/export-report", definition.identity.id),
        version: definition.identity.version.clone(),
        content_digest: "sha256:0000000000000000000000000000000000000000000000000000000000000000"
            .to_owned(),
        provenance: json!({"source": "UAR compatibility export refusal", "authors": ["Prometheus-AGS"]}),
        required_capabilities: vec!["collaboration_conversion_reports_v1".to_owned()],
        extensions: BTreeMap::new(),
        export_class: "portable-evidence".to_owned(),
        source,
        target: ConversionTarget { profile, harness },
        diagnostics,
        activation_blocked: true,
    };
    report.content_digest = canonical_digest(&serde_json::to_value(&report)?)?;
    Ok(report)
}
