use universal_agent_runtime::llm::{
    LlmRequest, Message, MessageContent, MessageRole, ToolCall, ToolCallFunction,
};

pub const TOOL_CALL_ID: &str = "protected-call-17";
pub const PAYLOAD_HEAD: &str = "receipt-head\r\n";
pub const PAYLOAD_MIDDLE: &str = "PROTECTED-MIDDLE-雪🙂é";
pub const PAYLOAD_TAIL: &str = "\r\nreceipt-tail";

pub fn text(role: MessageRole, content: impl Into<String>) -> Message {
    Message {
        role,
        content: MessageContent::text(content),
        tool_call_id: None,
        tool_calls: None,
    }
}

pub fn protected_tool_group() -> Vec<Message> {
    vec![
        Message {
            role: MessageRole::Assistant,
            content: MessageContent::text(""),
            tool_call_id: None,
            tool_calls: Some(vec![ToolCall {
                id: TOOL_CALL_ID.to_string(),
                call_type: "function".to_string(),
                function: ToolCallFunction {
                    name: "fixture_tool".to_string(),
                    arguments: "{\"preserve\":true}".to_string(),
                },
            }]),
        },
        Message {
            role: MessageRole::Tool,
            content: MessageContent::text(protected_payload()),
            tool_call_id: Some(TOOL_CALL_ID.to_string()),
            tool_calls: None,
        },
    ]
}

pub fn protected_payload() -> String {
    format!(
        "{PAYLOAD_HEAD}{}{PAYLOAD_MIDDLE}{}{PAYLOAD_TAIL}",
        "a".repeat(20_000),
        "z".repeat(20_000)
    )
}

pub fn contains_complete_tool_group(messages: &[Message]) -> bool {
    let Some(call_index) = messages.iter().position(|message| {
        message
            .tool_calls
            .iter()
            .flatten()
            .any(|call| call.id == TOOL_CALL_ID)
    }) else {
        return false;
    };
    messages.get(call_index + 1).is_some_and(|message| {
        message.role == MessageRole::Tool
            && message.tool_call_id.as_deref() == Some(TOOL_CALL_ID)
            && message.content.as_text() == Some(protected_payload().as_str())
    })
}

pub fn decode_request_messages(request: &LlmRequest) -> Vec<Message> {
    request
        .messages
        .iter()
        .cloned()
        .map(serde_json::from_value)
        .collect::<Result<_, _>>()
        .expect("captured provider request contains typed messages")
}
