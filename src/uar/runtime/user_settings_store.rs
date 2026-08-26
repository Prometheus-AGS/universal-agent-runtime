//! Persistence-backed access to per-user prompt-caching settings.
//!
//! Every read goes through the configured [`PersistenceLayer`], and every
//! update is written before it is returned. The in-memory persistence provider
//! intentionally gives process-lifetime behavior; configured durable providers
//! never silently fall back to process memory when a read or write fails.

use crate::uar::{
    domain::prompt_caching::{CachingScope, UserPromptCachingSettings},
    persistence::PersistenceLayer,
};
use anyhow::Result;
use chrono::Utc;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Thread-safe persistence-backed store for per-user caching preferences.
#[derive(Debug, Clone)]
pub struct UserSettingsStore {
    persistence: Arc<dyn PersistenceLayer>,
    update_lock: Arc<Mutex<()>>,
}

impl UserSettingsStore {
    /// Create a store backed by the configured persistence provider.
    #[must_use]
    pub fn new(persistence: Arc<dyn PersistenceLayer>) -> Self {
        Self {
            persistence,
            update_lock: Arc::new(Mutex::new(())),
        }
    }

    /// Retrieve settings for a verified principal.
    ///
    /// # Errors
    ///
    /// Returns an error when the configured persistence provider cannot serve
    /// the read. Durable-provider failures are never converted into absence.
    pub async fn get(&self, principal_id: &str) -> Result<Option<UserPromptCachingSettings>> {
        self.persistence
            .load_user_prompt_caching_settings(principal_id)
            .await
    }

    /// Apply a partial update and durably save the resulting record.
    ///
    /// The mutation lock keeps concurrent read-modify-write updates from
    /// clobbering fields that the other request omitted.
    ///
    /// # Errors
    ///
    /// Returns an error when the configured persistence provider cannot read
    /// or save the record.
    pub async fn upsert(&self, update: UserSettingsUpdate) -> Result<UserPromptCachingSettings> {
        let _guard = self.update_lock.lock().await;
        let mut record = self
            .persistence
            .load_user_prompt_caching_settings(&update.principal_id)
            .await?
            .unwrap_or_else(|| UserPromptCachingSettings::new(&update.principal_id));

        match update.prompt_caching_enabled {
            PromptCachingPreferenceUpdate::Preserve => {}
            PromptCachingPreferenceUpdate::Clear => record.prompt_caching_enabled = None,
            PromptCachingPreferenceUpdate::Set(enabled) => {
                record.prompt_caching_enabled = Some(enabled);
            }
        }
        if let Some(scope) = update.preferred_scope {
            record.preferred_scope = scope;
        }
        record.updated_at = Utc::now();

        self.persistence
            .save_user_prompt_caching_settings(&record)
            .await?;
        Ok(record)
    }

    /// Look up the prompt-caching preference for a verified principal.
    ///
    /// `Ok(None)` means the user inherits the lower-precedence setting.
    ///
    /// # Errors
    ///
    /// Returns an error when the configured persistence provider cannot serve
    /// the read.
    pub async fn caching_enabled_for(&self, principal_id: &str) -> Result<Option<bool>> {
        Ok(self
            .get(principal_id)
            .await?
            .and_then(|settings| settings.prompt_caching_enabled))
    }
}

/// Four-state patch value for the nullable prompt-caching preference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PromptCachingPreferenceUpdate {
    /// The JSON field was omitted; preserve the stored value.
    #[default]
    Preserve,
    /// The JSON field was `null`; clear the override and inherit.
    Clear,
    /// The JSON field was a boolean; set the override explicitly.
    Set(bool),
}

/// Partial update applied to one verified principal's settings record.
#[derive(Debug, Clone)]
pub struct UserSettingsUpdate {
    pub principal_id: String,
    pub prompt_caching_enabled: PromptCachingPreferenceUpdate,
    pub preferred_scope: Option<CachingScope>,
}

#[cfg(all(test, feature = "surreal-backend"))]
mod tests {
    use super::*;
    use crate::uar::persistence::providers::surreal::SurrealDbProvider;

    async fn store_with_shared_backend() -> (
        UserSettingsStore,
        Arc<dyn PersistenceLayer>,
        tempfile::TempDir,
    ) {
        let dir = tempfile::tempdir().expect("tempdir");
        let endpoint = format!("surrealkv://{}", dir.path().join("settings.db").display());
        let persistence: Arc<dyn PersistenceLayer> = Arc::new(
            SurrealDbProvider::new(&endpoint, None, None, None, None)
                .await
                .expect("connect embedded SurrealKV"),
        );
        (
            UserSettingsStore::new(Arc::clone(&persistence)),
            persistence,
            dir,
        )
    }

    fn update(
        principal_id: &str,
        prompt_caching_enabled: PromptCachingPreferenceUpdate,
    ) -> UserSettingsUpdate {
        UserSettingsUpdate {
            principal_id: principal_id.to_owned(),
            prompt_caching_enabled,
            preferred_scope: None,
        }
    }

    #[tokio::test]
    async fn four_state_updates_preserve_clear_enable_and_disable() {
        let (store, _, _dir) = store_with_shared_backend().await;
        let principal = "tenant:1:a:subject:1:u";

        let inherited = store
            .upsert(update(principal, PromptCachingPreferenceUpdate::Preserve))
            .await
            .expect("create inherited settings");
        assert_eq!(inherited.prompt_caching_enabled, None);

        let enabled = store
            .upsert(update(principal, PromptCachingPreferenceUpdate::Set(true)))
            .await
            .expect("enable caching");
        assert_eq!(enabled.prompt_caching_enabled, Some(true));

        let preserved = store
            .upsert(update(principal, PromptCachingPreferenceUpdate::Preserve))
            .await
            .expect("preserve enabled value");
        assert_eq!(preserved.prompt_caching_enabled, Some(true));

        let disabled = store
            .upsert(update(principal, PromptCachingPreferenceUpdate::Set(false)))
            .await
            .expect("disable caching");
        assert_eq!(disabled.prompt_caching_enabled, Some(false));

        let cleared = store
            .upsert(update(principal, PromptCachingPreferenceUpdate::Clear))
            .await
            .expect("clear override");
        assert_eq!(cleared.prompt_caching_enabled, None);
    }

    #[tokio::test]
    async fn records_are_isolated_and_reload_through_the_shared_backend() {
        let (first_store, persistence, _dir) = store_with_shared_backend().await;
        first_store
            .upsert(update(
                "tenant:1:a:subject:3:sam",
                PromptCachingPreferenceUpdate::Set(true),
            ))
            .await
            .expect("save tenant A");
        first_store
            .upsert(update(
                "tenant:1:b:subject:3:sam",
                PromptCachingPreferenceUpdate::Set(false),
            ))
            .await
            .expect("save tenant B");

        let reloaded = UserSettingsStore::new(persistence);
        assert_eq!(
            reloaded
                .caching_enabled_for("tenant:1:a:subject:3:sam")
                .await
                .expect("reload tenant A"),
            Some(true)
        );
        assert_eq!(
            reloaded
                .caching_enabled_for("tenant:1:b:subject:3:sam")
                .await
                .expect("reload tenant B"),
            Some(false)
        );
    }
}
