//! Prompt-caching settings and priority resolution.
//!
//! Defines the hierarchy by which effective prompt-caching state is determined
//! for any given request:
//!
//! ```text
//! session_override → user_setting → agent_setting → global_setting
//! (highest priority)                               (lowest priority)
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

/// Stored per-user prompt-caching preferences.
///
/// Keyed by `user_id`; persisted while the process is running and restored
/// from the database on startup (when a persistence layer is configured).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPromptCachingSettings {
    /// User identity that owns this record.
    pub user_id: String,
    /// Whether prompt caching is globally enabled for this user.
    ///
    /// `None` means "inherit from the system global setting".
    pub prompt_caching_enabled: Option<bool>,
    /// Which level the user prefers for the session-level scope.
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
/// 1. `session_override` — per-request body flag (e.g. from the chat toolbar)
/// 2. `user_setting`     — stored user preference (requires JWT)
/// 3. `agent_setting`    — per-agent default
/// 4. `global_setting`   — system-wide default from settings manager
#[must_use]
pub fn resolve_effective_caching(
    session_override: Option<bool>,
    user_setting: Option<bool>,
    agent_setting: Option<bool>,
    global_setting: bool,
) -> bool {
    session_override
        .or(user_setting)
        .or(agent_setting)
        .unwrap_or(global_setting)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn session_override_takes_highest_priority() {
        assert!(resolve_effective_caching(
            Some(true),
            Some(false),
            Some(false),
            false
        ));
        assert!(!resolve_effective_caching(
            Some(false),
            Some(true),
            Some(true),
            true
        ));
    }

    #[test]
    fn user_setting_used_when_no_session_override() {
        assert!(resolve_effective_caching(
            None,
            Some(true),
            Some(false),
            false
        ));
        assert!(!resolve_effective_caching(
            None,
            Some(false),
            Some(true),
            true
        ));
    }

    #[test]
    fn agent_setting_used_when_no_user_setting() {
        assert!(resolve_effective_caching(None, None, Some(true), false));
        assert!(!resolve_effective_caching(None, None, Some(false), true));
    }

    #[test]
    fn global_setting_is_the_fallback() {
        assert!(resolve_effective_caching(None, None, None, true));
        assert!(!resolve_effective_caching(None, None, None, false));
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
