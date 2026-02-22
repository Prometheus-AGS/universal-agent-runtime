//! In-memory store for per-user prompt-caching settings.
//!
//! Provides fast, thread-safe access to user preferences without requiring a
//! database round-trip on every request.  When the persistence layer is
//! configured, the store is populated from the database on startup and writes
//! are flushed asynchronously.

use crate::uar::domain::prompt_caching::UserPromptCachingSettings;
use chrono::Utc;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

/// Thread-safe in-memory store for per-user caching preferences.
#[derive(Debug, Clone, Default)]
pub struct UserSettingsStore {
    inner: Arc<RwLock<HashMap<String, UserPromptCachingSettings>>>,
}

impl UserSettingsStore {
    /// Create an empty store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Retrieve settings for `user_id`, returning `None` if the user has no
    /// stored preferences.
    pub async fn get(&self, user_id: &str) -> Option<UserPromptCachingSettings> {
        self.inner.read().await.get(user_id).cloned()
    }

    /// Upsert the settings record for `user_id`.
    ///
    /// Merges the incoming partial update with any existing record so that
    /// unset fields are preserved.
    pub async fn upsert(&self, update: UserSettingsUpdate) -> UserPromptCachingSettings {
        let mut map = self.inner.write().await;
        let record = map
            .entry(update.user_id.clone())
            .or_insert_with(|| UserPromptCachingSettings::new(&update.user_id));

        if let Some(enabled) = update.prompt_caching_enabled {
            record.prompt_caching_enabled = Some(enabled);
        }
        if let Some(scope) = update.preferred_scope {
            record.preferred_scope = scope;
        }
        record.updated_at = Utc::now();

        record.clone()
    }

    /// Look up the prompt-caching preference for a user, returning `None` when
    /// the user has no stored preference (caller should fall back to agent or
    /// global defaults).
    pub async fn caching_enabled_for(&self, user_id: &str) -> Option<bool> {
        self.inner
            .read()
            .await
            .get(user_id)
            .and_then(|s| s.prompt_caching_enabled)
    }
}

/// Partial update applied to a user's settings record.
#[derive(Debug, Clone)]
pub struct UserSettingsUpdate {
    pub user_id: String,
    pub prompt_caching_enabled: Option<bool>,
    pub preferred_scope: Option<crate::uar::domain::prompt_caching::CachingScope>,
}
