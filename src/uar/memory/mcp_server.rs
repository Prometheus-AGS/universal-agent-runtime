//! In-process memory MCP server for the UAR agent runtime.
//!
//! Runs the full `surreal-memory` tool suite in-process, with no external server process.
//! Exposes:
//! 1. An Axum router using rmcp's `StreamableHttpService` for HTTP MCP clients.
//! 2. All tools available as UAR-native calls through `MemoryService`.
//!
//! ## HTTP exposure
//!
//! Mount the router returned by `memory_mcp_router()` at the configured path:
//!
//! ```rust,ignore
//! let router = memory_mcp_router(Arc::clone(&memory_service));
//! app = app.nest(&config.memory.mcp_http_path, router);
//! ```

use std::sync::Arc;

use anyhow::Result;
use axum::Router;
use rmcp::{
    ErrorData as McpError, ServerHandler,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{CallToolResult, ContentBlock as Content, Implementation, ServerCapabilities, ServerInfo},
    tool, tool_handler, tool_router,
    transport::streamable_http_server::{
        StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
    },
};
use surreal_memory::{
    Memory, MemoryScope, MemoryType,
    entity::{Entity, Relation},
    storage::MemoryStorage,
    task_stream::TaskStream,
};

use super::service::MemoryService;

// ── Helper types ──────────────────────────────────────────────────────────────

fn ok_json<T: serde::Serialize>(value: &T) -> CallToolResult {
    let json =
        serde_json::to_string_pretty(value).unwrap_or_else(|e| format!("{{\"error\":\"{e}}}\")"));
    CallToolResult::success(vec![Content::text(json)])
}

fn err_mcp(e: impl std::fmt::Display) -> McpError {
    McpError::invalid_params(e.to_string(), None)
}

// ── Parameter structs ─────────────────────────────────────────────────────────

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct AddMemoryParams {
    pub content: String,
    #[serde(default)]
    pub scope: Option<String>,
    #[serde(default)]
    pub memory_type: Option<String>,
    #[serde(default)]
    pub user_id: Option<String>,
    #[serde(default)]
    pub agent_id: Option<String>,
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub categories: Vec<String>,
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
    #[serde(default = "AddMemoryParams::default_importance")]
    pub importance: f32,
}
impl AddMemoryParams {
    fn default_importance() -> f32 {
        0.5
    }
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct MemoryIdParams {
    pub id: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct UpdateMemoryParams {
    pub id: String,
    pub content: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ScopeFilterParams {
    #[serde(default)]
    pub user_id: Option<String>,
    #[serde(default)]
    pub agent_id: Option<String>,
    #[serde(default)]
    pub session_id: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SearchMemoriesParams {
    pub query: String,
    #[serde(default)]
    pub user_id: Option<String>,
    #[serde(default)]
    pub agent_id: Option<String>,
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub categories: Option<Vec<String>>,
    #[serde(default = "SearchMemoriesParams::default_limit")]
    pub limit: usize,
}
impl SearchMemoriesParams {
    fn default_limit() -> usize {
        10
    }
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct HybridSearchParams {
    pub query: String,
    #[serde(default)]
    pub user_id: Option<String>,
    #[serde(default)]
    pub agent_id: Option<String>,
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default = "HybridSearchParams::default_limit")]
    pub limit: usize,
    #[serde(default = "HybridSearchParams::default_vw")]
    pub vector_weight: f32,
    #[serde(default = "HybridSearchParams::default_bw")]
    pub bm25_weight: f32,
}
impl HybridSearchParams {
    fn default_limit() -> usize {
        10
    }
    fn default_vw() -> f32 {
        0.7
    }
    fn default_bw() -> f32 {
        0.3
    }
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ConversationParams {
    pub messages: Vec<serde_json::Value>,
    #[serde(default)]
    pub user_id: Option<String>,
    #[serde(default)]
    pub agent_id: Option<String>,
    #[serde(default)]
    pub session_id: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CompressMemoriesParams {
    #[serde(default)]
    pub user_id: Option<String>,
    #[serde(default)]
    pub agent_id: Option<String>,
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default = "CompressMemoriesParams::default_days")]
    pub older_than_days: u32,
}
impl CompressMemoriesParams {
    fn default_days() -> u32 {
        30
    }
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CreateEntityParams {
    pub name: String,
    pub entity_type: String,
    pub observations: Vec<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SearchParams {
    pub query: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SemanticSearchParams {
    pub query: String,
    #[serde(default = "SemanticSearchParams::default_limit")]
    pub limit: usize,
    #[serde(default = "SemanticSearchParams::default_threshold")]
    pub threshold: f32,
}
impl SemanticSearchParams {
    fn default_limit() -> usize {
        10
    }
    fn default_threshold() -> f32 {
        0.7
    }
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct DeleteEntityParams {
    pub name: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct AddObservationsParams {
    pub entity_name: String,
    pub observations: Vec<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CreateRelationParams {
    pub from: String,
    pub to: String,
    pub relation_type: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct DeleteRelationParams {
    pub from: String,
    pub to: String,
    pub relation_type: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct FindPathParams {
    pub from: String,
    pub to: String,
    #[serde(default = "FindPathParams::default_depth")]
    pub max_depth: u8,
}
impl FindPathParams {
    fn default_depth() -> u8 {
        4
    }
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ExpandNeighborsParams {
    pub entity_name: String,
    #[serde(default = "ExpandNeighborsParams::default_depth")]
    pub depth: u8,
    #[serde(default = "ExpandNeighborsParams::default_limit")]
    pub limit: usize,
}
impl ExpandNeighborsParams {
    fn default_depth() -> u8 {
        2
    }
    fn default_limit() -> usize {
        20
    }
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GetRelatedParams {
    pub entity_name: String,
    #[serde(default)]
    pub relation_type: Option<String>,
    #[serde(default = "GetRelatedParams::default_direction")]
    pub direction: String,
    #[serde(default = "GetRelatedParams::default_limit")]
    pub limit: usize,
}
impl GetRelatedParams {
    fn default_direction() -> String {
        "both".to_string()
    }
    fn default_limit() -> usize {
        20
    }
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct TaskStreamNameParams {
    pub name: String,
    #[serde(default)]
    pub user_id: Option<String>,
    #[serde(default)]
    pub agent_id: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CreateTaskStreamParams {
    pub name: String,
    #[serde(default)]
    pub agent_id: Option<String>,
    #[serde(default)]
    pub user_id: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct AddToTaskStreamParams {
    pub stream_name: String,
    pub content: String,
    #[serde(default)]
    pub categories: Vec<String>,
    #[serde(default)]
    pub user_id: Option<String>,
    #[serde(default)]
    pub agent_id: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GetContextParams {
    pub stream_name: String,
    pub model_name: String,
    #[serde(default)]
    pub max_tokens: Option<u64>,
    #[serde(default)]
    pub user_id: Option<String>,
    #[serde(default)]
    pub agent_id: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct AutoSummarizeParams {
    pub stream_name: String,
    #[serde(default)]
    pub user_id: Option<String>,
    #[serde(default)]
    pub agent_id: Option<String>,
    pub model_id: String,
}

// ── Server ────────────────────────────────────────────────────────────────────

/// In-process memory MCP server inheriting identical tools to the standalone
/// surreal-memory-server, but run inside the UAR process.
#[derive(Clone)]
pub struct UarMemoryMcpServer {
    storage: Arc<dyn MemoryStorage>,
    #[expect(
        dead_code,
        reason = "rmcp's generated tool handler retains this router for runtime dispatch"
    )]
    tool_router: ToolRouter<UarMemoryMcpServer>,
}

impl std::fmt::Debug for UarMemoryMcpServer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UarMemoryMcpServer").finish_non_exhaustive()
    }
}

impl UarMemoryMcpServer {
    pub fn new(storage: Arc<dyn MemoryStorage>) -> Self {
        Self {
            storage,
            tool_router: Self::tool_router(),
        }
    }
}

#[tool_router]
impl UarMemoryMcpServer {
    // ── Scoped Memory ─────────────────────────────────────────────────────────

    #[tool(description = "Add a new memory. Supports Global/Agent/User/Session/Task scoping.")]
    async fn memory_add(
        &self,
        Parameters(p): Parameters<AddMemoryParams>,
    ) -> Result<CallToolResult, McpError> {
        let scope = match p.scope.as_deref().unwrap_or("user") {
            "global" => MemoryScope::Global,
            "agent" => MemoryScope::Agent,
            "session" => MemoryScope::Session,
            "task" => MemoryScope::Task,
            _ => MemoryScope::User,
        };
        let memory_type = match p.memory_type.as_deref().unwrap_or("semantic") {
            "episodic" => MemoryType::Episodic,
            "procedural" => MemoryType::Procedural,
            "associative" => MemoryType::Associative,
            _ => MemoryType::Semantic,
        };
        let mut mem = Memory::new(p.content, p.user_id, p.agent_id, p.session_id, p.categories);
        mem.scope = scope;
        mem.memory_type = memory_type;
        mem.metadata = p.metadata;
        mem.importance = p.importance;
        let result = self.storage.add_memory(mem).await.map_err(err_mcp)?;
        Ok(ok_json(&result))
    }

    #[tool(description = "Retrieve a specific memory by its record ID.")]
    async fn memory_get(
        &self,
        Parameters(p): Parameters<MemoryIdParams>,
    ) -> Result<CallToolResult, McpError> {
        let result = self.storage.get_memory(&p.id).await.map_err(err_mcp)?;
        Ok(ok_json(&result))
    }

    #[tool(description = "Update the content of an existing memory (creates a history record).")]
    async fn memory_update(
        &self,
        Parameters(p): Parameters<UpdateMemoryParams>,
    ) -> Result<CallToolResult, McpError> {
        let result = self
            .storage
            .update_memory(&p.id, p.content)
            .await
            .map_err(err_mcp)?;
        Ok(ok_json(&result))
    }

    #[tool(description = "Delete a specific memory by ID.")]
    async fn memory_delete(
        &self,
        Parameters(p): Parameters<MemoryIdParams>,
    ) -> Result<CallToolResult, McpError> {
        self.storage.delete_memory(&p.id).await.map_err(err_mcp)?;
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Memory '{}' deleted",
            p.id
        ))]))
    }

    #[tool(description = "Delete all memories matching user/agent/session scope.")]
    async fn memory_delete_all(
        &self,
        Parameters(p): Parameters<ScopeFilterParams>,
    ) -> Result<CallToolResult, McpError> {
        let count = self
            .storage
            .delete_all_memories(
                p.user_id.as_deref(),
                p.agent_id.as_deref(),
                p.session_id.as_deref(),
            )
            .await
            .map_err(err_mcp)?;
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Deleted {} memories",
            count
        ))]))
    }

    #[tool(description = "List all memories for a given scope filter.")]
    async fn memory_list(
        &self,
        Parameters(p): Parameters<ScopeFilterParams>,
    ) -> Result<CallToolResult, McpError> {
        let result = self
            .storage
            .get_all_memories(
                p.user_id.as_deref(),
                p.agent_id.as_deref(),
                p.session_id.as_deref(),
            )
            .await
            .map_err(err_mcp)?;
        Ok(ok_json(&result))
    }

    #[tool(
        description = "Semantic search over scoped memories. Filters by user/agent/session and optional categories."
    )]
    async fn memory_search(
        &self,
        Parameters(p): Parameters<SearchMemoriesParams>,
    ) -> Result<CallToolResult, McpError> {
        let result = self
            .storage
            .search_memories(
                &p.query,
                p.user_id.as_deref(),
                p.agent_id.as_deref(),
                p.session_id.as_deref(),
                p.categories.as_deref(),
                p.limit,
            )
            .await
            .map_err(err_mcp)?;
        Ok(ok_json(&result))
    }

    #[tool(
        description = "Hybrid BM25 full-text + HNSW vector search with configurable weight ratio."
    )]
    async fn memory_hybrid_search(
        &self,
        Parameters(p): Parameters<HybridSearchParams>,
    ) -> Result<CallToolResult, McpError> {
        let result = self
            .storage
            .hybrid_search_memories(
                &p.query,
                p.user_id.as_deref(),
                p.agent_id.as_deref(),
                p.session_id.as_deref(),
                p.limit,
                p.vector_weight,
                p.bm25_weight,
            )
            .await
            .map_err(err_mcp)?;
        Ok(ok_json(&result))
    }

    #[tool(description = "Get the full edit history of a memory (all versions).")]
    async fn memory_history(
        &self,
        Parameters(p): Parameters<MemoryIdParams>,
    ) -> Result<CallToolResult, McpError> {
        let result = self
            .storage
            .get_memory_history(&p.id)
            .await
            .map_err(err_mcp)?;
        Ok(ok_json(&result))
    }

    #[tool(description = "Compress memories older than N days into a single summary memory.")]
    async fn memory_compress(
        &self,
        Parameters(p): Parameters<CompressMemoriesParams>,
    ) -> Result<CallToolResult, McpError> {
        let result = self
            .storage
            .compress_memories(
                p.user_id.as_deref(),
                p.agent_id.as_deref(),
                p.session_id.as_deref(),
                p.older_than_days,
            )
            .await
            .map_err(err_mcp)?;
        Ok(ok_json(&result))
    }

    #[tool(description = "Auto-extract and store memories from raw conversation messages.")]
    async fn memory_extract_from_conversation(
        &self,
        Parameters(p): Parameters<ConversationParams>,
    ) -> Result<CallToolResult, McpError> {
        let result = self
            .storage
            .add_memories_from_conversation(
                p.messages,
                p.user_id.as_deref(),
                p.agent_id.as_deref(),
                p.session_id.as_deref(),
            )
            .await
            .map_err(err_mcp)?;
        Ok(ok_json(&result))
    }

    // ── Knowledge Graph ───────────────────────────────────────────────────────

    #[tool(description = "Retrieve the complete knowledge graph with all entities and relations.")]
    async fn kg_read(&self) -> Result<CallToolResult, McpError> {
        let result = self.storage.get_graph().await.map_err(err_mcp)?;
        Ok(ok_json(&result))
    }

    #[tool(description = "Search entities by text match on name or type.")]
    async fn kg_search(
        &self,
        Parameters(p): Parameters<SearchParams>,
    ) -> Result<CallToolResult, McpError> {
        let result = self
            .storage
            .search_entities(&p.query)
            .await
            .map_err(err_mcp)?;
        Ok(ok_json(&result))
    }

    #[tool(description = "Find entities by vector similarity. Best for natural language queries.")]
    async fn kg_semantic_search(
        &self,
        Parameters(p): Parameters<SemanticSearchParams>,
    ) -> Result<CallToolResult, McpError> {
        let result = self
            .storage
            .semantic_search(&p.query, p.limit, p.threshold)
            .await
            .map_err(err_mcp)?;
        Ok(ok_json(&result))
    }

    #[tool(description = "Create a new entity in the knowledge graph.")]
    async fn kg_create_entity(
        &self,
        Parameters(p): Parameters<CreateEntityParams>,
    ) -> Result<CallToolResult, McpError> {
        let entity = Entity::new(p.name, p.entity_type, p.observations);
        let result = self.storage.create_entity(entity).await.map_err(err_mcp)?;
        Ok(ok_json(&result))
    }

    #[tool(description = "Add observations to an existing entity.")]
    async fn kg_add_observations(
        &self,
        Parameters(p): Parameters<AddObservationsParams>,
    ) -> Result<CallToolResult, McpError> {
        let result = self
            .storage
            .add_observations(&p.entity_name, p.observations)
            .await
            .map_err(err_mcp)?;
        Ok(ok_json(&result))
    }

    #[tool(description = "Create a directed relationship between two entities.")]
    async fn kg_create_relation(
        &self,
        Parameters(p): Parameters<CreateRelationParams>,
    ) -> Result<CallToolResult, McpError> {
        let relation = Relation::new(p.from, p.to, p.relation_type);
        let result = self
            .storage
            .create_relation(relation)
            .await
            .map_err(err_mcp)?;
        Ok(ok_json(&result))
    }

    #[tool(description = "Delete an entity and all its relations from the knowledge graph.")]
    async fn kg_delete_entity(
        &self,
        Parameters(p): Parameters<DeleteEntityParams>,
    ) -> Result<CallToolResult, McpError> {
        let relations = self.storage.get_relations(&p.name).await.map_err(err_mcp)?;
        for r in &relations {
            self.storage
                .delete_relation(&r.from, &r.to, &r.relation_type)
                .await
                .map_err(err_mcp)?;
        }
        self.storage.delete_entity(&p.name).await.map_err(err_mcp)?;
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Deleted '{}' and {} relation(s)",
            p.name,
            relations.len()
        ))]))
    }

    #[tool(description = "Delete a specific relation between two entities.")]
    async fn kg_delete_relation(
        &self,
        Parameters(p): Parameters<DeleteRelationParams>,
    ) -> Result<CallToolResult, McpError> {
        self.storage
            .delete_relation(&p.from, &p.to, &p.relation_type)
            .await
            .map_err(err_mcp)?;
        Ok(CallToolResult::success(vec![Content::text(
            "Relation deleted".to_string(),
        )]))
    }

    #[tool(description = "Return the subgraph of entities within N hops of a given entity.")]
    async fn kg_expand_neighbors(
        &self,
        Parameters(p): Parameters<ExpandNeighborsParams>,
    ) -> Result<CallToolResult, McpError> {
        let result = self
            .storage
            .expand_neighbors(&p.entity_name, p.depth, p.limit)
            .await
            .map_err(err_mcp)?;
        Ok(ok_json(&result))
    }

    #[tool(description = "Find shortest paths between two entities in the knowledge graph.")]
    async fn kg_find_path(
        &self,
        Parameters(p): Parameters<FindPathParams>,
    ) -> Result<CallToolResult, McpError> {
        let result = self
            .storage
            .find_path(&p.from, &p.to, p.max_depth)
            .await
            .map_err(err_mcp)?;
        Ok(ok_json(&result))
    }

    #[tool(
        description = "Return entities related to a given entity, optionally filtered by relation type and direction."
    )]
    async fn kg_get_related(
        &self,
        Parameters(p): Parameters<GetRelatedParams>,
    ) -> Result<CallToolResult, McpError> {
        let result = self
            .storage
            .get_related(
                &p.entity_name,
                p.relation_type.as_deref(),
                &p.direction,
                p.limit,
            )
            .await
            .map_err(err_mcp)?;
        Ok(ok_json(&result))
    }

    // ── TaskStreams ───────────────────────────────────────────────────────────

    #[tool(
        description = "Create a named TaskStream for tracking memories across a long-running task."
    )]
    async fn task_stream_create(
        &self,
        Parameters(p): Parameters<CreateTaskStreamParams>,
    ) -> Result<CallToolResult, McpError> {
        let stream = TaskStream::new(p.name, p.description, p.agent_id, p.user_id);
        let result = self
            .storage
            .create_task_stream(stream)
            .await
            .map_err(err_mcp)?;
        Ok(ok_json(&result))
    }

    #[tool(description = "Get the metadata of a TaskStream by name.")]
    async fn task_stream_get(
        &self,
        Parameters(p): Parameters<TaskStreamNameParams>,
    ) -> Result<CallToolResult, McpError> {
        let result = self
            .storage
            .get_task_stream(&p.name, p.user_id.as_deref(), p.agent_id.as_deref())
            .await
            .map_err(err_mcp)?;
        Ok(ok_json(&result))
    }

    #[tool(description = "Add a memory to a TaskStream.")]
    async fn task_stream_add(
        &self,
        Parameters(p): Parameters<AddToTaskStreamParams>,
    ) -> Result<CallToolResult, McpError> {
        let mem = Memory::new(p.content, None, None, None, p.categories);
        let result = self
            .storage
            .add_to_task_stream(
                &p.stream_name,
                p.user_id.as_deref(),
                p.agent_id.as_deref(),
                mem,
            )
            .await
            .map_err(err_mcp)?;
        Ok(ok_json(&result))
    }

    #[tool(
        description = "Get a model-aware context window for a TaskStream, respecting the model's token budget."
    )]
    async fn task_stream_context(
        &self,
        Parameters(p): Parameters<GetContextParams>,
    ) -> Result<CallToolResult, McpError> {
        let result = self
            .storage
            .get_context_for_task(
                &p.stream_name,
                p.user_id.as_deref(),
                p.agent_id.as_deref(),
                &p.model_name,
                p.max_tokens,
            )
            .await
            .map_err(err_mcp)?;
        Ok(ok_json(&result))
    }

    #[tool(description = "List all active TaskStreams, optionally filtered by agent/user.")]
    async fn task_stream_list(
        &self,
        Parameters(p): Parameters<ScopeFilterParams>,
    ) -> Result<CallToolResult, McpError> {
        let result = self
            .storage
            .list_task_streams(p.agent_id.as_deref(), p.user_id.as_deref())
            .await
            .map_err(err_mcp)?;
        Ok(ok_json(&result))
    }

    #[tool(description = "Archive a TaskStream by name.")]
    async fn task_stream_archive(
        &self,
        Parameters(p): Parameters<TaskStreamNameParams>,
    ) -> Result<CallToolResult, McpError> {
        let result = self
            .storage
            .archive_task_stream(&p.name, p.user_id.as_deref(), p.agent_id.as_deref())
            .await
            .map_err(err_mcp)?;
        Ok(ok_json(&result))
    }

    #[tool(
        description = "Trigger rolling auto-summarization on a TaskStream when total_tokens exceeds the model budget."
    )]
    async fn task_stream_auto_summarize(
        &self,
        Parameters(p): Parameters<AutoSummarizeParams>,
    ) -> Result<CallToolResult, McpError> {
        let result = self
            .storage
            .auto_summarize_task_stream(
                &p.stream_name,
                p.user_id.as_deref(),
                p.agent_id.as_deref(),
                &p.model_id,
            )
            .await
            .map_err(err_mcp)?;
        Ok(ok_json(&result))
    }
}

#[tool_handler]
impl ServerHandler for UarMemoryMcpServer {
    fn get_info(&self) -> ServerInfo {
        // rmcp 1.8: ServerInfo (InitializeResult) and Implementation are both
        // #[non_exhaustive] -- struct-literal syntax (even with
        // ..Default::default()) is rejected cross-crate; use the provided
        // constructors + field mutation instead (all fields are `pub`).
        let mut info = ServerInfo::new(ServerCapabilities::builder().enable_tools().build());
        info.server_info = Implementation::new("uar-memory-mcp", env!("CARGO_PKG_VERSION"));
        info.instructions = Some(
            "UAR in-process memory MCP server. \
            Scoped memory (mem0-compatible): memory_add, memory_get, memory_update, memory_delete, \
            memory_delete_all, memory_list, memory_search, memory_hybrid_search, memory_history, \
            memory_compress, memory_extract_from_conversation. \
            Knowledge graph (graph-RAG): kg_read, kg_search, kg_semantic_search, kg_create_entity, \
            kg_add_observations, kg_create_relation, kg_delete_entity, kg_delete_relation, \
            kg_expand_neighbors, kg_find_path, kg_get_related. \
            TaskStreams: task_stream_create, task_stream_get, task_stream_add, task_stream_context, \
            task_stream_list, task_stream_archive, task_stream_auto_summarize."
                .to_string(),
        );
        info
    }
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Build an Axum router that exposes the in-process memory MCP server over the
/// streamable-HTTP transport (GET SSE / POST / DELETE on the same path).
///
/// Mount on the UAR router at `config.memory.mcp_http_path`:
///
/// ```rust,ignore
/// if config.memory.mcp_http_enabled {
///     let router = memory_mcp_router(Arc::clone(&memory_service));
///     app = app.nest(&config.memory.mcp_http_path, router);
/// }
/// ```
pub fn memory_mcp_router(service: Arc<MemoryService>) -> Router {
    let storage = service.storage();

    let session_manager = Arc::new(LocalSessionManager::default());

    // #[non_exhaustive] -- struct-literal syntax rejected cross-crate even
    // with ..Default::default(); mutate the public field on a default instance.
    let mut config = StreamableHttpServerConfig::default();
    config.stateful_mode = true;

    let http_service = StreamableHttpService::new(
        move || -> Result<UarMemoryMcpServer, std::io::Error> {
            Ok(UarMemoryMcpServer::new(Arc::clone(&storage)))
        },
        Arc::clone(&session_manager),
        config,
    );

    Router::new().route_service("/", http_service)
}
