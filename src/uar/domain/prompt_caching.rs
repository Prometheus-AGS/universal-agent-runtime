//! Prompt-caching settings and priority resolution.
//!
//! Defines the hierarchy by which effective prompt-caching state is determined
//! for any given request:
//!
//! ```text
//! request_override → session_override → user_setting → global_setting
//! (highest priority)                                (lowest priority)
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Granularity at which prompt-caching preference is expressed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CachingScope {
    /// Per-conversation (session) override.
    #[default]
    Session,
    /// Per-user preference (requires JWT identity).
    User,
    /// Per-agent default applied to all sessions for that agent.
    Agent,
}

/// Source that supplied the effective prompt-caching value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PromptCachingSource {
    /// Explicit flag on this request.
    Request,
    /// Persisted conversation/session override.
    Session,
    /// Persisted JWT principal preference.
    User,
    /// System-wide default.
    Global,
}

/// Effective prompt-caching value and its authoritative source.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EffectivePromptCaching {
    /// Resolved on/off value.
    pub enabled: bool,
    /// First configured level in the precedence chain.
    pub source: PromptCachingSource,
}

/// Stored per-user prompt-caching preferences.
///
/// Keyed by a collision-safe verified principal identifier. Durable providers
/// store the record across restarts; the in-memory provider retains it only for
/// the process lifetime.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPromptCachingSettings {
    /// User identity that owns this record.
    pub user_id: String,
    /// Whether prompt caching is globally enabled for this user.
    ///
    /// `None` means "inherit from the system global setting".
    pub prompt_caching_enabled: Option<bool>,
    /// Deprecated compatibility field. It no longer affects precedence.
    #[serde(default)]
    pub preferred_scope: CachingScope,
    /// When this record was last modified.
    pub updated_at: DateTime<Utc>,
}

impl UserPromptCachingSettings {
    /// Create a new record for `user_id` with all defaults.
    #[must_use]
    pub fn new(user_id: impl Into<String>) -> Self {
        Self {
            user_id: user_id.into(),
            prompt_caching_enabled: None,
            preferred_scope: CachingScope::Session,
            updated_at: Utc::now(),
        }
    }
}

/// Resolve the effective prompt-caching flag for a request.
///
/// Priority (highest → lowest):
/// 1. `request_override` — per-request body flag
/// 2. `session_override` — persisted conversation override
/// 3. `user_setting`     — stored user preference (requires JWT)
/// 4. `global_setting`   — system-wide default from settings manager
#[must_use]
pub fn resolve_effective_caching(
    request_override: Option<bool>,
    session_override: Option<bool>,
    user_setting: Option<bool>,
    global_setting: bool,
) -> EffectivePromptCaching {
    if let Some(enabled) = request_override {
        EffectivePromptCaching {
            enabled,
            source: PromptCachingSource::Request,
        }
    } else if let Some(enabled) = session_override {
        EffectivePromptCaching {
            enabled,
            source: PromptCachingSource::Session,
        }
    } else if let Some(enabled) = user_setting {
        EffectivePromptCaching {
            enabled,
            source: PromptCachingSource::User,
        }
    } else {
        EffectivePromptCaching {
            enabled: global_setting,
            source: PromptCachingSource::Global,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn request_override_takes_highest_priority() {
        let enabled = resolve_effective_caching(Some(true), Some(false), Some(false), false);
        assert!(enabled.enabled);
        assert_eq!(enabled.source, PromptCachingSource::Request);

        let disabled = resolve_effective_caching(Some(false), Some(true), Some(true), true);
        assert!(!disabled.enabled);
        assert_eq!(disabled.source, PromptCachingSource::Request);
    }

    #[test]
    fn session_override_used_when_no_request_override() {
        let effective = resolve_effective_caching(None, Some(true), Some(false), false);
        assert!(effective.enabled);
        assert_eq!(effective.source, PromptCachingSource::Session);
    }

    #[test]
    fn user_setting_used_when_no_request_or_session_override() {
        let effective = resolve_effective_caching(None, None, Some(false), true);
        assert!(!effective.enabled);
        assert_eq!(effective.source, PromptCachingSource::User);
    }

    #[test]
    fn global_setting_is_the_fallback() {
        let enabled = resolve_effective_caching(None, None, None, true);
        assert!(enabled.enabled);
        assert_eq!(enabled.source, PromptCachingSource::Global);

        let disabled = resolve_effective_caching(None, None, None, false);
        assert!(!disabled.enabled);
        assert_eq!(disabled.source, PromptCachingSource::Global);
    }

    proptest! {
        #[test]
        fn user_prompt_caching_settings_serde_roundtrip(
            user_id in "[a-zA-Z0-9_-]{1,32}",
            prompt_caching_enabled in proptest::option::of(prop::bool::ANY),
            preferred_scope in prop::sample::select(&[
                CachingScope::Session,
                CachingScope::User,
                CachingScope::Agent,
            ]),
            updated_at in 0i64..4102444800i64,
        ) {
            let mut original = UserPromptCachingSettings::new(user_id);
            original.prompt_caching_enabled = prompt_caching_enabled;
            original.preferred_scope = preferred_scope;
            original.updated_at = DateTime::from_timestamp(updated_at, 0).unwrap_or(DateTime::UNIX_EPOCH);

            let json = serde_json::to_string(&original).expect("serialize");
            let deserialized: UserPromptCachingSettings = serde_json::from_str(&json).expect("deserialize");

            assert_eq!(original.user_id, deserialized.user_id);
            assert_eq!(original.prompt_caching_enabled, deserialized.prompt_caching_enabled);
            assert_eq!(original.preferred_scope, deserialized.preferred_scope);
            assert_eq!(original.updated_at, deserialized.updated_at);
        }
    }
}
