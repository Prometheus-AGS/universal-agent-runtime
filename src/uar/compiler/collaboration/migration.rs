//! Lossless legacy and draft.1 migration into the draft.2 definition profile.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use anyhow::{Result, anyhow};
use chrono::Utc;
use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};

use crate::uar::compiler::ir::{AgentDescriptorIR, LegacySourceSection, SectionName};
use crate::uar::domain::collaboration::{
    COLLABORATION_PROFILE_DRAFT_1, COLLABORATION_PROFILE_DRAFT_2, CollaborationDocumentIdentity,
    CollaborationKind, ConversionDisposition, ConversionReport, ConversionTarget, FieldDiagnostic,
    MigrationReceipt, SourceIdentity,
};

pub(crate) struct LegacyMigration {
    pub canonical_definition: Value,
    pub definition_ref: Value,
    pub receipt: MigrationReceipt,
    pub conversion_report: ConversionReport,
}

pub(crate) struct Draft1Migration {
    pub canonical_document: Value,
    pub receipt: MigrationReceipt,
}

pub(crate) fn migrate_legacy_agent(ir: &AgentDescriptorIR) -> LegacyMigration {
    let fallback_id = format!("urn:uar:legacy:{}", slugify(&ir.agent_name));
    let source_id = if ir.source.id.is_empty() {
        fallback_id
    } else {
        ir.source.id.clone()
    };
    let source_version = if ir.source.version.is_empty() {
        ir.metadata.version.clone()
    } else {
        ir.source.version.clone()
    };
    let source_digest = if ir.source.digest.is_empty() {
        canonical_digest(
            &serde_json::to_value(ir)
                .expect("compiler IR contains only serializable domain values"),
        )
    } else {
        ir.source.digest.clone()
    };
    let source = SourceIdentity {
        profile: if ir.source.profile.is_empty() {
            "urn:prometheus:uar:agent-md:1.1".to_owned()
        } else {
            ir.source.profile.clone()
        },
        id: source_id.clone(),
        version: source_version,
        digest: source_digest,
        revision: ir.source.revision,
    };
    let target_version = normalize_semver(&ir.metadata.version);
    let target_id = if ir.source.rename_mapping.target_id.is_empty() {
        source_id
    } else {
        ir.source.rename_mapping.target_id.clone()
    };
    let skills = canonical_skills(ir);
    let requirements = [
        ("modelRequirements", SectionName::ModelRequirements),
        ("promptDialect", SectionName::PromptDialect),
        ("ragConfiguration", SectionName::RagConfiguration),
        ("contextStrategy", SectionName::ContextStrategy),
        ("apiHarness", SectionName::ApiHarness),
    ];
    let mut diagnostics = Vec::new();
    for (index, skill) in ir.skills.skills.iter().enumerate() {
        let complete = skill.version.is_some() && skill.digest.is_some();
        diagnostics.push(field_diagnostic(
            format!("/skills/{index}"),
            if complete {
                ConversionDisposition::Translated
            } else if skill.required {
                ConversionDisposition::RequiredUnsupported
            } else {
                ConversionDisposition::OptionalUnsupported
            },
            if complete {
                "skill.private-binding-required"
            } else {
                "skill.legacy-identity-normalized"
            },
            "The complete SkillRef is retained for private binding; projection storage does not establish runtime support.",
        ));
    }
    let mut semantic_requirements = Map::new();
    for (target_name, section_name) in requirements {
        let requirement = semantic_requirement(ir.source.section(section_name));
        let required = requirement
            .get("required")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        semantic_requirements.insert(target_name.to_owned(), requirement);
        diagnostics.push(field_diagnostic(
            format!("/{target_name}"),
            if required {
                ConversionDisposition::RequiredUnsupported
            } else {
                ConversionDisposition::OptionalUnsupported
            },
            "requirement.private-binding-required",
            "The authored requirement is preserved for private binding; projection storage does not establish runtime support.",
        ));
    }

    let legacy_sections = Value::Object(
        ir.source
            .sections
            .iter()
            .map(|section| (section.heading.clone(), section.value.clone()))
            .collect(),
    );
    let mut definition = json!({
        "profile": COLLABORATION_PROFILE_DRAFT_2,
        "kind": "AgentDefinition",
        "id": target_id,
        "version": target_version,
        "provenance": {
            "source": ir.source.profile,
            "authors": [ir.metadata.author.clone().unwrap_or_else(|| "Prometheus-AGS".to_owned())]
        },
        "requiredCapabilities": ["collaboration_definition_packages_v2"],
        "extensions": {},
        "title": ir.agent_name,
        "role": ir.identity.role,
        "whenToUse": nonempty(ir.metadata.description.as_deref().unwrap_or(&ir.identity.persona), "Imported legacy agent descriptor."),
        "instructions": migration_instructions(ir),
        "input": {},
        "output": {},
        "skills": skills,
        "models": [{"role": "primary", "capabilities": [], "preferredAliases": []}],
        "permittedChildren": [],
        "context": {"mode": "none", "artifacts": [], "history": "none", "memoryScopes": []},
        "requestedLimits": requested_limits(ir),
        "legacySections": legacy_sections,
        "sourceDescriptor": ir.source,
        "sourceIdentity": source,
        "renameMapping": rename_mapping(ir, &source.id, &target_id),
        "authoredFields": ir.source.authored_fields,
    });
    definition
        .as_object_mut()
        .expect("legacy migration always builds an object")
        .extend(semantic_requirements);
    let definition_digest = canonical_digest(&definition);
    definition
        .as_object_mut()
        .expect("legacy migration always builds an object")
        .insert(
            "contentDigest".to_owned(),
            Value::String(definition_digest.clone()),
        );

    let target = CollaborationDocumentIdentity {
        profile: COLLABORATION_PROFILE_DRAFT_2.to_owned(),
        kind: CollaborationKind::AgentDefinition,
        id: definition["id"].as_str().unwrap_or_default().to_owned(),
        version: definition["version"]
            .as_str()
            .unwrap_or_default()
            .to_owned(),
        content_digest: definition_digest.clone(),
    };
    let receipt = MigrationReceipt {
        source: source.clone(),
        target: target.clone(),
        diagnostics: diagnostics.clone(),
        migrated_at: ir.source.migrated_at.unwrap_or_else(Utc::now),
    };
    let activation_blocked = diagnostics
        .iter()
        .any(|diagnostic| diagnostic.disposition == ConversionDisposition::RequiredUnsupported);
    let mut conversion_report = ConversionReport {
        profile: COLLABORATION_PROFILE_DRAFT_2.to_owned(),
        kind: CollaborationKind::ConversionReport,
        id: format!("urn:uar:conversion:{}", target.id),
        version: target.version.clone(),
        content_digest: String::new(),
        provenance: json!({"source": "UAR legacy Agent Markdown migration", "authors": ["Prometheus-AGS"]}),
        required_capabilities: Vec::new(),
        extensions: BTreeMap::new(),
        export_class: "portable-evidence".to_owned(),
        source,
        target: ConversionTarget {
            profile: Some(COLLABORATION_PROFILE_DRAFT_2.to_owned()),
            harness: None,
        },
        diagnostics,
        activation_blocked,
    };
    conversion_report.content_digest = canonical_digest(
        &serde_json::to_value(&conversion_report)
            .expect("conversion report contains only serializable domain values"),
    );

    LegacyMigration {
        definition_ref: json!({
            "id": target.id,
            "version": target.version,
            "digest": target.content_digest,
        }),
        canonical_definition: definition,
        receipt,
        conversion_report,
    }
}

/// Migrate a draft.1 JSON definition without modifying its source identity or digest.
pub(crate) fn migrate_draft1_document(document: &Value) -> Result<Draft1Migration> {
    let object = document
        .as_object()
        .ok_or_else(|| anyhow!("draft.1 collaboration document must be an object"))?;
    let source = SourceIdentity {
        profile: required_string(object, "profile")?.to_owned(),
        id: required_string(object, "id")?.to_owned(),
        version: required_string(object, "version")?.to_owned(),
        digest: required_string(object, "contentDigest")?.to_owned(),
        revision: object.get("revision").and_then(Value::as_u64),
    };
    if source.profile != COLLABORATION_PROFILE_DRAFT_1 {
        return Err(anyhow!(
            "migration input profile must be '{COLLABORATION_PROFILE_DRAFT_1}'"
        ));
    }
    let mut canonical_document = document.clone();
    let canonical = canonical_document
        .as_object_mut()
        .expect("draft.1 object shape was checked above");
    canonical.insert(
        "profile".to_owned(),
        Value::String(COLLABORATION_PROFILE_DRAFT_2.to_owned()),
    );
    canonical.remove("contentDigest");
    let target_digest = canonical_digest(&canonical_document);
    canonical_document
        .as_object_mut()
        .expect("draft.1 object shape was checked above")
        .insert(
            "contentDigest".to_owned(),
            Value::String(target_digest.clone()),
        );
    let kind: CollaborationKind = serde_json::from_value(
        canonical_document
            .get("kind")
            .cloned()
            .ok_or_else(|| anyhow!("draft.1 collaboration document is missing kind"))?,
    )?;
    let target = CollaborationDocumentIdentity {
        profile: COLLABORATION_PROFILE_DRAFT_2.to_owned(),
        kind,
        id: source.id.clone(),
        version: source.version.clone(),
        content_digest: target_digest,
    };
    let receipt = MigrationReceipt {
        source,
        target,
        diagnostics: vec![field_diagnostic(
            "/profile".to_owned(),
            ConversionDisposition::Translated,
            "profile.draft1-to-draft2",
            "The normalized document has a new draft.2 digest; the draft.1 identity and digest remain unchanged in this receipt.",
        )],
        migrated_at: Utc::now(),
    };
    Ok(Draft1Migration {
        canonical_document,
        receipt,
    })
}

fn canonical_skills(ir: &AgentDescriptorIR) -> Vec<Value> {
    ir.skills
        .skills
        .iter()
        .enumerate()
        .map(|(index, skill)| {
            let source_value = source_skill(ir, index).unwrap_or_else(|| {
                serde_json::to_value(skill)
                    .expect("compiler SkillRef contains only serializable values")
            });
            json!({
                "id": skill.id,
                "version": skill.version.clone().unwrap_or_else(|| "0.0.0".to_owned()),
                "digest": skill.digest.clone().unwrap_or_else(|| canonical_digest(&source_value)),
                "required": skill.required,
                "config": skill.config,
                "entrypoint": skill.entrypoint,
                "requiredTools": skill.required_tools,
            })
        })
        .collect()
}

fn source_skill(ir: &AgentDescriptorIR, index: usize) -> Option<Value> {
    let value = &ir.source.section(SectionName::Skills)?.value;
    value
        .as_array()
        .or_else(|| value.get("skills").and_then(Value::as_array))
        .and_then(|skills| skills.get(index))
        .cloned()
}

fn semantic_requirement(section: Option<&LegacySourceSection>) -> Value {
    let Some(section) = section else {
        return json!({"required": false, "value": null});
    };
    let required = section
        .value
        .get("required")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let value = if let Some(value) = section.value.get("value") {
        value.clone()
    } else if let Some(object) = section.value.as_object() {
        let mut value = object.clone();
        value.remove("required");
        Value::Object(value)
    } else {
        section.value.clone()
    };
    json!({"required": required, "value": value})
}

fn requested_limits(ir: &AgentDescriptorIR) -> Value {
    json!({
        "concurrentTurns": 1,
        "maxMembers": 1,
        "maxDepth": 0,
        "maxPendingTasks": ir.execution.max_iterations.unwrap_or(1).max(1),
    })
}

fn migration_instructions(ir: &AgentDescriptorIR) -> String {
    let mut instructions = Vec::new();
    if let Some(system) = ir.identity.system_prompt.as_deref() {
        instructions.push(system.to_owned());
    }
    instructions.extend(ir.identity.instructions.iter().cloned());
    if instructions.is_empty() {
        nonempty(
            &ir.identity.persona,
            "Follow the imported legacy agent descriptor.",
        )
        .to_owned()
    } else {
        instructions.join("\n")
    }
}

fn rename_mapping(ir: &AgentDescriptorIR, source_id: &str, target_id: &str) -> Value {
    if ir.source.rename_mapping.source_id.is_empty() {
        json!({
            "sourceId": source_id,
            "targetId": target_id,
            "reason": if source_id == target_id { "unchanged" } else { "heading-slug" },
        })
    } else {
        serde_json::to_value(&ir.source.rename_mapping)
            .expect("legacy rename mapping contains only serializable values")
    }
}

fn nonempty<'a>(value: &'a str, fallback: &'a str) -> &'a str {
    if value.trim().is_empty() {
        fallback
    } else {
        value
    }
}

fn slugify(value: &str) -> String {
    let mut slug = String::new();
    let mut separator = false;
    for character in value.chars() {
        if character.is_ascii_alphanumeric() {
            slug.push(character.to_ascii_lowercase());
            separator = false;
        } else if !separator && !slug.is_empty() {
            slug.push('-');
            separator = true;
        }
    }
    let slug = slug.trim_matches('-');
    if slug.is_empty() {
        "agent".to_owned()
    } else {
        slug.to_owned()
    }
}

fn normalize_semver(version: &str) -> String {
    match version.split('.').count() {
        1 => format!("{version}.0.0"),
        2 => format!("{version}.0"),
        _ => version.to_owned(),
    }
}

fn required_string<'a>(object: &'a Map<String, Value>, key: &str) -> Result<&'a str> {
    object
        .get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| anyhow!("draft.1 collaboration document is missing '{key}'"))
}

fn field_diagnostic(
    pointer: String,
    disposition: ConversionDisposition,
    reason_code: &str,
    message: &str,
) -> FieldDiagnostic {
    FieldDiagnostic {
        pointer,
        disposition,
        reason_code: reason_code.to_owned(),
        message: message.to_owned(),
        effective_binding_ref: None,
    }
}

fn canonical_digest(value: &Value) -> String {
    let mut without_digest = value.clone();
    if let Some(object) = without_digest.as_object_mut() {
        object.remove("contentDigest");
    }
    let canonical = canonical_json_value(&without_digest);
    let bytes = serde_json::to_vec(&canonical)
        .expect("serde_json::Value is always serializable to canonical JSON");
    let mut encoded = String::from("sha256:");
    for byte in Sha256::digest(bytes) {
        write!(&mut encoded, "{byte:02x}").expect("writing to String cannot fail");
    }
    encoded
}

fn canonical_json_value(value: &Value) -> Value {
    match value {
        Value::Object(object) => {
            let mut entries = object.iter().collect::<Vec<_>>();
            entries.sort_unstable_by(|left, right| left.0.cmp(right.0));
            Value::Object(
                entries
                    .into_iter()
                    .map(|(key, value)| (key.clone(), canonical_json_value(value)))
                    .collect(),
            )
        }
        Value::Array(values) => Value::Array(values.iter().map(canonical_json_value).collect()),
        _ => value.clone(),
    }
}
