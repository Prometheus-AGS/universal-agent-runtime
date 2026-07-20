//! Non-durable persistence for minimal deployments and deterministic tests.

use std::{collections::HashMap, sync::RwLock};

use anyhow::{Context, Result};
use async_trait::async_trait;

use crate::{
    session::Session,
    uar::{
        domain::{
            artifact::AgentArtifact,
            knowledge::{
                DocumentStatus, KnowledgeBase, KnowledgeChunk, KnowledgeDocument, KnowledgeMatch,
            },
            memory::{Memory, MemoryMatch},
            policy::ConversationPolicyRecord,
            skills::{Skill, SkillMatch},
        },
        persistence::PersistenceLayer,
    },
};

/// Process-local persistence. All state is discarded on shutdown.
#[derive(Debug, Default)]
pub struct InMemoryProvider {
    sessions: RwLock<HashMap<String, Session>>,
    conversation_policies: RwLock<HashMap<String, ConversationPolicyRecord>>,
    skills: RwLock<HashMap<String, Skill>>,
    knowledge_bases: RwLock<HashMap<String, KnowledgeBase>>,
    chunks: RwLock<HashMap<String, KnowledgeChunk>>,
    documents: RwLock<HashMap<String, KnowledgeDocument>>,
    agents: RwLock<HashMap<String, AgentArtifact>>,
    memories: RwLock<Vec<Memory>>,
}

impl InMemoryProvider {
    /// Create an empty, non-durable provider.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

fn read<T>(lock: &RwLock<T>) -> Result<std::sync::RwLockReadGuard<'_, T>> {
    lock.read()
        .map_err(|_| anyhow::anyhow!("in-memory persistence read lock poisoned"))
}

fn write<T>(lock: &RwLock<T>) -> Result<std::sync::RwLockWriteGuard<'_, T>> {
    lock.write()
        .map_err(|_| anyhow::anyhow!("in-memory persistence write lock poisoned"))
}

#[async_trait]
impl PersistenceLayer for InMemoryProvider {
    async fn save_session(&self, session: &Session) -> Result<()> {
        write(&self.sessions)?.insert(session.id().to_owned(), session.clone());
        Ok(())
    }
    async fn load_session(&self, id: &str) -> Result<Option<Session>> {
        Ok(read(&self.sessions)?.get(id).cloned())
    }
    async fn save_conversation_policy(&self, record: &ConversationPolicyRecord) -> Result<()> {
        write(&self.conversation_policies)?.insert(record.conversation_id.clone(), record.clone());
        Ok(())
    }
    async fn load_conversation_policy(
        &self,
        conversation_id: &str,
    ) -> Result<Option<ConversationPolicyRecord>> {
        Ok(read(&self.conversation_policies)?
            .get(conversation_id)
            .cloned())
    }
    async fn delete_conversation_policy(&self, conversation_id: &str) -> Result<()> {
        write(&self.conversation_policies)?.remove(conversation_id);
        Ok(())
    }
    async fn save_skill(&self, skill: &Skill, _embedding: &[f32]) -> Result<()> {
        write(&self.skills)?.insert(skill.skill_id.clone(), skill.clone());
        Ok(())
    }
    async fn search_skills(&self, _query_vec: &[f32], _limit: usize) -> Result<Vec<SkillMatch>> {
        Ok(Vec::new())
    }
    async fn list_skills(&self) -> Result<Vec<Skill>> {
        Ok(read(&self.skills)?.values().cloned().collect())
    }
    async fn delete_skill(&self, id: &str) -> Result<()> {
        write(&self.skills)?.remove(id);
        Ok(())
    }
    async fn save_knowledge_base(&self, kb: &KnowledgeBase) -> Result<()> {
        write(&self.knowledge_bases)?.insert(kb.id.clone(), kb.clone());
        Ok(())
    }
    async fn get_knowledge_base(&self, id: &str) -> Result<Option<KnowledgeBase>> {
        Ok(read(&self.knowledge_bases)?.get(id).cloned())
    }
    async fn get_knowledge_base_by_name(&self, name: &str) -> Result<Option<KnowledgeBase>> {
        Ok(read(&self.knowledge_bases)?
            .values()
            .find(|kb| kb.name == name)
            .cloned())
    }
    async fn list_knowledge_bases(&self) -> Result<Vec<KnowledgeBase>> {
        Ok(read(&self.knowledge_bases)?.values().cloned().collect())
    }
    async fn delete_knowledge_base(&self, id: &str) -> Result<()> {
        write(&self.knowledge_bases)?.remove(id);
        write(&self.chunks)?.retain(|_, chunk| chunk.kb_id != id);
        write(&self.documents)?.retain(|_, document| document.kb_id != id);
        Ok(())
    }
    async fn save_chunk(&self, chunk: &KnowledgeChunk) -> Result<()> {
        write(&self.chunks)?.insert(chunk.id.to_string(), chunk.clone());
        Ok(())
    }
    async fn search_knowledge(
        &self,
        _query_vec: &[f32],
        limit: usize,
        min_score: f32,
    ) -> Result<Vec<KnowledgeMatch>> {
        self.search_knowledge_scoped(&[], _query_vec, limit, min_score)
            .await
    }
    async fn search_knowledge_scoped(
        &self,
        kb_ids: &[&str],
        _query_vec: &[f32],
        limit: usize,
        _min_score: f32,
    ) -> Result<Vec<KnowledgeMatch>> {
        Ok(read(&self.chunks)?
            .values()
            .filter(|chunk| kb_ids.is_empty() || kb_ids.contains(&chunk.kb_id.as_str()))
            .take(limit)
            .cloned()
            .map(|chunk| KnowledgeMatch { chunk, score: 0.0 })
            .collect())
    }
    async fn save_document(&self, doc: &KnowledgeDocument) -> Result<()> {
        write(&self.documents)?.insert(doc.id.clone(), doc.clone());
        Ok(())
    }
    async fn get_document(&self, id: &str) -> Result<Option<KnowledgeDocument>> {
        Ok(read(&self.documents)?.get(id).cloned())
    }
    async fn list_documents(&self, kb_id: &str) -> Result<Vec<KnowledgeDocument>> {
        Ok(read(&self.documents)?
            .values()
            .filter(|doc| doc.kb_id == kb_id)
            .cloned()
            .collect())
    }
    async fn update_document_status(&self, doc_id: &str, status: &DocumentStatus) -> Result<()> {
        write(&self.documents)?
            .get_mut(doc_id)
            .context("document not found")?
            .status = status.clone();
        Ok(())
    }
    async fn delete_document(&self, doc_id: &str) -> Result<()> {
        write(&self.documents)?.remove(doc_id);
        write(&self.chunks)?.retain(|_, chunk| chunk.document_id.as_deref() != Some(doc_id));
        Ok(())
    }
    async fn save_agent(&self, agent: &AgentArtifact) -> Result<()> {
        write(&self.agents)?.insert(agent.id.clone(), agent.clone());
        Ok(())
    }
    async fn load_agent(&self, id: &str) -> Result<Option<AgentArtifact>> {
        Ok(read(&self.agents)?.get(id).cloned())
    }
    async fn load_agent_by_name(&self, name: &str) -> Result<Option<AgentArtifact>> {
        Ok(read(&self.agents)?
            .values()
            .find(|agent| agent.metadata.title == name)
            .cloned())
    }
    async fn list_agents(&self) -> Result<Vec<AgentArtifact>> {
        Ok(read(&self.agents)?.values().cloned().collect())
    }
    async fn delete_agent(&self, id: &str) -> Result<()> {
        write(&self.agents)?.remove(id);
        Ok(())
    }
    async fn save_memory(&self, memory: &Memory) -> Result<()> {
        write(&self.memories)?.push(memory.clone());
        Ok(())
    }
    async fn search_memory(
        &self,
        _agent_id: Option<&str>,
        _query_vec: &[f32],
        _limit: usize,
        _min_score: f32,
    ) -> Result<Vec<MemoryMatch>> {
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn sessions_round_trip_without_durable_storage() {
        let provider = InMemoryProvider::new();
        let session = crate::session::SessionStore::new().create();
        provider.save_session(&session).await.unwrap();
        assert_eq!(
            provider
                .load_session(session.id())
                .await
                .unwrap()
                .unwrap()
                .id(),
            session.id()
        );
    }

    #[tokio::test]
    async fn conversation_policy_round_trips_without_durable_storage() {
        let provider = InMemoryProvider::new();
        let record = ConversationPolicyRecord::new(
            "conversation-1",
            crate::uar::domain::policy::RunPolicy {
                memory_enabled: Some(false),
                ..crate::uar::domain::policy::RunPolicy::default()
            },
        );
        provider.save_conversation_policy(&record).await.unwrap();
        assert_eq!(
            provider
                .load_conversation_policy("conversation-1")
                .await
                .unwrap(),
            Some(record)
        );
        provider
            .delete_conversation_policy("conversation-1")
            .await
            .unwrap();
        assert!(
            provider
                .load_conversation_policy("conversation-1")
                .await
                .unwrap()
                .is_none()
        );
    }

    #[tokio::test]
    async fn deleting_a_knowledge_base_removes_owned_rows() {
        let provider = InMemoryProvider::new();
        let kb = KnowledgeBase {
            id: "kb-1".into(),
            name: "test".into(),
            description: None,
            config: Default::default(),
            created_at: String::new(),
            updated_at: String::new(),
        };
        provider.save_knowledge_base(&kb).await.unwrap();
        provider.delete_knowledge_base(&kb.id).await.unwrap();
        assert!(provider.get_knowledge_base(&kb.id).await.unwrap().is_none());
    }
}
