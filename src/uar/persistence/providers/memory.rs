//! Non-durable persistence for minimal deployments and deterministic tests.

use std::{collections::HashMap, sync::RwLock};

use anyhow::{Context, Result};
use async_trait::async_trait;
use uuid::Uuid;

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
            prompt_caching::UserPromptCachingSettings,
            skills::{Skill, SkillMatch},
        },
        persistence::PersistenceLayer,
        settings::schema::{Settings, SettingsType},
    },
};

/// Process-local persistence. All state is discarded on shutdown.
#[derive(Debug, Default)]
pub struct InMemoryProvider {
    sessions: RwLock<HashMap<String, Session>>,
    conversation_policies: RwLock<HashMap<String, ConversationPolicyRecord>>,
    user_prompt_caching_settings: RwLock<HashMap<String, UserPromptCachingSettings>>,
    skills: RwLock<HashMap<String, Skill>>,
    knowledge_bases: RwLock<HashMap<String, KnowledgeBase>>,
    chunks: RwLock<HashMap<String, KnowledgeChunk>>,
    documents: RwLock<HashMap<String, KnowledgeDocument>>,
    agents: RwLock<HashMap<String, AgentArtifact>>,
    memories: RwLock<Vec<Memory>>,
    /// Registered settings types keyed by their slug (e.g. `run_policy`).
    settings_types: RwLock<HashMap<String, SettingsType>>,
    /// Setting values keyed by their dotted key (e.g. `run_policy.global`).
    settings: RwLock<HashMap<String, Settings>>,
}

impl InMemoryProvider {
    /// Create an empty, non-durable provider.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

/// Validate a setting's data against a JSON Schema, mirroring the durable
/// providers so the in-memory provider honors the same write contract.
fn validate_setting_against_schema(
    data: &serde_json::Value,
    schema: &serde_json::Value,
    setting_key: &str,
) -> Result<()> {
    let validator = jsonschema::validator_for(schema)
        .map_err(|e| anyhow::anyhow!("Invalid JSON Schema for setting '{setting_key}': {e}"))?;
    let errors: Vec<String> = validator.iter_errors(data).map(|e| e.to_string()).collect();
    if !errors.is_empty() {
        return Err(anyhow::anyhow!(
            "Setting '{}' data failed JSON Schema validation:\n{}",
            setting_key,
            errors.join("\n")
        ));
    }
    Ok(())
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
        let key = crate::uar::persistence::tenant_storage_key(session.owner_id(), session.id());
        write(&self.sessions)?.insert(key, session.clone());
        Ok(())
    }
    async fn load_session(&self, owner_id: &str, id: &str) -> Result<Option<Session>> {
        let key = crate::uar::persistence::tenant_storage_key(owner_id, id);
        Ok(read(&self.sessions)?.get(&key).cloned())
    }
    async fn save_conversation_policy(&self, record: &ConversationPolicyRecord) -> Result<()> {
        let key =
            crate::uar::persistence::tenant_storage_key(&record.owner_id, &record.conversation_id);
        write(&self.conversation_policies)?.insert(key, record.clone());
        Ok(())
    }
    async fn load_conversation_policy(
        &self,
        owner_id: &str,
        conversation_id: &str,
    ) -> Result<Option<ConversationPolicyRecord>> {
        let key = crate::uar::persistence::tenant_storage_key(owner_id, conversation_id);
        Ok(read(&self.conversation_policies)?.get(&key).cloned())
    }
    async fn delete_conversation_policy(
        &self,
        owner_id: &str,
        conversation_id: &str,
    ) -> Result<()> {
        let key = crate::uar::persistence::tenant_storage_key(owner_id, conversation_id);
        write(&self.conversation_policies)?.remove(&key);
        Ok(())
    }
    async fn save_user_prompt_caching_settings(
        &self,
        settings: &UserPromptCachingSettings,
    ) -> Result<()> {
        write(&self.user_prompt_caching_settings)?
            .insert(settings.user_id.clone(), settings.clone());
        Ok(())
    }
    async fn load_user_prompt_caching_settings(
        &self,
        principal_id: &str,
    ) -> Result<Option<UserPromptCachingSettings>> {
        Ok(read(&self.user_prompt_caching_settings)?
            .get(principal_id)
            .cloned())
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
        let key = crate::uar::persistence::tenant_storage_key(&kb.owner_id, &kb.id);
        write(&self.knowledge_bases)?.insert(key, kb.clone());
        Ok(())
    }
    async fn get_knowledge_base(&self, owner_id: &str, id: &str) -> Result<Option<KnowledgeBase>> {
        let key = crate::uar::persistence::tenant_storage_key(owner_id, id);
        Ok(read(&self.knowledge_bases)?.get(&key).cloned())
    }
    async fn get_knowledge_base_by_name(
        &self,
        owner_id: &str,
        name: &str,
    ) -> Result<Option<KnowledgeBase>> {
        Ok(read(&self.knowledge_bases)?
            .values()
            .find(|kb| kb.owner_id == owner_id && kb.name == name)
            .cloned())
    }
    async fn list_knowledge_bases(&self, owner_id: &str) -> Result<Vec<KnowledgeBase>> {
        Ok(read(&self.knowledge_bases)?
            .values()
            .filter(|kb| kb.owner_id == owner_id)
            .cloned()
            .collect())
    }
    async fn delete_knowledge_base(&self, owner_id: &str, id: &str) -> Result<()> {
        let key = crate::uar::persistence::tenant_storage_key(owner_id, id);
        write(&self.knowledge_bases)?.remove(&key);
        write(&self.chunks)?.retain(|_, chunk| chunk.owner_id != owner_id || chunk.kb_id != id);
        write(&self.documents)?
            .retain(|_, document| document.owner_id != owner_id || document.kb_id != id);
        Ok(())
    }
    async fn save_chunk(&self, chunk: &KnowledgeChunk) -> Result<()> {
        let key =
            crate::uar::persistence::tenant_storage_key(&chunk.owner_id, &chunk.id.to_string());
        write(&self.chunks)?.insert(key, chunk.clone());
        Ok(())
    }
    async fn search_knowledge(
        &self,
        owner_id: &str,
        _query_vec: &[f32],
        limit: usize,
        min_score: f32,
    ) -> Result<Vec<KnowledgeMatch>> {
        self.search_knowledge_scoped(owner_id, &[], _query_vec, limit, min_score)
            .await
    }
    async fn search_knowledge_scoped(
        &self,
        owner_id: &str,
        kb_ids: &[&str],
        _query_vec: &[f32],
        limit: usize,
        _min_score: f32,
    ) -> Result<Vec<KnowledgeMatch>> {
        Ok(read(&self.chunks)?
            .values()
            .filter(|chunk| {
                chunk.owner_id == owner_id
                    && (kb_ids.is_empty() || kb_ids.contains(&chunk.kb_id.as_str()))
            })
            .take(limit)
            .cloned()
            .map(|chunk| KnowledgeMatch { chunk, score: 0.0 })
            .collect())
    }
    async fn save_document(&self, doc: &KnowledgeDocument) -> Result<()> {
        let key = crate::uar::persistence::tenant_storage_key(&doc.owner_id, &doc.id);
        write(&self.documents)?.insert(key, doc.clone());
        Ok(())
    }
    async fn get_document(&self, owner_id: &str, id: &str) -> Result<Option<KnowledgeDocument>> {
        let key = crate::uar::persistence::tenant_storage_key(owner_id, id);
        Ok(read(&self.documents)?.get(&key).cloned())
    }
    async fn list_documents(&self, owner_id: &str, kb_id: &str) -> Result<Vec<KnowledgeDocument>> {
        Ok(read(&self.documents)?
            .values()
            .filter(|doc| doc.owner_id == owner_id && doc.kb_id == kb_id)
            .cloned()
            .collect())
    }
    async fn update_document_status(
        &self,
        owner_id: &str,
        doc_id: &str,
        status: &DocumentStatus,
    ) -> Result<()> {
        let key = crate::uar::persistence::tenant_storage_key(owner_id, doc_id);
        write(&self.documents)?
            .get_mut(&key)
            .context("document not found")?
            .status = status.clone();
        Ok(())
    }
    async fn delete_document(&self, owner_id: &str, doc_id: &str) -> Result<()> {
        let key = crate::uar::persistence::tenant_storage_key(owner_id, doc_id);
        write(&self.documents)?.remove(&key);
        write(&self.chunks)?.retain(|_, chunk| {
            chunk.owner_id != owner_id || chunk.document_id.as_deref() != Some(doc_id)
        });
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

    // -------------------------------------------------------------------------
    // Settings storage
    //
    // The default trait methods are no-ops; the in-memory provider implements
    // real storage so the embedded admin surface (typed settings get/set) and
    // the Global run-policy scope work against in-process persistence. On a key
    // conflict the original type's id is preserved, matching the durable
    // providers' `ON CONFLICT` semantics.
    // -------------------------------------------------------------------------
    async fn upsert_settings_type(&self, st: &SettingsType) -> Result<Uuid> {
        let mut types = write(&self.settings_types)?;
        let actual_id = types.get(&st.key).map_or(st.id, |existing| existing.id);
        let stored = SettingsType {
            id: actual_id,
            ..st.clone()
        };
        types.insert(st.key.clone(), stored);
        Ok(actual_id)
    }

    async fn list_settings_types(&self) -> Result<Vec<SettingsType>> {
        Ok(read(&self.settings_types)?.values().cloned().collect())
    }

    async fn get_settings_type(&self, key: &str) -> Result<Option<SettingsType>> {
        Ok(read(&self.settings_types)?.get(key).cloned())
    }

    async fn upsert_setting(&self, setting: &Settings) -> Result<()> {
        // Validate against the leaf property schema extracted from the parent
        // type, mirroring the durable providers. Keys are dotted: `run_policy.global`
        // → type key `run_policy`, leaf key `global`. Validate only when a matching
        // leaf schema exists so namespace-level schemas without a 1:1 property map
        // do not cause false failures.
        let type_key = setting.key.split('.').next().unwrap_or("unknown");
        let leaf_key = setting.key.splitn(2, '.').nth(1).unwrap_or(&setting.key);
        if let Some(st) = read(&self.settings_types)?.get(type_key).cloned()
            && let Some(leaf_schema) = st
                .schema
                .get("properties")
                .and_then(|props| props.get(leaf_key))
                .or_else(|| st.schema.get("additionalProperties"))
        {
            validate_setting_against_schema(&setting.data, leaf_schema, &setting.key)?;
        }
        write(&self.settings)?.insert(setting.key.clone(), setting.clone());
        Ok(())
    }

    async fn get_setting(&self, key: &str) -> Result<Option<Settings>> {
        Ok(read(&self.settings)?.get(key).cloned())
    }

    async fn list_settings(
        &self,
        type_key: Option<&str>,
        parent_id: Option<Uuid>,
    ) -> Result<Vec<Settings>> {
        Ok(read(&self.settings)?
            .values()
            .filter(|s| {
                let type_ok = type_key.is_none_or(|tk| s.key.starts_with(&format!("{tk}.")));
                let parent_ok = parent_id.is_none_or(|pid| s.parent_id == Some(pid));
                type_ok && parent_ok
            })
            .cloned()
            .collect())
    }

    async fn delete_setting(&self, key: &str) -> Result<()> {
        write(&self.settings)?.remove(key);
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
        let session = crate::session::SessionStore::new().create_for_user("alice");
        provider.save_session(&session).await.unwrap();
        assert_eq!(
            provider
                .load_session("alice", session.id())
                .await
                .unwrap()
                .unwrap()
                .id(),
            session.id()
        );
        assert!(
            provider
                .load_session("bob", session.id())
                .await
                .unwrap()
                .is_none()
        );
    }

    #[tokio::test]
    async fn conversation_policy_round_trips_without_durable_storage() {
        let provider = InMemoryProvider::new();
        let record = ConversationPolicyRecord::new_for_user(
            "alice",
            "conversation-1",
            crate::uar::domain::policy::RunPolicy {
                memory_enabled: Some(false),
                ..crate::uar::domain::policy::RunPolicy::default()
            },
        );
        provider.save_conversation_policy(&record).await.unwrap();
        assert_eq!(
            provider
                .load_conversation_policy("alice", "conversation-1")
                .await
                .unwrap(),
            Some(record)
        );
        assert!(
            provider
                .load_conversation_policy("bob", "conversation-1")
                .await
                .unwrap()
                .is_none()
        );
        provider
            .delete_conversation_policy("bob", "conversation-1")
            .await
            .unwrap();
        assert!(
            provider
                .load_conversation_policy("alice", "conversation-1")
                .await
                .unwrap()
                .is_some()
        );
        provider
            .delete_conversation_policy("alice", "conversation-1")
            .await
            .unwrap();
        assert!(
            provider
                .load_conversation_policy("alice", "conversation-1")
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
            owner_id: "anonymous".into(),
            name: "test".into(),
            description: None,
            config: Default::default(),
            created_at: String::new(),
            updated_at: String::new(),
        };
        provider.save_knowledge_base(&kb).await.unwrap();
        provider
            .delete_knowledge_base(&kb.owner_id, &kb.id)
            .await
            .unwrap();
        assert!(
            provider
                .get_knowledge_base(&kb.owner_id, &kb.id)
                .await
                .unwrap()
                .is_none()
        );
    }

    #[tokio::test]
    async fn knowledge_rows_are_partitioned_by_owner() {
        let provider = InMemoryProvider::new();
        let now = chrono::Utc::now().to_rfc3339();
        let kb = KnowledgeBase {
            id: "alice-kb".into(),
            owner_id: "alice".into(),
            name: "private".into(),
            description: None,
            config: Default::default(),
            created_at: now.clone(),
            updated_at: now.clone(),
        };
        let document = KnowledgeDocument {
            id: "alice-doc".into(),
            owner_id: "alice".into(),
            kb_id: kb.id.clone(),
            filename: "private.txt".into(),
            file_path: None,
            mime_type: Some("text/plain".into()),
            chunk_count: 1,
            status: DocumentStatus::Indexed,
            created_at: now.clone(),
            updated_at: now.clone(),
        };
        let chunk = KnowledgeChunk {
            id: Uuid::new_v4(),
            owner_id: "alice".into(),
            kb_id: kb.id.clone(),
            document_id: Some(document.id.clone()),
            content: "alice secret".into(),
            metadata: None,
            embedding: vec![1.0],
            created_at: now,
        };

        provider.save_knowledge_base(&kb).await.unwrap();
        provider.save_document(&document).await.unwrap();
        provider.save_chunk(&chunk).await.unwrap();

        assert!(
            provider
                .get_knowledge_base("bob", &kb.id)
                .await
                .unwrap()
                .is_none()
        );
        assert!(
            provider
                .get_document("bob", &document.id)
                .await
                .unwrap()
                .is_none()
        );
        assert!(
            provider
                .search_knowledge_scoped("bob", &[&kb.id], &[1.0], 10, 0.0)
                .await
                .unwrap()
                .is_empty()
        );

        provider.delete_knowledge_base("bob", &kb.id).await.unwrap();
        assert!(
            provider
                .get_knowledge_base("alice", &kb.id)
                .await
                .unwrap()
                .is_some()
        );
        assert!(
            provider
                .get_document("alice", &document.id)
                .await
                .unwrap()
                .is_some()
        );
    }
}
