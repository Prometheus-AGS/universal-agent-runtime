use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use tokio::sync::RwLock;

use super::CompilerSession;

/// Storage abstraction for compiler sessions.
#[async_trait]
pub trait SessionStorage: Send + Sync + std::fmt::Debug {
    /// Save or update a session.
    async fn save_session(&self, session: CompilerSession) -> Result<CompilerSession>;

    /// Retrieve a session by ID.
    async fn get_session(&self, id: &str) -> Result<Option<CompilerSession>>;

    /// Delete a session.
    async fn delete_session(&self, id: &str) -> Result<bool>;

    /// List all sessions.
    async fn list_sessions(&self) -> Result<Vec<CompilerSession>>;
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

    async fn get_session(&self, id: &str) -> Result<Option<CompilerSession>> {
        let sessions = self.sessions.read().await;
        Ok(sessions.get(id).cloned())
    }

    async fn delete_session(&self, id: &str) -> Result<bool> {
        let mut sessions = self.sessions.write().await;
        Ok(sessions.remove(id).is_some())
    }

    async fn list_sessions(&self) -> Result<Vec<CompilerSession>> {
        let sessions = self.sessions.read().await;
        Ok(sessions.values().cloned().collect())
    }
}
