//! Protocol-neutral caller-owned tool declarations. These grant no host execution.
use serde_json::Value;
use std::collections::BTreeSet;

#[derive(Debug, Clone)]
pub struct ClientToolInput {
    /// OpenAI function schema format, normalized by each protocol adapter.
    pub tools: Vec<Value>,
    pub tool_choice: Option<crate::llm::ToolChoice>,
    pub parallel_tool_calls: Option<bool>,
    /// Canonical message format shared by providers, never protocol wire structs.
    pub history: Vec<Value>,
    /// Original caller-visible history retained when a host resumes hidden canonical history.
    pub visible_history: Option<Vec<Value>>,
    /// Host-only graph continuation captured from a trusted model node.
    pub(crate) graph_state: Option<Value>,
    pub(crate) output_token_cap: Option<u32>,
    pub(crate) budget_snapshot: Option<crate::uar::runtime::cost_budget::ClientToolBudgetSnapshot>,
    pub protocol: String,
}

impl ClientToolInput {
    pub fn validate(&self) -> anyhow::Result<()> {
        anyhow::ensure!(!self.protocol.is_empty(), "client_tool_protocol_required");
        let mut names = BTreeSet::new();
        for tool in &self.tools {
            anyhow::ensure!(tool["type"] == "function", "unsupported_client_tool_type");
            let name = tool.pointer("/function/name").and_then(Value::as_str)
                .filter(|name| !name.is_empty() && name.len() <= 64 && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-'))
                .ok_or_else(|| anyhow::anyhow!("invalid_client_tool_name"))?;
            anyhow::ensure!(names.insert(name), "duplicate_client_tool_name");
            let schema = tool.pointer("/function/parameters").filter(|schema| schema.is_object())
                .ok_or_else(|| anyhow::anyhow!("invalid_client_tool_schema"))?;
            crate::uar::tools::validate::compile(schema).map_err(|_| anyhow::anyhow!("invalid_client_tool_schema"))?;
        }
        if let Some(crate::llm::ToolChoice::Named(name)) = &self.tool_choice {
            anyhow::ensure!(names.contains(name.as_str()), "unknown_client_tool_choice");
        }
        if matches!(self.tool_choice, Some(crate::llm::ToolChoice::Required)) {
            anyhow::ensure!(!self.tools.is_empty(), "client_tools_required");
        }
        Ok(())
    }

    pub fn owns(&self, name: &str) -> bool {
        self.tools.iter().any(|tool| tool.pointer("/function/name").and_then(Value::as_str) == Some(name))
    }
}
