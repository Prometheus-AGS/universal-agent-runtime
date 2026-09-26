use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Append-only, content-free evidence for one exact tool invocation.
///
/// Arguments, outputs, credentials, digests, and host authorization material
/// deliberately never enter this record. Persisted evidence explains what the
/// runtime attempted without becoming authority that can be replayed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolAdmissionEvidence {
    pub schema_version: u32,
    pub evidence_id: String,
    pub owner_id: String,
    pub root_run_id: String,
    pub run_id: String,
    pub invocation_id: String,
    pub admission_id: String,
    pub tool_name: String,
    pub runtime_epoch: String,
    pub host_epoch: String,
    pub state: ToolAdmissionEvidenceState,
    pub occurred_at: DateTime<Utc>,
}

impl ToolAdmissionEvidence {
    pub const SCHEMA_VERSION: u32 = 1;

    pub fn validate(&self) -> anyhow::Result<()> {
        anyhow::ensure!(
            self.schema_version == Self::SCHEMA_VERSION,
            "Unsupported tool admission evidence schema"
        );
        for value in [
            &self.evidence_id,
            &self.owner_id,
            &self.root_run_id,
            &self.run_id,
            &self.invocation_id,
            &self.admission_id,
            &self.tool_name,
            &self.runtime_epoch,
            &self.host_epoch,
        ] {
            anyhow::ensure!(!value.trim().is_empty(), "Tool admission evidence is incomplete");
        }
        Ok(())
    }

    /// Produce sanitized terminal evidence for authority left behind by an old
    /// runtime epoch. This never reconstructs or retries the invocation.
    pub(crate) fn after_runtime_restart(&self) -> Option<Self> {
        let state = match self.state {
            ToolAdmissionEvidenceState::AwaitingApproval => {
                ToolAdmissionEvidenceState::Interrupted
            }
            ToolAdmissionEvidenceState::ClaimIntent => {
                ToolAdmissionEvidenceState::OutcomeUnknown
            }
            _ => return None,
        };
        Some(Self {
            evidence_id: uuid::Uuid::new_v4().to_string(),
            state,
            occurred_at: Utc::now(),
            ..self.clone()
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolAdmissionEvidenceState {
    AwaitingApproval,
    ClaimIntent,
    Succeeded,
    Failed,
    Denied,
    Cancelled,
    Invalidated,
    Interrupted,
    OutcomeUnknown,
}

impl ToolAdmissionEvidenceState {
    #[must_use]
    pub const fn is_terminal(self) -> bool {
        !matches!(self, Self::AwaitingApproval | Self::ClaimIntent)
    }
}
