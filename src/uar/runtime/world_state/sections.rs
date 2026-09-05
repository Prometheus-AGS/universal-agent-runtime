//! Stable, secret-free world-state sections. Only the host captures these inputs.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::uar::domain::policy::EffectiveRunPolicy;
use crate::uar::runtime::project_instructions::InstructionFile;

/// Clock seam used by runtime capture and deterministic integration fixtures.
pub trait Clock: Send + Sync {
    fn unix_seconds(&self) -> i64;
}

/// Wall clock used by the production host.
#[derive(Debug, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn unix_seconds(&self) -> i64 {
        chrono::Utc::now().timestamp()
    }
}

/// Operator-configured time granularity; zero cannot be deserialized.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(default)]
pub struct WorldStateConfig {
    pub time_granularity_secs: std::num::NonZeroU32,
}

impl Default for WorldStateConfig {
    fn default() -> Self {
        Self {
            time_granularity_secs: std::num::NonZeroU32::MIN.saturating_add(59),
        }
    }
}

/// Stable identities shared by rendering, history baselines, and merge patches.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SectionId {
    Environment,
    CurrentTime,
    Permissions,
    ProjectInstructions,
}

impl SectionId {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Environment => "environment",
            Self::CurrentTime => "current_time",
            Self::Permissions => "permissions",
            Self::ProjectInstructions => "project_instructions",
        }
    }

    pub const fn replacement_text(self) -> &'static str {
        match self {
            Self::Environment => "This replaces the previous environment section.",
            Self::CurrentTime => "This replaces the previous current-time section.",
            Self::Permissions => {
                "This replaces the previous permissions summary; enforced policy remains authoritative."
            }
            Self::ProjectInstructions => {
                "This replaces the previous project instructions; system and enforced policy remain authoritative."
            }
        }
    }

    pub const fn removal_text(self) -> &'static str {
        match self {
            Self::Environment => "The previous environment section has been removed.",
            Self::CurrentTime => "The previous current-time section has been removed.",
            Self::Permissions => {
                "The previous permissions summary has been removed; enforced policy remains authoritative."
            }
            Self::ProjectInstructions => {
                "The previous project instructions have been removed and no longer apply."
            }
        }
    }
}

/// A complete host snapshot. Its object-valued sections contain no explicit
/// nullable members, so every change is representable by RFC 7386.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct WorldStateSnapshot {
    pub sections: BTreeMap<SectionId, Value>,
}

impl WorldStateSnapshot {
    /// Capture runtime environment without copying process environment variables.
    pub fn capture(
        cwd: &Path,
        workspace_roots: &[PathBuf],
        policy: &EffectiveRunPolicy,
        instructions: &[InstructionFile],
        clock: &dyn Clock,
        config: WorldStateConfig,
    ) -> Self {
        let mut roots = workspace_roots.to_vec();
        roots.sort();
        roots.dedup();
        let granularity = i64::from(config.time_granularity_secs.get());
        // Store the bucket index instead of its product to avoid timestamp
        // overflow even with a substituted clock at the i64 boundary.
        let bucket = clock.unix_seconds().div_euclid(granularity);
        Self {
            sections: BTreeMap::from([
                (
                    SectionId::Environment,
                    json!({
                        "cwd": cwd,
                        "workspace_roots": roots,
                        "platform": std::env::consts::OS,
                        "architecture": std::env::consts::ARCH,
                    }),
                ),
                (
                    SectionId::CurrentTime,
                    json!({
                        "unix_time_bucket": bucket,
                        "granularity_seconds": granularity,
                        "timezone": "UTC",
                    }),
                ),
                (
                    SectionId::Permissions,
                    json!({
                        "tools": policy.tools,
                        "mcp_servers": policy.mcp_servers,
                        "skills": policy.skills,
                        "knowledge_bases": policy.knowledge_bases,
                        "memory_enabled": policy.memory_enabled,
                        "tool_approval": policy.tool_approval,
                    }),
                ),
                (
                    SectionId::ProjectInstructions,
                    json!({ "files": instructions }),
                ),
            ]),
        }
    }
}
