//! Collaboration package compilation pipeline.

mod authority;
mod canonical;
mod diagnostics;
mod graph;
mod projection;
mod schema;
mod schema_registry;

use std::collections::BTreeMap;

use anyhow::{Context, Result, anyhow, bail};
use serde_json::{Value, json};

use crate::uar::domain::collaboration::{
    COLLABORATION_PROFILE_DRAFT_2, CollaborationDefinitionRecord, CollaborationKind,
    ConversionDiagnostic, ConversionDisposition, ConversionReport, ConversionTarget,
    PackageManifest, PackagePreflightResponse, PackageSourceRequest, PreparedPackage,
    SourceIdentity,
};

use canonical::{digest, parse_unique_json, verify_content_digest};
use diagnostics::{complete_field_diagnostics, redact_projection_diagnostics};
use graph::validate_package_graph;
use projection::{conversion_diagnostics, project_agent};
use schema::{validate_manifest, validate_portable_document};
use schema_registry::validate_document;

pub(super) use canonical::{canonical_digest, request_digest};
pub(super) use schema::{validate_common_sections, validate_digest, validate_id, validate_semver};

pub fn prepare_package(request: &PackageSourceRequest) -> Result<PreparedPackage> {
    if request.command_id.trim().is_empty() {
        bail!("commandId must not be empty");
    }
    let manifest_document =
        parse_unique_json(&request.manifest).context("manifest is not canonicalizable JSON")?;
    let manifest_kind = validate_document(&manifest_document)?;
    if manifest_kind != CollaborationKind::PackageManifest {
        bail!("package source manifest selects the wrong collaboration document kind");
    }
    authority::validate_portable_authority(&manifest_document)?;
    verify_content_digest(&manifest_document)?;
    let manifest: PackageManifest = serde_json::from_value(manifest_document.clone())
        .context("manifest does not match the collaboration package shape")?;
    validate_manifest(&manifest, &request.files)?;

    let package_ref = manifest.definition_ref();
    let mut definitions = Vec::with_capacity(manifest.files.len());
    let mut documents_by_path = BTreeMap::new();
    let mut diagnostics = redact_projection_diagnostics(conversion_diagnostics(
        &manifest_document,
        &CollaborationKind::PackageManifest,
    ));

    for file in &manifest.files {
        let source = request
            .files
            .get(&file.path)
            .ok_or_else(|| anyhow!("manifest file '{}' was not supplied", file.path))?;
        let observed_byte_digest = digest(source.as_bytes());
        if observed_byte_digest != file.byte_digest {
            bail!(
                "byte digest mismatch for '{}': expected {}, observed {}",
                file.path,
                file.byte_digest,
                observed_byte_digest
            );
        }
        let document = parse_unique_json(source)
            .with_context(|| format!("{} is not canonicalizable JSON", file.path))?;
        let selected_kind = validate_portable_document(&document, &file.definition)?;
        if selected_kind != file.kind {
            bail!(
                "definition kind selected by the document disagrees with manifest entry '{}'",
                file.path
            );
        }
        let mut document_diagnostics =
            redact_projection_diagnostics(conversion_diagnostics(&document, &file.kind));
        if file.kind != CollaborationKind::AgentDefinition {
            document_diagnostics.push(ConversionDiagnostic {
                field: "kind".to_owned(),
                disposition: ConversionDisposition::RequiredUnsupported,
                message: format!(
                    "{} is cataloged but is not executable until durable collaboration runtime support lands",
                    file.kind.as_str()
                ),
            });
        }
        diagnostics.extend(document_diagnostics.iter().cloned());
        let compatibility_agent = match &file.kind {
            CollaborationKind::AgentDefinition => Some(project_agent(&document)?),
            _ => None,
        };
        documents_by_path.insert(file.path.clone(), document.clone());
        let conversion_report = build_conversion_report(&document, &document_diagnostics)?;
        definitions.push(CollaborationDefinitionRecord {
            identity: file.definition.clone(),
            kind: file.kind.clone(),
            document,
            source_json: source.clone(),
            byte_digest: file.byte_digest.clone(),
            package: package_ref.clone(),
            conversion_diagnostics: document_diagnostics,
            conversion_report: Some(conversion_report),
            compatibility_agent,
        });
    }

    validate_package_graph(&manifest, &documents_by_path)?;
    let activation_supported = !diagnostics
        .iter()
        .any(|item| item.disposition == ConversionDisposition::RequiredUnsupported);

    Ok(PreparedPackage {
        manifest,
        manifest_document,
        manifest_json: request.manifest.clone(),
        definitions,
        diagnostics,
        activation_supported,
        request_digest: request_digest(request)?,
    })
}

fn build_conversion_report(
    document: &Value,
    diagnostics: &[ConversionDiagnostic],
) -> Result<ConversionReport> {
    let id = schema::required_string(document, "id")?;
    let version = schema::required_string(document, "version")?;
    let content_digest = schema::required_string(document, "contentDigest")?;
    let source = match document.get("sourceIdentity") {
        Some(source) => serde_json::from_value(source.clone())
            .context("sourceIdentity does not match the draft.2 source identity shape")?,
        None => SourceIdentity {
            profile: COLLABORATION_PROFILE_DRAFT_2.to_owned(),
            id: id.to_owned(),
            version: version.to_owned(),
            digest: content_digest.to_owned(),
            revision: None,
        },
    };
    let field_diagnostics = complete_field_diagnostics(document, diagnostics);
    let activation_blocked = field_diagnostics
        .iter()
        .any(|diagnostic| diagnostic.disposition == ConversionDisposition::RequiredUnsupported);
    let mut report = ConversionReport {
        profile: COLLABORATION_PROFILE_DRAFT_2.to_owned(),
        kind: CollaborationKind::ConversionReport,
        id: format!("urn:uar:conversion:{id}"),
        version: version.to_owned(),
        content_digest: "sha256:0000000000000000000000000000000000000000000000000000000000000000"
            .to_owned(),
        provenance: json!({
            "source": "UAR collaboration compiler",
            "authors": ["Prometheus-AGS"]
        }),
        required_capabilities: Vec::new(),
        extensions: BTreeMap::new(),
        export_class: "portable-evidence".to_owned(),
        source,
        target: ConversionTarget {
            profile: Some(COLLABORATION_PROFILE_DRAFT_2.to_owned()),
            harness: None,
        },
        diagnostics: field_diagnostics,
        activation_blocked,
    };
    let report_value = serde_json::to_value(&report)?;
    report.content_digest = canonical::canonical_digest(&report_value)?;
    let report_value = serde_json::to_value(&report)?;
    let selected_kind = validate_document(&report_value)?;
    if selected_kind != CollaborationKind::ConversionReport {
        bail!("generated conversion report selected the wrong schema branch");
    }
    Ok(report)
}

pub fn preflight_response(prepared: &PreparedPackage) -> PackagePreflightResponse {
    PackagePreflightResponse {
        package: prepared.manifest.definition_ref(),
        definitions: prepared
            .definitions
            .iter()
            .map(|record| record.identity.clone())
            .collect(),
        diagnostics: prepared.diagnostics.clone(),
        activation_supported: prepared.activation_supported,
        request_digest: prepared.request_digest.clone(),
    }
}
