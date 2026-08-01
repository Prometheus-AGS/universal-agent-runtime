//! Native tool that lets an agent emit a validated interactive A2UI surface.

use crate::uar::a2ui::protocol::parse_message;
use crate::uar::runtime::native_skill::NativeSkill;

#[derive(Debug)]
pub struct A2uiRenderSkill;

#[async_trait::async_trait]
impl NativeSkill for A2uiRenderSkill {
    fn name(&self) -> &str {
        "a2ui_render"
    }

    fn description(&self) -> &str {
        "Render a safe interactive A2UI v0.9 surface in chat. Send canonical messages with top-level createSurface, updateComponents, or updateDataModel objects. Every message needs version; never use generic event/data or type/data envelopes."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "required": ["messages"],
            "properties": {
                "messages": {
                    "type": "array",
                    "minItems": 1,
                    "description": "Ordered canonical A2UI messages. Start with createSurface, then updateComponents, and optionally updateDataModel.",
                    "items": {
                        "oneOf": [
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
                                            "catalogId": { "type": "string", "const": "https://a2ui.org/specification/v0_9/catalogs/basic/catalog.json" }
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
                                                    "properties": { "id": { "const": "root" } }
                                                },
                                                "items": {
                                                    "type": "object",
                                                    "required": ["id", "component"],
                                                    "properties": {
                                                        "id": { "type": "string" },
                                                        "component": { "type": "string", "enum": ["Text", "Button", "TextField", "CheckBox", "ChoicePicker", "Row", "Column", "Card", "Divider"] },
                                                        "child": { "type": "string" },
                                                        "children": { "type": "array", "items": { "type": "string" } },
                                                        "text": {},
                                                        "variant": { "type": "string" },
                                                        "label": { "type": "string" },
                                                        "placeholder": { "type": "string" },
                                                        "value": {},
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
                                            "value": {}
                                        },
                                        "additionalProperties": false
                                    }
                                },
                                "additionalProperties": false
                            }
                        ]
                    },
                    "examples": [[
                        { "version": "v0.9", "createSurface": { "surfaceId": "demo", "catalogId": "https://a2ui.org/specification/v0_9/catalogs/basic/catalog.json" } },
                        { "version": "v0.9", "updateComponents": { "surfaceId": "demo", "components": [
                            { "id": "root", "component": "Card", "child": "body" },
                            { "id": "body", "component": "Column", "children": ["message", "continue"] },
                            { "id": "message", "component": "Text", "text": "Ready" },
                            { "id": "continue", "component": "Button", "child": "continueLabel", "action": { "event": { "name": "continueDemo", "context": {} } } },
                            { "id": "continueLabel", "component": "Text", "text": "Continue" }
                        ] } }
                    ]]
                }
            },
            "additionalProperties": false
        })
    }

    async fn execute(&self, args: serde_json::Value) -> anyhow::Result<serde_json::Value> {
        let messages = args
            .get("messages")
            .and_then(serde_json::Value::as_array)
            .ok_or_else(|| anyhow::anyhow!("messages must be a non-empty array"))?;
        if messages.is_empty() {
            anyhow::bail!("messages must be a non-empty array");
        }
        for message in messages {
            parse_message(message.clone()).map_err(anyhow::Error::msg)?;
        }
        Ok(serde_json::json!({
            "status": "rendered",
            "terminal": true,
            "instruction": "The interactive surface is rendered. Do not call a2ui_render again for this request; briefly confirm completion to the user.",
            "a2uiMessages": messages
        }))
    }
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
        assert_eq!(result["status"], "rendered");
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
}
