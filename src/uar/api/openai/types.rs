use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug)]
pub struct ModelList {
    pub object: String,
    pub data: Vec<ModelCard>,
}

#[derive(Serialize, Debug)]
pub struct ModelCard {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub owned_by: String,
}

#[derive(Deserialize, Debug)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<OpenAIMessage>,
    #[serde(default)]
    pub stream: bool,
    #[serde(default)]
    pub tools: Vec<OpenAITool>,
    pub tool_choice: Option<serde_json::Value>,
    pub parallel_tool_calls: Option<bool>,
    #[serde(flatten)]
    pub presentation_negotiation: crate::uar::a2ui::presentation_selection::PresentationNegotiation,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct OpenAITool {
    #[serde(rename = "type")]
    pub kind: String,
    pub function: OpenAIFunction,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct OpenAIFunction {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub parameters: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strict: Option<bool>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct OpenAIMessage {
    pub role: String,
    pub content: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tool_calls: Vec<OpenAIToolCall>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct OpenAIToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub function: OpenAIFunctionCall,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct OpenAIFunctionCall {
    pub name: String,
    pub arguments: String,
}

#[derive(Serialize, Debug)]
pub struct ChatCompletionChunk {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<ChatCompletionChunkChoice>,
}

#[derive(Serialize, Debug)]
pub struct ChatCompletionChunkChoice {
    pub index: usize,
    pub delta: ChatCompletionChunkDelta,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<String>,
}

#[derive(Serialize, Debug, Default)]
pub struct ChatCompletionChunkDelta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<OpenAIToolCallDelta>>,
}

#[derive(Serialize, Debug)]
pub struct OpenAIToolCallDelta {
    pub index: usize,
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub function: OpenAIFunctionCall,
}

impl OpenAIMessage {
    /// Convert the text chat wire contract without dropping tool-call history.
    pub fn into_history(self) -> Result<crate::llm::Message, String> {
        use crate::llm::{Message, MessageContent, MessageRole, ToolCall, ToolCallFunction};
        let role = match self.role.as_str() {
            "system" | "developer" => MessageRole::System,
            "user" => MessageRole::User,
            "assistant" => MessageRole::Assistant,
            "tool" => MessageRole::Tool,
            _ => return Err(format!("Unsupported message role '{}'", self.role)),
        };
        if role != MessageRole::Assistant && !self.tool_calls.is_empty() {
            return Err("tool_calls are only valid on assistant messages".to_owned());
        }
        if role == MessageRole::Tool {
            if self.tool_call_id.as_deref().is_none_or(str::is_empty) {
                return Err("tool messages require tool_call_id".to_owned());
            }
        } else if self.tool_call_id.is_some() {
            return Err("tool_call_id is only valid on tool messages".to_owned());
        }
        if self.content.is_none() && (role != MessageRole::Assistant || self.tool_calls.is_empty()) {
            return Err("content is required unless an assistant message contains tool_calls".to_owned());
        }
        let mut calls = Vec::with_capacity(self.tool_calls.len());
        for call in self.tool_calls {
            if call.kind != "function" || call.id.is_empty() || call.function.name.is_empty() {
                return Err("tool_calls require function type, id and function name".to_owned());
            }
            calls.push(ToolCall {
                id: call.id,
                call_type: call.kind,
                function: ToolCallFunction {
                    name: call.function.name,
                    arguments: call.function.arguments,
                },
            });
        }
        Ok(Message {
            role,
            content: MessageContent::text(self.content.unwrap_or_default()),
            tool_call_id: self.tool_call_id,
            tool_calls: (!calls.is_empty()).then_some(calls),
        })
    }
}

impl ChatCompletionRequest {
    pub fn client_tool_input(&self) -> Result<crate::uar::runtime::turn::client_tools::ClientToolInput, String> {
        let tool_choice = match self.tool_choice.as_ref() {
            None => None,
            Some(serde_json::Value::String(choice)) => Some(match choice.as_str() {
                "auto" => crate::llm::ToolChoice::Auto,
                "none" => crate::llm::ToolChoice::None,
                "required" => crate::llm::ToolChoice::Required,
                _ => return Err("Invalid tool_choice".to_owned()),
            }),
            Some(choice) if choice["type"] == "function" => Some(crate::llm::ToolChoice::Named(
                choice.pointer("/function/name").and_then(serde_json::Value::as_str)
                    .filter(|name| !name.is_empty()).ok_or("Named tool_choice requires a function name")?.to_owned()
            )),
            Some(_) => return Err("Invalid tool_choice".to_owned()),
        };
        let tools = self.tools.iter().map(serde_json::to_value).collect::<Result<Vec<_>, _>>()
            .map_err(|error| error.to_string())?;
        let history = self.messages.iter().cloned().map(|wire| {
            let message = wire.into_history()?;
            serde_json::to_value(message).map_err(|error| error.to_string())
        }).collect::<Result<Vec<_>, String>>()?;
        let input = crate::uar::runtime::turn::client_tools::ClientToolInput {
            tools, tool_choice, parallel_tool_calls: self.parallel_tool_calls, history,
            visible_history: None,
            graph_state: None,
            budget_snapshot: None, output_token_cap: None,
            protocol: "openai".to_owned(),
        };
        input.validate().map_err(|error| error.to_string())?;
        Ok(input)
    }
}
