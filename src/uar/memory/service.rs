//! `MemoryService` — the primary facade over `SurrealStorage`.
//!
//! Wraps the surreal-memory `MemoryStorage` trait implementation with UAR-level
//! scope enforcement, error handling, and logging. All callers should use this
//! struct rather than accessing `SurrealStorage` directly.

use anyhow::{Context, Result};
use std::sync::Arc;
use surreal_memory::{
    EmbeddingProvider, EmbeddingService, Memory, MemoryHistory, MemoryScope, MemoryType,
    SurrealStorage,
    entity::{Entity, KnowledgeGraph, Relation, SemanticSearchResult},
    mindmap::{MindMap, MindMapEdge, MindMapNode},
    storage::{
        MemoryStorage,
        surreal::{SurrealConfig, SurrealMode},
    },
    task_stream::{ContextWindow, TaskStream},
};

use crate::config::MemoryConfig;
use crate::uar::security::claims::UserContext;

use super::workflow_mirror::{
    WorkflowMirrorCandidate, WorkflowMirrorRecord, workflow_candidate_from_memory,
};

/// Facade over `SurrealStorage` with scope-aware helpers.
#[derive(Clone)]
pub struct MemoryService {
    pub(crate) storage: Arc<dyn MemoryStorage>,
    pub(crate) config: MemoryConfig,
}

impl std::fmt::Debug for MemoryService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MemoryService").finish_non_exhaustive()
    }
}

impl MemoryService {
    /// Initialise the service by connecting to (or creating) the embedded SurrealDB store.
    pub async fn new(config: MemoryConfig) -> Result<Self> {
        tracing::info!(
            db_path = %config.db_path,
            embedding_provider = %config.embedding_provider,
            "Initialising agent memory service"
        );

        // Build the embedding provider.
        let embedding_svc: Arc<dyn EmbeddingService> = match config.embedding_provider.as_str() {
            "openai" => {
                let api_key = config
                    .openai_api_key
                    .clone()
                    .or_else(|| std::env::var("OPENAI_API_KEY").ok())
                    .context("Memory: embedding_provider=openai requires OPENAI_API_KEY env var or memory.openai_api_key config")?;
                let provider = EmbeddingProvider::OpenAI {
                    api_key,
                    model: config.embedding_model.clone(),
                };
                Arc::from(surreal_memory::embeddings::create_embedding_service(provider).await?)
            }
            "cohere" => {
                let api_key = config
                    .cohere_api_key
                    .clone()
                    .or_else(|| std::env::var("COHERE_API_KEY").ok())
                    .context("Memory: embedding_provider=cohere requires COHERE_API_KEY env var or memory.cohere_api_key config")?;
                let provider = EmbeddingProvider::Cohere {
                    api_key,
                    model: config.embedding_model.clone(),
                };
                Arc::from(surreal_memory::embeddings::create_embedding_service(provider).await?)
            }
            _ => {
                // local / default
                let provider = EmbeddingProvider::Local {
                    model_id: config.embedding_model.clone(),
                    model_path: None,
                };
                Arc::from(surreal_memory::embeddings::create_embedding_service(provider).await?)
            }
        };

        tracing::info!(
            dimensions = embedding_svc.dimensions(),
            "Memory embedding service ready"
        );

        // Connect to SurrealDB — external server when `surreal_endpoint` is set, otherwise embedded.
        let surreal_config = if let Some(ref endpoint) = config.surreal_endpoint {
            tracing::info!(
                endpoint = %endpoint,
                "Memory: connecting to external SurrealDB instance"
            );
            SurrealConfig {
                mode: SurrealMode::Server,
                embedded_path: None,
                endpoint: Some(endpoint.clone()),
                username: config.surreal_user.clone(),
                password: config.surreal_pass.clone(),
                namespace: config.namespace.clone(),
                database: config.database.clone(),
                ..Default::default()
            }
        } else {
            tracing::info!(
                db_path = %config.db_path,
                "Memory: using embedded SurrealDB/RocksDB"
            );
            SurrealConfig {
                mode: SurrealMode::Embedded,
                embedded_path: Some(config.db_path.clone()),
                endpoint: None,
                username: None,
                password: None,
                namespace: config.namespace.clone(),
                database: config.database.clone(),
                ..Default::default()
            }
        };
        let storage = SurrealStorage::new(&surreal_config, embedding_svc)
            .await
            .context("Failed to initialise SurrealDB for memory")?;

        Ok(Self {
            storage: Arc::new(storage),
            config,
        })
    }

    // ── Scoped memory ─────────────────────────────────────────────────────────

    /// Add a memory with explicit scope. User-scoped memories require `user_ctx.user_id`.
    pub async fn add(
        &self,
        content: impl Into<String>,
        scope: MemoryScope,
        memory_type: MemoryType,
        user_ctx: &UserContext,
        agent_id: Option<&str>,
        session_id: Option<&str>,
        categories: Vec<String>,
        metadata: Option<serde_json::Value>,
        importance: f32,
    ) -> Result<Memory> {
        let user_id = match &scope {
            MemoryScope::User | MemoryScope::Session => {
                // Only authenticated users may write user/session-scoped memories.
                if user_ctx.user_id == "anonymous" {
                    anyhow::bail!(
                        "User-scoped memories require a valid JWT. Provide an Authorization: Bearer <token> header."
                    );
                }
                Some(user_ctx.user_id.clone())
            }
            _ => None,
        };

        let mut memory = Memory::new(
            content,
            user_id,
            agent_id.map(String::from),
            session_id.map(String::from),
            categories,
        );
        memory.scope = scope;
        memory.memory_type = memory_type;
        memory.metadata = metadata;
        memory.importance = importance;

        self.storage
            .add_memory(memory)
            .await
            .context("add_memory failed")
    }

    /// Semantic search over scoped memories. Automatically filters by user ownership.
    pub async fn search(
        &self,
        query: &str,
        user_ctx: &UserContext,
        agent_id: Option<&str>,
        session_id: Option<&str>,
        limit: usize,
        categories: Option<&[String]>,
    ) -> Result<Vec<Memory>> {
        let user_id = if user_ctx.user_id != "anonymous" {
            Some(user_ctx.user_id.as_str())
        } else {
            None
        };

        self.storage
            .search_memories(query, user_id, agent_id, session_id, categories, limit)
            .await
            .context("search_memories failed")
    }

    /// Hybrid BM25 + HNSW vector search. Weights are read from config by default.
    pub async fn hybrid_search(
        &self,
        query: &str,
        user_ctx: &UserContext,
        agent_id: Option<&str>,
        session_id: Option<&str>,
        limit: usize,
        vector_weight: Option<f32>,
        bm25_weight: Option<f32>,
    ) -> Result<Vec<Memory>> {
        let user_id = if user_ctx.user_id != "anonymous" {
            Some(user_ctx.user_id.as_str())
        } else {
            None
        };
        let vw = vector_weight.unwrap_or(self.config.vector_weight);
        let bw = bm25_weight.unwrap_or(self.config.bm25_weight);

        self.storage
            .hybrid_search_memories(query, user_id, agent_id, session_id, limit, vw, bw)
            .await
            .context("hybrid_search_memories failed")
    }

    /// Auto-extract memories from a conversation turn (calls the embedding LLM internally).
    pub async fn auto_capture(
        &self,
        messages: Vec<serde_json::Value>,
        user_ctx: &UserContext,
        agent_id: Option<&str>,
        session_id: Option<&str>,
    ) -> Result<Vec<Memory>> {
        let user_id = if user_ctx.user_id != "anonymous" {
            Some(user_ctx.user_id.clone())
        } else {
            None
        };

        self.storage
            .add_memories_from_conversation(messages, user_id.as_deref(), agent_id, session_id)
            .await
            .context("add_memories_from_conversation failed")
    }

    /// List all memories for a scope, filtered by caller identity.
    pub async fn list(
        &self,
        user_ctx: &UserContext,
        agent_id: Option<&str>,
        session_id: Option<&str>,
    ) -> Result<Vec<Memory>> {
        let user_id = if user_ctx.user_id != "anonymous" {
            Some(user_ctx.user_id.as_str())
        } else {
            None
        };

        self.storage
            .get_all_memories(user_id, agent_id, session_id)
            .await
            .context("get_all_memories failed")
    }

    /// Add a KBD/OpenSpec workflow mirror record through the memory service boundary.
    pub async fn add_workflow_mirror_record(
        &self,
        record: WorkflowMirrorRecord,
        user_ctx: &UserContext,
    ) -> Result<Memory> {
        let write = record.to_memory_write();
        self.add(
            write.content,
            write.scope,
            write.memory_type,
            user_ctx,
            None,
            None,
            write.categories,
            Some(write.metadata),
            write.importance,
        )
        .await
    }

    /// List workflow mirror recovery candidates from scoped memory records.
    pub async fn list_workflow_mirror_candidates(
        &self,
        user_ctx: &UserContext,
    ) -> Result<Vec<WorkflowMirrorCandidate>> {
        let memories = self.list(user_ctx, None, None).await?;
        Ok(memories
            .iter()
            .filter_map(|memory| workflow_candidate_from_memory(memory).ok())
            .collect())
    }

    /// Retrieve a single memory by its record ID string (e.g. `"memory:abc123"`).
    pub async fn get(&self, id: &str) -> Result<Option<Memory>> {
        self.storage
            .get_memory(id)
            .await
            .context("get_memory failed")
    }

    /// Update a memory's content (creates a history record).
    pub async fn update(&self, id: &str, content: String) -> Result<Memory> {
        self.storage
            .update_memory(id, content)
            .await
            .context("update_memory failed")
    }

    /// Delete a single memory. Ownership is not enforced here — callers should
    /// validate ownership before calling (e.g. via `get` + scope check).
    pub async fn delete(&self, id: &str) -> Result<()> {
        self.storage
            .delete_memory(id)
            .await
            .context("delete_memory failed")
    }

    /// Delete all memories for a scope filter.
    pub async fn delete_all(
        &self,
        user_ctx: &UserContext,
        agent_id: Option<&str>,
        session_id: Option<&str>,
    ) -> Result<u64> {
        let user_id = if user_ctx.user_id != "anonymous" {
            Some(user_ctx.user_id.as_str())
        } else {
            None
        };

        self.storage
            .delete_all_memories(user_id, agent_id, session_id)
            .await
            .context("delete_all_memories failed")
    }

    /// Get the audit history for a memory.
    pub async fn history(&self, memory_id: &str) -> Result<Vec<MemoryHistory>> {
        self.storage
            .get_memory_history(memory_id)
            .await
            .context("get_memory_history failed")
    }

    /// Compress memories older than `older_than_days` into a single summary.
    pub async fn compress(
        &self,
        user_ctx: &UserContext,
        agent_id: Option<&str>,
        session_id: Option<&str>,
        older_than_days: u32,
    ) -> Result<Option<Memory>> {
        let user_id = if user_ctx.user_id != "anonymous" {
            Some(user_ctx.user_id.as_str())
        } else {
            None
        };

        self.storage
            .compress_memories(user_id, agent_id, session_id, older_than_days)
            .await
            .context("compress_memories failed")
    }

    /// Expire stale memories (those past their `valid_until` TTL). Returns expired count.
    pub async fn expire_stale(&self) -> Result<u64> {
        self.storage
            .expire_stale_memories()
            .await
            .context("expire_stale_memories failed")
    }

    // ── Knowledge Graph ───────────────────────────────────────────────────────

    pub async fn knowledge_graph(&self) -> Result<KnowledgeGraph> {
        self.storage.get_graph().await.context("get_graph failed")
    }

    pub async fn create_entity(&self, entity: Entity) -> Result<Entity> {
        self.storage
            .create_entity(entity)
            .await
            .context("create_entity failed")
    }

    pub async fn create_entities(&self, entities: Vec<Entity>) -> Result<Vec<Entity>> {
        self.storage
            .create_entities(entities)
            .await
            .context("create_entities failed")
    }

    pub async fn get_entity(&self, name: &str) -> Result<Option<Entity>> {
        self.storage
            .get_entity(name)
            .await
            .context("get_entity failed")
    }

    pub async fn search_entities(&self, query: &str) -> Result<Vec<Entity>> {
        self.storage
            .search_entities(query)
            .await
            .context("search_entities failed")
    }

    pub async fn semantic_search_entities(
        &self,
        query: &str,
        limit: usize,
        threshold: f32,
    ) -> Result<Vec<SemanticSearchResult>> {
        self.storage
            .semantic_search(query, limit, threshold)
            .await
            .context("semantic_search failed")
    }

    pub async fn add_entity_observations(
        &self,
        name: &str,
        observations: Vec<String>,
    ) -> Result<Entity> {
        self.storage
            .add_observations(name, observations)
            .await
            .context("add_observations failed")
    }

    pub async fn delete_entity(&self, name: &str) -> Result<()> {
        self.storage
            .delete_entity(name)
            .await
            .context("delete_entity failed")
    }

    pub async fn create_relation(&self, relation: Relation) -> Result<Relation> {
        self.storage
            .create_relation(relation)
            .await
            .context("create_relation failed")
    }

    pub async fn create_relations(&self, relations: Vec<Relation>) -> Result<Vec<Relation>> {
        self.storage
            .create_relations(relations)
            .await
            .context("create_relations failed")
    }

    pub async fn get_relations(&self, entity_name: &str) -> Result<Vec<Relation>> {
        self.storage
            .get_relations(entity_name)
            .await
            .context("get_relations failed")
    }

    pub async fn delete_relation(&self, from: &str, to: &str, relation_type: &str) -> Result<()> {
        self.storage
            .delete_relation(from, to, relation_type)
            .await
            .context("delete_relation failed")
    }

    pub async fn expand_neighbors(
        &self,
        entity_name: &str,
        depth: u8,
        limit: usize,
    ) -> Result<KnowledgeGraph> {
        self.storage
            .expand_neighbors(entity_name, depth, limit)
            .await
            .context("expand_neighbors failed")
    }

    pub async fn find_path(&self, from: &str, to: &str, max_depth: u8) -> Result<Vec<Vec<String>>> {
        self.storage
            .find_path(from, to, max_depth)
            .await
            .context("find_path failed")
    }

    pub async fn get_related(
        &self,
        entity_name: &str,
        relation_type: Option<&str>,
        direction: &str,
        limit: usize,
    ) -> Result<Vec<Entity>> {
        self.storage
            .get_related(entity_name, relation_type, direction, limit)
            .await
            .context("get_related failed")
    }

    // ── TaskStreams ───────────────────────────────────────────────────────────

    pub async fn create_task_stream(&self, stream: TaskStream) -> Result<TaskStream> {
        self.storage
            .create_task_stream(stream)
            .await
            .context("create_task_stream failed")
    }

    pub async fn get_task_stream(&self, name: &str) -> Result<Option<TaskStream>> {
        self.storage
            .get_task_stream(name)
            .await
            .context("get_task_stream failed")
    }

    pub async fn add_to_task_stream(&self, stream_name: &str, memory: Memory) -> Result<Memory> {
        self.storage
            .add_to_task_stream(stream_name, memory)
            .await
            .context("add_to_task_stream failed")
    }

    /// Get a model-aware context window for a TaskStream, respecting the token budget.
    pub async fn task_stream_context(
        &self,
        stream_name: &str,
        model_id: &str,
        max_tokens: Option<u64>,
    ) -> Result<ContextWindow> {
        self.storage
            .get_context_for_task(stream_name, model_id, max_tokens)
            .await
            .context("get_context_for_task failed")
    }

    pub async fn list_task_streams(
        &self,
        agent_id: Option<&str>,
        user_id: Option<&str>,
    ) -> Result<Vec<TaskStream>> {
        self.storage
            .list_task_streams(agent_id, user_id)
            .await
            .context("list_task_streams failed")
    }

    pub async fn archive_task_stream(&self, name: &str) -> Result<TaskStream> {
        self.storage
            .archive_task_stream(name)
            .await
            .context("archive_task_stream failed")
    }

    pub async fn auto_summarize_task_stream(
        &self,
        stream_name: &str,
        user_id: Option<&str>,
        agent_id: Option<&str>,
        model_id: &str,
    ) -> Result<Option<Memory>> {
        self.storage
            .auto_summarize_task_stream(stream_name, user_id, agent_id, model_id)
            .await
            .context("auto_summarize_task_stream failed")
    }

    // ── Mindmaps ─────────────────────────────────────────────────────────────

    pub async fn create_mindmap(&self, mindmap: MindMap) -> Result<MindMap> {
        self.storage
            .create_mindmap(mindmap)
            .await
            .context("create_mindmap failed")
    }

    pub async fn get_mindmap(&self, name: &str, user_id: Option<&str>) -> Result<Option<MindMap>> {
        self.storage
            .get_mindmap(name, user_id, None)
            .await
            .context("get_mindmap failed")
    }

    pub async fn list_mindmaps(
        &self,
        user_id: Option<&str>,
        agent_id: Option<&str>,
    ) -> Result<Vec<MindMap>> {
        self.storage
            .list_mindmaps(user_id, agent_id)
            .await
            .context("list_mindmaps failed")
    }

    pub async fn add_mindmap_node(
        &self,
        mindmap_name: &str,
        user_id: Option<&str>,
        node: MindMapNode,
    ) -> Result<MindMap> {
        self.storage
            .add_mindmap_node(mindmap_name, user_id, None, node)
            .await
            .context("add_mindmap_node failed")
    }

    pub async fn add_mindmap_edge(
        &self,
        mindmap_name: &str,
        user_id: Option<&str>,
        edge: MindMapEdge,
    ) -> Result<MindMap> {
        self.storage
            .add_mindmap_edge(mindmap_name, user_id, None, edge)
            .await
            .context("add_mindmap_edge failed")
    }

    pub async fn delete_mindmap_node(
        &self,
        mindmap_name: &str,
        user_id: Option<&str>,
        node_id: &str,
    ) -> Result<MindMap> {
        self.storage
            .delete_mindmap_node(mindmap_name, user_id, None, node_id)
            .await
            .context("delete_mindmap_node failed")
    }

    pub async fn delete_mindmap(&self, name: &str, user_id: Option<&str>) -> Result<()> {
        self.storage
            .delete_mindmap(name, user_id, None)
            .await
            .context("delete_mindmap failed")
    }

    // ── Storage arc accessor (for MCP server handler) ─────────────────────────

    /// Return the underlying storage for direct use in the MCP server handler.
    pub fn storage(&self) -> Arc<dyn MemoryStorage> {
        Arc::clone(&self.storage)
    }

    /// Config accessor.
    pub fn config(&self) -> &MemoryConfig {
        &self.config
    }
}
