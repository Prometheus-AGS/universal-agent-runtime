//! A2UI projection of the immutable effective run policy.

use serde::Serialize;
use serde_json::{Value, json};

use crate::uar::domain::policy::{EffectiveResourceSelection, EffectiveRunPolicy};

use super::protocol::{CATALOG_ID, VERSION_1_0};

fn label<T: Serialize>(value: &T) -> String {
    // Two serde shapes reach here, and only one is a bare string.
    //
    // `SelectionMode`, `PolicyScope`, `ChatMode` and `ToolApprovalPolicy` are
    // externally tagged, so they serialize to `"sliding_window"`. But
    // `ContextStrategy` carries `#[serde(tag = "type")]`, so EVERY one of its
    // variants -- unit variants included -- serializes to an object
    // `{"type": "sliding_window", ...}`. Reading only `as_str()` therefore
    // rendered "Context · unknown" for all six strategies, on a surface whose
    // entire job is telling an operator what is in force.
    //
    // Falling back to the `type` tag covers the internally-tagged case without
    // touching the string case. "unknown" now means genuinely unlabelable.
    serde_json::to_value(value)
        .ok()
        .and_then(|value| match value {
            serde_json::Value::String(text) => Some(text),
            serde_json::Value::Object(map) => map
                .get("type")
                .and_then(serde_json::Value::as_str)
                .map(str::to_owned),
            _ => None,
        })
        .unwrap_or_else(|| "unknown".to_string())
        .replace('_', " ")
}

fn resource_summary(name: &str, selection: &EffectiveResourceSelection) -> String {
    format!(
        "{name} · {} · {} available · {} scope",
        label(&selection.mode),
        selection.ids.len(),
        label(&selection.source),
    )
}

/// Build a current-production A2UI 1.0 surface for the run policy.
///
/// The projection intentionally summarizes large capability sets. Their full
/// identifiers remain in the immutable run record; the chat surface presents
/// the operator-relevant mode and count instead of reproducing a JSON dump.
pub(crate) fn effective_policy_surface(run_id: &str, policy: &EffectiveRunPolicy) -> Value {
    let surface_id = format!("run-policy-{run_id}");
    let model = policy
        .model
        .as_ref()
        .map(|route| format!("{} / {}", route.provider_id, route.model_id))
        .unwrap_or_else(|| "Runtime default".to_string());
    let agent = policy
        .agent_id
        .as_deref()
        .unwrap_or("Runtime default")
        .to_string();
    let warning_summary = if policy.warnings.is_empty() {
        "No policy warnings.".to_string()
    } else {
        policy.warnings.join(" • ")
    };

    json!([
        {
            "version": VERSION_1_0,
            "createSurface": {
                "surfaceId": surface_id,
                "catalogId": CATALOG_ID,
                "components": [
                    { "id": "title", "component": "Text", "text": { "@path": "/title" }, "variant": "body" },
                    { "id": "subtitle", "component": "Text", "text": { "@path": "/subtitle" }, "variant": "caption" },
                    { "id": "overview_heading", "component": "Text", "text": "Execution", "variant": "body" },
                    { "id": "chat_mode", "component": "Text", "text": { "@path": "/chatMode" }, "variant": "body" },
                    { "id": "model", "component": "Text", "text": { "@path": "/model" }, "variant": "body" },
                    { "id": "agent", "component": "Text", "text": { "@path": "/agent" }, "variant": "body" },
                    { "id": "overview_column", "component": "Column", "children": ["overview_heading", "chat_mode", "model", "agent"] },
                    { "id": "overview_card", "component": "Card", "child": "overview_column" },
                    { "id": "capabilities_heading", "component": "Text", "text": "Available capabilities", "variant": "body" },
                    { "id": "skills", "component": "Text", "text": { "@path": "/skills" }, "variant": "body" },
                    { "id": "tools", "component": "Text", "text": { "@path": "/tools" }, "variant": "body" },
                    { "id": "mcp_servers", "component": "Text", "text": { "@path": "/mcpServers" }, "variant": "body" },
                    { "id": "knowledge_bases", "component": "Text", "text": { "@path": "/knowledgeBases" }, "variant": "body" },
                    { "id": "capabilities_column", "component": "Column", "children": ["capabilities_heading", "skills", "tools", "mcp_servers", "knowledge_bases"] },
                    { "id": "capabilities_card", "component": "Card", "child": "capabilities_column" },
                    { "id": "runtime_heading", "component": "Text", "text": "Runtime controls", "variant": "body" },
                    { "id": "memory", "component": "Text", "text": { "@path": "/memory" }, "variant": "body" },
                    { "id": "prompt_caching", "component": "Text", "text": { "@path": "/promptCaching" }, "variant": "body" },
                    { "id": "context", "component": "Text", "text": { "@path": "/context" }, "variant": "body" },
                    { "id": "approval", "component": "Text", "text": { "@path": "/approval" }, "variant": "body" },
                    { "id": "runtime_column", "component": "Column", "children": ["runtime_heading", "memory", "prompt_caching", "context", "approval"] },
                    { "id": "runtime_card", "component": "Card", "child": "runtime_column" },
                    { "id": "warnings_heading", "component": "Text", "text": { "@path": "/warningsHeading" }, "variant": "body" },
                    { "id": "warnings", "component": "Text", "text": { "@path": "/warnings" }, "variant": "body" },
                    { "id": "warnings_column", "component": "Column", "children": ["warnings_heading", "warnings"] },
                    { "id": "warnings_card", "component": "Card", "child": "warnings_column" },
                    { "id": "root", "component": "Column", "children": ["title", "subtitle", "overview_card", "capabilities_card", "runtime_card", "warnings_card"] }
                ],
                "dataModel": {
                    "title": "Effective run policy",
                    "subtitle": format!("Resolved policy v{} · A2UI {}", policy.version, VERSION_1_0),
                    "chatMode": format!("Chat mode · {}", label(&policy.chat_mode)),
                    "model": format!("Model · {model}"),
                    "agent": format!("Agent · {agent}"),
                    "skills": resource_summary("Skills", &policy.skills),
                    "tools": resource_summary("Tools", &policy.tools),
                    "mcpServers": resource_summary("MCP servers", &policy.mcp_servers),
                    "knowledgeBases": resource_summary("Knowledge bases", &policy.knowledge_bases),
                    "memory": format!("Memory · {}", if policy.memory_enabled { "on" } else { "off" }),
                    "promptCaching": format!("Prompt caching · {}", if policy.prompt_caching_enabled { "on" } else { "off" }),
                    "context": format!("Context · {}", label(&policy.context_strategy)),
                    "approval": format!("Tool approval · {}", label(&policy.tool_approval)),
                    "warningsHeading": format!("Warnings · {}", policy.warnings.len()),
                    "warnings": warning_summary
                }
            }
        }
    ])
}

#[cfg(test)]
mod tests {
    use crate::uar::domain::policy::{PolicyResolutionInput, resolve_run_policy};

    use super::effective_policy_surface;

    #[test]
    fn emits_current_production_policy_surface_without_capability_dump() {
        let mut input = PolicyResolutionInput::default();
        input
            .universe
            .tools
            .extend(["web_fetch".to_string(), "native_echo".to_string()]);
        let policy = resolve_run_policy(input);

        let surface = effective_policy_surface("run-1", &policy);
        let messages = surface.as_array().expect("surface messages");

        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0]["version"], "1.0");
        assert_eq!(
            messages[0]["createSurface"]["catalogId"],
            "urn:uar:a2ui:catalog:1"
        );
        assert!(messages[0]["createSurface"]["components"].is_array());
        assert_eq!(
            messages[0]["createSurface"]["dataModel"]["tools"],
            "Tools · auto · 2 available · legacy scope"
        );
        assert!(!surface.to_string().contains("web_fetch"));
    }
}

#[cfg(test)]
mod label_tests {
    use super::label;
    use crate::uar::context::ContextStrategy;
    use crate::uar::domain::policy::{ChatMode, PolicyScope, SelectionMode, ToolApprovalPolicy};

    /// `ContextStrategy` is internally tagged, so every variant -- including
    /// the unit ones -- serializes to an object. All six read "unknown" before
    /// this fix, not just the ones carrying fields.
    #[test]
    fn every_context_strategy_has_a_label() {
        let cases = [
            (ContextStrategy::None, "none"),
            (ContextStrategy::Auto, "auto"),
            (
                ContextStrategy::SlidingWindow { max_messages: 50 },
                "sliding window",
            ),
            (
                ContextStrategy::Summarize {
                    threshold: 100,
                    summary_max_tokens: 512,
                    model: None,
                },
                "summarize",
            ),
            (
                ContextStrategy::TruncateMiddle {
                    keep_first: 5,
                    keep_last: 20,
                },
                "truncate middle",
            ),
            (
                ContextStrategy::Hierarchical {
                    short_term_turns: 10,
                    mid_term_summary_tokens: 512,
                    long_term_facts_tokens: 256,
                },
                "hierarchical",
            ),
        ];
        for (strategy, expected) in cases {
            let rendered = label(&strategy);
            assert_eq!(rendered, expected, "{strategy:?} mislabelled");
            assert_ne!(rendered, "unknown", "{strategy:?} still unknown");
        }
    }

    /// The externally-tagged enums already worked. This asserts the fix did
    /// not regress them, which is the risk of touching a shared helper.
    #[test]
    fn externally_tagged_enums_are_unchanged() {
        assert_eq!(label(&SelectionMode::Selected), "selected");
        assert_eq!(label(&SelectionMode::None), "none");
        assert_eq!(label(&PolicyScope::Agent), "agent");
        assert_eq!(label(&ChatMode::Uar), "uar");
        assert_eq!(label(&ToolApprovalPolicy::Inherit), "inherit");
    }

    /// "unknown" must still mean something: a value with no label at all.
    #[test]
    fn genuinely_unlabelable_values_are_still_unknown() {
        assert_eq!(label(&42_u32), "unknown");
        assert_eq!(label(&serde_json::json!({"no_type_key": 1})), "unknown");
    }
}
