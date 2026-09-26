use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, anyhow, bail};
use serde_json::Value;

use crate::uar::domain::collaboration::{
    COLLABORATION_PROFILE, CollaborationKind, ImmutableDefinitionRef, PackageManifest,
};

use super::canonical::verify_content_digest;
use super::projection::reject_portable_authority;

pub(super) fn validate_manifest(
    manifest: &PackageManifest,
    files: &BTreeMap<String, String>,
) -> Result<()> {
    if manifest.profile != COLLABORATION_PROFILE
        || manifest.kind != CollaborationKind::PackageManifest
        || manifest.resolution != "exact-version-and-digest"
    {
        bail!("unsupported collaboration package profile, kind, or resolution mode");
    }
    validate_id(&manifest.id)?;
    validate_semver(&manifest.version)?;
    validate_digest(&manifest.content_digest)?;
    let document = serde_json::to_value(manifest).context("failed to inspect package manifest")?;
    validate_common_sections(&document)?;
    reject_portable_authority(&document, "$")?;
    if manifest.entrypoints.is_empty() || manifest.files.is_empty() {
        bail!("package must contain files and at least one entrypoint");
    }
    let declared_paths = manifest
        .files
        .iter()
        .map(|file| file.path.as_str())
        .collect::<BTreeSet<_>>();
    if declared_paths.len() != manifest.files.len() {
        bail!("package manifest contains duplicate file paths");
    }
    let supplied_paths = files.keys().map(String::as_str).collect::<BTreeSet<_>>();
    if declared_paths != supplied_paths {
        bail!("supplied files must exactly match the manifest inventory");
    }
    for file in &manifest.files {
        validate_relative_path(&file.path)?;
        validate_reference(&file.definition)?;
        validate_digest(&file.byte_digest)?;
        if matches!(
            &file.kind,
            CollaborationKind::PackageManifest | CollaborationKind::DeploymentBinding
        ) {
            bail!("portable package files may contain only agent, team, or workflow definitions");
        }
    }
    for entrypoint in &manifest.entrypoints {
        validate_reference(entrypoint)?;
        if !manifest
            .files
            .iter()
            .any(|file| &file.definition == entrypoint)
        {
            bail!(
                "entrypoint '{}' is absent from the package inventory",
                entrypoint.id
            );
        }
    }
    let lock_keys = manifest
        .lock
        .iter()
        .map(|entry| (entry.requested_by.as_str(), entry.reference.storage_key()))
        .collect::<BTreeSet<_>>();
    if lock_keys.len() != manifest.lock.len() {
        bail!("package manifest contains duplicate lock entries");
    }
    for entry in &manifest.lock {
        validate_id(&entry.requested_by)?;
        validate_reference(&entry.reference)?;
        validate_relative_path(&entry.resolved_path)?;
        if !manifest.files.iter().any(|file| {
            file.path == entry.resolved_path && file.definition == entry.reference
        }) {
            bail!(
                "lock entry for '{}' does not resolve to its declared package file",
                entry.reference.id
            );
        }
    }
    let declared_capabilities = manifest
        .capability_declarations
        .iter()
        .map(|declaration| declaration.capability.as_str())
        .collect::<BTreeSet<_>>();
    if declared_capabilities.len() != manifest.capability_declarations.len() {
        bail!("package manifest contains duplicate capability declarations");
    }
    for capability in &manifest.required_capabilities {
        if !manifest
            .capability_declarations
            .iter()
            .any(|declaration| declaration.required && declaration.capability == *capability)
        {
            bail!("required capability '{capability}' lacks a required declaration");
        }
    }
    Ok(())
}

pub(super) fn validate_portable_document(
    document: &Value,
    expected_kind: &CollaborationKind,
    expected_identity: &ImmutableDefinitionRef,
) -> Result<()> {
    let profile = required_string(document, "profile")?;
    let kind = required_string(document, "kind")?;
    let id = required_string(document, "id")?;
    let version = required_string(document, "version")?;
    if profile != COLLABORATION_PROFILE
        || kind != expected_kind.as_str()
        || id != expected_identity.id
        || version != expected_identity.version
    {
        bail!("definition identity does not match its manifest entry");
    }
    validate_id(id)?;
    validate_semver(version)?;
    validate_digest(&expected_identity.digest)?;
    verify_content_digest(document)?;
    if required_string(document, "contentDigest")? != expected_identity.digest {
        bail!("definition contentDigest does not match the immutable manifest reference");
    }
    validate_required_shape(document, expected_kind)?;
    validate_common_sections(document)?;
    reject_portable_authority(document, "$")?;
    validate_contracts(document)?;
    Ok(())
}

pub(in crate::uar::compiler::collaboration) fn validate_common_sections(
    document: &Value,
) -> Result<()> {
    let provenance = document
        .get("provenance")
        .and_then(Value::as_object)
        .ok_or_else(|| anyhow!("'provenance' must be an object"))?;
    if provenance.is_empty() {
        bail!("'provenance' must not be empty");
    }
    for capability in required_array(document, "requiredCapabilities")? {
        if capability.as_str().is_none_or(str::is_empty) {
            bail!("requiredCapabilities entries must be non-empty strings");
        }
    }
    let extensions = document
        .get("extensions")
        .and_then(Value::as_object)
        .ok_or_else(|| anyhow!("'extensions' must be an object"))?;
    for (name, extension) in extensions {
        if name.is_empty() || !extension.is_object() {
            bail!("extension entries must use a non-empty name and object value");
        }
        if extension
            .get("required")
            .is_some_and(|value| !value.is_boolean())
        {
            bail!("extension '{name}' required flag must be boolean");
        }
    }
    Ok(())
}

fn validate_required_shape(document: &Value, kind: &CollaborationKind) -> Result<()> {
    let common = [
        "profile",
        "kind",
        "id",
        "version",
        "contentDigest",
        "provenance",
        "requiredCapabilities",
        "extensions",
    ];
    let specific: &[&str] = match kind {
        CollaborationKind::AgentDefinition => &[
            "title",
            "role",
            "whenToUse",
            "instructions",
            "input",
            "output",
            "skills",
            "models",
            "permittedChildren",
            "context",
            "requestedLimits",
        ],
        CollaborationKind::TeamDefinition => &[
            "title",
            "purpose",
            "members",
            "coordinatorRole",
            "communication",
            "taskAcceptance",
            "routing",
            "limits",
            "budget",
            "input",
            "output",
        ],
        CollaborationKind::WorkflowDefinition => &[
            "title",
            "input",
            "output",
            "steps",
            "failurePolicy",
            "maxActivations",
        ],
        _ => bail!("unsupported portable definition kind"),
    };
    let object = document
        .as_object()
        .ok_or_else(|| anyhow!("definition must be a JSON object"))?;
    for field in common.iter().chain(specific) {
        if !object.contains_key(*field) {
            bail!("required field '{field}' is missing");
        }
    }
    let mut allowed = common
        .iter()
        .chain(specific)
        .copied()
        .collect::<BTreeSet<_>>();
    if *kind == CollaborationKind::AgentDefinition {
        allowed.extend(["legacySections", "sourceDescriptor"]);
    }
    for field in object.keys() {
        if !allowed.contains(field.as_str()) {
            bail!("definition contains unknown {} field '{field}'", kind.as_str());
        }
    }
    Ok(())
}

fn validate_contracts(document: &Value) -> Result<()> {
    for field in ["input", "output"] {
        if let Some(schema) = document.get(field) {
            jsonschema::validator_for(schema)
                .with_context(|| format!("{field} is not a valid JSON Schema"))?;
        }
    }
    if let Some(steps) = document.get("steps").and_then(Value::as_array) {
        for (index, step) in steps.iter().enumerate() {
            if let Some(schema) = step.get("output") {
                jsonschema::validator_for(schema).with_context(|| {
                    format!("steps[{index}].output is not a valid JSON Schema")
                })?;
            }
        }
    }
    Ok(())
}

pub(super) fn validate_reference(reference: &ImmutableDefinitionRef) -> Result<()> {
    validate_id(&reference.id)?;
    validate_semver(&reference.version)?;
    validate_digest(&reference.digest)
}

fn validate_relative_path(path: &str) -> Result<()> {
    if path.is_empty()
        || path.starts_with('/')
        || path
            .split('/')
            .any(|segment| segment.is_empty() || segment == "..")
        || !path
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '/' | '-'))
    {
        bail!("invalid manifest-relative path '{path}'");
    }
    Ok(())
}

pub(in crate::uar::compiler::collaboration) fn validate_id(value: &str) -> Result<()> {
    if value.is_empty()
        || !value.as_bytes()[0].is_ascii_alphanumeric()
        || !value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, ':' | '.' | '_' | '/' | '-'))
    {
        bail!("invalid collaboration id '{value}'");
    }
    Ok(())
}

pub(in crate::uar::compiler::collaboration) fn validate_semver(value: &str) -> Result<()> {
    let core = value.split_once('-').map_or(value, |(core, _)| core);
    let parts = core.split('.').collect::<Vec<_>>();
    if parts.len() != 3
        || parts
            .iter()
            .any(|part| part.is_empty() || !part.chars().all(|ch| ch.is_ascii_digit()))
    {
        bail!("invalid semantic version '{value}'");
    }
    Ok(())
}

pub(in crate::uar::compiler::collaboration) fn validate_digest(value: &str) -> Result<()> {
    if value.len() != 71
        || !value.starts_with("sha256:")
        || !value[7..]
            .chars()
            .all(|ch| ch.is_ascii_hexdigit() && !ch.is_ascii_uppercase())
    {
        bail!("invalid SHA-256 digest '{value}'");
    }
    Ok(())
}

pub(super) fn required_string<'a>(value: &'a Value, field: &str) -> Result<&'a str> {
    value
        .get(field)
        .and_then(Value::as_str)
        .filter(|text| !text.is_empty())
        .ok_or_else(|| anyhow!("'{field}' must be a non-empty string"))
}

pub(super) fn required_array<'a>(value: &'a Value, field: &str) -> Result<&'a Vec<Value>> {
    value
        .get(field)
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("'{field}' must be an array"))
}

pub(super) fn required_u64(value: &Value, field: &str) -> Result<u64> {
    value
        .get(field)
        .and_then(Value::as_u64)
        .ok_or_else(|| anyhow!("'{field}' must be a non-negative integer"))
}
