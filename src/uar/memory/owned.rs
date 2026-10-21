//! Host-bound memory access. Payload fields can narrow a query, never grant ownership.
use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use super::controls::{CaptureAdmission, CapturePolicy, CapturePolicyUpdate};
use surreal_memory::{Memory, MemoryHistory, MemoryScope, MemoryType, task_stream::{ContextWindow, TaskStream}};
use crate::uar::security::claims::CredentialPrincipal;
use crate::uar::runtime::lifecycle::OperationLease;
use super::service::MemoryService;

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
    pub fn owner_key(&self) -> String { self.principal.storage_key() }

    pub async fn capture_policy(&self) -> Result<CapturePolicy> {
        self.service.owner_capture_policy(&self.owner_key()).await
    }
    pub async fn set_capture_policy(&self, update: CapturePolicyUpdate) -> Result<CapturePolicy> {
        self.service.owner_set_capture_policy(&self.owner_key(), update).await
    }

    pub fn check_filter(&self, requested: Option<&str>) -> Result<()> {
        anyhow::ensure!(requested.is_none_or(|value| value == self.principal.subject()),
            "Memory owner cannot be replaced by request arguments");
        Ok(())
    }

    fn validate_id(id: &str) -> Result<()> {
        anyhow::ensure!(!id.trim().is_empty() && id.split_once(':').is_none_or(|(table, _)| table == "memory"),
            "Memory ID must address a memory record");
        Ok(())
    }

    pub async fn add_memory(&self, mut memory: Memory) -> Result<Memory> {
        self.check_filter(memory.user_id.as_deref())?;
        anyhow::ensure!(memory.id.is_none() && memory.task_stream_id.is_none(), "Memory record identity is host-owned");
        anyhow::ensure!(memory.metadata.as_ref().is_none_or(|value| value.as_object().is_none_or(|object| !object.keys().any(|key| key.starts_with("uar_owner") || key.starts_with("uar_migration") || key.starts_with("uar_capture")))), "Memory ownership metadata is host-owned");
        // Global is a classification inside this principal's partition. Only the
        // explicit trusted-host administrative path may create a shared record.
        memory.user_id = Some(self.owner_key());
        self.service.save_controlled_memory(memory, None, None).await?.context("Memory unavailable after save")
    }

    pub async fn get_memory(&self, id: &str) -> Result<Option<Memory>> {
        Self::validate_id(id)?;
        let owner = self.owner_key();
        match self.service.storage.get_memory(id).await?.filter(|memory| memory.user_id.as_deref() == Some(owner.as_str())) {
            Some(memory) => self.service.visible_memory(memory).await,
            None => Ok(None),
        }
    }

    pub async fn update_memory(&self, id: &str, content: String) -> Result<Memory> {
        self.get_memory(id).await?.context("Memory is unavailable to this owner")?;
        // Ownership is immutable: no ordinary service method can reassign a row.
        self.service.update_controlled_memory(id, &self.owner_key(), content).await
    }

    pub async fn delete_memory(&self, id: &str) -> Result<()> {
        Self::validate_id(id)?;
        self.service.delete_controlled_memory(id, &self.owner_key()).await
    }

    pub async fn get_memory_history(&self, id: &str) -> Result<Vec<MemoryHistory>> {
        let memory = self.get_memory(id).await?.context("Memory is unavailable to this owner")?;
        let mut history: Vec<MemoryHistory> = match memory.metadata.as_ref()
            .and_then(|value| value.get("uar_migration"))
            .and_then(|value| value.get("history")) {
            Some(value) => serde_json::from_value(value.clone()).context("Invalid preserved migration history")?,
            None => Vec::new(),
        };
        history.extend(self.service.storage.get_memory_history(id).await?);
        if self.get_memory(id).await?.is_none() { return Ok(Vec::new()); }
        Ok(history)
    }

    pub async fn get_all_memories(&self, user: Option<&str>, agent: Option<&str>, session: Option<&str>) -> Result<Vec<Memory>> {
        self.check_filter(user)?;
        let records = self.service.storage.get_all_memories(Some(&self.owner_key()), agent, session).await?;
        self.service.visible_memories(records).await
    }

    pub async fn search_memories(&self, query: &str, user: Option<&str>, agent: Option<&str>, session: Option<&str>, categories: Option<&[String]>, limit: usize) -> Result<Vec<Memory>> {
        self.check_filter(user)?;
        let records = self.service.storage.search_memories(query, Some(&self.owner_key()), agent, session, categories, limit).await?;
        self.service.visible_memories(records).await
    }

    pub async fn hybrid_search_memories(&self, query: &str, user: Option<&str>, agent: Option<&str>, session: Option<&str>, limit: usize, vector_weight: f32, bm25_weight: f32) -> Result<Vec<Memory>> {
        self.check_filter(user)?;
        let records = self.service.storage.hybrid_search_memories(query, Some(&self.owner_key()), agent, session, limit, vector_weight, bm25_weight).await?;
        self.service.visible_memories(records).await
    }

    pub async fn delete_all_memories(&self, user: Option<&str>, agent: Option<&str>, session: Option<&str>) -> Result<u64> {
        self.check_filter(user)?;
        let records = self.service.storage.get_all_memories(Some(&self.owner_key()), agent, session).await?;
        let count = records.len() as u64;
        for memory in records {
            if let Some(id) = memory.id { self.delete_memory(&record_id(&id)?).await?; }
        }
        Ok(count)
    }

    pub async fn add_memories_from_conversation(&self, messages: Vec<serde_json::Value>, user: Option<&str>, agent: Option<&str>, session: Option<&str>) -> Result<Vec<Memory>> {
        self.check_filter(user)?;
        let policy = self.capture_policy().await?;
        if !policy.enabled || !policy.host_capture_enabled { return Ok(Vec::new()); }
        let admission = CaptureAdmission { owner: self.owner_key(), epoch: policy.consent_epoch };
        self.capture_admitted(messages, agent, session, &admission).await
    }

    pub(super) async fn capture_admitted(&self, messages: Vec<serde_json::Value>, agent: Option<&str>, session: Option<&str>, admission: &CaptureAdmission) -> Result<Vec<Memory>> {
        anyhow::ensure!(admission.owner == self.owner_key(), "Capture admission owner mismatch");
        let mut captured = Vec::new();
        for message in messages {
            let content = message.get("content").and_then(serde_json::Value::as_str).unwrap_or_default().trim();
            if content.is_empty() { continue; }
            let role = message.get("role").and_then(serde_json::Value::as_str).unwrap_or("user");
            let mut memory = Memory::new(content, None, agent.map(str::to_owned), session.map(str::to_owned), vec!["conversation".into(), role.into()]);
            memory.scope = MemoryScope::Session;
            memory.memory_type = MemoryType::Episodic;
            // Stable owner/session/content identity makes uncertain retries use
            // the original anchor; retained tombstones prevent recapture.
            let key = format!("capture-{}", Sha256::digest(serde_json::to_vec(&(self.owner_key(), agent, session, role, content))?)
                .iter().map(|byte| format!("{byte:02x}")).collect::<String>());
            memory.user_id = Some(self.owner_key());
            if let Some(memory) = self.service.save_controlled_memory(memory, Some(&key), Some(admission.epoch)).await? {
                captured.push(memory);
            }
        }
        Ok(captured)
    }

    pub async fn compress_memories(&self, user: Option<&str>, agent: Option<&str>, session: Option<&str>, days: u32) -> Result<Option<Memory>> {
        self.check_filter(user)?;
        // The pinned compressor deletes sources before saving and suppresses
        // deletion failures. Preserve sources until a summary has committed.
        let cutoff = chrono::Utc::now() - chrono::Duration::days(i64::from(days));
        let memories: Vec<_> = self.get_all_memories(None, agent, session).await?.into_iter().filter(|memory| {
            let created: chrono::DateTime<chrono::Utc> = memory.created_at.clone().into();
            created < cutoff
        }).collect();
        if memories.is_empty() { return Ok(None); }
        let content = format!("[Compressed {} memories]\n{}", memories.len(), memories.iter().map(|memory| memory.content.as_str()).collect::<Vec<_>>().join("\n"));
        let summary = Memory::new(content, Some(self.owner_key()), agent.map(str::to_owned), session.map(str::to_owned), vec!["compressed".into()]);
        self.service.summarize_controlled_memories(&self.owner_key(), memories, summary).await
    }

    pub async fn create_task_stream(&self, mut stream: TaskStream) -> Result<TaskStream> {
        self.check_filter(stream.user_id.as_deref())?;
        anyhow::ensure!(stream.id.is_none(), "TaskStream identity is host-owned");
        stream.user_id = Some(self.owner_key());
        self.service.storage.create_task_stream(stream).await
    }

    pub async fn get_task_stream(&self, name: &str, user: Option<&str>, agent: Option<&str>) -> Result<Option<TaskStream>> {
        self.check_filter(user)?;
        self.service.storage.get_task_stream(name, Some(&self.owner_key()), agent).await
    }

    pub async fn add_to_task_stream(&self, name: &str, user: Option<&str>, agent: Option<&str>, mut memory: Memory) -> Result<Memory> {
        self.check_filter(user)?;
        self.check_filter(memory.user_id.as_deref())?;
        anyhow::ensure!(memory.id.is_none() && memory.task_stream_id.is_none(), "Memory record identity is host-owned");
        anyhow::ensure!(memory.metadata.as_ref().is_none_or(|value| value.as_object().is_none_or(|object| !object.keys().any(|key| key.starts_with("uar_owner") || key.starts_with("uar_migration") || key.starts_with("uar_capture")))), "Memory ownership metadata is host-owned");
        memory.user_id = Some(self.owner_key());
        memory.agent_id = agent.map(str::to_owned);
        memory.scope = MemoryScope::Task;
        let stream = self.get_task_stream(name, None, agent).await?.context("TaskStream not found")?;
        anyhow::ensure!(stream.status == surreal_memory::task_stream::TaskStreamStatus::Active, "TaskStream is not active");
        memory.task_stream_id = stream.id;
        self.service.save_controlled_memory(memory, None, None).await?.context("Memory unavailable after save")
    }

    pub async fn get_context_for_task(&self, name: &str, user: Option<&str>, agent: Option<&str>, model: &str, max: Option<u64>) -> Result<ContextWindow> {
        self.check_filter(user)?;
        let mut context = self.service.storage.get_context_for_task(name, Some(&self.owner_key()), agent, model, max).await?;
        let before = context.memories.len();
        context.memories = self.service.visible_memories(context.memories).await?;
        context.memories_omitted += (before - context.memories.len()) as u64;
        context.tokens_used = context.memories.iter().map(|memory| u64::from(memory.token_count.unwrap_or(0))).sum();
        Ok(context)
    }

    pub async fn list_task_streams(&self, agent: Option<&str>, user: Option<&str>) -> Result<Vec<TaskStream>> {
        self.check_filter(user)?;
        self.service.storage.list_task_streams(agent, Some(&self.owner_key())).await
    }

    pub async fn archive_task_stream(&self, name: &str, user: Option<&str>, agent: Option<&str>) -> Result<TaskStream> {
        self.check_filter(user)?;
        self.service.storage.archive_task_stream(name, Some(&self.owner_key()), agent).await
    }

    pub async fn auto_summarize_task_stream(&self, name: &str, user: Option<&str>, agent: Option<&str>, model: &str) -> Result<Option<Memory>> {
        self.check_filter(user)?;
        let context = self.get_context_for_task(name, None, agent, model, Some(u64::MAX)).await?;
        if context.memories.is_empty() { return Ok(None); }
        let stream = self.get_task_stream(name, None, agent).await?.context("TaskStream not found")?;
        let content = format!("[Task summary]\n{}", context.memories.iter().map(|memory| memory.content.as_str()).collect::<Vec<_>>().join("\n"));
        let mut summary = Memory::new(content, Some(self.owner_key()), agent.map(str::to_owned), None, vec!["compressed".into()]);
        summary.scope = MemoryScope::Task;
        summary.task_stream_id = stream.id;
        self.service.summarize_controlled_memories(&self.owner_key(), context.memories, summary).await
    }
}


pub(crate) fn record_id(id: &impl serde::Serialize) -> Result<String> {
    // The pinned SDK serializes RecordId as {table, key}, with key an
    // externally tagged enum. Do not assume serialization is a string.
    let value = serde_json::to_value(id)?;
    let table = value.get("table").and_then(serde_json::Value::as_str).context("Memory ID has no table")?;
    let key = value.get("key").context("Memory ID has no key")?;
    let key = if let Some(value) = key.get("String").and_then(serde_json::Value::as_str) {
        value.to_owned()
    } else if let Some(value) = key.get("Number").and_then(serde_json::Value::as_i64) {
        value.to_string()
    } else if let Some(value) = key.get("Uuid").and_then(serde_json::Value::as_str) {
        value.to_owned()
    } else { anyhow::bail!("Unsupported memory record key"); };
    Ok(format!("{table}:{key}"))
}
