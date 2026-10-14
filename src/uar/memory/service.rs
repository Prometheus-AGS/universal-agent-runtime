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
use crate::uar::security::claims::{CredentialPrincipal, UserContext};
use super::owned::OwnedMemory;

use super::workflow_mirror::{
    WorkflowMirrorCandidate, WorkflowMirrorRecord, workflow_candidate_from_memory,
};

/// Facade over `SurrealStorage` with scope-aware helpers.
#[derive(Clone)]
pub struct MemoryService {
    pub(crate) storage: Arc<dyn MemoryStorage>,
    pub(crate) config: MemoryConfig,
    pub(crate) embeddings: Arc<dyn EmbeddingService>,
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
                    .or_else(|| std::env::var("LLM_API_KEY").ok())
                    .context("Memory: embedding_provider=openai requires OPENAI_API_KEY, LLM_API_KEY, or memory.openai_api_key config")?;
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

        // Connect to SurrealDB — external server when `surreal_endpoint` is set, otherwise embedded SurrealKV.
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
                database: format!("{}__owner_v1", config.database),
                ..Default::default()
            }
        } else {
            tracing::info!(
                db_path = %config.db_path,
                "Memory: using embedded SurrealDB/SurrealKV"
            );
            SurrealConfig {
                // surreal-memory's Embedded mode currently hardcodes rocksdb://.
                // Use Server mode with a local surrealkv:// endpoint and no auth
                // so the shared SurrealDB crate only needs kv-surrealkv.
                mode: SurrealMode::Server,
                embedded_path: None,
                endpoint: Some(format!("surrealkv://{}", config.db_path)),
                username: None,
                password: None,
                namespace: config.namespace.clone(),
                database: format!("{}__owner_v1", config.database),
                ..Default::default()
            }
        };
        let storage = SurrealStorage::new(&surreal_config, Arc::clone(&embedding_svc))
            .await
            .context("Failed to initialise SurrealDB for memory")?;

        let service = Self {
            storage: Arc::new(storage),
            config,
            embeddings: embedding_svc,
        };
        service.initialize_controls().await?;
        Ok(service)
    }

    // ── Scoped memory ─────────────────────────────────────────────────────────

    /// Add a memory inside the complete verified principal namespace.
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
        let mut memory = Memory::new(
            content,
            None,
            agent_id.map(String::from),
            session_id.map(String::from),
            categories,
        );
        memory.scope = scope;
        memory.memory_type = memory_type;
        memory.metadata = metadata;
        memory.importance = importance;

        self.for_context(user_ctx)?
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

        self.for_context(user_ctx)?
            .search_memories(query, None, agent_id, session_id, categories, limit)
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
        let vw = vector_weight.unwrap_or(self.config.vector_weight);
        let bw = bm25_weight.unwrap_or(self.config.bm25_weight);

        self.for_context(user_ctx)?
            .hybrid_search_memories(query, None, agent_id, session_id, limit, vw, bw)
            .await
            .context("hybrid_search_memories failed")
    }

    /// Capture conversation records within the verified owner namespace.
    pub async fn auto_capture(
        &self,
        messages: Vec<serde_json::Value>,
        user_ctx: &UserContext,
        agent_id: Option<&str>,
        session_id: Option<&str>,
    ) -> Result<Vec<Memory>> {

        self.for_context(user_ctx)?
            .add_memories_from_conversation(messages, None, agent_id, session_id)
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

        self.for_context(user_ctx)?
            .get_all_memories(None, agent_id, session_id)
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
    pub async fn get(&self, id: &str, user_ctx: &UserContext) -> Result<Option<Memory>> {
        self.for_context(user_ctx)?
            .get_memory(id)
            .await
            .context("get_memory failed")
    }

    /// Update a memory's content (creates a history record).
    pub async fn update(&self, id: &str, content: String, user_ctx: &UserContext) -> Result<Memory> {
        self.for_context(user_ctx)?
            .update_memory(id, content)
            .await
            .context("update_memory failed")
    }

    /// Delete a single memory only after resolving the complete caller owner.
    pub async fn delete(&self, id: &str, user_ctx: &UserContext) -> Result<()> {
        self.for_context(user_ctx)?
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

        self.for_context(user_ctx)?
            .delete_all_memories(None, agent_id, session_id)
            .await
            .context("delete_all_memories failed")
    }

    /// Get the audit history for a memory.
    pub async fn history(&self, memory_id: &str, user_ctx: &UserContext) -> Result<Vec<MemoryHistory>> {
        self.for_context(user_ctx)?
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

        self.for_context(user_ctx)?
            .compress_memories(None, agent_id, session_id, older_than_days)
            .await
            .context("compress_memories failed")
    }

    /// Expire stale memories (those past their `valid_until` TTL). Returns expired count.
    pub(crate) async fn expire_stale(&self) -> Result<u64> {
        self.resume_memory_purge().await
    }

    // ── Knowledge Graph ───────────────────────────────────────────────────────

    pub async fn knowledge_graph(&self, user_ctx: &UserContext) -> Result<KnowledgeGraph> {
        self.admin_storage(user_ctx)?.get_graph().await.context("get_graph failed")
    }

    pub async fn create_entity(&self, entity: Entity, user_ctx: &UserContext) -> Result<Entity> {
        self.admin_storage(user_ctx)?
            .create_entity(entity)
            .await
            .context("create_entity failed")
    }

    pub async fn create_entities(&self, entities: Vec<Entity>, user_ctx: &UserContext) -> Result<Vec<Entity>> {
        self.admin_storage(user_ctx)?
            .create_entities(entities)
            .await
            .context("create_entities failed")
    }

    pub async fn get_entity(&self, name: &str, user_ctx: &UserContext) -> Result<Option<Entity>> {
        self.admin_storage(user_ctx)?
            .get_entity(name)
            .await
            .context("get_entity failed")
    }

    pub async fn search_entities(&self, query: &str, user_ctx: &UserContext) -> Result<Vec<Entity>> {
        self.admin_storage(user_ctx)?
            .search_entities(query)
            .await
            .context("search_entities failed")
    }

    pub async fn semantic_search_entities(
        &self,
        query: &str,
        limit: usize,
        threshold: f32,
        user_ctx: &UserContext,
    ) -> Result<Vec<SemanticSearchResult>> {
        self.admin_storage(user_ctx)?
            .semantic_search(query, limit, threshold)
            .await
            .context("semantic_search failed")
    }

    pub async fn add_entity_observations(
        &self,
        name: &str,
        observations: Vec<String>,
        user_ctx: &UserContext,
    ) -> Result<Entity> {
        self.admin_storage(user_ctx)?
            .add_observations(name, observations)
            .await
            .context("add_observations failed")
    }

    pub async fn delete_entity(&self, name: &str, user_ctx: &UserContext) -> Result<()> {
        self.admin_storage(user_ctx)?
            .delete_entity(name)
            .await
            .context("delete_entity failed")
    }

    pub async fn create_relation(&self, relation: Relation, user_ctx: &UserContext) -> Result<Relation> {
        self.admin_storage(user_ctx)?
            .create_relation(relation)
            .await
            .context("create_relation failed")
    }

    pub async fn create_relations(&self, relations: Vec<Relation>, user_ctx: &UserContext) -> Result<Vec<Relation>> {
        self.admin_storage(user_ctx)?
            .create_relations(relations)
            .await
            .context("create_relations failed")
    }

    pub async fn get_relations(&self, entity_name: &str, user_ctx: &UserContext) -> Result<Vec<Relation>> {
        self.admin_storage(user_ctx)?
            .get_relations(entity_name)
            .await
            .context("get_relations failed")
    }

    pub async fn delete_relation(&self, from: &str, to: &str, relation_type: &str, user_ctx: &UserContext) -> Result<()> {
        self.admin_storage(user_ctx)?
            .delete_relation(from, to, relation_type)
            .await
            .context("delete_relation failed")
    }

    pub async fn expand_neighbors(
        &self,
        entity_name: &str,
        depth: u8,
        limit: usize,
        user_ctx: &UserContext,
    ) -> Result<KnowledgeGraph> {
        self.admin_storage(user_ctx)?
            .expand_neighbors(entity_name, depth, limit)
            .await
            .context("expand_neighbors failed")
    }

    pub async fn find_path(&self, from: &str, to: &str, max_depth: u8, user_ctx: &UserContext) -> Result<Vec<Vec<String>>> {
        self.admin_storage(user_ctx)?
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
        user_ctx: &UserContext,
    ) -> Result<Vec<Entity>> {
        self.admin_storage(user_ctx)?
            .get_related(entity_name, relation_type, direction, limit)
            .await
            .context("get_related failed")
    }

    // ── TaskStreams ───────────────────────────────────────────────────────────

    pub async fn create_task_stream(&self, stream: TaskStream, user_ctx: &UserContext) -> Result<TaskStream> {
        self.for_context(user_ctx)?
            .create_task_stream(stream)
            .await
            .context("create_task_stream failed")
    }

    pub async fn get_task_stream(
        &self,
        name: &str,
        user_ctx: &UserContext,
        agent_id: Option<&str>,
    ) -> Result<Option<TaskStream>> {
        self.for_context(user_ctx)?
            .get_task_stream(name, None, agent_id)
            .await
            .context("get_task_stream failed")
    }

    pub async fn add_to_task_stream(
        &self,
        stream_name: &str,
        user_ctx: &UserContext,
        agent_id: Option<&str>,
        memory: Memory,
    ) -> Result<Memory> {
        self.for_context(user_ctx)?
            .add_to_task_stream(stream_name, None, agent_id, memory)
            .await
            .context("add_to_task_stream failed")
    }

    /// Get a model-aware context window for a TaskStream, respecting the token budget.
    pub async fn task_stream_context(
        &self,
        stream_name: &str,
        user_ctx: &UserContext,
        agent_id: Option<&str>,
        model_id: &str,
        max_tokens: Option<u64>,
    ) -> Result<ContextWindow> {
        self.for_context(user_ctx)?
            .get_context_for_task(stream_name, None, agent_id, model_id, max_tokens)
            .await
            .context("get_context_for_task failed")
    }

    pub async fn list_task_streams(
        &self,
        agent_id: Option<&str>,
        user_ctx: &UserContext,
    ) -> Result<Vec<TaskStream>> {
        self.for_context(user_ctx)?
            .list_task_streams(agent_id, None)
            .await
            .context("list_task_streams failed")
    }

    pub async fn archive_task_stream(
        &self,
        name: &str,
        user_ctx: &UserContext,
        agent_id: Option<&str>,
    ) -> Result<TaskStream> {
        self.for_context(user_ctx)?
            .archive_task_stream(name, None, agent_id)
            .await
            .context("archive_task_stream failed")
    }

    pub async fn auto_summarize_task_stream(
        &self,
        stream_name: &str,
        user_ctx: &UserContext,
        agent_id: Option<&str>,
        model_id: &str,
    ) -> Result<Option<Memory>> {
        self.for_context(user_ctx)?
            .auto_summarize_task_stream(stream_name, None, agent_id, model_id)
            .await
            .context("auto_summarize_task_stream failed")
    }

    // ── Mindmaps ─────────────────────────────────────────────────────────────

    pub async fn create_mindmap(&self, mindmap: MindMap, user_ctx: &UserContext) -> Result<MindMap> {
        self.admin_storage(user_ctx)?
            .create_mindmap(mindmap)
            .await
            .context("create_mindmap failed")
    }

    pub async fn get_mindmap(&self, name: &str, user_id: Option<&str>, user_ctx: &UserContext) -> Result<Option<MindMap>> {
        self.admin_storage(user_ctx)?
            .get_mindmap(name, user_id, None)
            .await
            .context("get_mindmap failed")
    }

    pub async fn list_mindmaps(
        &self,
        user_id: Option<&str>,
        agent_id: Option<&str>,
        user_ctx: &UserContext,
    ) -> Result<Vec<MindMap>> {
        self.admin_storage(user_ctx)?
            .list_mindmaps(user_id, agent_id)
            .await
            .context("list_mindmaps failed")
    }

    pub async fn add_mindmap_node(
        &self,
        mindmap_name: &str,
        user_id: Option<&str>,
        node: MindMapNode,
        user_ctx: &UserContext,
    ) -> Result<MindMap> {
        self.admin_storage(user_ctx)?
            .add_mindmap_node(mindmap_name, user_id, None, node)
            .await
            .context("add_mindmap_node failed")
    }

    pub async fn add_mindmap_edge(
        &self,
        mindmap_name: &str,
        user_id: Option<&str>,
        edge: MindMapEdge,
        user_ctx: &UserContext,
    ) -> Result<MindMap> {
        self.admin_storage(user_ctx)?
            .add_mindmap_edge(mindmap_name, user_id, None, edge)
            .await
            .context("add_mindmap_edge failed")
    }

    pub async fn delete_mindmap_node(
        &self,
        mindmap_name: &str,
        user_id: Option<&str>,
        node_id: &str,
        user_ctx: &UserContext,
    ) -> Result<MindMap> {
        self.admin_storage(user_ctx)?
            .delete_mindmap_node(mindmap_name, user_id, None, node_id)
            .await
            .context("delete_mindmap_node failed")
    }

    pub async fn delete_mindmap(&self, name: &str, user_id: Option<&str>, user_ctx: &UserContext) -> Result<()> {
        self.admin_storage(user_ctx)?
            .delete_mindmap(name, user_id, None)
            .await
            .context("delete_mindmap failed")
    }

    // ── Storage arc accessor (for MCP server handler) ─────────────────────────

    /// Bind a non-deserializable complete owner captured by the host.
    pub fn for_owner(&self, principal: &CredentialPrincipal) -> OwnedMemory<'_> {
        OwnedMemory { service: self, principal: principal.clone() }
    }

    pub fn for_context(&self, context: &UserContext) -> Result<OwnedMemory<'_>> {
        Ok(self.for_owner(&CredentialPrincipal::from_verified_context(context)?))
    }

    /// Global administrative access requires installation/service authority;
    /// role strings or user-supplied query filters do not establish it.
    pub fn admin_storage(&self, context: &UserContext) -> Result<Arc<dyn MemoryStorage>> {
        anyhow::ensure!(context.trusted_host_id().is_some(), "Trusted host memory administration required");
        Ok(Arc::clone(&self.storage))
    }

    /// Config accessor.
    pub fn config(&self) -> &MemoryConfig {
        &self.config
    }
}
