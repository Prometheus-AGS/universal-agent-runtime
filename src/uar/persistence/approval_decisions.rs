//! Owner-scoped approval history. Records describe decisions, never grant execution.
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ApprovalState { Pending, Approved, Denied, Cancelled, Expired, Interrupted, Invalidated }

#[derive(Debug, thiserror::Error)]
pub enum ApprovalRecoveryError {
    #[error("recovered approval persistence is unavailable; continuation ownership remains held")]
    PersistenceUnavailable,
}

/// Protected checkpoint identity and digests are evidence, never runnable authority.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ApprovalContinuation {
    pub checkpoint_id: String,
    pub checkpoint_revision: String,
    pub binding_digest: String,
    pub call_index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ApprovalDecision {
    pub decision_id: String,
    pub actor: String,
    pub approved: bool,
    pub decided_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ApprovalRecord {
    pub version: u32,
    pub issuer_id: String,
    pub challenge_id: String,
    pub owner_key: String,
    pub workspace_id: Option<String>,
    pub root_run_id: String,
    pub admission_id: Option<String>,
    pub admission_owner: super::tool_admission::AdmissionOwner,
    pub tool_call_id: String,
    pub tool_name: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub state: ApprovalState,
    pub decision: Option<ApprovalDecision>,
    pub updated_at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub continuation: Option<ApprovalContinuation>,
}

/// Read-time availability is local to this broker, never persisted authority.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApprovalRecordView {
    #[serde(flatten)]
    pub record: ApprovalRecord,
    pub resolvable: bool,
    /// Recovery still requires trusted predecessor exclusion and current admission.
    pub recovery_available: bool,
}

impl ApprovalRecord {
    pub fn storage_key(&self) -> String {
        super::tenant_storage_key(&self.owner_key, &format!("{}:{}", self.issuer_id, self.challenge_id))
    }

    pub fn finish(&self, state: ApprovalState, decision: Option<ApprovalDecision>) -> Self {
        Self { state, decision, updated_at: Utc::now(), ..self.clone() }
    }

    /// Expiry occurs at the immutable deadline, even when a delayed CAS observes it later.
    pub(crate) fn expire(&self) -> Self {
        Self { state: ApprovalState::Expired, decision: None, updated_at: self.expires_at, ..self.clone() }
    }

    pub fn validate_transition(&self, next: &Self) -> anyhow::Result<()> {
        anyhow::ensure!(self.state == ApprovalState::Pending && next.state != ApprovalState::Pending,
            "Approval transition requires a pending challenge");
        let mut identity = next.clone();
        identity.state = self.state;
        identity.decision = self.decision.clone();
        identity.updated_at = self.updated_at;
        anyhow::ensure!(identity == *self, "Approval identity is immutable");
        match &next.decision {
            Some(decision) => anyhow::ensure!(decision.actor == self.owner_key &&
                next.state == if decision.approved { ApprovalState::Approved } else { ApprovalState::Denied },
                "Approval decision does not match its authenticated owner or state"),
            None => anyhow::ensure!(!matches!(next.state, ApprovalState::Approved | ApprovalState::Denied),
                "Human resolution requires a decision"),
        }
        Ok(())
    }
}
