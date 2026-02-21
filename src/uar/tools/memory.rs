//! NativeTools for memory operations (legacy shim).
//!
//! These tools (`memory_save` / `memory_recall`) are kept for backward
//! compatibility with agents that use them directly. They now delegate to
//! `AppState::memory_service` when available, and fall back to the persistence
//! layer's no-op stubs otherwise.
//!
//! New agent implementations should prefer the MCP tools exposed by
//! `UarMemoryMcpServer` in `uar::memory::mcp_server`, which provide full
//! multi-scope, hybrid-search, and knowledge-graph capabilities.

use crate::mcp::registry::NativeTool;
use async_trait::async_trait;
use serde_json::json;
use std::sync::Arc;

use crate::uar::memory::service::MemoryService;
use surreal_memory::Memory;

#[derive(Debug)]
pub struct MemorySaveTool {
    memory_service: Option<Arc<MemoryService>>,
}

impl MemorySaveTool {
    pub fn new(memory_service: Option<Arc<MemoryService>>) -> Self {
        Self { memory_service }
    }
}

#[async_trait]
impl NativeTool for MemorySaveTool {
    fn name(&self) -> &'static str {
        "memory_save"
    }

    fn description(&self) -> &'static str {
        "Save a piece of information to long-term memory. Use to remember facts, user preferences, or important context."
    }

    fn schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "content": {
                    "type": "string",
                    "description": "The information content to memorize."
                },
                "categories": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Optional categories for classification."
                },
                "agent_id": {
                    "type": "string",
                    "description": "Optional ID of the agent owning this memory. Omit for global memory."
                },
                "user_id": {
                    "type": "string",
                    "description": "Optional user ID for user-scoped memory."
                }
            },
            "required": ["content"]
        })
    }

    async fn call(&self, args: serde_json::Value) -> anyhow::Result<serde_json::Value> {
        let content = args["content"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing content"))?;
        let categories: Vec<String> = args["categories"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default();
        let agent_id = args["agent_id"].as_str().map(str::to_string);
        let user_id = args["user_id"].as_str().map(str::to_string);

        if let Some(svc) = &self.memory_service {
            let memory = Memory::new(content.to_string(), user_id, agent_id, None, categories);
            let stored = svc
                .storage()
                .add_memory(memory)
                .await
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            let id = stored
                .id
                .as_ref()
                .map(|r| {
                    serde_json::to_value(r)
                        .ok()
                        .and_then(|v| v.as_str().map(str::to_string))
                        .unwrap_or_default()
                })
                .unwrap_or_default();
            Ok(json!({ "status": "success", "memory_id": id }))
        } else {
            tracing::warn!("memory_save called but MemoryService not enabled");
            Ok(json!({ "status": "disabled", "message": "Memory system not enabled" }))
        }
    }
}

#[derive(Debug)]
pub struct MemoryRecallTool {
    memory_service: Option<Arc<MemoryService>>,
}

impl MemoryRecallTool {
    pub fn new(memory_service: Option<Arc<MemoryService>>) -> Self {
        Self { memory_service }
    }
}

#[async_trait]
impl NativeTool for MemoryRecallTool {
    fn name(&self) -> &'static str {
        "memory_recall"
    }

    fn description(&self) -> &'static str {
        "Search long-term memory for relevant information."
    }

    fn schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Semantic search query."
                },
                "agent_id": {
                    "type": "string",
                    "description": "Optional. Filter to this agent's memories."
                },
                "user_id": {
                    "type": "string",
                    "description": "Optional. Filter to this user's memories."
                },
                "limit": {
                    "type": "integer",
                    "description": "Max results (default 5)."
                }
            },
            "required": ["query"]
        })
    }

    async fn call(&self, args: serde_json::Value) -> anyhow::Result<serde_json::Value> {
        let query = args["query"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing query"))?;
        let agent_id = args["agent_id"].as_str();
        let user_id = args["user_id"].as_str();
        let limit = args["limit"].as_u64().unwrap_or(5) as usize;

        let Some(svc) = &self.memory_service else {
            return Ok(json!([]));
        };

        let results = svc
            .storage()
            .search_memories(query, user_id, agent_id, None, None, limit)
            .await
            .map_err(|e| anyhow::anyhow!("{e}"))?;

        let json_results: Vec<serde_json::Value> = results
            .iter()
            .map(|m| {
                json!({
                    "content": m.content,
                    "score": 1.0, // embedding score not returned in basic search
                    "categories": m.categories,
                    "scope": format!("{:?}", m.scope),
                    "type": if m.agent_id.is_some() { "agent" } else { "global" }
                })
            })
            .collect();

        Ok(json!(json_results))
    }
}
