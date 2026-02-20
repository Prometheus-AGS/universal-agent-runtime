use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

// =============================================================================
// Database-persisted types
// =============================================================================

/// Defines a named category of settings (core UAR namespace or plugin-provided).
///
/// The `schema` field is a full JSON Schema (Draft 7) document. All writes to
/// `Settings.data` for this type are validated against it before being accepted.
/// This makes the system infinitely extensible — plugins declare their own type
/// with a JSON Schema and UAR validates writes automatically. No code changes
/// needed to add new setting domains.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
pub struct SettingsType {
    pub id: Uuid,
    /// Human-readable, globally unique. e.g. "Server Configuration"
    pub name: String,
    /// URL-safe slug, globally unique. e.g. "server" (name → lowercase + dashes).
    pub key: String,
    /// Required JSON Schema (Draft 7) for `.data` validation + UI generation.
    pub schema: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

/// A concrete setting value whose `data` must conform to the parent type's JSON Schema.
///
/// The `parent_id` field enables hierarchical nesting — e.g. per-knowledge-base
/// settings can be children of a root KB settings row.
#[allow(clippy::struct_field_names)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
pub struct Settings {
    pub id: Uuid,
    /// FK → settings_types.id
    pub settings_type_id: Uuid,
    /// Human-readable label for this record.
    pub name: String,
    /// Dotted key, globally unique. e.g. "server.port"
    pub key: String,
    /// JSONB data — validated against the parent type's JSON Schema at write time.
    pub data: Value,
    /// Optional FK → settings.id — enables nesting.
    pub parent_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

// =============================================================================
// Transient runtime-only types (NOT stored in DB)
// =============================================================================

/// Tracks where a setting value originated so drift can be detected on subsequent boots.
///
/// This is stored only in `SettingsManager`'s in-memory cache — never written to the DB.
/// This keeps the DB schema clean and extensible by plugins that know nothing about
/// `source` or `is_drift`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum SettingSource {
    #[default]
    Default,
    ConfigFile,
    EnvVar,
    CliArg,
    Api,
}

/// Transient metadata appended to settings in API responses.
///
/// Computed at runtime from boot-time state; never persisted.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsMeta {
    pub source: SettingSource,
    /// `true` when the DB value was set via the API and differs from the
    /// value that would be produced by the current config file/env.
    pub is_drift: bool,
}

impl SettingsMeta {
    pub fn from_config(source: SettingSource) -> Self {
        Self {
            source,
            is_drift: false,
        }
    }

    pub fn api_drift() -> Self {
        Self {
            source: SettingSource::Api,
            is_drift: true,
        }
    }
}

/// Statistics returned from `SettingsManager::initialize`.
#[derive(Debug, Default, Clone)]
pub struct BootstrapStats {
    /// Settings rows inserted for the first time.
    pub seeded: usize,
    /// Settings rows updated (config-sourced value changed since last boot).
    pub updated: usize,
    /// API-sourced settings that now differ from the current config (drift).
    pub drift_count: usize,
    /// Number of settings types upserted.
    pub types_upserted: usize,
}

/// Enriched API response that includes transient metadata alongside the stored value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsWithMeta {
    #[serde(flatten)]
    pub setting: Settings,
    pub meta: SettingsMeta,
}
