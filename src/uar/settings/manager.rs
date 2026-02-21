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
                }
            }
        });
        let st = make_type("Server Configuration", "server", schema);
        let settings = vec![
            make_setting(&st, "server.port", "Port", json!(config.server.port)),
            make_setting(&st, "server.host", "Host", json!(config.server.host)),
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
                "timeout_disabled": { "type": "boolean", "title": "Disable Timeout",
                    "x-control": "toggle", "x-order": 2 },
                "requests_per_second": { "type": "number", "minimum": 0.1, "title": "Req/s",
                    "x-control": "slider", "x-order": 3 },
                "burst_size": { "type": "number", "minimum": 1, "title": "Burst Size",
                    "x-control": "slider", "x-order": 4 }
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
                "resilience.timeout_disabled",
                "Timeout Disabled",
                json!(r.timeout_disabled),
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
                "provider": { "type": "string", "title": "Provider",
                    "enum": ["auto", "unstructured", "mistral", "kreuzberg"],
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
                    "additionalProperties": { "type": "object" } }
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
                    "embedding_provider": d.embedding_provider,
                    "embedding_model": d.embedding_model,
                    "chunking_strategy": d.chunking.strategy,
                    "chunk_size": d.chunking.chunk_size,
                })
            })
            .unwrap_or(serde_json::Value::Null);
        let named_keys: Vec<&str> = kb.named.keys().map(String::as_str).collect();
        let settings = vec![
            make_setting(&st, "knowledge_bases.default", "Default KB", default_value),
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
        let settings = if let Some(u) = &config.unstructured {
            vec![
                make_setting(&st, "unstructured.api_url", "API URL", json!(u.api_url)),
                make_setting(&st, "unstructured.api_key", "API Key", json!(u.api_key)),
            ]
        } else {
            vec![]
        };
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
                "model": { "type": "string", "title": "OCR Model", "x-control": "select" }
            }
        });
        let st = make_type("Mistral OCR", "mistral_ocr", schema);
        let settings = if let Some(m) = &config.mistral_ocr {
            vec![
                make_setting(&st, "mistral_ocr.api_key", "API Key", json!(m.api_key)),
                make_setting(
                    &st,
                    "mistral_ocr.ocr_model",
                    "OCR Model",
                    json!(m.ocr_model),
                ),
            ]
        } else {
            vec![]
        };
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
                "ocr_backend": { "type": "string", "title": "OCR Backend", "x-control": "select" },
                "output_format": { "type": "string", "title": "Output Format",
                    "enum": ["markdown", "plain", "html"], "x-control": "select" }
            }
        });
        let st = make_type("Kreuzberg OCR", "kreuzberg", schema);
        let settings = if let Some(k) = &config.kreuzberg {
            vec![
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
                    "kreuzberg.output_format",
                    "Output Format",
                    json!(k.output_format),
                ),
            ]
        } else {
            vec![]
        };
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
        let schema = json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "title": "Agent Configuration",
            "x-uar-ui": { "category": "Governance & Agents", "icon": "bot", "order": 12, "display_mode": "master-detail" },
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
                }
            }
        });
        let st = make_type("Agent Configuration", "agent_config", schema);
        // Seed default config for the built-in orchestrated agent
        let default_agent_cfg = json!({
            "enabled": true,
            "context_strategy": "inherit",
            "governance_mode": "inherit",
            "allowed_tools": []
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
