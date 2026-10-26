//! Lifecycle admission around the memory-storage capability.
//!
//! A retained administrative storage handle remains a capability, not an engine
//! ownership transfer. Every delegated operation is admitted before it starts.
//! A canceled caller only loses its reply. The retained worker records an
//! explicit unknown outcome only when a mutating operation lacks an
//! acknowledgement; it never infers rollback from waiter cancellation.

use std::{any::Any, sync::Arc};

use anyhow::Result;
use async_trait::async_trait;
use surreal_memory::{
    Memory, MemoryHistory,
    entity::{Entity, KnowledgeGraph, Relation, SemanticSearchResult},
    mindmap::{MindMap, MindMapEdge, MindMapNode},
    storage::MemoryStorage,
    task_step::{TaskStep, TaskStepStatus},
    task_stream::{ContextWindow, TaskStream},
};

use crate::uar::runtime::lifecycle::ResourceScope;

pub(crate) struct ScopedMemoryStorage {
    inner: Arc<dyn MemoryStorage>,
    scope: ResourceScope,
}

impl ScopedMemoryStorage {
    pub(crate) fn new(inner: Arc<dyn MemoryStorage>, scope: ResourceScope) -> Self {
        Self { inner, scope }
    }

    /// Keep the delegated storage future in the resource scope after admission.
    /// A canceled caller only loses its reply; closing the scope still joins the
    /// admitted work and retains uncertain mutations from a lost transport.
    async fn dispatch<T, F, Fut>(&self, method: &'static str, work: F) -> Result<T>
    where
        T: Send + 'static,
        F: FnOnce(Arc<dyn MemoryStorage>) -> Fut + Send + 'static,
        Fut: Future<Output = Result<T>> + Send + 'static,
    {
        let operation = uuid::Uuid::new_v4().to_string();
        let worker_scope = self.scope.clone();
        let inner = Arc::clone(&self.inner);
        let (sender, receiver) = tokio::sync::oneshot::channel();
        self.scope.spawn(format!("memory-storage:{method}:{operation}"), async move {
            let mut completion = MemoryCompletion::new(worker_scope, method, &operation);
            let result = work(inner).await;
            completion.observe_result(&result);
            let _ = sender.send(result);
            Ok(())
        })?;
        receiver.await.map_err(|_| anyhow::anyhow!(
            "retained memory storage operation '{method}' ended before reporting its outcome"
        ))?
    }
}

struct MemoryCompletion {
    scope: ResourceScope,
    operation: String,
    may_mutate: bool,
    acknowledged: bool,
}

impl MemoryCompletion {
    fn new(scope: ResourceScope, name: &str, identity: &str) -> Self {
        Self {
            scope,
            operation: format!("{name}:{identity}"),
            may_mutate: !read_only_memory_operation(name),
            acknowledged: false,
        }
    }

    fn observe_result<T>(&mut self, result: &Result<T>) {
        if self.may_mutate && result.as_ref().err().is_some_and(transport_outcome_unknown) {
            self.scope.record_unresolved_effect(self.operation.clone());
        }
        self.acknowledged = true;
    }
}

impl Drop for MemoryCompletion {
    fn drop(&mut self) {
        if !self.acknowledged && self.may_mutate {
            self.scope.record_unresolved_effect(self.operation.clone());
        }
    }
}

#[cfg(feature = "surreal-backend")]
fn transport_outcome_unknown(error: &anyhow::Error) -> bool {
    error.chain().any(|cause| {
        cause
            .downcast_ref::<surrealdb::types::Error>()
            .is_some_and(|error| error.is_connection())
    })
}

// Without a concrete backend classifier, an error cannot acknowledge a
// mutating outcome. Keep the original error and the unresolved obligation.
#[cfg(not(feature = "surreal-backend"))]
fn transport_outcome_unknown(_error: &anyhow::Error) -> bool {
    true
}

// Snapshots and searches have no durable effect to reconcile. New operations
// default to mutating until their contract is explicitly classified here.
fn read_only_memory_operation(name: &str) -> bool {
    matches!(name,
        "get_entity" | "search_entities" | "get_relations" | "get_graph" |
        "semantic_search" | "get_memory" | "get_all_memories" |
        "search_memories" | "get_memory_history" | "hybrid_search_memories" |
        "get_task_stream" | "get_context_for_task" | "list_task_streams" |
        "get_task_steps" | "get_current_step" | "get_mindmap" |
        "list_mindmaps" | "find_path" | "expand_neighbors" | "get_related" |
        "get_entity_history" | "get_graph_at_time"
    )
}

#[async_trait]
impl MemoryStorage for ScopedMemoryStorage {


    async fn create_entity(&self, entity: Entity) -> Result<Entity> {
        self.dispatch("create_entity", move |inner| async move {
            inner.create_entity(entity).await
        }).await
    }

    async fn create_entities(&self, entities: Vec<Entity>) -> Result<Vec<Entity>> {
        self.dispatch("create_entities", move |inner| async move {
            inner.create_entities(entities).await
        }).await
    }

    async fn get_entity(&self, name: &str) -> Result<Option<Entity>> {
        let name = name.to_owned();
        self.dispatch("get_entity", move |inner| async move {
            inner.get_entity(&name).await
        }).await
    }

    async fn update_entity(&self, entity: Entity) -> Result<Entity> {
        self.dispatch("update_entity", move |inner| async move {
            inner.update_entity(entity).await
        }).await
    }

    async fn delete_entity(&self, name: &str) -> Result<()> {
        let name = name.to_owned();
        self.dispatch("delete_entity", move |inner| async move {
            inner.delete_entity(&name).await
        }).await
    }

    async fn search_entities(&self, query: &str) -> Result<Vec<Entity>> {
        let query = query.to_owned();
        self.dispatch("search_entities", move |inner| async move {
            inner.search_entities(&query).await
        }).await
    }

    async fn create_relation(&self, relation: Relation) -> Result<Relation> {
        self.dispatch("create_relation", move |inner| async move {
            inner.create_relation(relation).await
        }).await
    }

    async fn create_relations(&self, relations: Vec<Relation>) -> Result<Vec<Relation>> {
        self.dispatch("create_relations", move |inner| async move {
            inner.create_relations(relations).await
        }).await
    }

    async fn get_relations(&self, entity_name: &str) -> Result<Vec<Relation>> {
        let entity_name = entity_name.to_owned();
        self.dispatch("get_relations", move |inner| async move {
            inner.get_relations(&entity_name).await
        }).await
    }

    async fn delete_relation(&self, from: &str, to: &str, relation_type: &str) -> Result<()> {
        let from = from.to_owned();
        let to = to.to_owned();
        let relation_type = relation_type.to_owned();
        self.dispatch("delete_relation", move |inner| async move {
            inner.delete_relation(&from, &to, &relation_type).await
        }).await
    }

    async fn get_graph(&self) -> Result<KnowledgeGraph> {
        self.dispatch("get_graph", move |inner| async move {
            inner.get_graph().await
        }).await
    }

    async fn add_observations(&self, entity_name: &str, observations: Vec<String>) -> Result<Entity> {
        let entity_name = entity_name.to_owned();
        self.dispatch("add_observations", move |inner| async move {
            inner.add_observations(&entity_name, observations).await
        }).await
    }

    async fn semantic_search(&self, query: &str, limit: usize, threshold: f32) -> Result<Vec<SemanticSearchResult>> {
        let query = query.to_owned();
        self.dispatch("semantic_search", move |inner| async move {
            inner.semantic_search(&query, limit, threshold).await
        }).await
    }

    async fn add_memory(&self, memory: Memory) -> Result<Memory> {
        self.dispatch("add_memory", move |inner| async move {
            inner.add_memory(memory).await
        }).await
    }

    async fn get_memory(&self, id: &str) -> Result<Option<Memory>> {
        let id = id.to_owned();
        self.dispatch("get_memory", move |inner| async move {
            inner.get_memory(&id).await
        }).await
    }

    async fn update_memory(&self, id: &str, content: String) -> Result<Memory> {
        let id = id.to_owned();
        self.dispatch("update_memory", move |inner| async move {
            inner.update_memory(&id, content).await
        }).await
    }

    async fn delete_memory(&self, id: &str) -> Result<()> {
        let id = id.to_owned();
        self.dispatch("delete_memory", move |inner| async move {
            inner.delete_memory(&id).await
        }).await
    }

    async fn delete_all_memories(&self, user_id: Option<&str>, agent_id: Option<&str>, session_id: Option<&str>) -> Result<u64> {
        let user_id = user_id.map(str::to_owned);
        let agent_id = agent_id.map(str::to_owned);
        let session_id = session_id.map(str::to_owned);
        self.dispatch("delete_all_memories", move |inner| async move {
            inner.delete_all_memories(user_id.as_deref(), agent_id.as_deref(), session_id.as_deref()).await
        }).await
    }

    async fn get_all_memories(&self, user_id: Option<&str>, agent_id: Option<&str>, session_id: Option<&str>) -> Result<Vec<Memory>> {
        let user_id = user_id.map(str::to_owned);
        let agent_id = agent_id.map(str::to_owned);
        let session_id = session_id.map(str::to_owned);
        self.dispatch("get_all_memories", move |inner| async move {
            inner.get_all_memories(user_id.as_deref(), agent_id.as_deref(), session_id.as_deref()).await
        }).await
    }

    async fn search_memories(&self, query: &str, user_id: Option<&str>, agent_id: Option<&str>, session_id: Option<&str>, categories: Option<&[String]>, limit: usize) -> Result<Vec<Memory>> {
        let query = query.to_owned();
        let user_id = user_id.map(str::to_owned);
        let agent_id = agent_id.map(str::to_owned);
        let session_id = session_id.map(str::to_owned);
        let categories = categories.map(|values| values.to_vec());
        self.dispatch("search_memories", move |inner| async move {
            inner.search_memories(&query, user_id.as_deref(), agent_id.as_deref(), session_id.as_deref(), categories.as_deref(), limit).await
        }).await
    }

    async fn get_memory_history(&self, memory_id: &str) -> Result<Vec<MemoryHistory>> {
        let memory_id = memory_id.to_owned();
        self.dispatch("get_memory_history", move |inner| async move {
            inner.get_memory_history(&memory_id).await
        }).await
    }

    async fn hybrid_search_memories(&self, query: &str, user_id: Option<&str>, agent_id: Option<&str>, session_id: Option<&str>, limit: usize, vector_weight: f32, bm25_weight: f32) -> Result<Vec<Memory>> {
        let query = query.to_owned();
        let user_id = user_id.map(str::to_owned);
        let agent_id = agent_id.map(str::to_owned);
        let session_id = session_id.map(str::to_owned);
        self.dispatch("hybrid_search_memories", move |inner| async move {
            inner.hybrid_search_memories(&query, user_id.as_deref(), agent_id.as_deref(), session_id.as_deref(), limit, vector_weight, bm25_weight).await
        }).await
    }

    async fn compress_memories(&self, user_id: Option<&str>, agent_id: Option<&str>, session_id: Option<&str>, older_than_days: u32) -> Result<Option<Memory>> {
        let user_id = user_id.map(str::to_owned);
        let agent_id = agent_id.map(str::to_owned);
        let session_id = session_id.map(str::to_owned);
        self.dispatch("compress_memories", move |inner| async move {
            inner.compress_memories(user_id.as_deref(), agent_id.as_deref(), session_id.as_deref(), older_than_days).await
        }).await
    }

    async fn add_memories_from_conversation(&self, messages: Vec<serde_json::Value>, user_id: Option<&str>, agent_id: Option<&str>, session_id: Option<&str>) -> Result<Vec<Memory>> {
        let user_id = user_id.map(str::to_owned);
        let agent_id = agent_id.map(str::to_owned);
        let session_id = session_id.map(str::to_owned);
        self.dispatch("add_memories_from_conversation", move |inner| async move {
            inner.add_memories_from_conversation(messages, user_id.as_deref(), agent_id.as_deref(), session_id.as_deref()).await
        }).await
    }

    async fn expire_stale_memories(&self) -> Result<u64> {
        self.dispatch("expire_stale_memories", move |inner| async move {
            inner.expire_stale_memories().await
        }).await
    }

    async fn create_task_stream(&self, stream: TaskStream) -> Result<TaskStream> {
        self.dispatch("create_task_stream", move |inner| async move {
            inner.create_task_stream(stream).await
        }).await
    }

    async fn get_task_stream(&self, name: &str, user_id: Option<&str>, agent_id: Option<&str>) -> Result<Option<TaskStream>> {
        let name = name.to_owned();
        let user_id = user_id.map(str::to_owned);
        let agent_id = agent_id.map(str::to_owned);
        self.dispatch("get_task_stream", move |inner| async move {
            inner.get_task_stream(&name, user_id.as_deref(), agent_id.as_deref()).await
        }).await
    }

    async fn add_to_task_stream(&self, stream_name: &str, user_id: Option<&str>, agent_id: Option<&str>, memory: Memory) -> Result<Memory> {
        let stream_name = stream_name.to_owned();
        let user_id = user_id.map(str::to_owned);
        let agent_id = agent_id.map(str::to_owned);
        self.dispatch("add_to_task_stream", move |inner| async move {
            inner.add_to_task_stream(&stream_name, user_id.as_deref(), agent_id.as_deref(), memory).await
        }).await
    }

    async fn get_context_for_task(&self, stream_name: &str, user_id: Option<&str>, agent_id: Option<&str>, model_name: &str, max_tokens: Option<u64>) -> Result<ContextWindow> {
        let stream_name = stream_name.to_owned();
        let user_id = user_id.map(str::to_owned);
        let agent_id = agent_id.map(str::to_owned);
        let model_name = model_name.to_owned();
        self.dispatch("get_context_for_task", move |inner| async move {
            inner.get_context_for_task(&stream_name, user_id.as_deref(), agent_id.as_deref(), &model_name, max_tokens).await
        }).await
    }

    async fn list_task_streams(&self, agent_id: Option<&str>, user_id: Option<&str>) -> Result<Vec<TaskStream>> {
        let agent_id = agent_id.map(str::to_owned);
        let user_id = user_id.map(str::to_owned);
        self.dispatch("list_task_streams", move |inner| async move {
            inner.list_task_streams(agent_id.as_deref(), user_id.as_deref()).await
        }).await
    }

    async fn delete_task_stream(&self, name: &str, user_id: Option<&str>, agent_id: Option<&str>) -> Result<()> {
        let name = name.to_owned();
        let user_id = user_id.map(str::to_owned);
        let agent_id = agent_id.map(str::to_owned);
        self.dispatch("delete_task_stream", move |inner| async move {
            inner.delete_task_stream(&name, user_id.as_deref(), agent_id.as_deref()).await
        }).await
    }

    async fn archive_task_stream(&self, name: &str, user_id: Option<&str>, agent_id: Option<&str>) -> Result<TaskStream> {
        let name = name.to_owned();
        let user_id = user_id.map(str::to_owned);
        let agent_id = agent_id.map(str::to_owned);
        self.dispatch("archive_task_stream", move |inner| async move {
            inner.archive_task_stream(&name, user_id.as_deref(), agent_id.as_deref()).await
        }).await
    }

    async fn pause_task_stream(&self, name: &str, user_id: Option<&str>, agent_id: Option<&str>) -> Result<TaskStream> {
        let name = name.to_owned();
        let user_id = user_id.map(str::to_owned);
        let agent_id = agent_id.map(str::to_owned);
        self.dispatch("pause_task_stream", move |inner| async move {
            inner.pause_task_stream(&name, user_id.as_deref(), agent_id.as_deref()).await
        }).await
    }

    async fn add_task_step(&self, stream_name: &str, user_id: Option<&str>, agent_id: Option<&str>, step: TaskStep) -> Result<TaskStep> {
        let stream_name = stream_name.to_owned();
        let user_id = user_id.map(str::to_owned);
        let agent_id = agent_id.map(str::to_owned);
        self.dispatch("add_task_step", move |inner| async move {
            inner.add_task_step(&stream_name, user_id.as_deref(), agent_id.as_deref(), step).await
        }).await
    }

    async fn update_task_step_status(&self, idempotency_key: &str, status: TaskStepStatus, result: Option<String>, error: Option<String>) -> Result<TaskStep> {
        let idempotency_key = idempotency_key.to_owned();
        self.dispatch("update_task_step_status", move |inner| async move {
            inner.update_task_step_status(&idempotency_key, status, result, error).await
        }).await
    }

    async fn get_task_steps(&self, stream_name: &str, user_id: Option<&str>, agent_id: Option<&str>) -> Result<Vec<TaskStep>> {
        let stream_name = stream_name.to_owned();
        let user_id = user_id.map(str::to_owned);
        let agent_id = agent_id.map(str::to_owned);
        self.dispatch("get_task_steps", move |inner| async move {
            inner.get_task_steps(&stream_name, user_id.as_deref(), agent_id.as_deref()).await
        }).await
    }

    async fn get_current_step(&self, stream_name: &str, user_id: Option<&str>, agent_id: Option<&str>) -> Result<Option<TaskStep>> {
        let stream_name = stream_name.to_owned();
        let user_id = user_id.map(str::to_owned);
        let agent_id = agent_id.map(str::to_owned);
        self.dispatch("get_current_step", move |inner| async move {
            inner.get_current_step(&stream_name, user_id.as_deref(), agent_id.as_deref()).await
        }).await
    }

    async fn complete_step(&self, idempotency_key: &str, result: Option<String>) -> Result<TaskStep> {
        let idempotency_key = idempotency_key.to_owned();
        self.dispatch("complete_step", move |inner| async move {
            inner.complete_step(&idempotency_key, result).await
        }).await
    }

    async fn create_mindmap(&self, mindmap: MindMap) -> Result<MindMap> {
        self.dispatch("create_mindmap", move |inner| async move {
            inner.create_mindmap(mindmap).await
        }).await
    }

    async fn get_mindmap(&self, name: &str, user_id: Option<&str>, agent_id: Option<&str>) -> Result<Option<MindMap>> {
        let name = name.to_owned();
        let user_id = user_id.map(str::to_owned);
        let agent_id = agent_id.map(str::to_owned);
        self.dispatch("get_mindmap", move |inner| async move {
            inner.get_mindmap(&name, user_id.as_deref(), agent_id.as_deref()).await
        }).await
    }

    async fn add_mindmap_node(&self, mindmap_name: &str, user_id: Option<&str>, agent_id: Option<&str>, node: MindMapNode) -> Result<MindMap> {
        let mindmap_name = mindmap_name.to_owned();
        let user_id = user_id.map(str::to_owned);
        let agent_id = agent_id.map(str::to_owned);
        self.dispatch("add_mindmap_node", move |inner| async move {
            inner.add_mindmap_node(&mindmap_name, user_id.as_deref(), agent_id.as_deref(), node).await
        }).await
    }

    async fn add_mindmap_edge(&self, mindmap_name: &str, user_id: Option<&str>, agent_id: Option<&str>, edge: MindMapEdge) -> Result<MindMap> {
        let mindmap_name = mindmap_name.to_owned();
        let user_id = user_id.map(str::to_owned);
        let agent_id = agent_id.map(str::to_owned);
        self.dispatch("add_mindmap_edge", move |inner| async move {
            inner.add_mindmap_edge(&mindmap_name, user_id.as_deref(), agent_id.as_deref(), edge).await
        }).await
    }

    async fn delete_mindmap_node(&self, mindmap_name: &str, user_id: Option<&str>, agent_id: Option<&str>, node_id: &str) -> Result<MindMap> {
        let mindmap_name = mindmap_name.to_owned();
        let user_id = user_id.map(str::to_owned);
        let agent_id = agent_id.map(str::to_owned);
        let node_id = node_id.to_owned();
        self.dispatch("delete_mindmap_node", move |inner| async move {
            inner.delete_mindmap_node(&mindmap_name, user_id.as_deref(), agent_id.as_deref(), &node_id).await
        }).await
    }

    async fn list_mindmaps(&self, user_id: Option<&str>, agent_id: Option<&str>) -> Result<Vec<MindMap>> {
        let user_id = user_id.map(str::to_owned);
        let agent_id = agent_id.map(str::to_owned);
        self.dispatch("list_mindmaps", move |inner| async move {
            inner.list_mindmaps(user_id.as_deref(), agent_id.as_deref()).await
        }).await
    }

    async fn delete_mindmap(&self, name: &str, user_id: Option<&str>, agent_id: Option<&str>) -> Result<()> {
        let name = name.to_owned();
        let user_id = user_id.map(str::to_owned);
        let agent_id = agent_id.map(str::to_owned);
        self.dispatch("delete_mindmap", move |inner| async move {
            inner.delete_mindmap(&name, user_id.as_deref(), agent_id.as_deref()).await
        }).await
    }

    async fn auto_summarize_task_stream(&self, stream_name: &str, user_id: Option<&str>, agent_id: Option<&str>, model_id: &str) -> Result<Option<Memory>> {
        let stream_name = stream_name.to_owned();
        let user_id = user_id.map(str::to_owned);
        let agent_id = agent_id.map(str::to_owned);
        let model_id = model_id.to_owned();
        self.dispatch("auto_summarize_task_stream", move |inner| async move {
            inner.auto_summarize_task_stream(&stream_name, user_id.as_deref(), agent_id.as_deref(), &model_id).await
        }).await
    }

    async fn try_update_persona_mindmap(&self, user_id: &str, memory: &Memory) -> Result<()> {
        let user_id = user_id.to_owned();
        let memory = memory.clone();
        self.dispatch("try_update_persona_mindmap", move |inner| async move {
            inner.try_update_persona_mindmap(&user_id, &memory).await
        }).await
    }

    async fn find_path(&self, from: &str, to: &str, max_depth: u8) -> Result<Vec<Vec<String>>> {
        let from = from.to_owned();
        let to = to.to_owned();
        self.dispatch("find_path", move |inner| async move {
            inner.find_path(&from, &to, max_depth).await
        }).await
    }

    async fn expand_neighbors(&self, entity_name: &str, depth: u8, limit: usize) -> Result<KnowledgeGraph> {
        let entity_name = entity_name.to_owned();
        self.dispatch("expand_neighbors", move |inner| async move {
            inner.expand_neighbors(&entity_name, depth, limit).await
        }).await
    }

    async fn get_related(&self, entity_name: &str, relation_type: Option<&str>, direction: &str, limit: usize) -> Result<Vec<Entity>> {
        let entity_name = entity_name.to_owned();
        let relation_type = relation_type.map(str::to_owned);
        let direction = direction.to_owned();
        self.dispatch("get_related", move |inner| async move {
            inner.get_related(&entity_name, relation_type.as_deref(), &direction, limit).await
        }).await
    }

    async fn get_entity_history(&self, name: &str) -> Result<Vec<MemoryHistory>> {
        let name = name.to_owned();
        self.dispatch("get_entity_history", move |inner| async move {
            inner.get_entity_history(&name).await
        }).await
    }

    async fn get_graph_at_time(&self, before_rfc3339: &str) -> Result<KnowledgeGraph> {
        let before_rfc3339 = before_rfc3339.to_owned();
        self.dispatch("get_graph_at_time", move |inner| async move {
            inner.get_graph_at_time(&before_rfc3339).await
        }).await
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

