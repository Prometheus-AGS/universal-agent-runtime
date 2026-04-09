use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use tokio::time;
use tracing::{info, warn};

use super::runner::SandboxRunner;
use super::types::*;

struct SessionEntry {
    handle: SandboxHandle,
    #[allow(dead_code)]
    config: SandboxConfig,
    last_accessed: Instant,
    #[allow(dead_code)]
    created_at: Instant,
}

pub struct SessionManager {
    sessions: RwLock<HashMap<String, SessionEntry>>,
    runner: Arc<dyn SandboxRunner>,
    ttl: Duration,
    max_concurrent: usize,
}

impl std::fmt::Debug for SessionManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SessionManager")
            .field("ttl", &self.ttl)
            .field("max_concurrent", &self.max_concurrent)
            .field("active_count", &self.active_count())
            .finish_non_exhaustive()
    }
}

impl SessionManager {
    #[must_use]
    pub fn new(runner: Arc<dyn SandboxRunner>, ttl_secs: u64, max_concurrent: usize) -> Self {
        Self {
            sessions: RwLock::new(HashMap::new()),
            runner,
            ttl: Duration::from_secs(ttl_secs),
            max_concurrent,
        }
    }

    pub async fn get_or_create(
        &self,
        session_id: &str,
        config: SandboxConfig,
    ) -> Result<SandboxHandle, SandboxError> {
        // Check for existing session
        {
            let mut sessions = self
                .sessions
                .write()
                .map_err(|e| SandboxError::CreationFailed(format!("lock poisoned: {e}")))?;
            if let Some(entry) = sessions.get_mut(session_id) {
                entry.last_accessed = Instant::now();
                return Ok(entry.handle.clone());
            }
            // Check capacity before creating
            if sessions.len() >= self.max_concurrent {
                return Err(SandboxError::CapacityExceeded(self.max_concurrent));
            }
        }

        // Create new sandbox outside the lock
        let handle = self.runner.create(config.clone()).await?;
        let handle_clone = handle.clone();

        let entry = SessionEntry {
            handle,
            config,
            last_accessed: Instant::now(),
            created_at: Instant::now(),
        };

        {
            let mut sessions = self
                .sessions
                .write()
                .map_err(|e| SandboxError::CreationFailed(format!("lock poisoned: {e}")))?;
            sessions.insert(session_id.to_string(), entry);
        }

        info!(session_id, "Created new sandbox session");
        Ok(handle_clone)
    }

    pub async fn destroy_session(&self, session_id: &str) -> Result<(), SandboxError> {
        let entry = {
            let mut sessions = self
                .sessions
                .write()
                .map_err(|e| SandboxError::NotFound(format!("lock poisoned: {e}")))?;
            sessions.remove(session_id)
        };

        if let Some(entry) = entry {
            info!(session_id, "Destroying sandbox session");
            self.runner.destroy(entry.handle).await?;
        } else {
            warn!(session_id, "Session not found for destruction");
        }

        Ok(())
    }

    pub async fn cleanup_expired(&self) {
        let now = Instant::now();
        let expired: Vec<(String, SandboxHandle)> = {
            let sessions = match self.sessions.read() {
                Ok(s) => s,
                Err(_) => return,
            };
            sessions
                .iter()
                .filter(|(_, entry)| now.duration_since(entry.last_accessed) > self.ttl)
                .map(|(id, entry)| (id.clone(), entry.handle.clone()))
                .collect()
        };

        if expired.is_empty() {
            return;
        }

        info!(count = expired.len(), "Cleaning up expired sandbox sessions");

        for (id, handle) in expired {
            if let Err(e) = self.runner.destroy(handle).await {
                warn!(session_id = %id, error = %e, "Failed to destroy expired sandbox");
            }
            let mut sessions = match self.sessions.write() {
                Ok(s) => s,
                Err(_) => continue,
            };
            sessions.remove(&id);
        }
    }

    /// Return a reference to the underlying sandbox runner.
    #[must_use]
    pub fn runner(&self) -> &dyn SandboxRunner {
        &*self.runner
    }

    #[must_use]
    pub fn active_count(&self) -> usize {
        self.sessions.read().map(|s| s.len()).unwrap_or(0)
    }

    /// Start background GC task that periodically cleans up expired sessions.
    pub fn start_gc_task(self: &Arc<Self>) {
        let mgr = Arc::clone(self);
        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(60));
            loop {
                interval.tick().await;
                mgr.cleanup_expired().await;
            }
        });
    }
}
