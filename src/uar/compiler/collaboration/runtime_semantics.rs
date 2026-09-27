//! Effective runtime semantics for collaboration deployment bindings.
//!
//! A retained definition field is not executable evidence. This module emits
//! `runtime.enforced` only when the selected configured model or the ordinary
//! turn policy can consume the authored value.

use std::collections::BTreeSet;

use serde_json::{Map, Value};

use crate::llm::{ProviderRegistry, prompt_dialect::PromptDialect};
use crate::uar::context::ContextStrategy;
use crate::uar::domain::collaboration::{
    CollaborationDefinitionRecord, ConversionDisposition, FieldDiagnostic,
};

pub(super) async fn resolve_runtime_semantics(
    definition: &CollaborationDefinitionRecord,
    resolved_models: &[Value],
    provider_registry: Option<&ProviderRegistry>,
    diagnostics: &mut Vec<FieldDiagnostic>,
) -> Value {
    let mut effective = Map::new();
    resolve_model_requirements(
        definition,
        resolved_models,
        provider_registry,
        &mut effective,
        diagnostics,
    )
    .await;
    resolve_prompt_dialect(
        definition,
        resolved_models,
        provider_registry,
        &mut effective,
        diagnostics,
    )
    .await;
    resolve_context_strategy(definition, &mut effective, diagnostics);
    unsupported_field(definition, "ragConfiguration", diagnostics);
    unsupported_field(definition, "apiHarness", diagnostics);
    Value::Object(effective)
}

async fn resolve_model_requirements(
    definition: &CollaborationDefinitionRecord,
    resolved_models: &[Value],
    provider_registry: Option<&ProviderRegistry>,
    effective: &mut Map<String, Value>,
    diagnostics: &mut Vec<FieldDiagnostic>,
) {
    let Some(requirement) = definition.document.get("modelRequirements") else {
        return;
    };
    let value = requirement.get("value").cloned().unwrap_or(Value::Null);
    let result = match provider_registry {
        Some(registry) if value.is_object() && !resolved_models.is_empty() => {
            model_requirements_satisfied(&value, resolved_models, registry).await
        }
        _ => Err("The selected configured provider/model cannot be inspected.".to_owned()),
    };
    match result {
        Ok(()) => supported(
            effective,
            diagnostics,
            "modelRequirements",
            value,
            "runtime.model-capabilities-enforced",
            "Every selected configured model satisfies the authored capability requirements.",
        ),
        Err(message) => unsupported(requirement, "modelRequirements", message, diagnostics),
    }
}

async fn model_requirements_satisfied(
    value: &Value,
    resolved_models: &[Value],
    registry: &ProviderRegistry,
) -> Result<(), String> {
    let object = value
        .as_object()
        .ok_or_else(|| "Model requirements must be an object.".to_owned())?;
    let allowed = BTreeSet::from([
        "capabilities",
        "needs_tools",
        "needs_reasoning",
        "needs_vision",
        "needs_structured_output",
        "minContext",
        "min_context",
        "preferredProvider",
        "preferred_provider",
    ]);
    if let Some(field) = object
        .keys()
        .find(|field| !allowed.contains(field.as_str()))
    {
        return Err(format!(
            "Model requirement field '{field}' has no enforcing configured-model check."
        ));
    }
    let mut capabilities = object
        .get("capabilities")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_owned)
        .collect::<BTreeSet<_>>();
    for (field, capability) in [
        ("needs_tools", "tool-calling"),
        ("needs_reasoning", "reasoning"),
        ("needs_vision", "vision"),
        ("needs_structured_output", "structured-output"),
    ] {
        if object.get(field).and_then(Value::as_bool).unwrap_or(false) {
            capabilities.insert(capability.to_owned());
        }
    }
    let min_context = object
        .get("minContext")
        .or_else(|| object.get("min_context"))
        .and_then(Value::as_u64);
    let preferred_provider = object
        .get("preferredProvider")
        .or_else(|| object.get("preferred_provider"))
        .and_then(Value::as_str);

    for resolved in resolved_models {
        let provider_id = string_field(resolved, "providerId")?;
        let model_id = string_field(resolved, "modelId")?;
        if preferred_provider.is_some_and(|preferred| preferred != provider_id) {
            return Err(format!(
                "Selected provider '{provider_id}' does not match the required provider."
            ));
        }
        let provider = registry
            .get(provider_id)
            .await
            .filter(|provider| provider.enabled)
            .ok_or_else(|| format!("Selected provider '{provider_id}' is not configured."))?;
        let model = provider
            .models
            .iter()
            .find(|model| model.id == model_id && model.enabled)
            .ok_or_else(|| {
                format!("Selected model '{provider_id}/{model_id}' is not configured and enabled.")
            })?;
        for capability in &capabilities {
            let supported = match capability.as_str() {
                "text" => true,
                "tool-calling" => model.supports_tools,
                "reasoning" => model.supports_reasoning,
                "vision" => model.supports_vision,
                "structured-output" => model.supports_structured_output,
                "streaming" => model.supports_streaming,
                _ => false,
            };
            if !supported {
                return Err(format!(
                    "Selected model '{provider_id}/{model_id}' does not satisfy capability '{capability}'."
                ));
            }
        }
        if min_context.is_some_and(|minimum| {
            model
                .context_window
                .is_none_or(|window| u64::from(window) < minimum)
        }) {
            return Err(format!(
                "Selected model '{provider_id}/{model_id}' does not satisfy the minimum context window."
            ));
        }
    }
    Ok(())
}

async fn resolve_prompt_dialect(
    definition: &CollaborationDefinitionRecord,
    resolved_models: &[Value],
    provider_registry: Option<&ProviderRegistry>,
    effective: &mut Map<String, Value>,
    diagnostics: &mut Vec<FieldDiagnostic>,
) {
    let Some(requirement) = definition.document.get("promptDialect") else {
        return;
    };
    let value = requirement.get("value").cloned().unwrap_or(Value::Null);
    let dialect = match &value {
        Value::String(value) if value == "default" => None,
        Value::Object(object) if object.is_empty() => None,
        Value::Object(object)
            if object
                .keys()
                .all(|field| matches!(field.as_str(), "dialect" | "wants_reasoning" | "hard"))
                && !object
                    .get("wants_reasoning")
                    .and_then(Value::as_bool)
                    .unwrap_or(false)
                && !object.get("hard").and_then(Value::as_bool).unwrap_or(false) =>
        {
            object.get("dialect").and_then(Value::as_str)
        }
        _ => {
            unsupported(
                requirement,
                "promptDialect",
                "The authored prompt dialect has no ordinary-run enforcement mapping.".to_owned(),
                diagnostics,
            );
            return;
        }
    };
    let Some(provider_registry) = provider_registry else {
        unsupported(
            requirement,
            "promptDialect",
            "The configured model registry is unavailable for prompt dialect enforcement."
                .to_owned(),
            diagnostics,
        );
        return;
    };
    if resolved_models.is_empty() {
        unsupported(
            requirement,
            "promptDialect",
            "No selected model is available for prompt dialect enforcement.".to_owned(),
            diagnostics,
        );
        return;
    }
    for model in resolved_models {
        let Ok(provider_id) = string_field(model, "providerId") else {
            unsupported(
                requirement,
                "promptDialect",
                "The selected model has no provider identity for prompt dialect enforcement."
                    .to_owned(),
                diagnostics,
            );
            return;
        };
        let Ok(model_id) = string_field(model, "modelId") else {
            unsupported(
                requirement,
                "promptDialect",
                "The selected model has no model identity for prompt dialect enforcement."
                    .to_owned(),
                diagnostics,
            );
            return;
        };
        let Some(provider) = provider_registry
            .get(provider_id)
            .await
            .filter(|provider| provider.enabled)
        else {
            unsupported(
                requirement,
                "promptDialect",
                "The selected provider is not configured for prompt dialect enforcement."
                    .to_owned(),
                diagnostics,
            );
            return;
        };
        let driver_model = if provider.base_url.is_empty() {
            format!("{provider_id}/{model_id}")
        } else {
            model_id.to_owned()
        };
        if dialect.is_some_and(|expected| PromptDialect::detect(&driver_model).name() != expected) {
            unsupported(
                requirement,
                "promptDialect",
                "The selected model does not enforce the authored prompt dialect.".to_owned(),
                diagnostics,
            );
            return;
        }
    }
    let effective_value = dialect.map_or_else(
        || serde_json::json!({"mode": "auto"}),
        |dialect| serde_json::json!({"dialect": dialect}),
    );
    supported(
        effective,
        diagnostics,
        "promptDialect",
        effective_value,
        "runtime.prompt-dialect-enforced",
        "The bound model uses the authored dialect in ordinary prompt and request assembly.",
    );
}

fn resolve_context_strategy(
    definition: &CollaborationDefinitionRecord,
    effective: &mut Map<String, Value>,
    diagnostics: &mut Vec<FieldDiagnostic>,
) {
    let Some(requirement) = definition.document.get("contextStrategy") else {
        return;
    };
    let value = requirement.get("value").cloned().unwrap_or(Value::Null);
    let strategy = if value == serde_json::json!({"mode": "selected"}) {
        Some(ContextStrategy::Auto)
    } else {
        serde_json::from_value::<ContextStrategy>(value).ok()
    };
    let Some(strategy) = strategy else {
        unsupported(
            requirement,
            "contextStrategy",
            "The authored context strategy has no ordinary-turn policy mapping.".to_owned(),
            diagnostics,
        );
        return;
    };
    supported(
        effective,
        diagnostics,
        "contextStrategy",
        serde_json::to_value(strategy).expect("ContextStrategy is serializable"),
        "runtime.context-policy-enforced",
        "The authored context strategy is installed as the ordinary turn policy.",
    );
}

fn unsupported_field(
    definition: &CollaborationDefinitionRecord,
    field: &str,
    diagnostics: &mut Vec<FieldDiagnostic>,
) {
    if let Some(requirement) = definition.document.get(field) {
        unsupported(
            requirement,
            field,
            "The field is preserved without an enforcing runtime component.".to_owned(),
            diagnostics,
        );
    }
}

fn supported(
    effective: &mut Map<String, Value>,
    diagnostics: &mut Vec<FieldDiagnostic>,
    field: &str,
    value: Value,
    reason_code: &str,
    message: &str,
) {
    effective.insert(field.to_owned(), value);
    diagnostics.push(FieldDiagnostic {
        pointer: format!("/{field}"),
        disposition: ConversionDisposition::Exact,
        reason_code: reason_code.to_owned(),
        message: message.to_owned(),
        effective_binding_ref: None,
    });
}

fn unsupported(
    requirement: &Value,
    field: &str,
    message: String,
    diagnostics: &mut Vec<FieldDiagnostic>,
) {
    let required = requirement
        .get("required")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    diagnostics.push(FieldDiagnostic {
        pointer: format!("/{field}"),
        disposition: if required {
            ConversionDisposition::RequiredUnsupported
        } else {
            ConversionDisposition::OptionalUnsupported
        },
        reason_code: "runtime.component-unavailable".to_owned(),
        message,
        effective_binding_ref: None,
    });
}

fn string_field<'a>(value: &'a Value, field: &str) -> Result<&'a str, String> {
    value
        .get(field)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("Resolved model is missing '{field}'."))
}
