//! XML tool definition injection for instruction-tuned models.
//!
//! Models that lack native tool calling support can still use tools by having
//! the tool definitions injected into the system prompt as XML. The model is
//! then instructed to emit `<tool_call>` blocks that the [`super::tool_extractor`]
//! can parse from the streaming text output.

use serde_json::Value;

/// Convert OpenAI-format tool definitions to an XML block suitable for
/// system prompt injection.
///
/// Each tool's `function.name`, `function.description`, and
/// `function.parameters` (JSON Schema) are rendered into a structured XML
/// format, followed by instructions telling the model how to emit tool calls.
#[must_use]
pub fn tools_to_xml(tools: &[Value]) -> String {
    let mut xml = String::from("<tool_definitions>\n");

    for tool in tools {
        if let Some(func) = tool.get("function") {
            let name = func
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or("unknown");
            let desc = func
                .get("description")
                .and_then(Value::as_str)
                .unwrap_or("");
            let params = func
                .get("parameters")
                .cloned()
                .unwrap_or(Value::Object(serde_json::Map::new()));

            xml.push_str(&format!("  <tool name=\"{name}\">\n"));
            xml.push_str(&format!("    <description>{desc}</description>\n"));
            xml.push_str(&format!(
                "    <parameters>{}</parameters>\n",
                serde_json::to_string_pretty(&params).unwrap_or_default()
            ));
            xml.push_str("  </tool>\n");
        }
    }

    xml.push_str("</tool_definitions>\n\n");
    xml.push_str("To call a tool, emit ONLY this exact format:\n");
    xml.push_str(
        "<tool_call>\n{\"name\": \"tool_name\", \"input\": {\"param\": \"value\"}}\n</tool_call>\n\n",
    );
    xml.push_str("After receiving tool results, continue your response.\n");
    xml
}

/// Inject XML tool definitions into the system prompt of an [`super::LlmRequest`].
///
/// If the request has tools, this function:
/// 1. Renders them as XML via [`tools_to_xml`].
/// 2. Prepends the XML block to the existing system message (or creates one).
/// 3. Clears `req.tools` so they are not sent via the native tool API.
pub fn inject_tools_into_system_prompt(req: &mut super::LlmRequest) {
    if req.tools.is_empty() {
        return;
    }

    let xml = tools_to_xml(&req.tools);

    // Look for an existing system message to prepend to.
    let system_index = req.messages.iter().position(|msg| {
        msg.get("role")
            .and_then(Value::as_str)
            .is_some_and(|r| r == "system")
    });

    match system_index {
        Some(idx) => {
            // Prepend the XML block to the existing system message content.
            if let Some(content) = req.messages[idx].get("content").and_then(Value::as_str) {
                let new_content = format!("{xml}\n{content}");
                req.messages[idx]["content"] = Value::String(new_content);
            } else {
                req.messages[idx]["content"] = Value::String(xml);
            }
        }
        None => {
            // Insert a new system message at the beginning.
            let system_msg = serde_json::json!({
                "role": "system",
                "content": xml,
            });
            req.messages.insert(0, system_msg);
        }
    }

    // Tools are now encoded in the prompt; clear the native tool list.
    req.tools.clear();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::LlmRequest;

    fn make_request(messages: Vec<serde_json::Value>, tools: Vec<serde_json::Value>) -> LlmRequest {
        LlmRequest {
            messages,
            tools,
            cache_strategy: None,
            thinking_config: None,
            anthropic_system: None,
            extra_params: None,
        }
    }

    #[test]
    fn test_tools_to_xml_basic() {
        let tools = vec![serde_json::json!({
            "type": "function",
            "function": {
                "name": "get_weather",
                "description": "Get current weather for a location",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "location": { "type": "string" }
                    },
                    "required": ["location"]
                }
            }
        })];

        let xml = tools_to_xml(&tools);
        assert!(xml.contains("<tool_definitions>"));
        assert!(xml.contains("<tool name=\"get_weather\">"));
        assert!(xml.contains("Get current weather"));
        assert!(xml.contains("</tool_definitions>"));
        assert!(xml.contains("<tool_call>"));
    }

    #[test]
    fn test_tools_to_xml_empty() {
        let xml = tools_to_xml(&[]);
        assert!(xml.contains("<tool_definitions>"));
        assert!(xml.contains("</tool_definitions>"));
    }

    #[test]
    fn test_inject_into_existing_system_prompt() {
        let mut req = make_request(
            vec![
                serde_json::json!({"role": "system", "content": "You are a helpful assistant."}),
                serde_json::json!({"role": "user", "content": "Hello"}),
            ],
            vec![serde_json::json!({
                "type": "function",
                "function": {
                    "name": "test_tool",
                    "description": "A test tool",
                    "parameters": {}
                }
            })],
        );

        inject_tools_into_system_prompt(&mut req);

        let content = req.messages[0]["content"].as_str().unwrap();
        assert!(content.contains("<tool_definitions>"));
        assert!(content.contains("You are a helpful assistant."));
        assert!(req.tools.is_empty());
    }

    #[test]
    fn test_inject_creates_system_prompt_if_missing() {
        let mut req = make_request(
            vec![serde_json::json!({"role": "user", "content": "Hello"})],
            vec![serde_json::json!({
                "type": "function",
                "function": {
                    "name": "test_tool",
                    "description": "A test tool",
                    "parameters": {}
                }
            })],
        );

        inject_tools_into_system_prompt(&mut req);

        assert_eq!(req.messages.len(), 2);
        assert_eq!(req.messages[0]["role"], "system");
        assert!(
            req.messages[0]["content"]
                .as_str()
                .unwrap()
                .contains("<tool_definitions>")
        );
        assert!(req.tools.is_empty());
    }

    #[test]
    fn test_inject_noop_when_no_tools() {
        let mut req = make_request(
            vec![serde_json::json!({"role": "user", "content": "Hello"})],
            vec![],
        );

        inject_tools_into_system_prompt(&mut req);

        assert_eq!(req.messages.len(), 1);
    }
}
