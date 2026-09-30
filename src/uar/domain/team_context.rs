//! Host-selected team guidance and attributed context provenance.

use serde::{Deserialize, Deserializer, Serialize};

use super::team_wait::{AttemptAuthority, TargetOutcome};

pub const MAX_TEAM_INSTRUCTION_BYTES: usize = 16_384;
pub const MAX_ROSTER_LABEL_BYTES: usize = 256;
pub const MAX_ROSTER_CAPABILITIES: usize = 32;
pub const MAX_CONTEXT_ROSTER_MEMBERS: usize = 16;
pub const MAX_CONTEXT_SELECTIONS: usize = 128;
pub const MAX_ROSTER_PAGE_MEMBERS: u32 = 50;
pub const MAX_SAFE_COUNTER: u64 = 9_007_199_254_740_991;

/// Immutable guidance authorized by the installed team definition, not task input.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TeamInstructions {
    pub revision: u64,
    pub digest: String,
    pub text: String,
}

/// Public member identity; never contains another member's prompt or history.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RosterMember {
    pub member_id: String,
    pub role: String,
    pub label: String,
    pub safe_capabilities: Vec<String>,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ContextSourceKind {
    TeamInput,
    TaskInput,
    Artifact,
    Message,
    Skill,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ContextDisposition {
    Selected,
    Excluded,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContextSelection {
    pub source_id: String,
    pub source_kind: ContextSourceKind,
    pub disposition: ContextDisposition,
    #[serde(default, skip_serializing_if = "Option::is_none", deserialize_with = "present")]
    pub reason_code: Option<String>,
    pub original_bytes: u64,
    pub selected_bytes: u64,
    pub truncated: bool,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum TeamInstructionLayer {
    HostPolicy,
    TeamInstructions,
    MemberInstructions,
    TaskInstructions,
}

pub const TEAM_INSTRUCTION_ORDER: [TeamInstructionLayer; 4] = [
    TeamInstructionLayer::HostPolicy,
    TeamInstructionLayer::TeamInstructions,
    TeamInstructionLayer::MemberInstructions,
    TeamInstructionLayer::TaskInstructions,
];

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TeamContextCountQuality {
    Exact,
    Conservative,
    Unknown,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum TeamOutcomeDataTrust {
    UntrustedAttributedData,
}

/// Selection evidence. Recorded selection alone is not model-input consumption.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TeamContextReceipt {
    pub authority: AttemptAuthority,
    pub root_id: String,
    pub approval_scope_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none", deserialize_with = "present")]
    pub team_instructions: Option<TeamInstructions>,
    pub instruction_order: [TeamInstructionLayer; 4],
    #[serde(rename = "self")]
    pub self_member: RosterMember,
    pub coordinator_member_id: String,
    pub roster: Vec<RosterMember>,
    pub authorization_revision: u64,
    pub selections: Vec<ContextSelection>,
    pub target_outcomes: Vec<TargetOutcome>,
    pub target_outcome_data_trust: TeamOutcomeDataTrust,
    pub context_budget_tokens: u64,
    pub count_quality: TeamContextCountQuality,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RosterRequest {
    #[serde(default, skip_serializing_if = "Option::is_none", deserialize_with = "present")]
    pub cursor: Option<String>,
    pub limit: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RosterResult {
    pub members: Vec<RosterMember>,
    pub team_revision: u64,
    pub authorization_revision: u64,
    #[serde(default, skip_serializing_if = "Option::is_none", deserialize_with = "present")]
    pub next_cursor: Option<String>,
}

/// Optional fields may be omitted, but explicit null is not a profile value.
fn present<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    T::deserialize(deserializer).map(Some)
}
