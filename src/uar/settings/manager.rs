//! Settings Manager — runtime configuration administration.
//!
//! `SettingsManager` is the single source of truth for all application settings
//! after the first boot. It:
//!
//! 1. **Seeds** the database on first boot from the resolved `AppConfig`
//!    (config file + env vars + CLI) using typed JSON Schemas.
//! 2. **Detects drift** on subsequent boots: settings changed via the REST API
//!    and not reflected in the config file are flagged `is_drift=true` in the
//!    in-memory cache, while the DB value is preserved.
//! 3. **Validates** every write against the parent type's JSON Schema.
//! 4. **Exposes** an extension API (`register_extension`) so plugins can add
//!    their own settings namespaces without touching any UAR code.
//! 5. **Caches** all settings in memory for sub-μs reads at runtime.

use crate::config::AppConfig;
use crate::uar::persistence::PersistenceLayer;
use crate::uar::settings::schema::{
    BootstrapStats, SettingSource, Settings, SettingsMeta, SettingsType, SettingsWithMeta,
};
use anyhow::{Context, Result};
use chrono::Utc;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

// =============================================================================
// Types
// =============================================================================

/// An entry in the in-memory cache — pairs the stored value with its transient metadata.
#[derive(Debug, Clone)]
struct CacheEntry {
    setting: Settings,
    meta: SettingsMeta,
}

// =============================================================================
// SettingsManager
// =============================================================================

/// Thread-safe runtime settings manager.
///
/// Obtain via `SettingsManager::new(persistence)` then call `initialize(&config)`.
#[derive(Debug)]
pub struct SettingsManager {
    persistence: Arc<dyn PersistenceLayer>,
    /// In-memory cache: key → (setting, transient metadata).
    cache: RwLock<HashMap<String, CacheEntry>>,
}

impl SettingsManager {
    /// Create a new manager wrapping the given persistence layer.
    pub fn new(persistence: Arc<dyn PersistenceLayer>) -> Self {
        Self {
            persistence,
            cache: RwLock::new(HashMap::new()),
        }
    }

    // =========================================================================
    // Public: Bootstrap
    // =========================================================================

    /// Seed or reconcile settings from `AppConfig` into the database.
    ///
    /// **First boot:** every setting is inserted with `source = ConfigFile/EnvVar/Default`.
    ///
    /// **Subsequent boots:**
    /// - If the DB value was set via the **API** (`source == Api`) and it differs
    ///   from the current config, the DB value is preserved and `is_drift = true`
    ///   is set in the in-memory cache.
    /// - Otherwise the DB value is overwritten with the new config value.
    ///
    /// After the upsert pass the entire settings table is loaded into the
    /// in-memory cache so runtime reads are O(1).
    pub async fn initialize(&self, config: &AppConfig) -> Result<BootstrapStats> {
        let desired = build_core_schema(config);
        let mut stats = BootstrapStats::default();

        // Upsert each settings type and immediately reconcile its settings using the
        // *actual* stored id returned by the persistence layer. ON CONFLICT DO UPDATE
        // keeps the original row's id, so we must not use st.id for FK references —
        // we use the value returned by upsert_settings_type instead.
        let mut new_cache: HashMap<String, CacheEntry> = HashMap::new();

        for (st, settings) in &desired {
            let actual_type_id = self
                .persistence
                .upsert_settings_type(st)
                .await
                .with_context(|| format!("upserting settings type '{}'", st.key))?;
            stats.types_upserted += 1;

            for desired_setting in settings {
                // Ensure the FK points to the row that actually exists in the DB.
                let desired_setting = if desired_setting.settings_type_id != actual_type_id {
                    Settings {
                        settings_type_id: actual_type_id,
                        ..desired_setting.clone()
                    }
                } else {
                    desired_setting.clone()
                };
                let key = &desired_setting.key;

                match self.persistence.get_setting(key).await? {
                    None => {
                        // First boot — insert.
                        self.persistence
                            .upsert_setting(&desired_setting)
                            .await
                            .with_context(|| format!("seeding setting '{key}'"))?;
                        new_cache.insert(
                            key.clone(),
                            CacheEntry {
                                setting: desired_setting.clone(),
                                meta: SettingsMeta::from_config(SettingSource::ConfigFile),
                            },
                        );
                        stats.seeded += 1;
                    }
                    Some(existing) => {
                        // Look up the existing cache entry to check the source.
                        let cached_source = {
                            let cache = self.cache.read().await;
                            cache
                                .get(key)
                                .map(|e| e.meta.source.clone())
                                .unwrap_or(SettingSource::Default)
                        };

                        let data_differs = existing.data != desired_setting.data;

                        if cached_source == SettingSource::Api && data_differs {
                            // API-controlled value — preserve DB, mark drift.
                            new_cache.insert(
                                key.clone(),
                                CacheEntry {
                                    setting: existing,
                                    meta: SettingsMeta::api_drift(),
                                },
                            );
                            stats.drift_count += 1;
                            tracing::warn!(
                                setting_key = %key,
                                settings_type = %st.key,
                                "Settings drift detected: DB value (set via API) differs from config"
                            );
                        } else if data_differs {
                            // Config-controlled value changed — overwrite DB.
                            self.persistence
                                .upsert_setting(&desired_setting)
                                .await
                                .with_context(|| format!("updating setting '{key}'"))?;
                            new_cache.insert(
                                key.clone(),
                                CacheEntry {
                                    setting: desired_setting.clone(),
                                    meta: SettingsMeta::from_config(SettingSource::ConfigFile),
                                },
                            );
                            stats.updated += 1;
                        } else {
                            // Unchanged — just load from DB into cache.
                            new_cache.insert(
                                key.clone(),
                                CacheEntry {
                                    setting: existing,
                                    meta: SettingsMeta::from_config(cached_source),
                                },
                            );
                        }
                    }
                }
            }
        }

        // 3. Replace cache atomically.
        {
            let mut cache = self.cache.write().await;
            *cache = new_cache;
        }

        tracing::info!(
            seeded = stats.seeded,
            updated = stats.updated,
            drift = stats.drift_count,
            types = stats.types_upserted,
            "Settings bootstrapped from config into DB"
        );

        Ok(stats)
    }

    // =========================================================================
    // Public: Plugin/Extension API
    // =========================================================================

    /// Register a new settings domain from a plugin or extension.
    ///
    /// This is the **only** entry point extensions need. They never touch the
    /// persistence layer directly. The `schema` field of `settings_type` must be
    /// a valid JSON Schema (Draft 7); `defaults` provides initial values.
    pub async fn register_extension(
        &self,
        settings_type: SettingsType,
        defaults: Vec<Settings>,
    ) -> Result<()> {
        // Use the actual stored id (ON CONFLICT keeps original) for FK references.
        let actual_type_id = self
            .persistence
            .upsert_settings_type(&settings_type)
            .await
            .with_context(|| format!("register_extension: upsert type '{}'", settings_type.key))?;

        let mut cache = self.cache.write().await;
        for setting in defaults {
            // Ensure the FK points to the id that actually exists in the DB.
            let setting = if setting.settings_type_id != actual_type_id {
                Settings {
                    settings_type_id: actual_type_id,
                    ..setting
                }
            } else {
                setting
            };
            // Only insert if not already present (don't overwrite existing values).
            if self.persistence.get_setting(&setting.key).await?.is_none() {
                self.persistence
                    .upsert_setting(&setting)
                    .await
                    .with_context(|| {
                        format!("register_extension: seed setting '{}'", setting.key)
                    })?;
                cache.insert(
                    setting.key.clone(),
                    CacheEntry {
                        setting: setting.clone(),
                        meta: SettingsMeta::from_config(SettingSource::Default),
                    },
                );
            }
        }
        Ok(())
    }

    // =========================================================================
    // Public: Consumer API
    // =========================================================================

    /// Get a raw JSON value by dotted key — cache-first, then DB fallback.
    pub async fn get_value(&self, key: &str) -> Option<Value> {
        {
            let cache = self.cache.read().await;
            if let Some(entry) = cache.get(key) {
                return Some(entry.setting.data.clone());
            }
        }
        // DB fallback (should be rare after initialize()).
        self.persistence
            .get_setting(key)
            .await
            .ok()
            .flatten()
            .map(|s| s.data)
    }

    /// Deserialise a typed value by key.
    pub async fn get_typed<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>> {
        match self.get_value(key).await {
            Some(v) => Ok(Some(serde_json::from_value(v)?)),
            None => Ok(None),
        }
    }

    /// Set a setting value via the API. The write is validated against the type's
    /// JSON Schema before being persisted.
    pub async fn set_value(&self, key: &str, value: Value) -> Result<()> {
        // Retrieve the existing DB record (we need the type + id).
        let existing = self
            .persistence
            .get_setting(key)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Setting '{key}' not found"))?;

        let updated = Settings {
            data: value.clone(),
            updated_at: Some(Utc::now()),
            ..existing
        };

        // upsert_setting validates against schema.
        self.persistence.upsert_setting(&updated).await?;

        // Update cache with Api source.
        let mut cache = self.cache.write().await;
        cache.insert(
            key.to_string(),
            CacheEntry {
                setting: updated,
                meta: SettingsMeta {
                    source: SettingSource::Api,
                    is_drift: false,
                },
            },
        );
        Ok(())
    }

    /// List all settings that are currently flagged as drifted.
    pub async fn list_drift(&self) -> Vec<SettingsWithMeta> {
        let cache = self.cache.read().await;
        cache
            .values()
            .filter(|e| e.meta.is_drift)
            .map(|e| SettingsWithMeta {
                setting: e.setting.clone(),
                meta: e.meta.clone(),
            })
            .collect()
    }

    /// List all settings with their transient metadata (for the REST API).
    pub async fn list_all_with_meta(&self) -> Vec<SettingsWithMeta> {
        let cache = self.cache.read().await;
        cache
            .values()
            .map(|e| SettingsWithMeta {
                setting: e.setting.clone(),
                meta: e.meta.clone(),
            })
            .collect()
    }

    /// List settings for a specific namespace (type key prefix).
    pub async fn list_namespace_with_meta(&self, type_key: &str) -> Vec<SettingsWithMeta> {
        let prefix = format!("{type_key}.");
        let cache = self.cache.read().await;
        cache
            .values()
            .filter(|e| e.setting.key.starts_with(&prefix) || e.setting.key == type_key)
            .map(|e| SettingsWithMeta {
                setting: e.setting.clone(),
                meta: e.meta.clone(),
            })
            .collect()
    }

    /// Get a single setting with its metadata.
    pub async fn get_with_meta(&self, key: &str) -> Option<SettingsWithMeta> {
        let cache = self.cache.read().await;
        cache.get(key).map(|e| SettingsWithMeta {
            setting: e.setting.clone(),
            meta: e.meta.clone(),
        })
    }

    /// Reset a setting to its compiled-in default by deleting the DB override.
    /// On next boot, `initialize()` will reseed the default from config.
    pub async fn reset_to_default(&self, key: &str) -> Result<()> {
        self.persistence.delete_setting(key).await?;
        let mut cache = self.cache.write().await;
        cache.remove(key);
        Ok(())
    }

    /// List all registered settings types.
    pub async fn list_types(&self) -> Result<Vec<SettingsType>> {
        self.persistence.list_settings_types().await
    }

    /// Get a settings type by key.
    pub async fn get_type(&self, key: &str) -> Result<Option<SettingsType>> {
        self.persistence.get_settings_type(key).await
    }

    // =========================================================================
    // Public: LLM provider persistence (`provider.{id}` settings rows)
    // =========================================================================

    /// Seed the env/YAML/CLI-configured providers (held in the in-memory
    /// [`ProviderRegistry`] after `seed_from_llm_config` / `seed_from_configs`)
    /// into the DB on **first boot**, then persist the default provider id.
    ///
    /// Source-of-truth rule: **DB-wins-after-first-boot.** A provider whose
    /// `provider.{id}` row already exists is left untouched — it may have been
    /// edited via the admin API, and re-seeding would clobber that edit. Only
    /// rows absent from the DB are inserted, tagged [`SettingSource::ConfigFile`]
    /// (NOT `Api`, so a later config change can still reconcile them the way
    /// `initialize()` reconciles scalar settings).
    ///
    /// Call this AFTER [`Self::initialize`] (which creates the `provider`
    /// settings type) and BEFORE `hydrate_provider_registry_from_settings`.
    pub async fn seed_providers_from_registry(
        &self,
        registry: &crate::llm::registry::ProviderRegistry,
    ) -> Result<usize> {
        // The `provider` settings type is created by initialize(); bail clearly
        // if it isn't present rather than silently writing orphan rows.
        let st = self
            .persistence
            .get_settings_type("provider")
            .await?
            .ok_or_else(|| {
                anyhow::anyhow!("settings type 'provider' not found; call initialize() first")
            })?;

        let configured = registry.list().await;
        let mut seeded = 0usize;
        for config in &configured {
            let key = format!("provider.{}", config.id);
            // DB-wins-after-first-boot: never overwrite an existing row.
            if self.persistence.get_setting(&key).await?.is_some() {
                continue;
            }
            let data = serde_json::to_value(config).context("serialize ProviderConfig")?;
            let now = Utc::now();
            let setting = Settings {
                id: Uuid::new_v4(),
                settings_type_id: st.id,
                name: config.display_name.clone(),
                key: key.clone(),
                data,
                parent_id: None,
                created_at: now,
                updated_at: Some(now),
            };
            self.persistence
                .upsert_setting(&setting)
                .await
                .with_context(|| format!("seeding provider setting '{key}'"))?;
            // Config-sourced (not Api) so config can still reconcile later.
            let mut cache = self.cache.write().await;
            cache.insert(
                key,
                CacheEntry {
                    setting,
                    meta: SettingsMeta::from_config(SettingSource::ConfigFile),
                },
            );
            seeded += 1;
        }

        // Persist the default provider id (`llm.default_provider`) on first boot
        // so the UI shows which provider is the default, mirroring the registry's
        // notion of default. Only set it if not already persisted (DB wins).
        if self.get_default_provider_id().await.is_none()
            && let Some(default_id) = registry.default_id().await
            && let Err(e) = self.set_default_provider_id(&default_id).await
        {
            tracing::warn!(error = ?e, "failed to persist default provider id on seed");
        }

        if seeded > 0 {
            tracing::info!(seeded, "Seeded configured providers into the settings DB");
        }
        Ok(seeded)
    }

    /// Upsert a full provider configuration to `provider.{id}` (insert or update).
    ///
    /// Unlike [`Self::set_value`], this does **not** require a pre-existing row: new
    /// providers created only via the API are inserted on first save.
    pub async fn upsert_provider_config(
        &self,
        config: &crate::llm::registry::ProviderConfig,
    ) -> Result<()> {
        let st = self
            .persistence
            .get_settings_type("provider")
            .await?
            .ok_or_else(|| {
                anyhow::anyhow!("settings type 'provider' not found; call initialize() first")
            })?;

        let key = format!("provider.{}", config.id);
        let data = serde_json::to_value(config).context("serialize ProviderConfig")?;

        let existing = self.persistence.get_setting(&key).await?;
        let now = Utc::now();
        let setting = Settings {
            id: existing.as_ref().map(|e| e.id).unwrap_or_else(Uuid::new_v4),
            settings_type_id: st.id,
            name: config.display_name.clone(),
            key: key.clone(),
            data,
            parent_id: None,
            created_at: existing.as_ref().map(|e| e.created_at).unwrap_or(now),
            updated_at: Some(now),
        };

        self.persistence
            .upsert_setting(&setting)
            .await
            .with_context(|| format!("upsert provider setting '{key}'"))?;

        let mut cache = self.cache.write().await;
        cache.insert(
            key,
            CacheEntry {
                setting,
                meta: SettingsMeta {
                    source: SettingSource::Api,
                    is_drift: false,
                },
            },
        );
        Ok(())
    }

    /// Remove `provider.{id}` from the database and cache.
    pub async fn delete_provider_config(&self, provider_id: &str) -> Result<()> {
        let key = format!("provider.{provider_id}");
        self.persistence
            .delete_setting(&key)
            .await
            .with_context(|| format!("delete provider setting '{key}'"))?;
        let mut cache = self.cache.write().await;
        cache.remove(&key);
        Ok(())
    }

    /// Load all persisted provider configs from the database.
    pub async fn load_provider_configs_from_db(
        &self,
    ) -> Result<Vec<crate::llm::registry::ProviderConfig>> {
        let rows = self
            .persistence
            .list_settings(Some("provider"), None)
            .await?;
        let mut out = Vec::with_capacity(rows.len());
        for s in rows {
            if !s.key.starts_with("provider.") {
                continue;
            }
            let pc: crate::llm::registry::ProviderConfig = serde_json::from_value(s.data)
                .with_context(|| format!("deserialize ProviderConfig for setting key {}", s.key))?;
            out.push(pc);
        }
        Ok(out)
    }

    /// Persist the default provider id (`llm.default_provider`).
    pub async fn set_default_provider_id(&self, provider_id: &str) -> Result<()> {
        self.set_value("llm.default_provider", json!(provider_id))
            .await
    }

    /// Read the persisted default provider id, if any.
    pub async fn get_default_provider_id(&self) -> Option<String> {
        match self.get_value("llm.default_provider").await {
            Some(Value::String(s)) if !s.is_empty() => Some(s),
            Some(v) => v
                .as_str()
                .map(std::string::ToString::to_string)
                .filter(|s| !s.is_empty()),
            None => None,
        }
    }
}

// =============================================================================
// Core Config → (SettingsType, Vec<Settings>) mapping
// =============================================================================

/// Flatten the resolved `AppConfig` into a list of `(SettingsType, Vec<Settings>)` pairs.
///
/// One `SettingsType` per namespace. Each namespace has a JSON Schema that includes
/// `x-uar-ui` extension hints for the embedded React admin UI.
///
/// **Bootstrap-only fields are excluded:** `persistence.database_url` and
/// `persistence.provider` must come from config/env (the DB can't tell you how
/// to reach itself).
fn build_core_schema(config: &AppConfig) -> Vec<(SettingsType, Vec<Settings>)> {
    let mut result = Vec::new();

    // -------------------------------------------------------------------------
    // server
    // -------------------------------------------------------------------------
    {
        let schema = json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "title": "Server Configuration",
            "x-uar-ui": { "category": "Runtime", "icon": "server", "order": 1 },
            "type": "object",
            "required": ["port", "host"],
            "properties": {
                "port": {
                    "type": "integer", "minimum": 1, "maximum": 65535,
                    "title": "Port", "x-order": 1
                },
                "host": {
                    "type": "string", "title": "Host",
                    "x-control": "text", "x-order": 2
                },
                "shutdown_timeout_secs": {
                    "type": "integer", "minimum": 1, "title": "Shutdown Timeout (s)",
                    "x-control": "number", "x-order": 3
                },
                "log_format": {
                    "type": "string", "title": "Log Format",
                    "enum": ["json", "compact", "pretty"], "x-control": "select", "x-order": 4
                },
                "grpc_port": {
                    "type": "integer", "minimum": 1, "maximum": 65535,
                    "title": "A2A gRPC Port", "x-order": 5
                }
            }
        });
        let st = make_type("Server Configuration", "server", schema);
        let settings = vec![
            make_setting(&st, "server.port", "Port", json!(config.server.port)),
            make_setting(&st, "server.host", "Host", json!(config.server.host)),
            make_setting(
                &st,
                "server.shutdown_timeout_secs",
                "Shutdown Timeout (s)",
                json!(config.server.shutdown_timeout_secs),
            ),
            make_setting(
                &st,
                "server.log_format",
                "Log Format",
                json!(match config.server.log_format {
                    crate::config::LogFormat::Json => "json",
                    crate::config::LogFormat::Compact => "compact",
                    crate::config::LogFormat::Pretty => "pretty",
                }),
            ),
            make_setting(
                &st,
                "server.grpc_port",
                "A2A gRPC Port",
                json!(config.server.grpc_port),
            ),
        ];
        result.push((st, settings));
    }

    // -------------------------------------------------------------------------
    // security
    // -------------------------------------------------------------------------
    {
        let schema = json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "title": "Security Configuration",
            "x-uar-ui": { "category": "Runtime", "icon": "shield", "order": 2 },
            "type": "object",
            "required": ["jwt_required"],
            "properties": {
                "jwt_required": {
                    "type": "boolean", "title": "JWT Required",
                    "x-control": "toggle", "x-order": 1
                },
                "jwt_secret": {
                    "type": "string", "title": "JWT Secret",
                    "x-sensitive": true, "x-control": "password", "x-order": 2
                },
                "settings_mutation_auth_required": {
                    "type": "boolean", "title": "Require Admin Key for Settings Mutation",
                    "x-control": "toggle", "x-order": 3
                }
            }
        });
        let st = make_type("Security Configuration", "security", schema);
        let settings = vec![
            make_setting(
                &st,
                "security.jwt_required",
                "JWT Required",
                json!(config.security.jwt_required),
            ),
            make_setting(
                &st,
                "security.jwt_secret",
                "JWT Secret",
                json!(config.security.jwt_secret),
            ),
            make_setting(
                &st,
                "security.settings_mutation_auth_required",
                "Require Admin Key for Settings Mutation",
                json!(config.security.settings_mutation_auth_required),
            ),
        ];
        result.push((st, settings));
    }

    // -------------------------------------------------------------------------
    // resilience
    // -------------------------------------------------------------------------
    {
        let schema = json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "title": "Resilience Configuration",
            "x-uar-ui": { "category": "Runtime", "icon": "activity", "order": 3 },
            "type": "object",
            "properties": {
                "rate_limit_enabled": { "type": "boolean", "title": "Rate Limiting",
                    "x-control": "toggle", "x-order": 1 },
                "requests_per_second": { "type": "number", "minimum": 0.1, "title": "Req/s",
                    "x-control": "slider", "x-order": 2 },
                "burst_size": { "type": "number", "minimum": 1, "title": "Burst Size",
                    "x-control": "slider", "x-order": 3 },
                "request_timeout_ms": { "type": "integer", "minimum": 1000, "title": "Request Timeout (ms)",
                    "x-control": "slider", "x-order": 4 },
                "stream_start_timeout_ms": { "type": "integer", "minimum": 1000, "title": "Stream Start Timeout (ms)",
                    "x-control": "slider", "x-order": 5 },
                "retries_enabled": { "type": "boolean", "title": "Retries Enabled",
                    "x-control": "toggle", "x-order": 6 },
                "retry_max_attempts": { "type": "integer", "minimum": 0, "maximum": 10, "title": "Max Retry Attempts",
                    "x-control": "slider", "x-order": 7 },
                "retry_base_delay_ms": { "type": "integer", "minimum": 100, "title": "Retry Base Delay (ms)",
                    "x-control": "slider", "x-order": 8 },
                "retry_backoff_multiplier": { "type": "number", "minimum": 1.1, "maximum": 5.0, "title": "Retry Backoff Multiplier",
                    "x-control": "slider", "x-order": 9 },
                "retry_max_delay_ms": { "type": "integer", "minimum": 100, "title": "Retry Max Delay (ms)",
                    "x-control": "slider", "x-order": 10 },
                "retry_jitter_mode": {
                    "type": "string",
                    "enum": ["none", "full", "equal", "decorrelated"],
                    "title": "Retry Jitter Mode",
                    "x-control": "select",
                    "x-order": 11
                },
                "retry_respect_retry_after": { "type": "boolean", "title": "Respect Retry-After",
                    "x-control": "toggle", "x-order": 12 },
                "retryable_http_statuses": {
                    "type": "array",
                    "title": "Retryable HTTP Statuses",
                    "items": { "type": "integer", "minimum": 100, "maximum": 599 },
                    "minItems": 1,
                    "x-control": "tag-input",
                    "x-order": 13
                },
                "retryable_transport_errors": { "type": "boolean", "title": "Retryable Transport Errors",
                    "x-control": "toggle", "x-order": 14 },
                "retry_budget_ms": { "type": "integer", "minimum": 0, "title": "Retry Budget (ms)",
                    "x-control": "slider", "x-order": 15 },
                "timeout_disabled": {
                    "type": "boolean",
                    "title": "Disable Timeout (legacy)",
                    "x-control": "toggle",
                    "x-order": 99,
                    "x-deprecated": true
                }
            }
        });
        let st = make_type("Resilience Configuration", "resilience", schema);
        let r = &config.resilience;
        let settings = vec![
            make_setting(
                &st,
                "resilience.rate_limit_enabled",
                "Rate Limiting",
                json!(r.rate_limit_enabled),
            ),
            make_setting(
                &st,
                "resilience.requests_per_second",
                "Req/s",
                json!(r.requests_per_second),
            ),
            make_setting(
                &st,
                "resilience.burst_size",
                "Burst Size",
                json!(r.burst_size),
            ),
            make_setting(
                &st,
                "resilience.request_timeout_ms",
                "Request Timeout (ms)",
                json!(r.request_timeout_ms),
            ),
            make_setting(
                &st,
                "resilience.stream_start_timeout_ms",
                "Stream Start Timeout (ms)",
                json!(r.stream_start_timeout_ms),
            ),
            make_setting(
                &st,
                "resilience.retries_enabled",
                "Retries Enabled",
                json!(r.retries_enabled),
            ),
            make_setting(
                &st,
                "resilience.retry_max_attempts",
                "Max Retry Attempts",
                json!(r.retry_max_attempts),
            ),
            make_setting(
                &st,
                "resilience.retry_base_delay_ms",
                "Retry Base Delay (ms)",
                json!(r.retry_base_delay_ms),
            ),
            make_setting(
                &st,
                "resilience.retry_backoff_multiplier",
                "Retry Backoff Multiplier",
                json!(r.retry_backoff_multiplier),
            ),
            make_setting(
                &st,
                "resilience.retry_max_delay_ms",
                "Retry Max Delay (ms)",
                json!(r.retry_max_delay_ms),
            ),
            make_setting(
                &st,
                "resilience.retry_jitter_mode",
                "Retry Jitter Mode",
                json!(r.retry_jitter_mode),
            ),
            make_setting(
                &st,
                "resilience.retry_respect_retry_after",
                "Respect Retry-After",
                json!(r.retry_respect_retry_after),
            ),
            make_setting(
                &st,
                "resilience.retryable_http_statuses",
                "Retryable HTTP Statuses",
                json!(r.retryable_http_statuses),
            ),
            make_setting(
                &st,
                "resilience.retryable_transport_errors",
                "Retryable Transport Errors",
                json!(r.retryable_transport_errors),
            ),
            make_setting(
                &st,
                "resilience.retry_budget_ms",
                "Retry Budget (ms)",
                json!(r.retry_budget_ms),
            ),
            make_setting(
                &st,
                "resilience.timeout_disabled",
                "Timeout Disabled",
                json!(r.timeout_disabled),
            ),
        ];
        result.push((st, settings));
    }

    // -------------------------------------------------------------------------
    // persistence
    // -------------------------------------------------------------------------
    {
        let p = &config.persistence;
        let schema = json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "title": "Persistence",
            "x-uar-ui": { "category": "Infrastructure", "icon": "database", "order": 4 },
            "type": "object",
            "properties": {
                "provider": {
                    "type": "string", "title": "Provider",
                    "enum": ["surreal", "surrealdb", "postgres"], "x-control": "select"
                },
                "database_url": { "type": "string", "title": "Database URL", "x-control": "text" },
                "vector_dimension": { "type": "integer", "minimum": 1, "title": "Vector Dimension" },
                "external_cache_enabled": { "type": "boolean", "title": "External Cache Enabled", "x-control": "toggle" },
                "surreal_user": { "type": ["string", "null"], "title": "SurrealDB User", "x-control": "text" },
                "surreal_pass": { "type": ["string", "null"], "title": "SurrealDB Password", "x-sensitive": true, "x-control": "password" }
            }
        });
        let st = make_type("Persistence", "persistence", schema);
        let settings = vec![
            make_setting(&st, "persistence.provider", "Provider", json!(p.provider)),
            make_setting(
                &st,
                "persistence.database_url",
                "Database URL",
                json!(p.database_url),
            ),
            make_setting(
                &st,
                "persistence.vector_dimension",
                "Vector Dimension",
                json!(p.vector_dimension),
            ),
            make_setting(
                &st,
                "persistence.external_cache_enabled",
                "External Cache Enabled",
                json!(p.external_cache_enabled),
            ),
            make_setting(
                &st,
                "persistence.surreal_user",
                "SurrealDB User",
                json!(p.surreal_user),
            ),
            make_setting(
                &st,
                "persistence.surreal_pass",
                "SurrealDB Password",
                json!(p.surreal_pass.as_deref().unwrap_or("")),
            ),
        ];
        result.push((st, settings));
    }

    // -------------------------------------------------------------------------
    // file_processing
    // -------------------------------------------------------------------------
    {
        let schema = json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "title": "File Processing",
            "x-uar-ui": { "category": "Integrations", "icon": "file-text", "order": 4 },
            "type": "object",
            "properties": {
                "provider": { "type": "string", "title": "Provider", "default": "kreuzberg",
                    "enum": ["kreuzberg", "auto", "unstructured", "mistral", "local"],
                    "x-control": "select" },
                "upload_dir": { "type": "string", "title": "Upload Directory" },
                "max_files_per_prompt": { "type": "integer", "minimum": 1, "title": "Max Files Per Prompt" },
                "max_file_size": { "type": "integer", "minimum": 0, "title": "Max File Size (bytes)" },
                "max_total_size": { "type": "integer", "minimum": 0, "title": "Max Total Size (bytes)" },
                "allowed_mime_types": { "type": "array", "items": { "type": "string" },
                    "title": "Allowed MIME Types", "x-control": "multi-select" }
            }
        });
        let st = make_type("File Processing", "file_processing", schema);
        let fp = &config.file_processing;
        let settings = vec![
            make_setting(
                &st,
                "file_processing.provider",
                "Provider",
                json!(fp.provider),
            ),
            make_setting(
                &st,
                "file_processing.upload_dir",
                "Upload Directory",
                json!(fp.upload_dir),
            ),
            make_setting(
                &st,
                "file_processing.max_files_per_prompt",
                "Max Files Per Prompt",
                json!(fp.max_files_per_prompt),
            ),
            make_setting(
                &st,
                "file_processing.max_file_size",
                "Max File Size",
                json!(fp.max_file_size),
            ),
            make_setting(
                &st,
                "file_processing.max_total_size",
                "Max Total Size",
                json!(fp.max_total_size),
            ),
            make_setting(
                &st,
                "file_processing.allowed_mime_types",
                "Allowed MIME Types",
                json!(fp.allowed_mime_types),
            ),
        ];
        result.push((st, settings));
    }

    // -------------------------------------------------------------------------
    // vision
    // -------------------------------------------------------------------------
    {
        let schema = json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "title": "Vision Configuration",
            "x-uar-ui": { "category": "Integrations", "icon": "eye", "order": 5 },
            "type": "object",
            "properties": {
                "model": { "type": ["string", "null"], "title": "Vision Model Override", "x-control": "text" },
                "auto_detect": { "type": "boolean", "title": "Auto-detect Vision Capability", "x-control": "toggle" }
            }
        });
        let st = make_type("Vision Configuration", "vision", schema);
        let v = &config.vision;
        let settings = vec![
            make_setting(&st, "vision.model", "Model Override", json!(v.model)),
            make_setting(
                &st,
                "vision.auto_detect",
                "Auto-detect",
                json!(v.auto_detect),
            ),
        ];
        result.push((st, settings));
    }

    // -------------------------------------------------------------------------
    // models
    // -------------------------------------------------------------------------
    {
        let schema = json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "title": "Model Files",
            "x-uar-ui": { "category": "AI & LLM", "icon": "box", "order": 5 },
            "type": "object",
            "properties": {
                "models_dir": { "type": "string", "title": "Models Directory", "x-control": "text" },
                "vector_threshold": { "type": "number", "minimum": 0.0, "maximum": 1.0,
                    "title": "Vector Match Threshold", "x-control": "slider" }
            }
        });
        let st = make_type("Model Files", "models", schema);
        let settings = vec![
            make_setting(
                &st,
                "models.models_dir",
                "Models Directory",
                json!(config.models.models_dir),
            ),
            make_setting(
                &st,
                "models.vector_threshold",
                "Vector Match Threshold",
                json!(config.models.vector_threshold),
            ),
        ];
        result.push((st, settings));
    }

    // -------------------------------------------------------------------------
    // knowledge_bases (global defaults)
    // -------------------------------------------------------------------------
    {
        let schema = json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "title": "Knowledge Bases",
            "x-uar-ui": { "category": "Knowledge", "icon": "database", "order": 6, "display_mode": "master-detail" },
            "type": "object",
            "properties": {
                "default": { "type": ["object", "null"], "title": "Default KB Config" },
                "named": { "type": "object", "title": "Named KBs",
                    "additionalProperties": { "type": "object" } },
                "named_keys": { "type": "array", "items": { "type": "string" },
                    "title": "Named KB Keys" }
            }
        });
        let st = make_type("Knowledge Bases", "knowledge_bases", schema);
        let kb = &config.knowledge_bases;
        // KnowledgeBaseConfig doesn\u2019t necessarily implement Serialize,
        // so we manually construct the JSON value we want to store.
        let default_value = kb
            .default
            .as_ref()
            .map(|d| {
                json!({
                    "name": d.name,
                    "description": d.description,
                    "embedding_provider": d.embedding_provider,
                    "embedding_model": d.embedding_model,
                    "vector_dimensions": d.vector_dimensions,
                    "file_processor": d.file_processor,
                    "chunking_strategy": d.chunking.strategy,
                    "chunk_size": d.chunking.chunk_size,
                    "semantic_threshold": d.chunking.semantic_threshold,
                })
            })
            .unwrap_or(serde_json::Value::Null);
        let named_value = serde_json::Value::Object(
            kb.named
                .iter()
                .map(|(name, cfg)| {
                    (
                        name.clone(),
                        json!({
                            "name": cfg.name,
                            "description": cfg.description,
                            "embedding_provider": cfg.embedding_provider,
                            "embedding_model": cfg.embedding_model,
                            "vector_dimensions": cfg.vector_dimensions,
                            "file_processor": cfg.file_processor,
                            "chunking_strategy": cfg.chunking.strategy,
                            "chunk_size": cfg.chunking.chunk_size,
                            "semantic_threshold": cfg.chunking.semantic_threshold,
                        }),
                    )
                })
                .collect(),
        );
        let named_keys: Vec<&str> = kb.named.keys().map(String::as_str).collect();
        let settings = vec![
            make_setting(&st, "knowledge_bases.default", "Default KB", default_value),
            make_setting(&st, "knowledge_bases.named", "Named KBs", named_value),
            make_setting(
                &st,
                "knowledge_bases.named_keys",
                "Named KB Keys",
                json!(named_keys),
            ),
        ];
        result.push((st, settings));
    }

    // -------------------------------------------------------------------------
    // intent_classifier
    // -------------------------------------------------------------------------
    {
        let schema = json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "title": "Intent Classifier",
            "x-uar-ui": { "category": "Runtime", "icon": "zap", "order": 7 },
            "type": "object",
            "properties": {
                "backend": { "type": "string", "title": "Backend",
                    "enum": ["rules", "tfidf", "wasm", "hybrid", "localembedding", "llm"],
                    "x-control": "select" },
                "topk": { "type": "integer", "minimum": 1, "title": "Top-K" },
                "accept_threshold": { "type": "number", "minimum": 0.0, "maximum": 1.0,
                    "title": "Accept Threshold", "x-control": "slider" },
                "margin_threshold": { "type": "number", "minimum": 0.0, "maximum": 1.0,
                    "title": "Margin Threshold", "x-control": "slider" },
                "wasm_component_path": { "type": ["string", "null"], "title": "WASM Component Path" }
            }
        });
        let st = make_type("Intent Classifier", "intent_classifier", schema);
        let ic = &config.intent_classifier;
        let settings = vec![
            make_setting(
                &st,
                "intent_classifier.backend",
                "Backend",
                json!(ic.backend),
            ),
            make_setting(&st, "intent_classifier.topk", "Top-K", json!(ic.topk)),
            make_setting(
                &st,
                "intent_classifier.accept_threshold",
                "Accept Threshold",
                json!(ic.accept_threshold),
            ),
            make_setting(
                &st,
                "intent_classifier.margin_threshold",
                "Margin Threshold",
                json!(ic.margin_threshold),
            ),
            make_setting(
                &st,
                "intent_classifier.wasm_component_path",
                "WASM Path",
                json!(ic.wasm_component_path),
            ),
        ];
        result.push((st, settings));
    }

    // -------------------------------------------------------------------------
    // providers (LLM) — stored as a single blob per provider entry
    // -------------------------------------------------------------------------
    {
        let schema = json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "title": "LLM Provider",
            "x-uar-ui": { "category": "Providers", "icon": "server", "order": 8, "display_mode": "master-detail" },
            "type": "object",
            "required": ["id", "display_name", "base_url", "protocol"],
            "properties": {
                "id": { "type": "string", "title": "ID", "x-readonly": true },
                "display_name": { "type": "string", "title": "Display Name" },
                "base_url": { "type": "string", "title": "Base URL", "x-control": "url" },
                "api_key": { "type": "string", "title": "API Key", "x-sensitive": true, "x-control": "password" },
                "protocol": { "type": "string", "title": "Protocol",
                    "enum": ["auto", "chat", "responses"], "default": "auto", "x-control": "select" },
                "enabled": { "type": "boolean", "title": "Enabled", "x-control": "toggle", "default": true },
                "default_model": { "type": "string", "title": "Default Model" },
                "models": {
                    "type": "array", "title": "Models", "x-control": "model-card",
                    "items": {
                        "type": "object",
                        "properties": {
                            "id": { "type": "string" },
                            "display_name": { "type": "string" },
                            "enabled": { "type": "boolean", "default": true },
                            "context_window": { "type": "integer" },
                            "max_output_tokens": { "type": "integer" },
                            "capabilities": {
                                "type": "object", "x-control": "capabilities-grid",
                                "properties": {
                                    "supports_vision": { "type": "boolean", "title": "Vision" },
                                    "supports_tools": { "type": "boolean", "title": "Tool Use" },
                                    "supports_reasoning": { "type": "boolean", "title": "Reasoning" },
                                    "supports_embeddings": { "type": "boolean", "title": "Embeddings" },
                                    "supports_audio_input": { "type": "boolean", "title": "Audio Input" },
                                    "supports_audio_output": { "type": "boolean", "title": "Audio Output" },
                                    "supports_image_generation": { "type": "boolean", "title": "Image Generation" },
                                    "supports_streaming": { "type": "boolean", "title": "Streaming", "default": true },
                                    "supports_parallel_tool_calls": { "type": "boolean", "title": "Parallel Tools" },
                                    "supports_structured_output": { "type": "boolean", "title": "JSON Output" },
                                    "supports_prompt_caching": { "type": "boolean", "title": "Prompt Caching" },
                                    "supports_fine_tuning": { "type": "boolean", "title": "Fine-Tuning" }
                                }
                            },
                            "pricing": {
                                "type": "object", "x-control": "pricing-table",
                                "properties": {
                                    "input_per_1m": { "type": "number", "title": "Input/1M" },
                                    "output_per_1m": { "type": "number", "title": "Output/1M" },
                                    "cached_input_per_1m": { "type": "number", "title": "Cached/1M" },
                                    "reasoning_per_1m": { "type": "number", "title": "Reasoning/1M" },
                                    "embedding_per_1m": { "type": "number", "title": "Embedding/1M" },
                                    "image_input_per_1m": { "type": "number", "title": "Image/1M" },
                                    "audio_input_per_1m": { "type": "number", "title": "Audio/1M" },
                                    "currency": { "type": "string", "enum": ["USD", "EUR", "GBP"], "default": "USD" }
                                }
                            }
                        }
                    }
                }
            }
        });
        let st = make_type("LLM Provider", "provider", schema);
        let mut provider_settings = Vec::new();
        for p in &config.providers {
            let data = serde_json::to_value(p).unwrap_or(json!({}));
            let key = format!("provider.{}", p.id);
            let label = p.display_name.clone();
            provider_settings.push(make_setting(&st, &key, &label, data));
        }
        result.push((st, provider_settings));
    }

    // -------------------------------------------------------------------------
    // unstructured (optional)
    // -------------------------------------------------------------------------
    {
        let schema = json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "title": "Unstructured API",
            "x-uar-ui": { "category": "Integrations", "icon": "file-code", "order": 9 },
            "type": "object",
            "properties": {
                "api_url": { "type": "string", "title": "API URL", "x-control": "url" },
                "api_key": { "type": "string", "title": "API Key", "x-sensitive": true, "x-control": "password" }
            }
        });
        let st = make_type("Unstructured API", "unstructured", schema);
        let u = config.unstructured.clone().unwrap_or_default();
        let settings = vec![
            make_setting(&st, "unstructured.api_url", "API URL", json!(u.api_url)),
            // `Option` serializes as null; schema requires a string for this key.
            make_setting(
                &st,
                "unstructured.api_key",
                "API Key",
                json!(u.api_key.as_deref().unwrap_or("")),
            ),
        ];
        result.push((st, settings));
    }

    // -------------------------------------------------------------------------
    // mistral_ocr (optional)
    // -------------------------------------------------------------------------
    {
        let schema = json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "title": "Mistral OCR",
            "x-uar-ui": { "category": "Integrations", "icon": "scan", "order": 10 },
            "type": "object",
            "properties": {
                "api_key": { "type": "string", "title": "API Key", "x-sensitive": true, "x-control": "password" },
                "ocr_model": { "type": "string", "title": "OCR Model", "x-control": "select" }
            }
        });
        let st = make_type("Mistral OCR", "mistral_ocr", schema);
        let m = config.mistral_ocr.clone().unwrap_or_default();
        let settings = vec![
            make_setting(
                &st,
                "mistral_ocr.api_key",
                "API Key",
                json!(m.api_key.as_deref().unwrap_or("")),
            ),
            make_setting(
                &st,
                "mistral_ocr.ocr_model",
                "OCR Model",
                json!(m.ocr_model),
            ),
        ];
        result.push((st, settings));
    }

    // -------------------------------------------------------------------------
    // kreuzberg (optional)
    // -------------------------------------------------------------------------
    {
        let schema = json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "title": "Kreuzberg OCR",
            "x-uar-ui": { "category": "Integrations", "icon": "scan-line", "order": 11 },
            "type": "object",
            "properties": {
                "ocr_enabled": { "type": "boolean", "title": "OCR Enabled", "x-control": "toggle" },
                "force_ocr": { "type": "boolean", "title": "Force OCR", "x-control": "toggle" },
                "ocr_backend": { "type": "string", "title": "OCR Backend",
                    "enum": ["tesseract", "paddleocr", "easyocr"], "x-control": "select" },
                "ocr_language": { "type": "string", "title": "OCR Language", "x-control": "text" },
                "pdf_dpi": { "type": "integer", "minimum": 72, "maximum": 600,
                    "title": "PDF DPI", "x-control": "slider" },
                "extract_tables": { "type": "boolean", "title": "Extract Tables", "x-control": "toggle" },
                "extract_metadata": { "type": "boolean", "title": "Extract Metadata", "x-control": "toggle" },
                "output_format": { "type": "string", "title": "Output Format",
                    "enum": ["markdown", "text", "html"], "x-control": "select" }
            }
        });
        let st = make_type("Kreuzberg OCR", "kreuzberg", schema);
        let k = config.kreuzberg.clone().unwrap_or_default();
        let settings = vec![
            make_setting(
                &st,
                "kreuzberg.ocr_enabled",
                "OCR Enabled",
                json!(k.ocr_enabled),
            ),
            make_setting(&st, "kreuzberg.force_ocr", "Force OCR", json!(k.force_ocr)),
            make_setting(
                &st,
                "kreuzberg.ocr_backend",
                "OCR Backend",
                json!(k.ocr_backend),
            ),
            make_setting(
                &st,
                "kreuzberg.ocr_language",
                "OCR Language",
                json!(k.ocr_language),
            ),
            make_setting(&st, "kreuzberg.pdf_dpi", "PDF DPI", json!(k.pdf_dpi)),
            make_setting(
                &st,
                "kreuzberg.extract_tables",
                "Extract Tables",
                json!(k.extract_tables),
            ),
            make_setting(
                &st,
                "kreuzberg.extract_metadata",
                "Extract Metadata",
                json!(k.extract_metadata),
            ),
            make_setting(
                &st,
                "kreuzberg.output_format",
                "Output Format",
                json!(k.output_format),
            ),
            make_setting(
                &st,
                "kreuzberg.chunking.max_characters",
                "Chunk Max Characters",
                json!(k.chunking.as_ref().map_or(0usize, |c| c.max_characters)),
            ),
            make_setting(
                &st,
                "kreuzberg.chunking.overlap",
                "Chunk Overlap",
                json!(k.chunking.as_ref().map_or(0usize, |c| c.overlap)),
            ),
        ];
        result.push((st, settings));
    }

    // -------------------------------------------------------------------------
    // context_management — global context window strategy
    // -------------------------------------------------------------------------
    {
        let schema = json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "title": "Context Management",
            "x-uar-ui": { "category": "AI & LLM", "icon": "layers", "order": 9 },
            "type": "object",
            "properties": {
                "strategy": {
                    "type": "string", "title": "Strategy",
                    "enum": ["sliding_window", "keep_first_last", "progressive_summarization", "hierarchical_memory", "none"],
                    "default": "sliding_window", "x-control": "select"
                },
                "max_tokens": { "type": ["integer", "null"], "minimum": 512, "title": "Max Tokens", "description": "Leave blank to use model limit" },
                "trigger_threshold": { "type": "number", "minimum": 0.1, "maximum": 1.0, "title": "Trigger Threshold (0-1)", "x-control": "slider", "default": 0.85 },
                "max_messages": { "type": ["integer", "null"], "minimum": 1, "title": "Max Messages (sliding window)" },
                "summary_budget": { "type": "integer", "minimum": 100, "title": "Summary Budget (tokens)", "default": 1000 },
                "summarization_model": { "type": ["string", "null"], "title": "Summarization Model Override" }
            }
        });
        let st = make_type("Context Management", "context_management", schema);
        let settings = vec![
            make_setting(
                &st,
                "context_management.strategy",
                "Strategy",
                json!("sliding_window"),
            ),
            make_setting(
                &st,
                "context_management.max_tokens",
                "Max Tokens",
                serde_json::Value::Null,
            ),
            make_setting(
                &st,
                "context_management.trigger_threshold",
                "Trigger Threshold",
                json!(0.85),
            ),
            make_setting(
                &st,
                "context_management.max_messages",
                "Max Messages",
                serde_json::Value::Null,
            ),
            make_setting(
                &st,
                "context_management.summary_budget",
                "Summary Budget",
                json!(1000),
            ),
            make_setting(
                &st,
                "context_management.summarization_model",
                "Summarization Model",
                serde_json::Value::Null,
            ),
        ];
        result.push((st, settings));
    }

    // -------------------------------------------------------------------------
    // context_strategy — typed global conversation trimming strategy
    // -------------------------------------------------------------------------
    {
        let strategy_json = serde_json::to_value(&config.context_strategy).unwrap_or(json!({
            "type": "sliding_window",
            "max_messages": 20
        }));
        let strategy_type = strategy_json
            .get("type")
            .and_then(Value::as_str)
            .unwrap_or("sliding_window");
        let schema = json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "title": "Context Strategy",
            "x-uar-ui": { "category": "AI & LLM", "icon": "layers", "order": 9, "display_mode": "form" },
            "type": "object",
            "properties": {
                "type": {
                    "type": "string", "title": "Strategy",
                    "enum": ["none", "sliding_window", "summarize", "truncate_middle", "hierarchical"],
                    "x-control": "select"
                },
                "config": { "type": "object", "title": "Strategy Config", "x-control": "json" }
            }
        });
        let st = make_type("Context Strategy", "context_strategy", schema);
        let settings = vec![
            make_setting(
                &st,
                "context_strategy.type",
                "Strategy",
                json!(strategy_type),
            ),
            make_setting(
                &st,
                "context_strategy.config",
                "Strategy Config",
                strategy_json,
            ),
        ];
        result.push((st, settings));
    }

    // -------------------------------------------------------------------------
    // rag — retrieval-augmented generation / chunking / embedding config
    // -------------------------------------------------------------------------
    {
        let schema = json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "title": "RAG & Chunking",
            "x-uar-ui": { "category": "AI & LLM", "icon": "scissors", "order": 10 },
            "type": "object",
            "properties": {
                "chunking_strategy": {
                    "type": "string", "title": "Chunking Strategy",
                    "enum": ["fixed_size", "token", "recursive", "sentence", "document", "semantic", "agentic"],
                    "default": "recursive", "x-control": "select"
                },
                "chunk_size": { "type": "integer", "minimum": 64, "title": "Chunk Size (chars)", "default": 1024 },
                "chunk_tokens": { "type": "integer", "minimum": 16, "title": "Chunk Size (tokens)", "default": 256 },
                "semantic_threshold": { "type": "number", "minimum": 0.0, "maximum": 1.0, "title": "Semantic Merge Threshold", "x-control": "slider", "default": 0.75 },
                "embedding_provider": { "type": "string", "title": "Embedding Provider ID" },
                "embedding_model": { "type": "string", "title": "Embedding Model ID" }
            }
        });
        let st = make_type("RAG & Chunking", "rag", schema);
        let kb = &config.knowledge_bases;
        let (ep, em) = kb
            .default
            .as_ref()
            .map(|d| (d.embedding_provider.clone(), d.embedding_model.clone()))
            .unwrap_or_default();
        let settings = vec![
            make_setting(
                &st,
                "rag.chunking_strategy",
                "Chunking Strategy",
                json!("recursive"),
            ),
            make_setting(&st, "rag.chunk_size", "Chunk Size", json!(1024)),
            make_setting(&st, "rag.chunk_tokens", "Chunk Tokens", json!(256)),
            make_setting(
                &st,
                "rag.semantic_threshold",
                "Semantic Threshold",
                json!(0.75),
            ),
            make_setting(
                &st,
                "rag.embedding_provider",
                "Embedding Provider",
                json!(ep),
            ),
            make_setting(&st, "rag.embedding_model", "Embedding Model", json!(em)),
        ];
        result.push((st, settings));
    }

    // -------------------------------------------------------------------------
    // governance — Cedar-based authorization defaults
    // -------------------------------------------------------------------------
    {
        let schema = json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "title": "Governance",
            "x-uar-ui": { "category": "Governance & Agents", "icon": "shield-check", "order": 11 },
            "type": "object",
            "properties": {
                "default_mode": {
                    "type": "string", "title": "Default Authorization Mode",
                    "enum": ["permit_all", "deny_all", "custom"],
                    "default": "permit_all", "x-control": "select"
                },
                "allowed_actions": {
                    "type": "array", "title": "Globally Allowed Actions",
                    "items": {
                        "type": "string",
                        "enum": ["execute_tool", "call_llm", "spawn_agent", "collaborate", "access_knowledge"]
                    },
                    "x-control": "multi-select"
                },
                "policy_reload_enabled": { "type": "boolean", "title": "Enable Hot Policy Reload", "default": true, "x-control": "toggle" }
            }
        });
        let st = make_type("Governance", "governance", schema);
        let all_actions = json!([
            "execute_tool",
            "call_llm",
            "spawn_agent",
            "collaborate",
            "access_knowledge"
        ]);
        let settings = vec![
            make_setting(
                &st,
                "governance.default_mode",
                "Default Mode",
                json!("permit_all"),
            ),
            make_setting(
                &st,
                "governance.allowed_actions",
                "Allowed Actions",
                all_actions,
            ),
            make_setting(
                &st,
                "governance.policy_reload_enabled",
                "Hot Reload",
                json!(true),
            ),
        ];
        result.push((st, settings));
    }

    // -------------------------------------------------------------------------
    // agent_config — per built-in agent configuration overrides
    // -------------------------------------------------------------------------
    {
        let resilience_override_schema = json!({
            "type": "object",
            "properties": {
                "mode": {
                    "type": "string",
                    "title": "Resilience Override Mode",
                    "enum": ["inherit", "override"],
                    "default": "inherit",
                    "x-control": "select"
                },
                "rate_limit_enabled": { "type": "boolean" },
                "requests_per_second": { "type": "number", "minimum": 0.1 },
                "burst_size": { "type": "number", "minimum": 1 },
                "request_timeout_ms": { "type": "integer", "minimum": 1000 },
                "stream_start_timeout_ms": { "type": "integer", "minimum": 1000 },
                "retries_enabled": { "type": "boolean" },
                "retry_max_attempts": { "type": "integer", "minimum": 0, "maximum": 10 },
                "retry_base_delay_ms": { "type": "integer", "minimum": 100 },
                "retry_backoff_multiplier": { "type": "number", "minimum": 1.1, "maximum": 5.0 },
                "retry_max_delay_ms": { "type": "integer", "minimum": 100 },
                "retry_jitter_mode": { "type": "string", "enum": ["none", "full", "equal", "decorrelated"] },
                "retry_respect_retry_after": { "type": "boolean" },
                "retryable_http_statuses": {
                    "type": "array",
                    "items": { "type": "integer", "minimum": 100, "maximum": 599 },
                    "minItems": 1
                },
                "retryable_transport_errors": { "type": "boolean" },
                "retry_budget_ms": { "type": "integer", "minimum": 0 }
            }
        });

        let agent_value_schema = json!({
            "type": "object",
            "properties": {
                "enabled": { "type": "boolean", "title": "Enabled", "default": true, "x-control": "toggle" },
                "context_strategy": {
                    "type": "string", "title": "Context Strategy Override",
                    "enum": ["inherit", "sliding_window", "keep_first_last", "progressive_summarization", "none"],
                    "default": "inherit", "x-control": "select"
                },
                "governance_mode": {
                    "type": "string", "title": "Governance Mode Override",
                    "enum": ["inherit", "permit_all", "deny_all"],
                    "default": "inherit", "x-control": "select"
                },
                "allowed_tools": {
                    "type": "array", "title": "Allowed Tools (empty = all)",
                    "items": { "type": "string" },
                    "x-control": "tag-input"
                },
                "resilience": resilience_override_schema
            },
            "required": ["enabled", "context_strategy", "governance_mode", "allowed_tools"]
        });

        let schema = json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "title": "Agent Configuration",
            "x-uar-ui": { "category": "Governance & Agents", "icon": "bot", "order": 12, "display_mode": "master-detail" },
            "type": "object",
            "additionalProperties": agent_value_schema
        });
        let st = make_type("Agent Configuration", "agent_config", schema);
        // Seed default config for the built-in orchestrated agent
        let default_agent_cfg = json!({
            "enabled": true,
            "context_strategy": "inherit",
            "governance_mode": "inherit",
            "allowed_tools": [],
            "resilience": {
                "mode": "inherit"
            }
        });
        let settings = vec![make_setting(
            &st,
            "agent_config.orchestrated",
            "Orchestrated Agent",
            default_agent_cfg,
        )];
        result.push((st, settings));
    }

    // -------------------------------------------------------------------------
    // skill_config — per built-in skill enabled/disabled configuration
    // -------------------------------------------------------------------------
    {
        let schema = json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "title": "Skill Configuration",
            "x-uar-ui": { "category": "Governance & Agents", "icon": "zap", "order": 13, "display_mode": "master-detail" },
            "type": "object",
            "properties": {
                "enabled": { "type": "boolean", "title": "Enabled", "default": true, "x-control": "toggle" },
                "description": { "type": "string", "title": "Description (override)" }
            }
        });
        let st = make_type("Skill Configuration", "skill_config", schema);
        let skills = [
            ("echo", "Echo — reflects input back to caller"),
            (
                "system_info",
                "System Info — returns runtime environment details",
            ),
            (
                "compiler_agent",
                "Skill Compiler — compiles new agent skills via LLM",
            ),
            (
                "update_section",
                "Update Section — compiler session section update tool",
            ),
            (
                "check_completeness",
                "Check Completeness — compiler completeness validation",
            ),
            (
                "compile_session",
                "Compile Session — finalizes compiler session and signs skill",
            ),
        ];
        let mut skill_settings = Vec::new();
        for (id, description) in &skills {
            let key = format!("skill_config.{id}");
            let data = json!({ "enabled": true, "description": description });
            skill_settings.push(make_setting(&st, &key, id, data));
        }
        result.push((st, skill_settings));
    }

    // -------------------------------------------------------------------------
    // memory (agent memory system)
    // -------------------------------------------------------------------------
    {
        let schema = json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "title": "Agent Memory",
            "x-uar-ui": { "category": "AI & LLM", "icon": "brain", "order": 2, "display_mode": "form" },
            "type": "object",
            "properties": {
                "enabled": { "type": "boolean", "title": "Enable Memory System",
                    "x-control": "toggle" },
                "auto_capture": { "type": "boolean", "title": "Auto-capture",
                    "x-control": "toggle" },
                "inject_context": { "type": "boolean", "title": "Inject Memory Context",
                    "x-control": "toggle" },
                "embedding_provider": { "type": "string", "title": "Embedding Provider",
                    "enum": ["openai", "cohere"], "x-control": "select" },
                "embedding_model": { "type": "string", "title": "Embedding Model",
                    "x-control": "text" },
                "openai_api_key": { "type": ["string", "null"], "title": "OpenAI API Key",
                    "x-sensitive": true, "x-control": "password" },
                "cohere_api_key": { "type": ["string", "null"], "title": "Cohere API Key",
                    "x-sensitive": true, "x-control": "password" },
                "max_context_tokens": { "type": "integer", "minimum": 0,
                    "title": "Max Context Tokens" },
                "vector_weight": { "type": "number", "minimum": 0.0, "maximum": 1.0,
                    "title": "Vector Search Weight", "x-control": "slider" },
                "bm25_weight": { "type": "number", "minimum": 0.0, "maximum": 1.0,
                    "title": "BM25 Search Weight", "x-control": "slider" },
                "db_path": { "type": "string", "title": "DB Path", "x-control": "text" },
                "mcp_http_enabled": { "type": "boolean", "title": "Expose via MCP HTTP",
                    "x-control": "toggle" },
                "mcp_http_path": { "type": "string", "title": "MCP HTTP Path",
                    "x-control": "text" },
                "namespace": { "type": "string", "title": "Namespace", "x-control": "text" },
                "database": { "type": "string", "title": "Database", "x-control": "text" },
                "surreal_endpoint": { "type": ["string", "null"], "title": "SurrealDB Endpoint",
                    "x-control": "url" },
                "surreal_user": { "type": ["string", "null"], "title": "SurrealDB User",
                    "x-control": "text" },
                "surreal_pass": { "type": ["string", "null"], "title": "SurrealDB Password",
                    "x-sensitive": true, "x-control": "password" }
            }
        });
        let st = make_type("Agent Memory", "memory", schema);
        let m = &config.memory;
        let settings = vec![
            make_setting(&st, "memory.enabled", "Enable Memory", json!(m.enabled)),
            make_setting(
                &st,
                "memory.auto_capture",
                "Auto-capture",
                json!(m.auto_capture),
            ),
            make_setting(
                &st,
                "memory.inject_context",
                "Inject Context",
                json!(m.inject_context),
            ),
            make_setting(
                &st,
                "memory.embedding_provider",
                "Embedding Provider",
                json!(m.embedding_provider),
            ),
            make_setting(
                &st,
                "memory.embedding_model",
                "Embedding Model",
                json!(m.embedding_model),
            ),
            make_setting(
                &st,
                "memory.openai_api_key",
                "OpenAI API Key",
                json!(m.openai_api_key.as_deref().unwrap_or("")),
            ),
            make_setting(
                &st,
                "memory.cohere_api_key",
                "Cohere API Key",
                json!(m.cohere_api_key.as_deref().unwrap_or("")),
            ),
            make_setting(
                &st,
                "memory.max_context_tokens",
                "Max Context Tokens",
                json!(m.max_context_tokens),
            ),
            make_setting(
                &st,
                "memory.vector_weight",
                "Vector Weight",
                json!(m.vector_weight),
            ),
            make_setting(
                &st,
                "memory.bm25_weight",
                "BM25 Weight",
                json!(m.bm25_weight),
            ),
            make_setting(&st, "memory.db_path", "DB Path", json!(m.db_path)),
            make_setting(
                &st,
                "memory.mcp_http_enabled",
                "MCP HTTP Enabled",
                json!(m.mcp_http_enabled),
            ),
            make_setting(
                &st,
                "memory.mcp_http_path",
                "MCP HTTP Path",
                json!(m.mcp_http_path),
            ),
            make_setting(&st, "memory.namespace", "Namespace", json!(m.namespace)),
            make_setting(&st, "memory.database", "Database", json!(m.database)),
            make_setting(
                &st,
                "memory.surreal_endpoint",
                "SurrealDB Endpoint",
                json!(m.surreal_endpoint),
            ),
            make_setting(
                &st,
                "memory.surreal_user",
                "SurrealDB User",
                json!(m.surreal_user),
            ),
            make_setting(
                &st,
                "memory.surreal_pass",
                "SurrealDB Password",
                json!(m.surreal_pass.as_deref().unwrap_or("")),
            ),
        ];
        result.push((st, settings));
    }

    // -------------------------------------------------------------------------
    // llm — unified liter-llm client configuration
    // -------------------------------------------------------------------------
    {
        let schema = json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "title": "LLM Configuration",
            "x-uar-ui": {
                "category": "AI & LLM",
                "icon": "cpu",
                "order": 1,
                "display_mode": "form"
            },
            "type": "object",
            "properties": {
                "model": {
                    "type": "string",
                    "title": "Default Model",
                    "description": "Model in provider/model format (e.g. openai/gpt-4o). See models.dev for the full list.",
                    "x-control": "text"
                },
                "api_key": {
                    "type": ["string", "null"],
                    "title": "API Key",
                    "description": "API key for the default provider. Stored encrypted.",
                    "x-control": "password"
                },
                "base_url": {
                    "type": ["string", "null"],
                    "title": "Base URL Override",
                    "description": "Override provider base URL (e.g. for Ollama or a custom proxy).",
                    "x-control": "url"
                },
                "protocol": {
                    "type": "string",
                    "title": "Protocol",
                    "enum": ["auto", "chat", "responses"],
                    "x-control": "select"
                },
                "timeout_secs": {
                    "type": "integer",
                    "title": "Request Timeout (seconds)",
                    "minimum": 1,
                    "maximum": 600,
                    "x-control": "number"
                },
                "max_retries": {
                    "type": "integer",
                    "title": "Max Retries",
                    "minimum": 0,
                    "maximum": 10,
                    "x-control": "number"
                },
                "parallel_tool_calls": {
                    "type": ["boolean", "null"],
                    "title": "Parallel Tool Calls",
                    "description": "Allow the model to issue multiple tool calls in a single turn.",
                    "x-control": "toggle"
                },
                "cost_tracking": {
                    "type": "boolean",
                    "title": "Cost Tracking",
                    "description": "Log per-request token cost via tracing.",
                    "x-control": "toggle"
                },
                "tracing": {
                    "type": "boolean",
                    "title": "OpenTelemetry Tracing",
                    "description": "Emit OTEL spans for every LLM request.",
                    "x-control": "toggle"
                },
                "cooldown_secs": {
                    "type": ["integer", "null"],
                    "title": "Error Cooldown (seconds)",
                    "minimum": 0,
                    "x-control": "number"
                },
                "health_check_secs": {
                    "type": ["integer", "null"],
                    "title": "Health Check Interval (seconds)",
                    "minimum": 0,
                    "x-control": "number"
                },
                "thinking_budget": {
                    "type": ["integer", "null"],
                    "title": "Thinking Budget",
                    "minimum": 0,
                    "x-control": "number"
                },
                "cache": {
                    "type": ["object", "null"],
                    "title": "Response Cache",
                    "properties": {
                        "max_entries": { "type": "integer", "minimum": 1 },
                        "ttl_secs": { "type": "integer", "minimum": 1 }
                    }
                },
                "budget": {
                    "type": ["object", "null"],
                    "title": "Budget",
                    "properties": {
                        "global_limit": { "type": "number", "minimum": 0 },
                        "model_limits": { "type": "object" },
                        "enforcement": { "type": "string", "enum": ["hard", "soft"] }
                    }
                },
                "rate_limit": {
                    "type": ["object", "null"],
                    "title": "Rate Limit",
                    "properties": {
                        "rpm": { "type": ["integer", "null"], "minimum": 1 },
                        "tpm": { "type": ["integer", "null"], "minimum": 1 }
                    }
                },
                "default_provider": {
                    "type": "string",
                    "title": "Default LLM Provider ID",
                    "description": "Which registered provider is default (e.g. openai). Persisted with provider settings.",
                    "x-control": "text"
                }
            },
            "required": ["model"]
        });
        let llm = &config.llm;
        let default_provider_seed = llm
            .model
            .split_once('/')
            .map(|(p, _)| p.to_string())
            .unwrap_or_else(|| "default".to_string());
        let st = make_type("LLM Configuration", "llm", schema);
        let settings = vec![
            make_setting(&st, "llm.model", "Default Model", json!(llm.model)),
            make_setting(&st, "llm.api_key", "API Key", json!(llm.api_key)),
            make_setting(&st, "llm.base_url", "Base URL", json!(llm.base_url)),
            make_setting(&st, "llm.protocol", "Protocol", json!(llm.protocol)),
            make_setting(
                &st,
                "llm.timeout_secs",
                "Timeout (s)",
                json!(llm.timeout_secs),
            ),
            make_setting(
                &st,
                "llm.max_retries",
                "Max Retries",
                json!(llm.max_retries),
            ),
            make_setting(
                &st,
                "llm.parallel_tool_calls",
                "Parallel Tool Calls",
                json!(llm.parallel_tool_calls),
            ),
            make_setting(
                &st,
                "llm.cost_tracking",
                "Cost Tracking",
                json!(llm.cost_tracking),
            ),
            make_setting(&st, "llm.tracing", "OTEL Tracing", json!(llm.tracing)),
            make_setting(
                &st,
                "llm.cooldown_secs",
                "Cooldown (s)",
                json!(llm.cooldown_secs),
            ),
            make_setting(
                &st,
                "llm.health_check_secs",
                "Health Check Interval (s)",
                json!(llm.health_check_secs),
            ),
            make_setting(
                &st,
                "llm.thinking_budget",
                "Thinking Budget",
                json!(llm.thinking_budget),
            ),
            make_setting(&st, "llm.cache", "Response Cache", json!(llm.cache)),
            make_setting(&st, "llm.budget", "Budget", json!(llm.budget)),
            make_setting(&st, "llm.rate_limit", "Rate Limit", json!(llm.rate_limit)),
            make_setting(
                &st,
                "llm.default_provider",
                "Default LLM Provider ID",
                json!(default_provider_seed),
            ),
        ];
        result.push((st, settings));
    }

    // -------------------------------------------------------------------------
    // llm_failover — runtime model failover configuration
    // -------------------------------------------------------------------------
    {
        let fo = &config.failover;
        let schema = json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "title": "LLM Failover",
            "x-uar-ui": { "category": "AI & LLM", "icon": "shield", "order": 15, "display_mode": "form" },
            "type": "object",
            "properties": {
                "enabled": { "type": "boolean", "title": "Enabled", "x-control": "toggle" },
                "strategy": {
                    "type": "string", "title": "Strategy",
                    "enum": ["priority", "round_robin", "cost_optimized"],
                    "default": "priority", "x-control": "select"
                },
                "error_threshold": {
                    "type": "integer", "title": "Error Threshold",
                    "minimum": 1, "x-control": "slider",
                    "x-slider-min": 1, "x-slider-max": 20
                },
                "cooldown_secs": {
                    "type": "integer", "title": "Cooldown (seconds)",
                    "minimum": 0, "x-control": "number"
                },
                "fallback_models": {
                    "type": "array", "title": "Fallback Models",
                    "items": {
                        "type": "object",
                        "properties": {
                            "model": { "type": "string" },
                            "api_key": { "type": ["string", "null"], "x-sensitive": true },
                            "base_url": { "type": ["string", "null"] }
                        },
                        "required": ["model"]
                    },
                    "x-control": "json"
                }
            }
        });
        let st = make_type("LLM Failover", "llm_failover", schema);
        let settings = vec![
            make_setting(&st, "llm_failover.enabled", "Enabled", json!(fo.enabled)),
            make_setting(
                &st,
                "llm_failover.strategy",
                "Strategy",
                serde_json::to_value(&fo.strategy).unwrap_or(json!("priority")),
            ),
            make_setting(
                &st,
                "llm_failover.error_threshold",
                "Error Threshold",
                json!(fo.error_threshold),
            ),
            make_setting(
                &st,
                "llm_failover.cooldown_secs",
                "Cooldown (seconds)",
                json!(fo.cooldown_secs),
            ),
            make_setting(
                &st,
                "llm_failover.fallback_models",
                "Fallback Models",
                json!(fo.fallback_models),
            ),
        ];
        result.push((st, settings));
    }

    // -------------------------------------------------------------------------
    // sandbox — code execution runtime configuration
    // -------------------------------------------------------------------------
    {
        let sb = &config.sandbox;
        let schema = json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "title": "Sandbox Runtime",
            "x-uar-ui": { "category": "Runtime", "icon": "terminal", "order": 15, "display_mode": "form" },
            "type": "object",
            "properties": {
                "runner": {
                    "type": "string", "title": "Runner",
                    "enum": ["auto", "microsandbox", "wasmtime", "remote"], "x-control": "select"
                },
                "remote_url": { "type": ["string", "null"], "title": "Remote URL", "x-control": "url" },
                "default_memory_mib": { "type": "integer", "minimum": 16, "title": "Default Memory (MiB)", "x-control": "number" },
                "default_timeout_secs": { "type": "integer", "minimum": 1, "title": "Default Timeout (s)", "x-control": "number" },
                "network_enabled": { "type": "boolean", "title": "Network Enabled", "x-control": "toggle" },
                "max_concurrent": { "type": "integer", "minimum": 1, "title": "Max Concurrent Sessions", "x-control": "number" }
            }
        });
        let st = make_type("Sandbox Runtime", "sandbox", schema);
        let settings = vec![
            make_setting(&st, "sandbox.runner", "Runner", json!(sb.runner)),
            make_setting(
                &st,
                "sandbox.remote_url",
                "Remote URL",
                json!(sb.remote_url),
            ),
            make_setting(
                &st,
                "sandbox.default_memory_mib",
                "Default Memory (MiB)",
                json!(sb.default_memory_mib),
            ),
            make_setting(
                &st,
                "sandbox.default_timeout_secs",
                "Default Timeout (s)",
                json!(sb.default_timeout_secs),
            ),
            make_setting(
                &st,
                "sandbox.network_enabled",
                "Network Enabled",
                json!(sb.network_enabled),
            ),
            make_setting(
                &st,
                "sandbox.max_concurrent",
                "Max Concurrent Sessions",
                json!(sb.max_concurrent),
            ),
        ];
        result.push((st, settings));
    }

    // -------------------------------------------------------------------------
    // sycophancy — response quality guardrail
    // -------------------------------------------------------------------------
    {
        let sy = &config.sycophancy;
        let schema = json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "title": "Sycophancy Detection",
            "x-uar-ui": { "category": "Governance & Agents", "icon": "shield", "order": 16, "display_mode": "form" },
            "type": "object",
            "properties": {
                "enabled": { "type": "boolean", "title": "Enabled", "x-control": "toggle" },
                "strictness": {
                    "type": "string", "title": "Strictness",
                    "enum": ["permissive", "standard", "strict", "adversarial"], "x-control": "select"
                },
                "auto_correct_threshold": { "type": "number", "minimum": 0.0, "maximum": 1.0, "title": "Auto-correct Threshold", "x-control": "slider" },
                "reflect_threshold": { "type": "number", "minimum": 0.0, "maximum": 1.0, "title": "Reflect Threshold", "x-control": "slider" },
                "log_only": { "type": "boolean", "title": "Log Only", "x-control": "toggle" }
            }
        });
        let st = make_type("Sycophancy Detection", "sycophancy", schema);
        let settings = vec![
            make_setting(&st, "sycophancy.enabled", "Enabled", json!(sy.enabled)),
            make_setting(
                &st,
                "sycophancy.strictness",
                "Strictness",
                json!(sy.strictness),
            ),
            make_setting(
                &st,
                "sycophancy.auto_correct_threshold",
                "Auto-correct Threshold",
                json!(sy.auto_correct_threshold),
            ),
            make_setting(
                &st,
                "sycophancy.reflect_threshold",
                "Reflect Threshold",
                json!(sy.reflect_threshold),
            ),
            make_setting(&st, "sycophancy.log_only", "Log Only", json!(sy.log_only)),
        ];
        result.push((st, settings));
    }

    // -------------------------------------------------------------------------
    // native_tools — file, web-fetch, terminal, session-search tools
    // -------------------------------------------------------------------------
    {
        let nt = &config.native_tools;
        let schema = json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "title": "Native Tools",
            "x-uar-ui": { "category": "Runtime", "icon": "tool", "order": 16, "display_mode": "form" },
            "type": "object",
            "properties": {
                "file_tools_enabled": {
                    "type": "boolean", "title": "File Tools Enabled", "x-control": "toggle"
                },
                "file_allowed_paths": {
                    "type": "array", "title": "Allowed File Paths",
                    "items": { "type": "string" }, "x-control": "tag-input"
                },
                "file_max_size_kb": {
                    "type": "integer", "title": "Max Read Size (KB)",
                    "minimum": 1, "x-control": "number"
                },
                "file_write_max_kb": {
                    "type": "integer", "title": "Max Write Size (KB)",
                    "minimum": 1, "x-control": "number"
                },
                "web_fetch_enabled": {
                    "type": "boolean", "title": "Web Fetch Enabled", "x-control": "toggle"
                },
                "web_fetch_timeout_secs": {
                    "type": "integer", "title": "Web Fetch Timeout (s)",
                    "minimum": 1, "x-control": "slider",
                    "x-slider-min": 5, "x-slider-max": 120
                },
                "web_fetch_max_size_kb": {
                    "type": "integer", "title": "Web Fetch Max Size (KB)",
                    "minimum": 1, "x-control": "number"
                },
                "web_fetch_allowed_domains": {
                    "type": "array", "title": "Allowed Domains",
                    "items": { "type": "string" }, "x-control": "tag-input"
                },
                "terminal_exec_enabled": {
                    "type": "boolean", "title": "Terminal Exec Enabled", "x-control": "toggle"
                },
                "terminal_shell": {
                    "type": "string", "title": "Shell", "default": "sh", "x-control": "text"
                },
                "terminal_timeout_secs": {
                    "type": "integer", "title": "Terminal Timeout (s)",
                    "minimum": 1, "x-control": "number"
                },
                "terminal_use_sandbox": {
                    "type": "boolean", "title": "Use Sandbox", "x-control": "toggle"
                },
                "session_search_enabled": {
                    "type": "boolean", "title": "Session Search Enabled", "x-control": "toggle"
                },
                "session_search_max_results": {
                    "type": "integer", "title": "Max Search Results",
                    "minimum": 1, "x-control": "slider",
                    "x-slider-min": 1, "x-slider-max": 50
                }
            }
        });
        let st = make_type("Native Tools", "native_tools", schema);
        let settings = vec![
            make_setting(
                &st,
                "native_tools.file_tools_enabled",
                "File Tools Enabled",
                json!(nt.file_tools_enabled),
            ),
            make_setting(
                &st,
                "native_tools.file_allowed_paths",
                "Allowed File Paths",
                json!(nt.file_allowed_paths),
            ),
            make_setting(
                &st,
                "native_tools.file_max_size_kb",
                "Max Read Size (KB)",
                json!(nt.file_max_size_kb),
            ),
            make_setting(
                &st,
                "native_tools.file_write_max_kb",
                "Max Write Size (KB)",
                json!(nt.file_write_max_kb),
            ),
            make_setting(
                &st,
                "native_tools.web_fetch_enabled",
                "Web Fetch Enabled",
                json!(nt.web_fetch_enabled),
            ),
            make_setting(
                &st,
                "native_tools.web_fetch_timeout_secs",
                "Web Fetch Timeout (s)",
                json!(nt.web_fetch_timeout_secs),
            ),
            make_setting(
                &st,
                "native_tools.web_fetch_max_size_kb",
                "Web Fetch Max Size (KB)",
                json!(nt.web_fetch_max_size_kb),
            ),
            make_setting(
                &st,
                "native_tools.web_fetch_allowed_domains",
                "Allowed Domains",
                json!(nt.web_fetch_allowed_domains),
            ),
            make_setting(
                &st,
                "native_tools.terminal_exec_enabled",
                "Terminal Exec Enabled",
                json!(nt.terminal_exec_enabled),
            ),
            make_setting(
                &st,
                "native_tools.terminal_shell",
                "Shell",
                json!(nt.terminal_shell),
            ),
            make_setting(
                &st,
                "native_tools.terminal_timeout_secs",
                "Terminal Timeout (s)",
                json!(nt.terminal_timeout_secs),
            ),
            make_setting(
                &st,
                "native_tools.terminal_use_sandbox",
                "Use Sandbox",
                json!(nt.terminal_use_sandbox),
            ),
            make_setting(
                &st,
                "native_tools.session_search_enabled",
                "Session Search Enabled",
                json!(nt.session_search_enabled),
            ),
            make_setting(
                &st,
                "native_tools.session_search_max_results",
                "Max Search Results",
                json!(nt.session_search_max_results),
            ),
        ];
        result.push((st, settings));
    }

    // -------------------------------------------------------------------------
    // skill_evolution — Hermes learning cycle / auto-skill creation
    // -------------------------------------------------------------------------
    {
        let se = &config.skill_evolution;
        let schema = json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "title": "Skill Evolution",
            "x-uar-ui": {
                "category": "Governance & Agents", "icon": "sparkles",
                "order": 17, "display_mode": "form"
            },
            "type": "object",
            "properties": {
                "enabled": { "type": "boolean", "title": "Enabled", "x-control": "toggle" },
                "trigger_model": {
                    "type": ["string", "null"], "title": "Trigger Model", "x-control": "text"
                },
                "min_tool_calls": {
                    "type": "integer", "title": "Min Tool Calls to Trigger",
                    "minimum": 1, "x-control": "slider",
                    "x-slider-min": 1, "x-slider-max": 20
                },
                "max_skills_per_run": {
                    "type": "integer", "title": "Max Skills per Run",
                    "minimum": 1, "x-control": "number"
                },
                "allow_update": {
                    "type": "boolean", "title": "Allow Updating Skills", "x-control": "toggle"
                },
                "allow_deletion": {
                    "type": "boolean", "title": "Allow Deleting Skills", "x-control": "toggle"
                },
                "min_executions_before_delete": {
                    "type": "integer", "title": "Min Executions Before Delete",
                    "minimum": 0, "x-control": "number"
                },
                "reflection_prompt": {
                    "type": ["string", "null"], "title": "Reflection Prompt", "x-control": "textarea"
                }
            }
        });
        let st = make_type("Skill Evolution", "skill_evolution", schema);
        let settings = vec![
            make_setting(&st, "skill_evolution.enabled", "Enabled", json!(se.enabled)),
            make_setting(
                &st,
                "skill_evolution.trigger_model",
                "Trigger Model",
                json!(se.trigger_model),
            ),
            make_setting(
                &st,
                "skill_evolution.min_tool_calls",
                "Min Tool Calls to Trigger",
                json!(se.min_tool_calls),
            ),
            make_setting(
                &st,
                "skill_evolution.max_skills_per_run",
                "Max Skills per Run",
                json!(se.max_skills_per_run),
            ),
            make_setting(
                &st,
                "skill_evolution.allow_update",
                "Allow Updating Skills",
                json!(se.allow_update),
            ),
            make_setting(
                &st,
                "skill_evolution.allow_deletion",
                "Allow Deleting Skills",
                json!(se.allow_deletion),
            ),
            make_setting(
                &st,
                "skill_evolution.min_executions_before_delete",
                "Min Executions Before Delete",
                json!(se.min_executions_before_delete),
            ),
            make_setting(
                &st,
                "skill_evolution.reflection_prompt",
                "Reflection Prompt",
                json!(se.reflection_prompt),
            ),
        ];
        result.push((st, settings));
    }

    // -------------------------------------------------------------------------
    // acp — BeeAI Agent Communication Protocol server endpoint
    // -------------------------------------------------------------------------
    {
        let acp = &config.acp;
        let schema = json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "title": "ACP Server",
            "x-uar-ui": { "category": "Protocols", "icon": "plug", "order": 18, "display_mode": "form" },
            "type": "object",
            "properties": {
                "enabled": { "type": "boolean", "title": "Enabled", "x-control": "toggle" },
                "path": {
                    "type": "string", "title": "Mount Path",
                    "default": "/acp", "x-control": "text"
                },
                "auth_required": {
                    "type": "boolean", "title": "Auth Required", "x-control": "toggle"
                },
                "session_ttl_secs": {
                    "type": "integer", "title": "Session TTL (seconds)",
                    "minimum": 60, "x-control": "number"
                }
            }
        });
        let st = make_type("ACP Server", "acp", schema);
        let settings = vec![
            make_setting(&st, "acp.enabled", "Enabled", json!(acp.enabled)),
            make_setting(&st, "acp.path", "Mount Path", json!(acp.path)),
            make_setting(
                &st,
                "acp.auth_required",
                "Auth Required",
                json!(acp.auth_required),
            ),
            make_setting(
                &st,
                "acp.session_ttl_secs",
                "Session TTL (seconds)",
                json!(acp.session_ttl_secs),
            ),
        ];
        result.push((st, settings));
    }

    result
}

// =============================================================================
// Helpers
// =============================================================================

fn make_type(name: &str, key: &str, schema: Value) -> SettingsType {
    SettingsType {
        id: Uuid::new_v4(),
        name: name.to_string(),
        key: key.to_string(),
        schema,
        created_at: Utc::now(),
        updated_at: None,
    }
}

fn make_setting(st: &SettingsType, key: &str, name: &str, data: Value) -> Settings {
    Settings {
        id: Uuid::new_v4(),
        settings_type_id: st.id,
        name: name.to_string(),
        key: key.to_string(),
        data,
        parent_id: None,
        created_at: Utc::now(),
        updated_at: None,
    }
}
