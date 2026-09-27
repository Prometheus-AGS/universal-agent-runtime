use std::collections::HashMap;

use anyhow::{Context, Result, anyhow, bail};
use jsonschema::{Retrieve, Uri};
use serde_json::Value;

use crate::uar::domain::collaboration::{COLLABORATION_PROFILE_DRAFT_2, CollaborationKind};

const SCHEMA_BASE: &str = "https://schemas.prometheus-ags.dev/uar/collaboration/0.1.0-draft.2/";

const SCHEMAS: &[(&str, &str)] = &[
    (
        "common.schema.json",
        include_str!(
            "../../../../../docs/agents/collaboration/v0.1.0-draft.2/schemas/common.schema.json"
        ),
    ),
    (
        "collaboration-document.schema.json",
        include_str!(
            "../../../../../docs/agents/collaboration/v0.1.0-draft.2/schemas/collaboration-document.schema.json"
        ),
    ),
    (
        "agent-definition.schema.json",
        include_str!(
            "../../../../../docs/agents/collaboration/v0.1.0-draft.2/schemas/agent-definition.schema.json"
        ),
    ),
    (
        "team-definition.schema.json",
        include_str!(
            "../../../../../docs/agents/collaboration/v0.1.0-draft.2/schemas/team-definition.schema.json"
        ),
    ),
    (
        "workflow-definition.schema.json",
        include_str!(
            "../../../../../docs/agents/collaboration/v0.1.0-draft.2/schemas/workflow-definition.schema.json"
        ),
    ),
    (
        "package-manifest.schema.json",
        include_str!(
            "../../../../../docs/agents/collaboration/v0.1.0-draft.2/schemas/package-manifest.schema.json"
        ),
    ),
    (
        "deployment-binding.schema.json",
        include_str!(
            "../../../../../docs/agents/collaboration/v0.1.0-draft.2/schemas/deployment-binding.schema.json"
        ),
    ),
    (
        "representation-grant.schema.json",
        include_str!(
            "../../../../../docs/agents/collaboration/v0.1.0-draft.2/schemas/representation-grant.schema.json"
        ),
    ),
    (
        "conversion-report.schema.json",
        include_str!(
            "../../../../../docs/agents/collaboration/v0.1.0-draft.2/schemas/conversion-report.schema.json"
        ),
    ),
    (
        "effective-binding-receipt.schema.json",
        include_str!(
            "../../../../../docs/agents/collaboration/v0.1.0-draft.2/schemas/effective-binding-receipt.schema.json"
        ),
    ),
    (
        "deployment-binding-template.schema.json",
        include_str!(
            "../../../../../docs/agents/collaboration/v0.1.0-draft.2/schemas/deployment-binding-template.schema.json"
        ),
    ),
];

#[derive(Clone)]
struct EmbeddedSchemas {
    by_uri: HashMap<String, Value>,
}

impl EmbeddedSchemas {
    fn load() -> Result<Self> {
        let mut by_uri = HashMap::with_capacity(SCHEMAS.len());
        for (name, source) in SCHEMAS {
            let schema = serde_json::from_str(source).with_context(|| {
                format!("committed collaboration schema '{name}' is invalid JSON")
            })?;
            by_uri.insert(format!("{SCHEMA_BASE}{name}"), schema);
        }
        Ok(Self { by_uri })
    }

    fn schema(&self, name: &str) -> Result<&Value> {
        self.by_uri
            .get(&format!("{SCHEMA_BASE}{name}"))
            .ok_or_else(|| anyhow!("embedded collaboration schema '{name}' is unavailable"))
    }
}

impl Retrieve for EmbeddedSchemas {
    fn retrieve(
        &self,
        uri: &Uri<String>,
    ) -> std::result::Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        self.by_uri
            .get(uri.as_str())
            .cloned()
            .ok_or_else(|| format!("embedded collaboration schema is unavailable: {uri}").into())
    }
}

pub(super) fn validate_document(document: &Value) -> Result<CollaborationKind> {
    let object = document
        .as_object()
        .ok_or_else(|| anyhow!("schema.invalid at /: collaboration document must be an object"))?;
    let profile = object
        .get("profile")
        .and_then(Value::as_str)
        .unwrap_or_default();
    if profile != COLLABORATION_PROFILE_DRAFT_2 {
        bail!("schema.invalid at /profile: unsupported collaboration profile");
    }
    let kind = object
        .get("kind")
        .and_then(Value::as_str)
        .and_then(parse_kind)
        .ok_or_else(|| anyhow!("schema.invalid at /kind: unsupported collaboration kind"))?;

    let schemas = EmbeddedSchemas::load()?;
    validate_against(
        &schemas,
        schemas.schema("collaboration-document.schema.json")?,
        document,
    )?;
    validate_against(&schemas, schemas.schema(schema_name(&kind))?, document)?;
    Ok(kind)
}

fn validate_against(schemas: &EmbeddedSchemas, schema: &Value, document: &Value) -> Result<()> {
    let validator = jsonschema::options()
        .with_retriever(schemas.clone())
        .build(schema)
        .map_err(|_| anyhow!("embedded collaboration schema could not be compiled"))?;
    let mut paths = validator
        .iter_errors(document)
        .map(|error| {
            let pointer = error.instance_path().as_str();
            if pointer.is_empty() {
                "/".to_owned()
            } else {
                pointer.to_owned()
            }
        })
        .collect::<Vec<_>>();
    if paths.is_empty() {
        return Ok(());
    }
    paths.sort();
    paths.dedup();
    bail!(
        "schema.invalid at {}: document does not satisfy the selected schema",
        paths.join(", ")
    )
}

fn parse_kind(kind: &str) -> Option<CollaborationKind> {
    Some(match kind {
        "AgentDefinition" => CollaborationKind::AgentDefinition,
        "TeamDefinition" => CollaborationKind::TeamDefinition,
        "WorkflowDefinition" => CollaborationKind::WorkflowDefinition,
        "PackageManifest" => CollaborationKind::PackageManifest,
        "DeploymentBinding" => CollaborationKind::DeploymentBinding,
        "RepresentationGrant" => CollaborationKind::RepresentationGrant,
        "ConversionReport" => CollaborationKind::ConversionReport,
        "EffectiveBindingReceipt" => CollaborationKind::EffectiveBindingReceipt,
        "DeploymentBindingTemplate" => CollaborationKind::DeploymentBindingTemplate,
        _ => return None,
    })
}

fn schema_name(kind: &CollaborationKind) -> &'static str {
    match kind {
        CollaborationKind::AgentDefinition => "agent-definition.schema.json",
        CollaborationKind::TeamDefinition => "team-definition.schema.json",
        CollaborationKind::WorkflowDefinition => "workflow-definition.schema.json",
        CollaborationKind::PackageManifest => "package-manifest.schema.json",
        CollaborationKind::DeploymentBinding => "deployment-binding.schema.json",
        CollaborationKind::RepresentationGrant => "representation-grant.schema.json",
        CollaborationKind::ConversionReport => "conversion-report.schema.json",
        CollaborationKind::EffectiveBindingReceipt => "effective-binding-receipt.schema.json",
        CollaborationKind::DeploymentBindingTemplate => "deployment-binding-template.schema.json",
    }
}
