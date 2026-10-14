//! Host-owned durable call settlement and aggregate budget contracts.
//! Legacy append-only rows are history, never inferred opening balances.
use serde::{Deserialize, Serialize};

/// Authorized attempts are cumulative even when their execution later fails.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoteGrantActivity { Inspect, Model, Tool }

/// Destination accounting seals the first admission deadline. Resubmitting the
/// same peer contract never refreshes time, attempt counts, or financial scopes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RemoteGrantState {
    pub owner: String,
    pub accounting_id: String,
    pub grant: crate::uar::api::a2a::contract::UarUsageGrant,
    pub first_admitted_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub model_requests: u64,
    pub tool_calls: u64,
    pub revision: u64,
}

impl RemoteGrantState {
    pub fn key(owner: &str, id: &str) -> String { format!("{}:{owner}{}:{id}", owner.len(), id.len()) }

    pub fn admit(before: Option<&Self>, owner: &str, id: &str,
        grant: &crate::uar::api::a2a::contract::UarUsageGrant,
        activity: RemoteGrantActivity, now: chrono::DateTime<chrono::Utc>) -> anyhow::Result<Self> {
        grant.validate()?;
        anyhow::ensure!(!owner.is_empty() && !id.is_empty(), "remote_grant_identity_invalid");
        let mut next = match before {
            Some(before) => {
                anyhow::ensure!(before.owner == owner && before.accounting_id == id && &before.grant == grant,
                    "remote_grant_identity_conflict");
                before.clone()
            }
            None => Self { owner: owner.into(), accounting_id: id.into(), grant: grant.clone(),
                first_admitted_at: now,
                expires_at: grant.expires_after_seconds.map(|seconds| {
                    i64::try_from(seconds).ok().and_then(chrono::Duration::try_seconds)
                        .and_then(|duration| now.checked_add_signed(duration))
                        .ok_or_else(|| anyhow::anyhow!("remote_grant_deadline_invalid"))
                }).transpose()?, model_requests: 0, tool_calls: 0, revision: 0 },
        };
        anyhow::ensure!(next.expires_at.is_none_or(|deadline| now < deadline), "remote_grant_expired");
        match activity {
            RemoteGrantActivity::Inspect => {}
            RemoteGrantActivity::Model => {
                anyhow::ensure!(grant.max_total_model_requests.is_none_or(|limit| next.model_requests < limit), "remote_model_budget_exhausted");
                next.model_requests = next.model_requests.checked_add(1).ok_or(AccountingError::Exhausted)?;
            }
            RemoteGrantActivity::Tool => {
                anyhow::ensure!(grant.max_total_tool_calls.is_none_or(|limit| next.tool_calls < limit), "remote_tool_budget_exhausted");
                next.tool_calls = next.tool_calls.checked_add(1).ok_or(AccountingError::Exhausted)?;
            }
        }
        next.revision = next.revision.checked_add(1).ok_or(AccountingError::Conflict)?;
        Ok(next)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AccountingError {
    #[error("durable_accounting_unsupported")] Unsupported,
    #[error("accounting_revision_conflict")] Conflict,
    #[error("accounting_identity_conflict")] IdentityConflict,
    #[error("accounting_scope_reconciliation_required")] ReconciliationRequired,
    #[error("accounting_budget_exhausted")] Exhausted,
    #[error("accounting_usage_unresolved")] Unresolved,
    #[error("accounting_record_missing")] Missing,
}

/// Exact verified principal key is supplied by the host, never model arguments.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ScopeIdentity { pub owner: String, pub scope: String, pub scope_id: String }
impl ScopeIdentity {
    pub fn key(&self) -> String {
        format!("{}:{}:{}:{}:{}", self.owner.len(), self.owner, self.scope.len(), self.scope, self.scope_id)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ScopeLimits { pub cost_usd: Option<f64>, pub tokens: Option<u64> }
impl ScopeLimits {
    pub fn validate(&self) -> anyhow::Result<()> {
        anyhow::ensure!(self.cost_usd.is_none_or(|v| v.is_finite() && v >= 0.0), "invalid accounting cost limit");
        Ok(())
    }
}

/// A bounded aggregate; call history is stored separately and never rewritten here.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AccountingScope {
    pub identity: ScopeIdentity,
    pub revision: u64,
    pub generation: u64,
    pub limits: ScopeLimits,
    pub settled_cost_usd: f64,
    pub settled_tokens: u64,
    pub reserved_cost_usd: f64,
    pub reserved_tokens: u64,
    pub unknown_calls: u64,
    pub pending_calls: u64,
    pub reconciled: bool,
    pub reconciliation_evidence: Option<String>,
}
impl AccountingScope {
    pub fn new(identity: ScopeIdentity, limits: ScopeLimits, legacy: bool) -> Self {
        Self { identity, revision: 0, generation: 1, limits, settled_cost_usd: 0.0,
            settled_tokens: 0, reserved_cost_usd: 0.0, reserved_tokens: 0,
            unknown_calls: 0, pending_calls: 0, reconciled: !legacy, reconciliation_evidence: None }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScopeBinding { pub identity: ScopeIdentity, pub generation: u64 }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CallIdentity { pub owner: String, pub run_id: String, pub call_id: String, pub attempt: u64 }
impl CallIdentity {
    pub fn key(&self) -> String {
        format!("{}:{}:{}:{}:{}:{}:{}", self.owner.len(), self.owner, self.run_id.len(), self.run_id,
            self.call_id.len(), self.call_id, self.attempt)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CallAdmission {
    pub identity: CallIdentity,
    pub model: String,
    pub scopes: Vec<ScopeBinding>,
    /// None retains unknown liability; finite cost limits require a known bound.
    pub reserved_cost_usd: Option<f64>,
    pub reserved_tokens: Option<u64>,
    /// Shared-ledger funding must still be pending in the admission transaction.
    #[serde(default)]
    pub source_reservation: Option<CallIdentity>,
}
impl CallAdmission {
    pub fn validate(&self) -> anyhow::Result<()> {
        anyhow::ensure!(!self.identity.owner.is_empty() && !self.identity.run_id.is_empty() && !self.identity.call_id.is_empty(), "accounting identity missing");
        anyhow::ensure!(!self.scopes.is_empty() && self.scopes.len() <= 8, "invalid accounting scope set");
        anyhow::ensure!(self.reserved_cost_usd.is_none_or(|v| v.is_finite() && v >= 0.0), "invalid accounting reservation");
        anyhow::ensure!(self.source_reservation.as_ref() != Some(&self.identity), "accounting self-funding is invalid");
        let mut keys = std::collections::BTreeSet::new();
        for scope in &self.scopes {
            anyhow::ensure!(scope.identity.owner == self.identity.owner || (scope.identity.owner == "deployment" && scope.identity.scope == "global"), "accounting owner mismatch");
            anyhow::ensure!(keys.insert(scope.identity.key()), "duplicate accounting scope");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccountingState { Pending, Settled }
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AccountingCall {
    pub admission: CallAdmission,
    pub revision: u64,
    pub state: AccountingState,
    pub observed_cost_usd: Option<f64>,
    pub observed_tokens: u64,
    pub reconciliation_evidence: Option<String>,
}
impl AccountingCall {
    pub fn new(admission: CallAdmission) -> Self {
        Self { admission, revision: 0, state: AccountingState::Pending,
            observed_cost_usd: None, observed_tokens: 0, reconciliation_evidence: None }
    }
    pub fn liability_cost(&self) -> f64 { self.admission.reserved_cost_usd.unwrap_or(0.0).max(self.observed_cost_usd.unwrap_or(0.0)) }
    pub fn liability_tokens(&self) -> u64 { self.admission.reserved_tokens.unwrap_or(0).max(self.observed_tokens) }
    pub fn unknown(&self) -> bool { self.admission.reserved_cost_usd.is_none() || self.admission.reserved_tokens.is_none() }
}

/// Cumulative provider counter. Only a trustworthy terminal counter releases a reservation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallUsage { pub identity: CallIdentity, pub cost_usd: Option<f64>, pub tokens: u64, pub terminal: bool, pub evidence: Option<String> }

/// Administrative CAS. Opening balances require external reconciliation evidence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopeUpdate {
    pub identity: ScopeIdentity,
    pub expected_generation: u64,
    pub limits: ScopeLimits,
    pub opening_cost_usd: Option<f64>,
    pub opening_tokens: Option<u64>,
    pub evidence: String,
}

pub(crate) fn reserve(scope: &mut AccountingScope, admission: &CallAdmission) -> anyhow::Result<()> {
    if !scope.reconciled { return Err(AccountingError::ReconciliationRequired.into()); }
    let binding = admission.scopes.iter().find(|s| s.identity == scope.identity).ok_or(AccountingError::IdentityConflict)?;
    if binding.generation != scope.generation { return Err(AccountingError::Conflict.into()); }
    if scope.limits.cost_usd.is_some() || scope.limits.tokens.is_some() {
        if scope.unknown_calls != 0 { return Err(AccountingError::Unresolved.into()); }
    }
    if let Some(limit) = scope.limits.cost_usd {
        let reserved = admission.reserved_cost_usd.ok_or(AccountingError::Unresolved)?;
        if scope.settled_cost_usd + scope.reserved_cost_usd + reserved > limit { return Err(AccountingError::Exhausted.into()); }
    }
    if let Some(limit) = scope.limits.tokens {
        let reserved = admission.reserved_tokens.ok_or(AccountingError::Unresolved)?;
        if scope.settled_tokens.saturating_add(scope.reserved_tokens).saturating_add(reserved) > limit { return Err(AccountingError::Exhausted.into()); }
    }
    let call = AccountingCall::new(admission.clone());
    scope.reserved_cost_usd += call.liability_cost();
    scope.reserved_tokens = scope.reserved_tokens.checked_add(call.liability_tokens()).ok_or(AccountingError::Exhausted)?;
    scope.unknown_calls += u64::from(call.unknown());
    scope.pending_calls += 1;
    scope.revision += 1;
    Ok(())
}

pub(crate) fn observe(call: &AccountingCall, usage: &CallUsage) -> anyhow::Result<AccountingCall> {
    if usage.identity != call.admission.identity { return Err(AccountingError::IdentityConflict.into()); }
    anyhow::ensure!(usage.cost_usd.is_none_or(|v| v.is_finite() && v >= 0.0), "invalid accounting usage");
    if call.state == AccountingState::Settled {
        if usage.terminal && usage.cost_usd == call.observed_cost_usd && usage.tokens == call.observed_tokens { return Ok(call.clone()); }
        return Err(AccountingError::IdentityConflict.into());
    }
    anyhow::ensure!(usage.tokens >= call.observed_tokens, "accounting token counter decreased");
    let mut after = call.clone();
    after.observed_cost_usd = if usage.terminal { usage.cost_usd.or(call.observed_cost_usd) } else {
        match (usage.cost_usd,call.observed_cost_usd) { (Some(next),Some(prior)) => Some(next.max(prior)), (next,prior) => next.or(prior) }
    };
    after.observed_tokens = usage.tokens;
    // Unpriced work is never represented as free settled usage.
    if usage.terminal && usage.cost_usd.is_some() { after.state = AccountingState::Settled; }
    after.reconciliation_evidence = usage.evidence.clone();
    after.revision += 1;
    Ok(after)
}

pub(crate) fn settle(scope: &mut AccountingScope, before: &AccountingCall, after: &AccountingCall) -> anyhow::Result<()> {
    if before == after { return Ok(()); }
    scope.reserved_cost_usd = (scope.reserved_cost_usd - before.liability_cost()).max(0.0);
    scope.reserved_tokens = scope.reserved_tokens.checked_sub(before.liability_tokens()).ok_or(AccountingError::Conflict)?;
    if after.state == AccountingState::Settled {
        scope.settled_cost_usd += after.observed_cost_usd.ok_or(AccountingError::Unresolved)?;
        scope.settled_tokens = scope.settled_tokens.checked_add(after.observed_tokens).ok_or(AccountingError::Exhausted)?;
        scope.pending_calls = scope.pending_calls.checked_sub(1).ok_or(AccountingError::Conflict)?;
        scope.unknown_calls = scope.unknown_calls.checked_sub(u64::from(before.unknown())).ok_or(AccountingError::Conflict)?;
    } else {
        scope.reserved_cost_usd += after.liability_cost();
        scope.reserved_tokens = scope.reserved_tokens.checked_add(after.liability_tokens()).ok_or(AccountingError::Exhausted)?;
    }
    scope.revision += 1;
    Ok(())
}

pub(crate) fn update(scope: &AccountingScope, change: &ScopeUpdate) -> anyhow::Result<AccountingScope> {
    change.limits.validate()?;
    if scope.identity != change.identity || scope.generation != change.expected_generation { return Err(AccountingError::Conflict.into()); }
    anyhow::ensure!(!change.evidence.trim().is_empty(), "accounting reconciliation evidence required");
    let mut after = scope.clone();
    if change.opening_cost_usd.is_some() || change.opening_tokens.is_some() {
        anyhow::ensure!(!scope.reconciled && scope.pending_calls == 0, "opening balance cannot rewrite active accounting");
        let cost = change.opening_cost_usd.ok_or(AccountingError::ReconciliationRequired)?;
        anyhow::ensure!(cost.is_finite() && cost >= 0.0, "invalid opening balance");
        after.settled_cost_usd = cost;
        after.settled_tokens = change.opening_tokens.ok_or(AccountingError::ReconciliationRequired)?;
        after.reconciled = true;
    }
    after.limits = change.limits.clone();
    after.generation += 1;
    after.revision += 1;
    after.reconciliation_evidence = Some(change.evidence.clone());
    Ok(after)
}

/// Immutable link from the existing authenticated contract digest to the source
/// reservation. Its presence in the destination store proves shared accounting
/// authority; network/process identity alone does not.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountingDelegation { pub contract_digest: String, pub source: CallIdentity }
