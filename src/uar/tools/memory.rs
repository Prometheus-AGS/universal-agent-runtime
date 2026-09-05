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
use crate::uar::tools::descriptor::ToolEffect;
use async_trait::async_trait;
use serde_json::json;
use std::sync::Arc;

use crate::uar::memory::service::MemoryService;
use surreal_memory::Memory;

fn check_memory_policy(
    policy: &crate::uar::runtime::thread::policy_intersection::ThreadPolicy,
) -> anyhow::Result<()> {
    anyhow::ensure!(
        policy.effective().memory_enabled,
        "Memory is disabled for this delegated turn"
    );
    Ok(())
}

/// Model owner fields are filters, never authority. Preserve the legacy direct
/// host API, but every verified run is confined to its captured user's records.
async fn scoped_arguments(
    mut args: serde_json::Value,
    context: &crate::uar::runtime::native_skill::NativeExecutionContext,
    service: Option<&MemoryService>,
    by_id: bool,
) -> anyhow::Result<serde_json::Value> {
    if let Some(policy) = &context.thread_policy {
        check_memory_policy(policy)?;
    }
    let Some(owner) = &context.verified_owner else {
        anyhow::ensure!(
            context.thread_policy.is_none(),
            "Memory requires a verified delegated owner"
        );
        return Ok(args);
    };
    let object = args
        .as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("Memory arguments must be an object"))?;
    if let Some(requested) = object.get("user_id") {
        anyhow::ensure!(
            requested.as_str() == Some(owner.user_id()),
            "Memory owner cannot be replaced by tool arguments"
        );
    }
    if by_id {
        let id = object
            .get("memory_id")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("Missing memory_id"))?;
        anyhow::ensure!(
            !id.trim().is_empty()
                && id
                    .split_once(':')
                    .is_none_or(|(table, _)| table == "memory"),
            "Memory ID must address a memory record"
        );
        if let Some(service) = service {
            let memory = service
                .get(id)
                .await?
                .ok_or_else(|| anyhow::anyhow!("Memory is unavailable to this owner"))?;
            anyhow::ensure!(
                memory.user_id.as_deref() == Some(owner.user_id()),
                "Memory is unavailable to this owner"
            );
        }
    } else {
        object.insert("user_id".into(), json!(owner.user_id()));
    }
    Ok(args)
}

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
    fn check_thread_policy(
        &self,
        policy: &crate::uar::runtime::thread::policy_intersection::ThreadPolicy,
    ) -> anyhow::Result<()> {
        check_memory_policy(policy)
    }

    async fn call_with_context(
        &self,
        args: serde_json::Value,
        context: &crate::uar::runtime::native_skill::NativeExecutionContext,
    ) -> anyhow::Result<serde_json::Value> {
        self.call(scoped_arguments(args, context, self.memory_service.as_deref(), false).await?)
            .await
    }

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

    fn effect(&self) -> ToolEffect {
        ToolEffect::ExternalMutation
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
            let mut memory = Memory::new(content.to_string(), user_id, agent_id, None, categories);
            if memory.user_id.is_some() {
                memory.scope = surreal_memory::MemoryScope::User;
            }
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

// ── Admin tools (list, delete, update, history) ────────────────────────────

/// List all memories visible to the current session/user.
#[derive(Debug)]
pub struct MemoryListTool {
    memory_service: Option<Arc<MemoryService>>,
}

impl MemoryListTool {
    pub fn new(memory_service: Option<Arc<MemoryService>>) -> Self {
        Self { memory_service }
    }
}

#[async_trait]
impl NativeTool for MemoryListTool {
    fn check_thread_policy(
        &self,
        policy: &crate::uar::runtime::thread::policy_intersection::ThreadPolicy,
    ) -> anyhow::Result<()> {
        check_memory_policy(policy)
    }

    async fn call_with_context(
        &self,
        args: serde_json::Value,
        context: &crate::uar::runtime::native_skill::NativeExecutionContext,
    ) -> anyhow::Result<serde_json::Value> {
        self.call(scoped_arguments(args, context, self.memory_service.as_deref(), false).await?)
            .await
    }

    fn name(&self) -> &'static str {
        "memory_list"
    }

    fn description(&self) -> &'static str {
        "List stored memories. Optionally filter by user_id, agent_id, or session_id. Returns up to `limit` results."
    }

    fn schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "user_id": {
                    "type": "string",
                    "description": "Optional user ID to filter memories by owner."
                },
                "agent_id": {
                    "type": "string",
                    "description": "Optional agent ID to filter agent-scoped memories."
                },
                "session_id": {
                    "type": "string",
                    "description": "Optional session ID to filter session-scoped memories."
                },
                "limit": {
                    "type": "integer",
                    "description": "Maximum number of memories to return (default 20)."
                }
            },
            "required": []
        })
    }

    fn effect(&self) -> ToolEffect {
        ToolEffect::ReadOnly
    }

    async fn call(&self, args: serde_json::Value) -> anyhow::Result<serde_json::Value> {
        let Some(svc) = &self.memory_service else {
            return Ok(json!({ "memories": [], "message": "Memory system not enabled" }));
        };

        let user_id = args["user_id"].as_str();
        let agent_id = args["agent_id"].as_str();
        let session_id = args["session_id"].as_str();
        let limit = args["limit"].as_u64().unwrap_or(20) as usize;

        let results = svc
            .storage()
            .get_all_memories(user_id, agent_id, session_id)
            .await
            .map_err(|e| anyhow::anyhow!("{e}"))?;

        let truncated: Vec<serde_json::Value> = results
            .into_iter()
            .take(limit)
            .map(|m| {
                let id =
                    m.id.as_ref()
                        .map(|r| {
                            serde_json::to_value(r)
                                .ok()
                                .and_then(|v| v.as_str().map(str::to_string))
                                .unwrap_or_default()
                        })
                        .unwrap_or_default();
                json!({
                    "memory_id": id,
                    "content": m.content,
                    "scope": format!("{:?}", m.scope).to_lowercase(),
                    "memory_type": format!("{:?}", m.memory_type).to_lowercase(),
                    "importance": m.importance,
                    "categories": m.categories
                })
            })
            .collect();

        let count = truncated.len();
        Ok(json!({ "memories": truncated, "count": count }))
    }
}

/// Delete a specific memory by its record ID.
#[derive(Debug)]
pub struct MemoryDeleteTool {
    memory_service: Option<Arc<MemoryService>>,
}

impl MemoryDeleteTool {
    pub fn new(memory_service: Option<Arc<MemoryService>>) -> Self {
        Self { memory_service }
    }
}

#[async_trait]
impl NativeTool for MemoryDeleteTool {
    fn check_thread_policy(
        &self,
        policy: &crate::uar::runtime::thread::policy_intersection::ThreadPolicy,
    ) -> anyhow::Result<()> {
        check_memory_policy(policy)
    }

    async fn call_with_context(
        &self,
        args: serde_json::Value,
        context: &crate::uar::runtime::native_skill::NativeExecutionContext,
    ) -> anyhow::Result<serde_json::Value> {
        self.call(scoped_arguments(args, context, self.memory_service.as_deref(), true).await?)
            .await
    }

    fn name(&self) -> &'static str {
        "memory_delete_by_id"
    }

    fn description(&self) -> &'static str {
        "Permanently delete a memory by its ID. Use memory_list to find the memory_id first."
    }

    fn schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "memory_id": {
                    "type": "string",
                    "description": "The ID of the memory to delete (e.g. 'memory:abc123')."
                }
            },
            "required": ["memory_id"]
        })
    }

    fn effect(&self) -> ToolEffect {
        ToolEffect::ExternalMutation
    }

    async fn call(&self, args: serde_json::Value) -> anyhow::Result<serde_json::Value> {
        let Some(svc) = &self.memory_service else {
            return Ok(json!({ "status": "disabled", "message": "Memory system not enabled" }));
        };

        let memory_id = args["memory_id"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing memory_id"))?;

        svc.storage()
            .delete_memory(memory_id)
            .await
            .map_err(|e| anyhow::anyhow!("{e}"))?;

        Ok(json!({ "status": "deleted", "memory_id": memory_id }))
    }
}

/// Update the content of an existing memory.
#[derive(Debug)]
pub struct MemoryUpdateTool {
    memory_service: Option<Arc<MemoryService>>,
}

impl MemoryUpdateTool {
    pub fn new(memory_service: Option<Arc<MemoryService>>) -> Self {
        Self { memory_service }
    }
}

#[async_trait]
impl NativeTool for MemoryUpdateTool {
    fn check_thread_policy(
        &self,
        policy: &crate::uar::runtime::thread::policy_intersection::ThreadPolicy,
    ) -> anyhow::Result<()> {
        check_memory_policy(policy)
    }

    async fn call_with_context(
        &self,
        args: serde_json::Value,
        context: &crate::uar::runtime::native_skill::NativeExecutionContext,
    ) -> anyhow::Result<serde_json::Value> {
        self.call(scoped_arguments(args, context, self.memory_service.as_deref(), true).await?)
            .await
    }

    fn name(&self) -> &'static str {
        "memory_update_by_id"
    }

    fn description(&self) -> &'static str {
        "Update the content of an existing memory by ID. Creates a history record of the previous content."
    }

    fn schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "memory_id": {
                    "type": "string",
                    "description": "The ID of the memory to update."
                },
                "content": {
                    "type": "string",
                    "description": "The new content to store in the memory."
                }
            },
            "required": ["memory_id", "content"]
        })
    }

    fn effect(&self) -> ToolEffect {
        ToolEffect::ExternalMutation
    }

    async fn call(&self, args: serde_json::Value) -> anyhow::Result<serde_json::Value> {
        let Some(svc) = &self.memory_service else {
            return Ok(json!({ "status": "disabled", "message": "Memory system not enabled" }));
        };

        let memory_id = args["memory_id"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing memory_id"))?;
        let content = args["content"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing content"))?;

        let updated = svc
            .storage()
            .update_memory(memory_id, content.to_string())
            .await
            .map_err(|e| anyhow::anyhow!("{e}"))?;

        let new_id = updated
            .id
            .as_ref()
            .map(|r| {
                serde_json::to_value(r)
                    .ok()
                    .and_then(|v| v.as_str().map(str::to_string))
                    .unwrap_or_default()
            })
            .unwrap_or_default();

        Ok(json!({
            "status": "updated",
            "memory_id": new_id,
            "content": updated.content
        }))
    }
}

/// Retrieve the edit history for a memory.
#[derive(Debug)]
pub struct MemoryHistoryTool {
    memory_service: Option<Arc<MemoryService>>,
}

impl MemoryHistoryTool {
    pub fn new(memory_service: Option<Arc<MemoryService>>) -> Self {
        Self { memory_service }
    }
}

#[async_trait]
impl NativeTool for MemoryHistoryTool {
    fn check_thread_policy(
        &self,
        policy: &crate::uar::runtime::thread::policy_intersection::ThreadPolicy,
    ) -> anyhow::Result<()> {
        check_memory_policy(policy)
    }

    async fn call_with_context(
        &self,
        args: serde_json::Value,
        context: &crate::uar::runtime::native_skill::NativeExecutionContext,
    ) -> anyhow::Result<serde_json::Value> {
        self.call(scoped_arguments(args, context, self.memory_service.as_deref(), true).await?)
            .await
    }

    fn name(&self) -> &'static str {
        "memory_history"
    }

    fn description(&self) -> &'static str {
        "Retrieve the full edit history for a memory, including previous content values and timestamps."
    }

    fn schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "memory_id": {
                    "type": "string",
                    "description": "The ID of the memory to inspect."
                }
            },
            "required": ["memory_id"]
        })
    }

    fn effect(&self) -> ToolEffect {
        ToolEffect::ReadOnly
    }

    async fn call(&self, args: serde_json::Value) -> anyhow::Result<serde_json::Value> {
        let Some(svc) = &self.memory_service else {
            return Ok(json!({ "history": [], "message": "Memory system not enabled" }));
        };

        let memory_id = args["memory_id"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing memory_id"))?;

        let history = svc
            .storage()
            .get_memory_history(memory_id)
            .await
            .map_err(|e| anyhow::anyhow!("{e}"))?;

        let entries: Vec<serde_json::Value> = history
            .iter()
            .map(|h| {
                json!({
                    "version": h.version,
                    "change_type": h.change_type,
                    "old_content": h.old_content,
                    "new_content": h.new_content,
                    "changed_at": h.changed_at.to_string(),
                })
            })
            .collect();

        let count = entries.len();
        Ok(json!({ "memory_id": memory_id, "history": entries, "count": count }))
    }
}

// ── Legacy recall tool ──────────────────────────────────────────────────────

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
    fn check_thread_policy(
        &self,
        policy: &crate::uar::runtime::thread::policy_intersection::ThreadPolicy,
    ) -> anyhow::Result<()> {
        check_memory_policy(policy)
    }

    async fn call_with_context(
        &self,
        args: serde_json::Value,
        context: &crate::uar::runtime::native_skill::NativeExecutionContext,
    ) -> anyhow::Result<serde_json::Value> {
        self.call(scoped_arguments(args, context, self.memory_service.as_deref(), false).await?)
            .await
    }

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

    fn effect(&self) -> ToolEffect {
        ToolEffect::ReadOnly
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
