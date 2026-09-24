use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use super::HostInputError;
use crate::llm::{Message, MessageContent, MessageRole, ToolCall};

const MAX_HOST_HISTORY_MESSAGES: usize = 1_000;
const MAX_HOST_HISTORY_BYTES: usize = 4 * 1024 * 1024;

#[derive(Debug, Clone, Deserialize)]
pub struct HostHistoryInput {
    pub session_id: String,
    pub messages: Vec<HostHistoryMessage>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HostHistoryMessage {
    pub role: String,
    pub content: String,
    #[serde(default)]
    pub tool_call_id: Option<String>,
    #[serde(default)]
    pub tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HistorySeedStatus {
    None,
    Seeded,
    IgnoredWarmSession,
}

impl HostHistoryInput {
    pub fn validate(
        self,
        request_session_id: Option<&str>,
    ) -> Result<Vec<Message>, HostInputError> {
        if request_session_id != Some(self.session_id.as_str()) {
            return Err(HostInputError::new(
                "history_session_mismatch",
                "history must name the request session",
            ));
        }
        if self.messages.len() > MAX_HOST_HISTORY_MESSAGES {
            return Err(HostInputError::new(
                "history_too_large",
                "history exceeds the message limit",
            ));
        }
        let mut bytes = 0usize;
        let mut calls = HashSet::new();
        let mut results = HashSet::new();
        let mut messages = Vec::with_capacity(self.messages.len());
        for input in self.messages {
            bytes = bytes.saturating_add(input.content.len());
            let role = match input.role.as_str() {
                "user" => MessageRole::User,
                "assistant" => MessageRole::Assistant,
                "tool" => MessageRole::Tool,
                _ => {
                    return Err(HostInputError::new(
                        "history_invalid",
                        "history contains an unsupported role",
                    ));
                }
            };
            if input.content.is_empty() && input.tool_calls.as_ref().map_or(true, Vec::is_empty) {
                return Err(HostInputError::new(
                    "history_invalid",
                    "history contains an empty message",
                ));
            }
            if role == MessageRole::Assistant {
                for call in input.tool_calls.as_deref().unwrap_or_default() {
                    bytes = bytes
                        .saturating_add(call.id.len())
                        .saturating_add(call.function.name.len())
                        .saturating_add(call.function.arguments.len());
                    if call.id.is_empty()
                        || call.function.name.is_empty()
                        || serde_json::from_str::<serde_json::Value>(&call.function.arguments)
                            .is_err()
                        || !calls.insert(call.id.clone())
                    {
                        return Err(HostInputError::new(
                            "history_invalid",
                            "history contains an invalid or duplicate tool call",
                        ));
                    }
                }
            } else if input.tool_calls.is_some() {
                return Err(HostInputError::new(
                    "history_invalid",
                    "only assistant history may contain tool calls",
                ));
            }
            if role == MessageRole::Tool {
                let call_id = input
                    .tool_call_id
                    .as_deref()
                    .filter(|id| calls.contains(*id) && results.insert((*id).to_owned()));
                if call_id.is_none() {
                    return Err(HostInputError::new(
                        "history_invalid",
                        "history contains an orphaned or duplicate tool result",
                    ));
                }
                bytes = bytes.saturating_add(call_id.map_or(0, str::len));
            } else if input.tool_call_id.is_some() {
                return Err(HostInputError::new(
                    "history_invalid",
                    "only tool history may carry tool_call_id",
                ));
            }
            if bytes > MAX_HOST_HISTORY_BYTES {
                return Err(HostInputError::new(
                    "history_too_large",
                    "history exceeds the byte limit",
                ));
            }
            messages.push(Message {
                role,
                content: MessageContent::text(input.content),
                tool_call_id: input.tool_call_id,
                tool_calls: input.tool_calls,
            });
        }
        Ok(messages)
    }
}
