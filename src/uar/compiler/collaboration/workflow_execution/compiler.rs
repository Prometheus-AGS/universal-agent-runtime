//! Closed interpreter for the installed portable workflow extension.
use super::super::{CollaborationError, validation::canonical_digest};
use crate::uar::domain::{
    collaboration::{CollaborationDefinitionRecord, CollaborationKind},
    workflow_execution::*,
};
use serde_json::{Value, json};

fn unsupported(field: &str, message: &str) -> CollaborationError {
    CollaborationError::Invalid(format!(
        "WORKFLOW_REQUIRED_UNSUPPORTED at {field}: {message}"
    ))
}

pub(super) fn compile(
    record: &CollaborationDefinitionRecord,
) -> Result<WorkflowCompiledPlan, CollaborationError> {
    let doc = &record.document;
    if record.kind != CollaborationKind::WorkflowDefinition {
        return Err(unsupported("/kind", "expected WorkflowDefinition"));
    }
    let extension = &doc["extensions"][WORKFLOW_EXTENSION];
    let schema: Value = serde_json::from_str(include_str!(
        "../../../../../docs/agents/collaboration/workflow-execution/1.0.0/extension.schema.json"
    ))?;
    let validator = jsonschema::validator_for(&schema)
        .map_err(|_| unsupported("/extensions", "extension schema unavailable"))?;
    if !validator.is_valid(extension) {
        return Err(unsupported(
            "/extensions/prometheus.workflow-execution",
            "closed version 1.0.0 extension required",
        ));
    }
    if !doc["requiredCapabilities"].as_array().is_some_and(|items| {
        items
            .iter()
            .any(|v| v.as_str() == Some(WORKFLOW_CAPABILITY))
    }) {
        return Err(unsupported(
            "/requiredCapabilities",
            "workflow execution capability must be required",
        ));
    }
    for capability in doc["requiredCapabilities"].as_array().into_iter().flatten() {
        if !matches!(
            capability.as_str(),
            Some(
                WORKFLOW_CAPABILITY
                    | "collaboration_definition_packages_v1"
                    | "collaboration_definition_packages_v2"
            )
        ) {
            return Err(unsupported(
                "/requiredCapabilities",
                "unknown required capability",
            ));
        }
    }
    for (name, extension) in doc["extensions"].as_object().into_iter().flatten() {
        if name != WORKFLOW_EXTENSION && extension["required"] == true {
            return Err(unsupported("/extensions", "unknown required extension"));
        }
    }
    if doc["failurePolicy"] != "stop-dependent" {
        return Err(unsupported(
            "/failurePolicy",
            "only stop-dependent is implemented",
        ));
    }
    let input = &doc["input"];
    if input["type"] != "object"
        || input["additionalProperties"] != false
        || input["required"] != json!(["feedback"])
        || input["properties"].as_object().map(|v| v.len()) != Some(1)
        || input["properties"]["feedback"]["type"] != "string"
        || input["properties"]["feedback"]["maxLength"]
            .as_u64()
            .filter(|n| *n > 0)
            .is_none()
    {
        return Err(unsupported(
            "/input",
            "a closed bounded feedback string contract is required",
        ));
    }
    let steps = doc["steps"]
        .as_array()
        .filter(|s| s.len() == 2)
        .ok_or_else(|| unsupported("/steps", "exactly classify then draft is supported"))?;
    let mut compiled = Vec::new();
    for (index, id) in ["classify", "draft"].into_iter().enumerate() {
        let step = &steps[index];
        let deps = if index == 0 {
            json!([])
        } else {
            json!(["classify"])
        };
        if step["id"] != id
            || step["dependsOn"] != deps
            || step["effect"] != "none"
            || step["approval"] != "current-authority"
            || step["retry"] != json!({"maxAttempts":1,"onUnknownEffect":"reconcile-before-retry"})
            || !matches!(
                step["completion"].as_str(),
                Some("artifact" | "all-dependencies-and-artifact")
            )
        {
            return Err(unsupported(
                &format!("/steps/{index}"),
                "unsupported order, dependencies, effects, approval, retry or completion",
            ));
        }
        let mappings: std::collections::BTreeMap<String, String> =
            serde_json::from_value(step["inputMapping"].clone())?;
        let expected = if index == 0 {
            json!({"feedback":"workflow-input:feedback"})
        } else {
            json!({"feedback":"workflow-input:feedback","classification":"step-artifact:classify"})
        };
        if serde_json::to_value(&mappings)? != expected {
            return Err(unsupported(
                &format!("/steps/{index}/inputMapping"),
                "exact feedback and predecessor artifact selectors required",
            ));
        }
        jsonschema::validator_for(&step["output"])
            .map_err(|_| unsupported(&format!("/steps/{index}/output"), "invalid output schema"))?;
        let fields = if index == 0 {
            ["category", "rationale"]
        } else {
            ["title", "body"]
        };
        if step["output"]["type"] != "object"
            || step["output"]["additionalProperties"] != false
            || fields.iter().any(|field| {
                step["output"]["properties"][*field]["type"] != "string"
                    || !step["output"]["required"]
                        .as_array()
                        .is_some_and(|a| a.iter().any(|v| v.as_str() == Some(*field)))
            })
        {
            return Err(unsupported(
                &format!("/steps/{index}/output"),
                "closed declared string fields required",
            ));
        }
        compiled.push(WorkflowCompiledStep {
            id: id.into(),
            role: step["role"].as_str().unwrap_or_default().into(),
            instructions: step["instructions"].as_str().unwrap_or_default().into(),
            input_mapping: mappings,
            output: step["output"].clone(),
        });
    }
    if doc["output"]["type"] != "object"
        || doc["output"]["additionalProperties"] != false
        || doc["output"]["properties"].as_object().map(|p| p.len()) != Some(3)
        || ["decision", "artifactId", "artifactDigest"]
            .iter()
            .any(|field| {
                doc["output"]["properties"][*field]["type"] != "string"
                    || !doc["output"]["required"]
                        .as_array()
                        .is_some_and(|a| a.iter().any(|v| v.as_str() == Some(*field)))
            })
    {
        return Err(unsupported(
            "/output",
            "closed decision and artifact reference contract required",
        ));
    }
    let mut plan = WorkflowCompiledPlan {
        interpretation_version: "1.0.0".into(),
        digest: String::new(),
        input: input.clone(),
        output: doc["output"].clone(),
        steps: compiled,
        max_activations: doc["maxActivations"]
            .as_u64()
            .ok_or_else(|| unsupported("/maxActivations", "required activation ceiling"))?,
    };
    jsonschema::validator_for(&plan.output)
        .map_err(|_| unsupported("/output", "invalid final output schema"))?;
    plan.digest = canonical_digest(&serde_json::to_value(&plan)?)?;
    Ok(plan)
}
