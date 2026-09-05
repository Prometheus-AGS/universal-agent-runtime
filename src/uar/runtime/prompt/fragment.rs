//! Typed prompt fragments and their stable content identity.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::assemble::PromptSection;

/// Model-visible authority carried by a prompt fragment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Authority {
    System,
    Policy,
    Host,
    Skill,
    Retrieved,
    User,
}

impl Authority {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Policy => "policy",
            Self::Host => "host",
            Self::Skill => "skill",
            Self::Retrieved => "retrieved",
            Self::User => "user",
        }
    }

    /// Markers make untrusted skill and retrieval bodies recognizable as data.
    #[must_use]
    pub const fn markers(self) -> (&'static str, &'static str) {
        match self {
            Self::Host => ("<uar-host-content>\n", "\n</uar-host-content>"),
            Self::Skill => ("<uar-skill-content>\n", "\n</uar-skill-content>"),
            Self::Retrieved => ("<uar-retrieved-content>\n", "\n</uar-retrieved-content>"),
            Self::System | Self::Policy | Self::User => ("", ""),
        }
    }
}

/// Response role that owns a fragment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PromptRole {
    System,
    User,
    Assistant,
    Tool,
}

impl PromptRole {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::User => "user",
            Self::Assistant => "assistant",
            Self::Tool => "tool",
        }
    }
}

/// How long a fragment remains eligible for reuse by later turns.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Retention {
    Session,
    Turn,
    Ephemeral,
    /// Reclaimed before history compaction and reattached from host state.
    Reclaimable,
}

/// One independently attributable unit of model-visible prompt context.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromptFragment {
    pub id: String,
    pub section: PromptSection,
    pub source: String,
    pub authority: Authority,
    pub role: PromptRole,
    pub retention: Retention,
    pub content_hash: String,
    pub content: String,
}

impl PromptFragment {
    #[must_use]
    pub fn new(
        id: impl Into<String>,
        section: PromptSection,
        source: impl Into<String>,
        authority: Authority,
        role: PromptRole,
        retention: Retention,
        content: impl Into<String>,
    ) -> Self {
        let id = id.into();
        let content = content.into();
        let content_hash = content_hash(role, section, &content);

        Self {
            id,
            section,
            source: source.into(),
            authority,
            role,
            retention,
            content_hash,
            content,
        }
    }

    #[must_use]
    pub const fn markers(&self) -> (&'static str, &'static str) {
        self.authority.markers()
    }

    #[must_use]
    pub fn marked_content(&self) -> String {
        let (start, end) = self.markers();
        if start.is_empty() && end.is_empty() {
            return self.content.clone();
        }

        let content = if self.authority == Authority::Host {
            // Project files cannot close the host envelope to imitate policy.
            self.content
                .replace('&', "&amp;")
                .replace('<', "&lt;")
                .replace('>', "&gt;")
        } else {
            self.content.clone()
        };
        format!("{start}{content}{end}")
    }
}

/// Stable SHA-256 identity over the response role, fragment kind, and body.
///
/// The section is the fragment's stable kind. Line endings are normalized so
/// equivalent Unix and Windows text has the same identity.
#[must_use]
pub fn content_hash(role: PromptRole, section: PromptSection, content: &str) -> String {
    let normalized = content.replace("\r\n", "\n");
    let mut hasher = Sha256::new();
    hasher.update(role.as_str().as_bytes());
    hasher.update([0]);
    hasher.update(section.as_str().as_bytes());
    hasher.update([0]);
    hasher.update(normalized.as_bytes());
    hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
