//! Owner consent and deletion authority. No content is stored in control records.
use anyhow::Result;
use serde::{Deserialize, Serialize};
use surreal_memory::Memory;
use crate::uar::security::claims::UserContext;
use super::service::MemoryService;

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Retention {
    Days { days: u16 },
    UntilDeleted,
}
impl Retention {
    pub fn validate(&self) -> Result<()> {
        if let Self::Days { days } = self {
            anyhow::ensure!(matches!(days, 7 | 30 | 90 | 365), "Retention days must be 7, 30, 90 or 365");
        }
        Ok(())
    }
    pub(crate) fn days(&self) -> Option<u16> {
        match self { Self::Days { days } => Some(*days), Self::UntilDeleted => None }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CapturePolicyUpdate {
    pub enabled: bool,
    pub retention: Option<Retention>,
}
impl CapturePolicyUpdate {
    pub fn validate(&self) -> Result<()> {
        anyhow::ensure!(!self.enabled || self.retention.is_some(), "Enabling capture requires explicit retention");
        if let Some(retention) = &self.retention { retention.validate()?; }
        Ok(())
    }
}

/// Host-issued queue capability; cannot be constructed or deserialized by a model.
#[derive(Debug, Clone)]
pub struct CaptureAdmission { pub(super) owner: String, pub(super) epoch: u64 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapturePolicy {
    pub enabled: bool,
    pub retention: Option<Retention>,
    pub revision: u64,
    pub consent_epoch: u64,
    pub host_capture_enabled: bool,
}

// The optional Surreal backend is the only supported durable control backend.
// Builds without it fail explicitly; they never infer consent from legacy rows.
#[cfg(feature = "surreal-backend")]
#[path = "controls_store.rs"]
mod backend;
#[cfg(not(feature = "surreal-backend"))]
mod backend {
    use super::*;
    macro_rules! unsupported {
        ($($name:ident ($($arg:ident : $ty:ty),*) -> $out:ty;)*) => {$(
            pub(super) async fn $name($($arg: $ty),*) -> Result<$out> {
                $(let _ = $arg;)*
                anyhow::bail!("Durable memory controls require surreal-backend")
            }
        )*};
    }
    unsupported! {
        initialize(service: &MemoryService) -> ();
        policy(service: &MemoryService, owner: &str) -> CapturePolicy;
        set_policy(service: &MemoryService, owner: &str, update: CapturePolicyUpdate) -> CapturePolicy;
        visible(service: &MemoryService, memory: Memory) -> Option<Memory>;
        save(service: &MemoryService, memory: Memory, key: Option<&str>, epoch: Option<u64>) -> Option<Memory>;
        update(service: &MemoryService, id: &str, owner: &str, content: String) -> Memory;
        delete(service: &MemoryService, id: &str, owner: &str) -> ();
        purge(service: &MemoryService) -> u64;
        status(service: &MemoryService, owner: &str, id: &str) -> Option<serde_json::Value>;
        summarize(service: &MemoryService, owner: &str, sources: Vec<Memory>, summary: Memory) -> Option<Memory>;
    }
}

impl MemoryService {
    pub async fn capture_admission(&self, user: &UserContext) -> Result<Option<CaptureAdmission>> {
        let owned = self.for_context(user)?;
        let policy = owned.capture_policy().await?;
        Ok((policy.enabled && policy.host_capture_enabled).then(|| CaptureAdmission { owner: owned.owner_key(), epoch: policy.consent_epoch }))
    }
    pub async fn auto_capture_admitted(&self, messages: Vec<serde_json::Value>, user: &UserContext, agent: Option<&str>, session: Option<&str>, admission: &CaptureAdmission) -> Result<Vec<Memory>> {
        self.for_context(user)?.capture_admitted(messages, agent, session, admission).await
    }

    pub async fn memory_control_status(&self, user: &UserContext, id: &str) -> Result<Option<serde_json::Value>> {
        backend::status(self, &self.for_context(user)?.owner_key(), id).await
    }
    pub async fn capture_policy(&self, user: &UserContext) -> Result<CapturePolicy> {
        self.for_context(user)?.capture_policy().await
    }
    pub async fn set_capture_policy(&self, user: &UserContext, update: CapturePolicyUpdate) -> Result<CapturePolicy> {
        self.for_context(user)?.set_capture_policy(update).await
    }
    pub(crate) async fn initialize_controls(&self) -> Result<()> { backend::initialize(self).await }
    pub(crate) async fn owner_capture_policy(&self, owner: &str) -> Result<CapturePolicy> { backend::policy(self, owner).await }
    pub(crate) async fn owner_set_capture_policy(&self, owner: &str, update: CapturePolicyUpdate) -> Result<CapturePolicy> {
        update.validate()?;
        anyhow::ensure!(!update.enabled || self.config.auto_capture, "Automatic capture is disabled by this host");
        backend::set_policy(self, owner, update).await
    }
    pub(crate) async fn visible_memory(&self, memory: Memory) -> Result<Option<Memory>> { backend::visible(self, memory).await }
    pub(crate) async fn visible_memories(&self, memories: Vec<Memory>) -> Result<Vec<Memory>> {
        let mut visible = Vec::new();
        for memory in memories {
            if let Some(memory) = self.visible_memory(memory).await? { visible.push(memory); }
        }
        Ok(visible)
    }
    pub(crate) async fn save_controlled_memory(&self, memory: Memory, key: Option<&str>, epoch: Option<u64>) -> Result<Option<Memory>> {
        backend::save(self, memory, key, epoch).await
    }
    pub(crate) async fn update_controlled_memory(&self, id: &str, owner: &str, content: String) -> Result<Memory> {
        backend::update(self, id, owner, content).await
    }
    pub(crate) async fn delete_controlled_memory(&self, id: &str, owner: &str) -> Result<()> { backend::delete(self, id, owner).await }
    /// Resume expiry and content/index/history cleanup. Tombstones are retained.
    pub async fn purge_memories(&self, administrator: &UserContext) -> Result<u64> {
        self.admin_storage(administrator)?;
        backend::purge(self).await
    }
    pub(crate) async fn summarize_controlled_memories(&self, owner: &str, sources: Vec<Memory>, summary: Memory) -> Result<Option<Memory>> {
        backend::summarize(self, owner, sources, summary).await
    }
    pub(crate) async fn resume_memory_purge(&self) -> Result<u64> { backend::purge(self).await }
}
