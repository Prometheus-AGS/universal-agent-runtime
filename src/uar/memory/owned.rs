//! Host-bound memory access. Payload fields can narrow a query, never grant ownership.
use super::controls::{CaptureAdmission, CapturePolicy, CapturePolicyUpdate};
use super::service::MemoryService;
use crate::uar::runtime::lifecycle::OperationLease;
use crate::uar::security::claims::CredentialPrincipal;
use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use surreal_memory::{
    Memory, MemoryHistory, MemoryScope, MemoryType,
    task_stream::{ContextWindow, TaskStream},
};

pub struct OwnedMemory<'a> {
    pub(super) service: &'a MemoryService,
    pub(super) principal: CredentialPrincipal,
    pub(super) _lease: OperationLease,
}

impl std::fmt::Debug for OwnedMemory<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OwnedMemory").finish_non_exhaustive()
    }
}

impl OwnedMemory<'_> {
    pub fn owner_key(&self) -> String {
        self.principal.storage_key()
    }

    pub async fn capture_policy(&self) -> Result<CapturePolicy> {
        self.service.owner_capture_policy(&self.owner_key()).await
    }
    pub async fn set_capture_policy(&self, update: CapturePolicyUpdate) -> Result<CapturePolicy> {
        self.service
            .owner_set_capture_policy(&self.owner_key(), update)
            .await
    }

    pub fn check_filter(&self, requested: Option<&str>) -> Result<()> {
        anyhow::ensure!(
            requested.is_none_or(|value| value == self.principal.subject()),
            "Memory owner cannot be replaced by request arguments"
        );
        Ok(())
    }

    fn validate_id(id: &str) -> Result<()> {
        anyhow::ensure!(
            !id.trim().is_empty()
                && id
                    .split_once(':')
                    .is_none_or(|(table, _)| table == "memory"),
            "Memory ID must address a memory record"
        );
        Ok(())
    }

    pub async fn add_memory(&self, mut memory: Memory) -> Result<Memory> {
        self.check_filter(memory.user_id.as_deref())?;
        anyhow::ensure!(
            memory.id.is_none() && memory.task_stream_id.is_none(),
            "Memory record identity is host-owned"
        );
        anyhow::ensure!(
            memory
                .metadata
                .as_ref()
                .is_none_or(
                    |value| value.as_object().is_none_or(|object| !object
                        .keys()
                        .any(|key| key.starts_with("uar_owner")
                            || key.starts_with("uar_migration")
                            || key.starts_with("uar_capture")))
                ),
            "Memory ownership metadata is host-owned"
        );
        // Global is a classification inside this principal's partition. Only the
        // explicit trusted-host administrative path may create a shared record.
        memory.user_id = Some(self.owner_key());
        self.service
            .save_controlled_memory(memory, None, None)
            .await?
            .context("Memory unavailable after save")
    }

    pub async fn get_memory(&self, id: &str) -> Result<Option<Memory>> {
        Self::validate_id(id)?;
        let owner = self.owner_key();
        match self
            .service
            .storage
            .get_memory(id)
            .await?
            .filter(|memory| memory.user_id.as_deref() == Some(owner.as_str()))
        {
            Some(memory) => self.service.visible_memory(memory).await,
            None => Ok(None),
        }
    }

    pub async fn update_memory(&self, id: &str, content: String) -> Result<Memory> {
        self.get_memory(id)
            .await?
            .context("Memory is unavailable to this owner")?;
        // Ownership is immutable: no ordinary service method can reassign a row.
        self.service
            .update_controlled_memory(id, &self.owner_key(), content)
            .await
    }

    pub async fn delete_memory(&self, id: &str) -> Result<()> {
        Self::validate_id(id)?;
        self.service
            .delete_controlled_memory(id, &self.owner_key())
            .await
    }

    pub async fn get_memory_history(&self, id: &str) -> Result<Vec<MemoryHistory>> {
        let memory = self
            .get_memory(id)
            .await?
            .context("Memory is unavailable to this owner")?;
        let mut history: Vec<MemoryHistory> = match memory
            .metadata
            .as_ref()
            .and_then(|value| value.get("uar_migration"))
            .and_then(|value| value.get("history"))
        {
            Some(value) => serde_json::from_value(value.clone())
                .context("Invalid preserved migration history")?,
            None => Vec::new(),
        };
        history.extend(self.service.storage.get_memory_history(id).await?);
        if self.get_memory(id).await?.is_none() {
            return Ok(Vec::new());
        }
        Ok(history)
    }

    pub async fn get_all_memories(
        &self,
        user: Option<&str>,
        agent: Option<&str>,
        session: Option<&str>,
    ) -> Result<Vec<Memory>> {
        self.check_filter(user)?;
        let records = self
            .service
            .storage
            .get_all_memories(Some(&self.owner_key()), agent, session)
            .await?;
        self.service.visible_memories(records).await
    }

    pub async fn search_memories(
        &self,
        query: &str,
        user: Option<&str>,
        agent: Option<&str>,
        session: Option<&str>,
        categories: Option<&[String]>,
        limit: usize,
    ) -> Result<Vec<Memory>> {
        self.check_filter(user)?;
        let records = self
            .service
            .storage
            .search_memories(
                query,
                Some(&self.owner_key()),
                agent,
                session,
                categories,
                limit,
            )
            .await?;
        self.service.visible_memories(records).await
    }

    pub async fn hybrid_search_memories(
        &self,
        query: &str,
        user: Option<&str>,
        agent: Option<&str>,
        session: Option<&str>,
        limit: usize,
        vector_weight: f32,
        bm25_weight: f32,
    ) -> Result<Vec<Memory>> {
        self.check_filter(user)?;
        let records = self
            .service
            .storage
            .hybrid_search_memories(
                query,
                Some(&self.owner_key()),
                agent,
                session,
                limit,
                vector_weight,
                bm25_weight,
            )
            .await?;
        self.service.visible_memories(records).await
    }

    /// Bounded automatic-context waterfall. Generic tool retrieval keeps its
    /// exact caller filters; only this host path joins applicable scope lanes.
    pub(crate) async fn injection_candidates(
        &self,
        query: &str,
        agent: Option<&str>,
        session: Option<&str>,
    ) -> Result<Vec<Memory>> {
        const LIMIT: usize = 20;
        let config = self.service.config();
        let mut lanes = Vec::with_capacity(3);
        if let Some(session_id) = session {
            let records = self
                .hybrid_search_memories(
                    query,
                    None,
                    None,
                    Some(session_id),
                    LIMIT,
                    config.vector_weight,
                    config.bm25_weight,
                )
                .await?;
            lanes.push(
                records
                    .into_iter()
                    .filter(|memory| {
                        memory.scope == MemoryScope::Session
                            && injection_scope_visible(memory, agent, session)
                    })
                    .collect::<Vec<_>>(),
            );
        }
        if let Some(agent_id) = agent {
            let records = self
                .hybrid_search_memories(
                    query,
                    None,
                    Some(agent_id),
                    None,
                    LIMIT,
                    config.vector_weight,
                    config.bm25_weight,
                )
                .await?;
            lanes.push(
                records
                    .into_iter()
                    .filter(|memory| {
                        memory.scope == MemoryScope::Agent
                            && injection_scope_visible(memory, agent, session)
                    })
                    .collect::<Vec<_>>(),
            );
        }
        let records = self
            .hybrid_search_memories(
                query,
                None,
                None,
                None,
                LIMIT,
                config.vector_weight,
                config.bm25_weight,
            )
            .await?;
        lanes.push(
            records
                .into_iter()
                .filter(|memory| {
                    matches!(memory.scope, MemoryScope::User | MemoryScope::Global)
                        && injection_scope_visible(memory, agent, session)
                })
                .collect::<Vec<_>>(),
        );

        // Stable rank round-robin gives each retained scope lane a place without
        // inventing comparable scores from rank-only backend results. Preserve
        // per-lane hybrid order; ties use Session, Agent, owner lane order.
        let mut seen = std::collections::BTreeSet::new();
        let mut candidates = Vec::with_capacity(LIMIT);
        for rank in 0..LIMIT {
            for lane in &lanes {
                if let Some(memory) = lane.get(rank) {
                    let id = record_id(
                        memory
                            .id
                            .as_ref()
                            .context("Injection memory identity unavailable")?,
                    )?;
                    if seen.insert(id) {
                        candidates.push(memory.clone());
                        if candidates.len() == LIMIT {
                            return Ok(candidates);
                        }
                    }
                }
            }
        }
        Ok(candidates)
    }

    /// Apply an explicit relevance boundary to already ranked owner-visible rows.
    /// Re-embedding both sides uses the same retained configured engine, avoiding
    /// comparison against a historical row vector from another model revision.
    #[cfg(feature = "surreal-backend")]
    pub(crate) async fn injection_eligible(
        &self,
        query: &str,
        candidates: Vec<Memory>,
        minimum: f32,
    ) -> Result<Vec<Memory>> {
        anyhow::ensure!(
            minimum.is_finite() && (-1.0..=1.0).contains(&minimum),
            "Invalid injection cutoff"
        );
        anyhow::ensure!(candidates.len() <= 20, "Injection candidate bound exceeded");
        let mut visible = Vec::with_capacity(candidates.len());
        for candidate in candidates {
            let id = candidate
                .id
                .as_ref()
                .context("Injection memory identity unavailable")?;
            if let Some(current) = self.get_memory(&record_id(id)?).await? {
                // Revisions/content changed during retrieval cannot inherit an
                // earlier ranking or include a superseded private fact.
                if current.version == candidate.version && current.content == candidate.content {
                    visible.push(current);
                }
            }
        }
        if visible.is_empty() {
            return Ok(visible);
        }
        let engine = &self.service.embeddings;
        let query_vector = engine.embed(query).await?;
        anyhow::ensure!(
            query_vector.len() == engine.dimensions(),
            "Injection embedding dimension unavailable"
        );
        let vectors = engine
            .embed_batch(
                visible
                    .iter()
                    .map(|memory| memory.content.clone())
                    .collect(),
            )
            .await?;
        anyhow::ensure!(
            vectors.len() == visible.len(),
            "Injection embedding count unavailable"
        );
        let mut eligible = Vec::with_capacity(visible.len());
        for (memory, vector) in visible.into_iter().zip(vectors) {
            if injection_cosine(&query_vector, &vector)? >= minimum {
                // Deletion/expiry/owner filtering is repeated after inference.
                let id = memory
                    .id
                    .as_ref()
                    .context("Injection memory identity unavailable")?;
                if let Some(current) = self.get_memory(&record_id(id)?).await?
                    && current.version == memory.version
                    && current.content == memory.content
                {
                    eligible.push(current);
                }
            }
        }
        Ok(eligible)
    }

    #[cfg(not(feature = "surreal-backend"))]
    pub(crate) async fn injection_eligible(
        &self,
        _query: &str,
        _candidates: Vec<Memory>,
        _minimum: f32,
    ) -> Result<Vec<Memory>> {
        anyhow::bail!("Automatic memory relevance requires surreal-backend")
    }

    pub async fn delete_all_memories(
        &self,
        user: Option<&str>,
        agent: Option<&str>,
        session: Option<&str>,
    ) -> Result<u64> {
        self.check_filter(user)?;
        let records = self
            .service
            .storage
            .get_all_memories(Some(&self.owner_key()), agent, session)
            .await?;
        let count = records.len() as u64;
        for memory in records {
            if let Some(id) = memory.id {
                self.delete_memory(&record_id(&id)?).await?;
            }
        }
        Ok(count)
    }

    pub async fn add_memories_from_conversation(
        &self,
        messages: Vec<serde_json::Value>,
        user: Option<&str>,
        agent: Option<&str>,
        session: Option<&str>,
    ) -> Result<Vec<Memory>> {
        self.check_filter(user)?;
        let policy = self.capture_policy().await?;
        if !policy.enabled || !policy.host_capture_enabled {
            return Ok(Vec::new());
        }
        let admission = CaptureAdmission {
            owner: self.owner_key(),
            epoch: policy.consent_epoch,
        };
        self.capture_admitted(messages, agent, session, &admission)
            .await
    }

    pub(super) async fn capture_admitted(
        &self,
        messages: Vec<serde_json::Value>,
        agent: Option<&str>,
        session: Option<&str>,
        admission: &CaptureAdmission,
    ) -> Result<Vec<Memory>> {
        anyhow::ensure!(
            admission.owner == self.owner_key(),
            "Capture admission owner mismatch"
        );
        let mut captured = Vec::new();
        for message in messages {
            let content = message
                .get("content")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default()
                .trim();
            if content.is_empty() {
                continue;
            }
            let role = message
                .get("role")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("user");
            let mut memory = Memory::new(
                content,
                None,
                agent.map(str::to_owned),
                session.map(str::to_owned),
                vec!["conversation".into(), role.into()],
            );
            memory.scope = MemoryScope::Session;
            memory.memory_type = MemoryType::Episodic;
            // Stable owner/session/content identity makes uncertain retries use
            // the original anchor; retained tombstones prevent recapture.
            let key = format!(
                "capture-{}",
                Sha256::digest(serde_json::to_vec(&(
                    self.owner_key(),
                    agent,
                    session,
                    role,
                    content
                ))?)
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>()
            );
            memory.user_id = Some(self.owner_key());
            if let Some(memory) = self
                .service
                .save_controlled_memory(memory, Some(&key), Some(admission.epoch))
                .await?
            {
                captured.push(memory);
            }
        }
        Ok(captured)
    }

    pub async fn compress_memories(
        &self,
        user: Option<&str>,
        agent: Option<&str>,
        session: Option<&str>,
        days: u32,
    ) -> Result<Option<Memory>> {
        self.check_filter(user)?;
        // The pinned compressor deletes sources before saving and suppresses
        // deletion failures. Preserve sources until a summary has committed.
        let cutoff = chrono::Utc::now() - chrono::Duration::days(i64::from(days));
        let memories: Vec<_> = self
            .get_all_memories(None, agent, session)
            .await?
            .into_iter()
            .filter(|memory| {
                let created: chrono::DateTime<chrono::Utc> = memory.created_at.clone().into();
                created < cutoff
            })
            .collect();
        if memories.is_empty() {
            return Ok(None);
        }
        let content = format!(
            "[Compressed {} memories]\n{}",
            memories.len(),
            memories
                .iter()
                .map(|memory| memory.content.as_str())
                .collect::<Vec<_>>()
                .join("\n")
        );
        let summary = Memory::new(
            content,
            Some(self.owner_key()),
            agent.map(str::to_owned),
            session.map(str::to_owned),
            vec!["compressed".into()],
        );
        self.service
            .summarize_controlled_memories(&self.owner_key(), memories, summary)
            .await
    }

    pub async fn create_task_stream(&self, mut stream: TaskStream) -> Result<TaskStream> {
        self.check_filter(stream.user_id.as_deref())?;
        anyhow::ensure!(stream.id.is_none(), "TaskStream identity is host-owned");
        stream.user_id = Some(self.owner_key());
        self.service.storage.create_task_stream(stream).await
    }

    pub async fn get_task_stream(
        &self,
        name: &str,
        user: Option<&str>,
        agent: Option<&str>,
    ) -> Result<Option<TaskStream>> {
        self.check_filter(user)?;
        self.service
            .storage
            .get_task_stream(name, Some(&self.owner_key()), agent)
            .await
    }

    pub async fn add_to_task_stream(
        &self,
        name: &str,
        user: Option<&str>,
        agent: Option<&str>,
        mut memory: Memory,
    ) -> Result<Memory> {
        self.check_filter(user)?;
        self.check_filter(memory.user_id.as_deref())?;
        anyhow::ensure!(
            memory.id.is_none() && memory.task_stream_id.is_none(),
            "Memory record identity is host-owned"
        );
        anyhow::ensure!(
            memory
                .metadata
                .as_ref()
                .is_none_or(
                    |value| value.as_object().is_none_or(|object| !object
                        .keys()
                        .any(|key| key.starts_with("uar_owner")
                            || key.starts_with("uar_migration")
                            || key.starts_with("uar_capture")))
                ),
            "Memory ownership metadata is host-owned"
        );
        memory.user_id = Some(self.owner_key());
        memory.agent_id = agent.map(str::to_owned);
        memory.scope = MemoryScope::Task;
        let stream = self
            .get_task_stream(name, None, agent)
            .await?
            .context("TaskStream not found")?;
        anyhow::ensure!(
            stream.status == surreal_memory::task_stream::TaskStreamStatus::Active,
            "TaskStream is not active"
        );
        memory.task_stream_id = stream.id;
        self.service
            .save_controlled_memory(memory, None, None)
            .await?
            .context("Memory unavailable after save")
    }

    pub async fn get_context_for_task(
        &self,
        name: &str,
        user: Option<&str>,
        agent: Option<&str>,
        model: &str,
        max: Option<u64>,
    ) -> Result<ContextWindow> {
        self.check_filter(user)?;
        let mut context = self
            .service
            .storage
            .get_context_for_task(name, Some(&self.owner_key()), agent, model, max)
            .await?;
        let before = context.memories.len();
        context.memories = self.service.visible_memories(context.memories).await?;
        context.memories_omitted += (before - context.memories.len()) as u64;
        context.tokens_used = context
            .memories
            .iter()
            .map(|memory| u64::from(memory.token_count.unwrap_or(0)))
            .sum();
        Ok(context)
    }

    pub async fn list_task_streams(
        &self,
        agent: Option<&str>,
        user: Option<&str>,
    ) -> Result<Vec<TaskStream>> {
        self.check_filter(user)?;
        self.service
            .storage
            .list_task_streams(agent, Some(&self.owner_key()))
            .await
    }

    pub async fn archive_task_stream(
        &self,
        name: &str,
        user: Option<&str>,
        agent: Option<&str>,
    ) -> Result<TaskStream> {
        self.check_filter(user)?;
        self.service
            .storage
            .archive_task_stream(name, Some(&self.owner_key()), agent)
            .await
    }

    pub async fn auto_summarize_task_stream(
        &self,
        name: &str,
        user: Option<&str>,
        agent: Option<&str>,
        model: &str,
    ) -> Result<Option<Memory>> {
        self.check_filter(user)?;
        let context = self
            .get_context_for_task(name, None, agent, model, Some(u64::MAX))
            .await?;
        if context.memories.is_empty() {
            return Ok(None);
        }
        let stream = self
            .get_task_stream(name, None, agent)
            .await?
            .context("TaskStream not found")?;
        let content = format!(
            "[Task summary]\n{}",
            context
                .memories
                .iter()
                .map(|memory| memory.content.as_str())
                .collect::<Vec<_>>()
                .join("\n")
        );
        let mut summary = Memory::new(
            content,
            Some(self.owner_key()),
            agent.map(str::to_owned),
            None,
            vec!["compressed".into()],
        );
        summary.scope = MemoryScope::Task;
        summary.task_stream_id = stream.id;
        self.service
            .summarize_controlled_memories(&self.owner_key(), context.memories, summary)
            .await
    }
}

/// Inclusive cosine comparison; invalid or zero vectors cannot prove relevance.
#[cfg(any(feature = "surreal-backend", test))]
fn injection_cosine(left: &[f32], right: &[f32]) -> Result<f32> {
    anyhow::ensure!(
        !left.is_empty() && left.len() == right.len(),
        "Injection vector dimensions unavailable"
    );
    anyhow::ensure!(
        left.iter().chain(right).all(|value| value.is_finite()),
        "Injection vector invalid"
    );
    let dot: f64 = left
        .iter()
        .zip(right)
        .map(|(a, b)| f64::from(*a) * f64::from(*b))
        .sum();
    let left_norm: f64 = left.iter().map(|value| f64::from(*value).powi(2)).sum();
    let right_norm: f64 = right.iter().map(|value| f64::from(*value).powi(2)).sum();
    anyhow::ensure!(
        left_norm > 0.0 && right_norm > 0.0,
        "Injection vector has no direction"
    );
    let similarity = dot / (left_norm.sqrt() * right_norm.sqrt());
    anyhow::ensure!(similarity.is_finite(), "Injection similarity unavailable");
    Ok(similarity.clamp(-1.0, 1.0) as f32)
}

fn injection_scope_visible(memory: &Memory, agent: Option<&str>, session: Option<&str>) -> bool {
    let agent_matches = memory
        .agent_id
        .as_deref()
        .is_none_or(|id| Some(id) == agent);
    let session_matches = memory
        .session_id
        .as_deref()
        .is_none_or(|id| Some(id) == session);
    match memory.scope {
        MemoryScope::User | MemoryScope::Global => agent_matches && session_matches,
        MemoryScope::Agent => {
            agent.is_some() && memory.agent_id.as_deref() == agent && session_matches
        }
        MemoryScope::Session => {
            session.is_some() && memory.session_id.as_deref() == session && agent_matches
        }
        MemoryScope::Task => false,
    }
}

#[cfg(test)]
mod injection_tests {
    use super::{injection_cosine, injection_scope_visible};
    use surreal_memory::{Memory, MemoryScope};

    #[test]
    fn similarity_retains_inclusive_cosine_semantics() {
        assert_eq!(injection_cosine(&[1.0, 0.0], &[1.0, 0.0]).unwrap(), 1.0);
        assert_eq!(injection_cosine(&[1.0, 0.0], &[-1.0, 0.0]).unwrap(), -1.0);
        assert_eq!(injection_cosine(&[1.0, 0.0], &[0.0, 1.0]).unwrap(), 0.0);
    }

    #[test]
    fn invalid_vectors_cannot_authorize_injection() {
        for vector in [
            &[][..],
            &[0.0, 0.0][..],
            &[f32::NAN, 0.0][..],
            &[f32::INFINITY, 0.0][..],
            &[1.0][..],
        ] {
            assert!(injection_cosine(&[1.0, 0.0], vector).is_err());
        }
    }

    #[test]
    fn owner_user_fact_is_applicable_to_a_new_session_without_ambient_task_access() {
        let mut memory = Memory::new("owner private fact", None, None, None, vec![]);
        memory.scope = MemoryScope::User;
        assert!(injection_scope_visible(
            &memory,
            Some("agent-a"),
            Some("new-session")
        ));
        memory.scope = MemoryScope::Global;
        assert!(injection_scope_visible(
            &memory,
            Some("agent-a"),
            Some("new-session")
        ));
        memory.scope = MemoryScope::Task;
        assert!(!injection_scope_visible(
            &memory,
            Some("agent-a"),
            Some("new-session")
        ));
    }

    #[test]
    fn classification_never_broadens_explicit_agent_or_session_narrowing() {
        let mut memory = Memory::new(
            "private session fact",
            None,
            Some("agent-a".into()),
            Some("session-a".into()),
            vec![],
        );
        memory.scope = MemoryScope::Session;
        assert!(injection_scope_visible(
            &memory,
            Some("agent-a"),
            Some("session-a")
        ));
        assert!(!injection_scope_visible(
            &memory,
            Some("agent-a"),
            Some("session-b")
        ));
        assert!(!injection_scope_visible(
            &memory,
            Some("agent-b"),
            Some("session-a")
        ));
        memory.scope = MemoryScope::Agent;
        memory.session_id = None;
        assert!(injection_scope_visible(
            &memory,
            Some("agent-a"),
            Some("session-b")
        ));
        assert!(!injection_scope_visible(
            &memory,
            Some("agent-b"),
            Some("session-b")
        ));
        memory.scope = MemoryScope::User;
        assert!(!injection_scope_visible(
            &memory,
            Some("agent-b"),
            Some("session-b")
        ));
    }
}

pub(crate) fn record_id(id: &impl serde::Serialize) -> Result<String> {
    // The pinned SDK serializes RecordId as {table, key}, with key an
    // externally tagged enum. Do not assume serialization is a string.
    let value = serde_json::to_value(id)?;
    let table = value
        .get("table")
        .and_then(serde_json::Value::as_str)
        .context("Memory ID has no table")?;
    let key = value.get("key").context("Memory ID has no key")?;
    let key = if let Some(value) = key.get("String").and_then(serde_json::Value::as_str) {
        value.to_owned()
    } else if let Some(value) = key.get("Number").and_then(serde_json::Value::as_i64) {
        value.to_string()
    } else if let Some(value) = key.get("Uuid").and_then(serde_json::Value::as_str) {
        value.to_owned()
    } else {
        anyhow::bail!("Unsupported memory record key");
    };
    Ok(format!("{table}:{key}"))
}
