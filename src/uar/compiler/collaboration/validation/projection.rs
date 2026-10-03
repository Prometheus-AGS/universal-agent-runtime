use anyhow::{Result, bail};
use serde_json::Value;

use crate::uar::defaults::default_agent;
use crate::uar::domain::{
    artifact::AgentArtifact,
    collaboration::{CollaborationKind, ConversionDiagnostic, ConversionDisposition},
};

use super::schema::{required_array, required_string};

const SUPPORTED_INSTALL_CAPABILITIES: &[&str] = &[
    "collaboration_definition_packages_v1",
    "collaboration_definition_packages_v2",
];

pub(super) fn conversion_diagnostics(
    document: &Value,
    kind: &CollaborationKind,
) -> Vec<ConversionDiagnostic> {
    let mut diagnostics = Vec::new();
    for capability in document
        .get("requiredCapabilities")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
    {
        if !(crate::uar::api::capabilities::workflow_execution_enabled() && capability == crate::uar::domain::workflow_execution::WORKFLOW_CAPABILITY) && !SUPPORTED_INSTALL_CAPABILITIES.contains(&capability) && !(crate::uar::api::capabilities::team_execution_b_enabled() && crate::uar::api::capabilities::TEAM_EXECUTION_B_CAPABILITIES.contains(&capability)) {
            diagnostics.push(ConversionDiagnostic {
                field: "requiredCapabilities".to_owned(),
                disposition: ConversionDisposition::RequiredUnsupported,
                message: format!("required capability '{capability}' is not implemented"),
            });
        }
    }
    for (name, extension) in document
        .get("extensions")
        .and_then(Value::as_object)
        .into_iter()
        .flatten()
    {
        if name == crate::uar::domain::workflow_execution::WORKFLOW_EXTENSION
            && crate::uar::api::capabilities::workflow_execution_enabled()
            && serde_json::from_str::<Value>(include_str!("../../../../../docs/agents/collaboration/workflow-execution/1.0.0/extension.schema.json"))
                .ok().and_then(|schema|jsonschema::validator_for(&schema).ok()).is_some_and(|validator|validator.is_valid(extension)) {
            continue;
        }
        diagnostics.push(ConversionDiagnostic {
            field: format!("extensions.{name}"),
            disposition: if extension
                .get("required")
                .and_then(Value::as_bool)
                .unwrap_or(false)
            {
                ConversionDisposition::RequiredUnsupported
            } else {
                ConversionDisposition::OptionalUnsupported
            },
            message: format!("extension '{name}' has no installed adapter"),
        });
    }
    if *kind == CollaborationKind::AgentDefinition {
        for field in document
            .as_object()
            .into_iter()
            .flatten()
            .map(|(field, _)| field)
            .filter(|field| {
                field.as_str() != "extensions" && field.as_str() != "requiredCapabilities"
            })
        {
            diagnostics.push(ConversionDiagnostic {
                field: format!("/{field}"),
                disposition: ConversionDisposition::Translated,
                message: format!(
                    "'{field}' is retained for compatibility projection and private binding"
                ),
            });
        }
        for field in [
            "modelRequirements",
            "promptDialect",
            "ragConfiguration",
            "contextStrategy",
            "apiHarness",
        ] {
            let required = document
                .get(field)
                .and_then(|value| value.get("required"))
                .and_then(Value::as_bool)
                .unwrap_or(false);
            diagnostics.push(ConversionDiagnostic {
                field: format!("/{field}"),
                disposition: if required {
                    ConversionDisposition::RequiredUnsupported
                } else {
                    ConversionDisposition::OptionalUnsupported
                },
                message: format!(
                    "'{field}' is retained for private binding without a runtime support claim"
                ),
            });
        }
        for (index, _skill) in document
            .get("skills")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .enumerate()
        {
            diagnostics.push(ConversionDiagnostic {
                field: format!("/skills/{index}"),
                disposition: ConversionDisposition::Translated,
                message: "the complete SkillRef is retained for private binding without a runtime support claim".to_owned(),
            });
        }
    }
    diagnostics
}

pub(super) fn project_agent(document: &Value) -> Result<AgentArtifact> {
    let mut agent = default_agent();
    agent.id = required_string(document, "id")?.to_owned();
    agent.version = required_string(document, "version")?.to_owned();
    agent.metadata.title = required_string(document, "title")?.to_owned();
    agent.metadata.description = required_string(document, "whenToUse")?.to_owned();
    agent.prompt.system = required_string(document, "role")?.to_owned();
    agent.prompt.instructions = vec![required_string(document, "instructions")?.to_owned()];
    agent.schemas.inputs = document.get("input").cloned();
    agent.schemas.outputs = document.get("output").cloned();
    agent.policy.skills.prefer = required_array(document, "skills")?
        .iter()
        .filter_map(|skill| skill.get("id").and_then(Value::as_str))
        .map(str::to_owned)
        .collect();
    agent
        .extensions
        .insert("uar.collaboration/definition".to_owned(), document.clone());
    agent.extensions.insert(
        "uar.collaboration/definition-ref".to_owned(),
        serde_json::json!({
            "id": required_string(document, "id")?,
            "version": required_string(document, "version")?,
            "digest": required_string(document, "contentDigest")?,
        }),
    );
    agent.extensions.insert(
        "uar.collaboration/skill-refs".to_owned(),
        Value::Array(required_array(document, "skills")?.to_vec()),
    );
    for field in [
        "modelRequirements",
        "promptDialect",
        "ragConfiguration",
        "contextStrategy",
        "apiHarness",
    ] {
        agent.extensions.insert(
            format!("uar.collaboration/{field}"),
            document.get(field).cloned().unwrap_or(Value::Null),
        );
    }
    Ok(agent.with_catalog_metadata("collaboration-package"))
}

pub(super) fn reject_portable_authority(value: &Value, path: &str) -> Result<()> {
    match value {
        Value::Object(object) => {
            for (key, child) in object {
                let normalized = key.to_ascii_lowercase().replace(['-', '_'], "");
                if matches!(
                    normalized.as_str(),
                    "apikey"
                        | "password"
                        | "secret"
                        | "token"
                        | "credentialref"
                        | "connectionref"
                        | "ownerid"
                        | "workspaceid"
                        | "runtimeinstanceid"
                        | "representationgrantrefs"
                        | "approvaltoken"
                        | "consentevidence"
                ) {
                    bail!("portable document contains private authority field '{path}.{key}'");
                }
                reject_portable_authority(child, &format!("{path}.{key}"))?;
            }
        }
        Value::Array(values) => {
            for (index, child) in values.iter().enumerate() {
                reject_portable_authority(child, &format!("{path}[{index}]"))?;
            }
        }
        _ => {}
    }
    Ok(())
}
