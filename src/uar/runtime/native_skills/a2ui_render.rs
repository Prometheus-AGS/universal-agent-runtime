//! Native tool that lets an agent emit a validated interactive A2UI surface.

use crate::uar::a2ui::protocol::{
    BASIC_CATALOG_ID, BASIC_CATALOG_ID_1_0, CATALOG_ID, VERSION_1_0, parse_messages,
};
use crate::uar::runtime::native_skill::NativeSkill;
use crate::uar::tools::descriptor::{ToolEffect, ToolSource};

#[derive(Debug)]
pub struct A2uiRenderSkill;

#[async_trait::async_trait]
impl NativeSkill for A2uiRenderSkill {
    fn name(&self) -> &str {
        "a2ui_render"
    }

    fn description(&self) -> &str {
        "Render a safe interactive A2UI 1.0 surface in chat. Use the exact version 1.0, an approved catalog, and @path bindings. Initialize the data model and component tree with inline createSurface fields."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        let mut message_branches = modern_message_schemas();
        message_branches.extend(legacy_message_schemas());
        serde_json::json!({
            "type": "object",
            "required": ["messages"],
            "properties": {
                "messages": {
                    "type": "array",
                    "minItems": 1,
                    "description": "Ordered A2UI 1.0 messages. Create a new surface before updating it. Use id root for the root component. Updates can address fewer components. Functions are unavailable.",
                    "items": {"anyOf": message_branches},
                    "examples": [[{
                        "version": VERSION_1_0,
                        "createSurface": {
                            "surfaceId": "demo", "catalogId": CATALOG_ID,
                            "dataModel": {"status": "Ready"},
                            "components": [
                                {"id": "root", "component": "Column", "children": ["status", "continue"]},
                                {"id": "status", "component": "Text", "text": {"@path": "/status"}},
                                {"id": "continue", "component": "Button", "child": "continueLabel",
                                    "action": {"event": {"name": "continueDemo", "context": {"status": {"@path": "/status"}}}}},
                                {"id": "continueLabel", "component": "Text", "text": "Continue"}
                            ]
                        }
                    }]]
                }
            },
            "additionalProperties": false
        })
    }

    fn effect(&self) -> ToolEffect {
        ToolEffect::ReadOnly
    }

    fn source(&self) -> ToolSource {
        ToolSource::BuiltIn
    }

    fn check_thread_policy(
        &self,
        _policy: &crate::uar::runtime::thread::policy_intersection::ThreadPolicy,
    ) -> anyhow::Result<()> {
        // This implementation only validates and returns declarative messages.
        // The governed host publishes them under its actual run ID; arguments
        // cannot select another run or execute the actions they describe.
        Ok(())
    }

    async fn execute(&self, args: serde_json::Value) -> anyhow::Result<serde_json::Value> {
        let messages = args
            .get("messages")
            .and_then(serde_json::Value::as_array)
            .ok_or_else(|| anyhow::anyhow!("messages must be a non-empty array"))?;
        parse_messages(messages.iter().cloned()).map_err(anyhow::Error::msg)?;
        Ok(serde_json::json!({
            "status": "prepared",
            "terminal": true,
            "instruction": "The interactive surface messages are prepared for host publication. Do not call a2ui_render again for this request; describe the content without claiming client display.",
            "a2uiMessages": messages
        }))
    }
}

/// One typed branch per A2UI 1.0 operation: `version` plus exactly one
/// operation. Strict tool-schema dialects (Moonshot's rejects a request whose
/// tool has an untyped property or union branch) cannot express the former
/// `oneOf` of bare `required` lists, so the operation choice lives in the branch.
fn modern_message_schemas() -> Vec<serde_json::Value> {
    let operations = [
        ("createSurface", serde_json::json!({
            "type": "object",
            "required": ["surfaceId", "catalogId"],
            "properties": {
                "surfaceId": {"type": "string"},
                "catalogId": {"type": "string", "enum": [CATALOG_ID, BASIC_CATALOG_ID_1_0]},
                "dataModel": {"type": "object"},
                "components": {"type": "array", "minItems": 1, "items": component_schema()}
            },
            "additionalProperties": false
        })),
        ("updateComponents", serde_json::json!({
            "type": "object",
            "required": ["surfaceId", "components"],
            "properties": {
                "surfaceId": {"type": "string"},
                "components": {"type": "array", "minItems": 1, "items": component_schema()}
            },
            "additionalProperties": false
        })),
        ("updateDataModel", serde_json::json!({
            "type": "object",
            "required": ["surfaceId", "value"],
            "properties": {
                "surfaceId": {"type": "string"},
                "path": {"type": "string", "description": "JSON pointer. Omit or use / for the root."},
                "value": json_value_schema("A JSON value. Use null to delete the addressed value.")
            },
            "additionalProperties": false
        })),
        ("deleteSurface", serde_json::json!({
            "type": "object",
            "required": ["surfaceId"],
            "properties": {"surfaceId": {"type": "string"}},
            "additionalProperties": false
        })),
    ];
    operations
        .into_iter()
        .map(|(operation, schema)| {
            let mut properties = serde_json::Map::new();
            properties.insert(
                "version".to_owned(),
                serde_json::json!({"type": "string", "const": VERSION_1_0}),
            );
            properties.insert(operation.to_owned(), schema);
            serde_json::json!({
                "type": "object",
                "required": ["version", operation],
                "properties": properties,
                "additionalProperties": false
            })
        })
        .collect()
}

/// A string literal or a `{"@path": JSON pointer}` data binding.
fn bindable_text_schema() -> serde_json::Value {
    serde_json::json!({
        "description": "A string or {@path: JSON pointer}.",
        "anyOf": [{"type": "string"}, path_binding_schema()]
    })
}

fn path_binding_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "required": ["@path"],
        "properties": {"@path": {"type": "string"}},
        "additionalProperties": false
    })
}

/// Any JSON value, with every union branch typed. Array elements cover every
/// JSON type except a nested array.
fn json_value_schema(description: &str) -> serde_json::Value {
    serde_json::json!({
        "description": description,
        "anyOf": [
            {"type": "string"},
            {"type": "number"},
            {"type": "boolean"},
            {"type": "object"},
            {"type": "array", "items": {"anyOf": [
                {"type": "string"},
                {"type": "number"},
                {"type": "boolean"},
                {"type": "object"},
                {"type": "null"}
            ]}},
            {"type": "null"}
        ]
    })
}

/// The three v0.9/v0.9.1 message shapes, each a typed branch of the same union
/// as the 1.0 operations.
fn legacy_message_schemas() -> Vec<serde_json::Value> {
    let branches = serde_json::json!([
            {
                "type": "object",
                "required": ["version", "createSurface"],
                "properties": {
                    "version": { "type": "string", "enum": ["v0.9", "v0.9.1"] },
                    "createSurface": {
                        "type": "object",
                        "required": ["surfaceId", "catalogId"],
                        "properties": {
                            "surfaceId": { "type": "string" },
                            "catalogId": { "type": "string", "enum": [CATALOG_ID, BASIC_CATALOG_ID] }
                        },
                        "additionalProperties": false
                    }
                },
                "additionalProperties": false
            },
            {
                "type": "object",
                "required": ["version", "updateComponents"],
                "properties": {
                    "version": { "type": "string", "enum": ["v0.9", "v0.9.1"] },
                    "updateComponents": {
                        "type": "object",
                        "required": ["surfaceId", "components"],
                        "properties": {
                            "surfaceId": { "type": "string" },
                            "components": {
                                "type": "array",
                                "minItems": 1,
                                "description": "Component graph. Exactly one component must have id `root`; it is the rendered surface root.",
                                "contains": {
                                    "type": "object",
                                    "required": ["id"],
                                    "properties": { "id": { "type": "string", "const": "root" } }
                                },
                                "items": {
                                    "type": "object",
                                    "required": ["id", "component"],
                                    "properties": {
                                        "id": { "type": "string" },
                                        "component": { "type": "string", "enum": ["Text", "Button", "TextField", "CheckBox", "ChoicePicker", "Row", "Column", "Card", "Divider"] },
                                        "child": { "type": "string" },
                                        "children": { "type": "array", "items": { "type": "string" } },
                                        "text": json_value_schema("A string or binding."),
                                        "variant": { "type": "string" },
                                        "label": { "type": "string" },
                                        "placeholder": { "type": "string" },
                                        "value": json_value_schema("A literal or binding."),
                                        "options": { "type": "array", "items": { "type": "object" } },
                                        "action": {
                                            "type": "object",
                                            "required": ["event"],
                                            "properties": {
                                                "event": {
                                                    "type": "object",
                                                    "required": ["name"],
                                                    "properties": {
                                                        "name": { "type": "string" },
                                                        "context": { "type": "object" }
                                                    },
                                                    "additionalProperties": false
                                                }
                                            },
                                            "additionalProperties": false
                                        }
                                    },
                                    "additionalProperties": false
                                }
                            }
                        },
                        "additionalProperties": false
                    }
                },
                "additionalProperties": false
            },
            {
                "type": "object",
                "required": ["version", "updateDataModel"],
                "properties": {
                    "version": { "type": "string", "enum": ["v0.9", "v0.9.1"] },
                    "updateDataModel": {
                        "type": "object",
                        "required": ["surfaceId", "path", "value"],
                        "properties": {
                            "surfaceId": { "type": "string" },
                            "path": { "type": "string" },
                            "value": json_value_schema("A JSON value.")
                        },
                        "additionalProperties": false
                    }
                },
                "additionalProperties": false
            }
        ]);
    match branches {
        serde_json::Value::Array(branches) => branches,
        other => vec![other],
    }
}

fn component_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "required": ["id", "component"],
        "properties": {
            "id": {"type": "string"},
            "component": {"type": "string", "enum": ["Text", "Button", "TextField", "CheckBox", "ChoicePicker", "Row", "Column", "Card", "Divider"]},
            "@catalogId": {"type": "string", "enum": [CATALOG_ID, BASIC_CATALOG_ID_1_0]},
            "child": {"type": "string"},
            "children": {"type": "array", "items": {"type": "string"}},
            "text": bindable_text_schema(),
            "variant": {"type": "string"},
            "label": bindable_text_schema(),
            "placeholder": bindable_text_schema(),
            "value": {
                "description": "TextField: a string; CheckBox: a boolean; ChoicePicker: an array of strings; or {@path: JSON pointer}.",
                "anyOf": [
                    {"type": "string"},
                    {"type": "boolean"},
                    {"type": "array", "items": {"type": "string"}},
                    path_binding_schema()
                ]
            },
            "options": {"type": "array", "items": {"type": "object"}},
            "axis": {"type": "string", "enum": ["horizontal", "vertical"]},
            "align": {"type": "string", "enum": ["start", "center", "end", "stretch"]},
            "justify": {"type": "string", "enum": ["start", "center", "end", "spaceBetween", "spaceAround", "spaceEvenly", "stretch"]},
            "action": {
                "type": "object",
                "required": ["event"],
                "properties": {
                    "event": {
                        "type": "object",
                        "required": ["name"],
                        "properties": {
                            "name": {"type": "string"},
                            "context": {"type": "object"},
                            "userMessage": bindable_text_schema()
                        },
                        "additionalProperties": false
                    }
                },
                "additionalProperties": false
            }
        },
        "additionalProperties": false
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn accepts_safe_surface_and_rejects_executable_components() {
        let safe = serde_json::json!({"messages": [{
            "version": "v0.9.1", "profile": "uar.a2ui/1",
            "createSurface": {"surfaceId": "demo", "catalogId": "urn:uar:a2ui:catalog:1"}
        }]});
        let result = A2uiRenderSkill.execute(safe).await.expect("safe surface");
        assert_eq!(result["status"], "prepared");
        assert_eq!(result["terminal"], true);
        assert!(
            result["instruction"]
                .as_str()
                .is_some_and(|value| value.contains("Do not call a2ui_render again"))
        );

        let unsafe_value = serde_json::json!({"messages": [{
            "version": "v0.9.1", "profile": "uar.a2ui/1",
            "updateComponents": {"surfaceId": "demo", "components": [
                {"id": "bad", "component": "Text", "text": "<script>bad()</script>"}
            ]}
        }]});
        assert!(A2uiRenderSkill.execute(unsafe_value).await.is_err());
    }

    /// Moonshot checks tool parameters against a strict JSON Schema subset and
    /// rejects the whole request on the first violation: every property and
    /// union branch declares `type`, `items` is a schema object, and `required`
    /// keys exist in `properties` (MoonshotAI/kimi-cli#1595).
    fn strict_dialect_violations(
        node: &serde_json::Value,
        path: &str,
        needs_type: bool,
        out: &mut Vec<String>,
    ) {
        let Some(object) = node.as_object() else {
            out.push(format!("{path}: schema is not an object"));
            return;
        };
        let union = object.get("anyOf").or_else(|| object.get("oneOf"));
        if let Some(branches) = union.and_then(serde_json::Value::as_array) {
            for (index, branch) in branches.iter().enumerate() {
                let branch_path = format!("{path}.union[{index}]");
                if branch.get("type").is_none() {
                    out.push(format!("{branch_path}: union branch has no type"));
                }
                strict_dialect_violations(branch, &branch_path, false, out);
            }
        } else if needs_type && !object.contains_key("type") {
            out.push(format!("{path}: type is not defined"));
        }
        match object.get("items") {
            Some(serde_json::Value::Array(_)) => out.push(format!("{path}.items: tuple items")),
            Some(items) => strict_dialect_violations(items, &format!("{path}.items"), true, out),
            None => {}
        }
        if let Some(contains) = object.get("contains") {
            strict_dialect_violations(contains, &format!("{path}.contains"), true, out);
        }
        let properties = object.get("properties").and_then(serde_json::Value::as_object);
        if let Some(required) = object.get("required").and_then(serde_json::Value::as_array) {
            for key in required.iter().filter_map(serde_json::Value::as_str) {
                if !properties.is_some_and(|properties| properties.contains_key(key)) {
                    out.push(format!("{path}: required {key} is not a property"));
                }
            }
        }
        for (key, value) in properties.into_iter().flatten() {
            strict_dialect_violations(value, &format!("{path}.properties.{key}"), true, out);
        }
    }

    #[test]
    fn parameters_schema_satisfies_the_strict_moonshot_dialect() {
        let mut violations = Vec::new();
        strict_dialect_violations(&A2uiRenderSkill.parameters_schema(), "root", true, &mut violations);
        assert!(violations.is_empty(), "{violations:#?}");
    }

    #[test]
    fn parameters_schema_keeps_one_operation_per_message() {
        let validator = jsonschema::validator_for(&A2uiRenderSkill.parameters_schema())
            .expect("the a2ui_render schema compiles");
        let surface = serde_json::json!({"surfaceId": "demo", "catalogId": CATALOG_ID});
        let single = serde_json::json!({"messages": [{"version": VERSION_1_0, "createSurface": surface}]});
        let double = serde_json::json!({"messages": [{
            "version": VERSION_1_0, "createSurface": surface, "deleteSurface": {"surfaceId": "demo"}
        }]});
        let none = serde_json::json!({"messages": [{"version": VERSION_1_0}]});
        assert!(validator.is_valid(&single));
        assert!(!validator.is_valid(&double));
        assert!(!validator.is_valid(&none));
    }
}
