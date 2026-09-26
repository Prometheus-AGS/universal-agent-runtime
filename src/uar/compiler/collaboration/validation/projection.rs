use anyhow::{Result, bail};
use serde_json::Value;

use crate::uar::defaults::default_agent;
use crate::uar::domain::{
    artifact::AgentArtifact,
    collaboration::{CollaborationKind, ConversionDiagnostic, ConversionDisposition},
};

use super::schema::{required_array, required_string};

const SUPPORTED_INSTALL_CAPABILITIES: &[&str] = &["collaboration_definition_packages_v1"];

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
        if !SUPPORTED_INSTALL_CAPABILITIES.contains(&capability) {
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
        for field in ["title", "role", "whenToUse", "instructions", "input", "output"] {
            diagnostics.push(ConversionDiagnostic {
                field: field.to_owned(),
                disposition: ConversionDisposition::Exact,
                message: format!("'{field}' is preserved in the compatibility projection"),
            });
        }
        for field in ["skills", "models", "context", "requestedLimits", "permittedChildren"] {
            diagnostics.push(ConversionDiagnostic {
                field: field.to_owned(),
                disposition: ConversionDisposition::Translated,
                message: format!(
                    "'{field}' is retained losslessly; only supported legacy runtime semantics are projected"
                ),
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
    agent.extensions.insert(
        "uar.collaboration/definition".to_owned(),
        document.clone(),
    );
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
