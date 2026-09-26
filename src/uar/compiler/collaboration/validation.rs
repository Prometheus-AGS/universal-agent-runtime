//! Collaboration package compilation pipeline.

mod canonical;
mod graph;
mod projection;
mod schema;

use std::collections::BTreeMap;

use anyhow::{Context, Result, anyhow, bail};

use crate::uar::domain::collaboration::{
    CollaborationDefinitionRecord, CollaborationKind, ConversionDiagnostic,
    ConversionDisposition, PackageManifest, PackagePreflightResponse, PackageSourceRequest,
    PreparedPackage,
};

use canonical::{digest, parse_unique_json, verify_content_digest};
use graph::validate_package_graph;
use projection::{conversion_diagnostics, project_agent};
use schema::{validate_manifest, validate_portable_document};

pub(super) use canonical::{canonical_digest, request_digest};
pub(super) use schema::{
    validate_common_sections, validate_digest, validate_id, validate_semver,
};

pub fn prepare_package(request: &PackageSourceRequest) -> Result<PreparedPackage> {
    if request.command_id.trim().is_empty() {
        bail!("commandId must not be empty");
    }
    let manifest_document =
        parse_unique_json(&request.manifest).context("manifest is not canonicalizable JSON")?;
    verify_content_digest(&manifest_document)?;
    let manifest: PackageManifest = serde_json::from_value(manifest_document.clone())
        .context("manifest does not match the collaboration package shape")?;
    validate_manifest(&manifest, &request.files)?;

    let package_ref = manifest.definition_ref();
    let mut definitions = Vec::with_capacity(manifest.files.len());
    let mut documents_by_path = BTreeMap::new();
    let mut diagnostics = conversion_diagnostics(
        &manifest_document,
        &CollaborationKind::PackageManifest,
    );

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
        validate_portable_document(&document, &file.kind, &file.definition)?;
        let mut document_diagnostics = conversion_diagnostics(&document, &file.kind);
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
        definitions.push(CollaborationDefinitionRecord {
            identity: file.definition.clone(),
            kind: file.kind.clone(),
            document,
            source_json: source.clone(),
            byte_digest: file.byte_digest.clone(),
            package: package_ref.clone(),
            conversion_diagnostics: document_diagnostics,
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
