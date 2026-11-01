use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use tokio::sync::RwLock;

use super::CompilerSession;
use crate::uar::compiler::owner::CompilerOwner;

/// Storage abstraction for compiler sessions.
///
/// Reads and deletes take the verified [`CompilerOwner`]; a session that belongs
/// to another owner is indistinguishable from an absent one. [`save_session`]
/// persists the owner carried by the session itself.
///
/// [`save_session`]: SessionStorage::save_session
#[async_trait]
pub trait SessionStorage: Send + Sync + std::fmt::Debug {
    /// Save or update a session.
    async fn save_session(&self, session: CompilerSession) -> Result<CompilerSession>;

    /// Retrieve a session by ID if `owner` owns it.
    async fn get_session(&self, id: &str, owner: &CompilerOwner)
    -> Result<Option<CompilerSession>>;

    /// Delete a session `owner` owns.
    async fn delete_session(&self, id: &str, owner: &CompilerOwner) -> Result<bool>;

    /// List the sessions `owner` owns.
    async fn list_sessions(&self, owner: &CompilerOwner) -> Result<Vec<CompilerSession>>;
}

/// Thread-safe in-memory session storage. Suitable for development and testing.
#[derive(Debug, Default)]
pub struct InMemorySessionStorage {
    sessions: RwLock<HashMap<String, CompilerSession>>,
}

impl InMemorySessionStorage {
    /// Create a new empty in-memory store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl SessionStorage for InMemorySessionStorage {
    async fn save_session(&self, session: CompilerSession) -> Result<CompilerSession> {
        let mut sessions = self.sessions.write().await;
        sessions.insert(session.id.clone(), session.clone());
        Ok(session)
    }

    async fn get_session(
        &self,
        id: &str,
        owner: &CompilerOwner,
    ) -> Result<Option<CompilerSession>> {
        let sessions = self.sessions.read().await;
        Ok(sessions
            .get(id)
            .filter(|session| owner.owns(session.owner.as_deref()))
            .cloned())
    }

    async fn delete_session(&self, id: &str, owner: &CompilerOwner) -> Result<bool> {
        let mut sessions = self.sessions.write().await;
        if !sessions
            .get(id)
            .is_some_and(|session| owner.owns(session.owner.as_deref()))
        {
            return Ok(false);
        }
        Ok(sessions.remove(id).is_some())
    }

    async fn list_sessions(&self, owner: &CompilerOwner) -> Result<Vec<CompilerSession>> {
        let sessions = self.sessions.read().await;
        Ok(sessions
            .values()
            .filter(|session| owner.owns(session.owner.as_deref()))
            .cloned()
            .collect())
    }
}
