//! Per-scope cost budgets + spend aggregation (CH-06, fable §R11 / §7).
//!
//! Builds on the existing opt-in cost estimation (`LlmConfig.cost_tracking`,
//! `estimate_cost`, the `uar_llm_cost_usd` metric). This module adds the
//! *accounting* layer the runtime was missing: aggregate spend per run / task /
//! session / agent, declare budget envelopes, and emit threshold-crossing
//! events so the UI and operators can react before a runaway burns the budget.
//!
//! The synchronous tracker measures live activity; the trusted driver wrapper
//! reserves and settles durable aggregate liability before exposing provider events.

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::uar::persistence::accounting::{AccountingError, CallAdmission, CallIdentity, CallUsage, ScopeBinding, ScopeIdentity, ScopeLimits};
use crate::uar::persistence::PersistenceLayer;

use crate::uar::api::a2a::contract::{UarUsageGrant, UarUsageReceipt};
use crate::uar::runtime::thread::policy_intersection::ThreadBudgets;

/// The trusted team host allocates this maximum response slice before granting
/// the remaining finite envelope to the request input allowance.
pub(crate) const TEAM_MAX_OUTPUT_TOKENS: u64 = 64_000;

/// Split a trusted team allocation into independently finite input and output
/// allowances without treating either as a measured provider token count.
pub(crate) fn team_token_allocation(total: u64) -> anyhow::Result<(u64, u64)> {
    let output = TEAM_MAX_OUTPUT_TOKENS.min(total / 2);
    let input = total
        .checked_sub(output)
        .ok_or_else(|| anyhow::anyhow!("team token allocation overflows"))?;
    anyhow::ensure!(input > 0 && output > 0,
        "team token allocation must leave nonzero input and output allowances");
    Ok((input, output))
}

/// Closed, field-free storage classification for admission diagnostics; never exposes error text.
fn storage_error_kind(error: &anyhow::Error) -> &'static str {
    #[cfg(feature = "surreal-backend")]
    for cause in error.chain() {
        if let Some(storage) = cause.downcast_ref::<surrealdb::Error>() {
            return if matches!(storage.query_details(), Some(surrealdb::types::QueryError::TransactionConflict)) { "transaction_conflict" } else { "other_storage_error" };
        }
    }
    if error.chain().any(|cause| cause.is::<AccountingError>()) { "accounting_error" } else { "non_storage" }
}

/// Closed SurrealDB kind/subkind chain; never formats error text or detail fields.
fn storage_error_detail(error: &anyhow::Error) -> String {
    #[cfg(feature = "surreal-backend")]
    for cause in error.chain() {
        if let Some(storage) = cause.downcast_ref::<surrealdb::Error>() {
            use surrealdb::types::{ConfigurationError, ErrorDetails, NotFoundError, QueryError, SerializationError, ValidationError};
            let label = |storage: &surrealdb::Error| {
                let subtype = match storage.details() {
                    ErrorDetails::Query(Some(QueryError::NotExecuted)) => Some("NotExecuted"),
                    ErrorDetails::Query(Some(QueryError::TimedOut { .. })) => Some("TimedOut"),
                    ErrorDetails::Query(Some(QueryError::Cancelled)) => Some("Cancelled"),
                    ErrorDetails::Query(Some(QueryError::TransactionConflict)) => Some("TransactionConflict"),
                    ErrorDetails::Validation(Some(ValidationError::Parse)) => Some("Parse"),
                    ErrorDetails::Validation(Some(ValidationError::InvalidRequest)) => Some("InvalidRequest"),
                    ErrorDetails::Validation(Some(ValidationError::InvalidParams)) => Some("InvalidParams"),
                    ErrorDetails::Validation(Some(ValidationError::NamespaceEmpty)) => Some("NamespaceEmpty"),
                    ErrorDetails::Validation(Some(ValidationError::DatabaseEmpty)) => Some("DatabaseEmpty"),
                    ErrorDetails::Validation(Some(ValidationError::InvalidParameter { .. })) => Some("InvalidParameter"),
                    ErrorDetails::Validation(Some(ValidationError::InvalidContent { .. })) => Some("InvalidContent"),
                    ErrorDetails::Validation(Some(ValidationError::InvalidMerge { .. })) => Some("InvalidMerge"),
                    ErrorDetails::Configuration(Some(ConfigurationError::LiveQueryNotSupported)) => Some("LiveQueryNotSupported"),
                    ErrorDetails::Configuration(Some(ConfigurationError::BadLiveQueryConfig)) => Some("BadLiveQueryConfig"),
                    ErrorDetails::Configuration(Some(ConfigurationError::BadGraphqlConfig)) => Some("BadGraphqlConfig"),
                    ErrorDetails::Configuration(Some(ConfigurationError::DialectNotSupported)) => Some("DialectNotSupported"),
                    ErrorDetails::Serialization(Some(SerializationError::Serialization)) => Some("Serialization"),
                    ErrorDetails::Serialization(Some(SerializationError::Deserialization)) => Some("Deserialization"),
                    ErrorDetails::NotFound(Some(NotFoundError::Table { .. })) => Some("Table"),
                    ErrorDetails::NotFound(Some(NotFoundError::Record { .. })) => Some("Record"),
                    ErrorDetails::NotFound(Some(NotFoundError::Namespace { .. })) => Some("Namespace"),
                    ErrorDetails::NotFound(Some(NotFoundError::Database { .. })) => Some("Database"),
                    ErrorDetails::NotFound(Some(NotFoundError::Transaction)) => Some("Transaction"),
                    ErrorDetails::Query(None) | ErrorDetails::Validation(None)
                    | ErrorDetails::Configuration(None) | ErrorDetails::Serialization(None)
                    | ErrorDetails::NotFound(None) => Some("Unspecified"),
                    _ => None,
                };
                subtype.map_or_else(|| storage.kind_str().to_owned(), |subtype| format!("{}:{subtype}", storage.kind_str()))
            };
            let mut detail = label(storage);
            let mut current = storage;
            while let Some(inner) = current.cause() {
                detail.push_str("<-");
                detail.push_str(&label(inner));
                current = inner;
            }
            return detail;
        }
    }
    let _ = error;
    "none".to_owned()
}

/// Budget-owned failure provenance, independent of provider response content.
#[derive(Debug, Clone, Copy, thiserror::Error)]
pub(crate) enum BudgetFailureOrigin {
    #[error("budget_admission_failed")]
    Admission,
    #[error("Model call cancelled with its root run")]
    RootCancellation,
    #[error("Root budget timeout exceeded")]
    RootDeadline,
    #[error("budget_usage_accounting_failed")]
    UsageAccounting,
}

impl BudgetFailureOrigin {
    pub(crate) const fn category(self) -> &'static str {
        match self {
            Self::Admission => "budget_admission_failed",
            Self::RootCancellation => "budget_root_cancelled",
            Self::RootDeadline => "budget_root_deadline_exceeded",
            Self::UsageAccounting => "budget_usage_accounting_failed",
        }
    }
}

/// Closed client-facing `(code, message)` for a budget-owned refusal: the category plus the
/// `AccountingError` cause. Typed provider failures keep their own code; no error text is used.
pub(crate) fn client_failure(error: &anyhow::Error) -> Option<(&'static str, String)> {
    if crate::llm::ProviderError::from_anyhow(error).is_some() { return None; }
    let category = error.downcast_ref::<BudgetFailureOrigin>()?.category();
    let message = match error.chain().find_map(|cause| cause.downcast_ref::<AccountingError>()) {
        Some(cause) => format!("{category}: {cause}"),
        None => category.to_owned(),
    };
    Some((category, message))
}

/// The dimension a budget / spend record is keyed on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum BudgetScope {
    Run,
    Task,
    Session,
    Agent,
    /// Whole-deployment ("house") spend.
    Global,
}

impl BudgetScope {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Run => "run",
            Self::Task => "task",
            Self::Session => "session",
            Self::Agent => "agent",
            Self::Global => "global",
        }
    }
}

/// A configured budget limit for one `(scope, scope_id)`.
#[derive(Debug, Clone, Copy)]
pub struct BudgetLimit {
    /// Hard ceiling in USD. Spend beyond this reports `Exceeded`.
    pub limit_usd: f64,
    /// Fraction (0.0–1.0) at which a `Warning` fires (e.g. 0.8 = 80%).
    pub warn_at: f64,
}

impl Default for BudgetLimit {
    fn default() -> Self {
        Self {
            limit_usd: f64::INFINITY,
            warn_at: 0.8,
        }
    }
}

/// Result of recording spend against a budget.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BudgetStatus {
    /// Under the warning threshold.
    Ok,
    /// At/over `warn_at * limit` but under `limit`.
    Warning { spent_usd: f64, limit_usd: f64 },
    /// At/over the hard limit.
    Exceeded { spent_usd: f64, limit_usd: f64 },
}

impl BudgetStatus {
    #[must_use]
    pub fn is_exceeded(self) -> bool {
        matches!(self, Self::Exceeded { .. })
    }
}

#[derive(Debug, Default)]
struct Inner {
    /// (scope, scope_id) -> accumulated USD spend.
    spend: HashMap<(BudgetScope, String), f64>,
    /// (scope, scope_id) -> configured limit.
    limits: HashMap<(BudgetScope, String), BudgetLimit>,
    tokens: HashMap<(BudgetScope, String), u64>,
    runs: HashMap<String, RunUsage>,
    remote_reservations: HashMap<String, RemoteReservationState>,
    accounting_owners: HashMap<String, String>,
}

#[derive(Debug, Clone)]
struct RemoteReservationState {
    scopes: Vec<(BudgetScope, String)>,
    usage_id: String,
    grant: UarUsageGrant,
    used: UarUsageReceipt,
}

/// Source-owned lease for one governed remote child. Dropping this handle does
/// not release capacity: the host must prove no dispatch occurred or obtain
/// confirmed terminal cleanup before releasing it.
#[derive(Debug, Clone)]
pub(crate) struct RemoteBudgetReservation {
    tracker: CostBudgetTracker,
    id: String,
    grant: UarUsageGrant,
    released: Arc<std::sync::atomic::AtomicBool>,
    durable: Option<DurableCall>,
    unresolved: Arc<std::sync::atomic::AtomicBool>,
}

#[derive(Debug, Default)]
struct RunUsage {
    usage_reports: u64,
    completed_usage_reports: u64,
    unpriced_usage: bool,
    total_tokens: u64,
    cost_usd: f64,
    tool_calls: u64,
    total_requests: u64,
    requests: VecDeque<(Instant, u64)>,
    token_window: VecDeque<(Instant, u64)>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub(crate) struct RunUsageSnapshot {
    pub(crate) usage_reports: u64,
    pub(crate) completed_usage_reports: u64,
    pub(crate) unpriced_usage: bool,
    pub(crate) total_tokens: u64,
    pub(crate) cost_usd: f64,
    pub(crate) model_requests: u64,
    pub(crate) tool_calls: u64,
}

/// Immutable local-accounting evidence retained across a caller-tool handoff.
/// Durable monetary liabilities remain in the existing accounting store.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClientToolBudgetSnapshot {
    version: u32,
    owner: String,
    source_usage_id: String,
    captured_at_ms: i64,
    elapsed_root_ms: u64,
    deadline_at_ms: Option<i64>,
    limits: ThreadBudgets,
    scopes: Vec<ClientToolScopeSnapshot>,
    activity: ClientToolActivitySnapshot,
    remote: Option<ClientToolRemoteSnapshot>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
struct ClientToolScopeSnapshot {
    scope: BudgetScope, id: String, tokens: u64, cost_usd: f64,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
struct ClientToolActivitySnapshot {
    cumulative: RunUsageSnapshot,
    requests: Vec<(i64, u64)>,
    tokens: Vec<(i64, u64)>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
struct ClientToolRemoteSnapshot {
    accounting_id: String, elapsed_ms: u64, activity: ClientToolActivitySnapshot,
}

fn handoff_activity(usage: Option<&RunUsage>, now: Instant, wall_ms: i64) -> anyhow::Result<ClientToolActivitySnapshot> {
    let windows = |entries: Option<&VecDeque<(Instant, u64)>>| -> anyhow::Result<Vec<(i64, u64)>> {
        entries.into_iter().flatten().filter(|(at, _)| now.saturating_duration_since(*at) < Duration::from_secs(60))
            .map(|(at, count)| {
                let age = i64::try_from(now.saturating_duration_since(*at).as_millis())?;
                Ok((wall_ms.checked_sub(age).ok_or_else(|| anyhow::anyhow!("client_budget_clock_range"))?, *count))
            }).collect()
    };
    Ok(ClientToolActivitySnapshot {
        cumulative: RunUsageSnapshot {
            usage_reports: usage.map_or(0, |v| v.usage_reports),
            completed_usage_reports: usage.map_or(0, |v| v.completed_usage_reports),
            unpriced_usage: usage.is_some_and(|v| v.unpriced_usage),
            total_tokens: usage.map_or(0, |v| v.total_tokens), cost_usd: usage.map_or(0.0, |v| v.cost_usd),
            model_requests: usage.map_or(0, |v| v.total_requests), tool_calls: usage.map_or(0, |v| v.tool_calls),
        },
        requests: windows(usage.map(|v| &v.requests))?, tokens: windows(usage.map(|v| &v.token_window))?,
    })
}

fn restore_handoff_activity(usage: &mut RunUsage, saved: &ClientToolActivitySnapshot, now: Instant, wall_ms: i64) -> anyhow::Result<()> {
    let total = &saved.cumulative;
    anyhow::ensure!(total.cost_usd.is_finite() && total.cost_usd >= 0.0, "invalid_client_budget_usage");
    usage.usage_reports = usage.usage_reports.max(total.usage_reports);
    usage.completed_usage_reports = usage.completed_usage_reports.max(total.completed_usage_reports);
    usage.unpriced_usage |= total.unpriced_usage;
    usage.total_tokens = usage.total_tokens.max(total.total_tokens);
    usage.cost_usd = usage.cost_usd.max(total.cost_usd);
    usage.total_requests = usage.total_requests.max(total.model_requests);
    usage.tool_calls = usage.tool_calls.max(total.tool_calls);
    // Per-wall-second maxima make repeated restoration idempotent and preserve
    // already observed activity without charging a usage delta a second time.
    let merge = |current: &mut VecDeque<(Instant, u64)>, entries: &[(i64, u64)]| -> anyhow::Result<()> {
        let mut buckets = std::collections::BTreeMap::<i64, u64>::new();
        for (at, count) in current.iter() {
            let age = i64::try_from(now.saturating_duration_since(*at).as_millis())?;
            let timestamp = wall_ms.checked_sub(age).ok_or_else(|| anyhow::anyhow!("client_budget_clock_range"))?;
            let bucket = buckets.entry(timestamp.div_euclid(1000)).or_default();
            *bucket = bucket.saturating_add(*count);
        }
        let mut saved_buckets = std::collections::BTreeMap::<i64, u64>::new();
        for (timestamp, count) in entries {
            anyhow::ensure!(*timestamp <= wall_ms, "client_budget_clock_reversed");
            let bucket = saved_buckets.entry(timestamp.div_euclid(1000)).or_default();
            *bucket = bucket.saturating_add(*count);
        }
        for (second, count) in saved_buckets {
            let bucket = buckets.entry(second).or_default();
            *bucket = (*bucket).max(count);
        }
        let mut merged = VecDeque::new();
        for (second, count) in buckets {
            // Use the end of the bucket: rounding may extend the restriction
            // by under a second, but cannot expire observed spend early.
            let timestamp = second.checked_mul(1000).and_then(|v| v.checked_add(999))
                .ok_or_else(|| anyhow::anyhow!("client_budget_clock_range"))?.min(wall_ms);
            let age = u64::try_from(wall_ms - timestamp)?;
            if age < 60_000 {
                merged.push_back((now.checked_sub(Duration::from_millis(age))
                    .ok_or_else(|| anyhow::anyhow!("client_budget_clock_range"))?, count));
            }
        }
        *current = merged;
        Ok(())
    };
    merge(&mut usage.requests, &saved.requests)?;
    merge(&mut usage.token_window, &saved.tokens)?;
    Ok(())
}

/// Target-local binding for the cumulative capacity leased by a source UAR.
/// The start instant is fixed when the governed actor is created so follow-up
/// turns cannot reset the lease timeout.
#[derive(Debug, Clone)]
pub(crate) struct RemoteUsageGrantBinding {
    pub(crate) accounting_id: String,
    pub(crate) grant: UarUsageGrant,
    pub(crate) started_at: Instant,
}

impl RunUsage {
    fn prune(&mut self, now: Instant) {
        let minute = Duration::from_secs(60);
        while self
            .requests
            .front()
            .is_some_and(|(at, _)| now.duration_since(*at) >= minute)
        {
            self.requests.pop_front();
        }
        while self
            .token_window
            .front()
            .is_some_and(|(at, _)| now.duration_since(*at) >= minute)
        {
            self.token_window.pop_front();
        }
    }
}

/// Thread-safe cost-budget tracker. Cloneable handle over shared state.
#[derive(Debug, Clone, Default)]
pub struct CostBudgetTracker {
    // No I/O or awaits while held. Synchronous updates let a received usage
    // event charge every scope before a caller can cancel its stream future.
    inner: Arc<Mutex<Inner>>,
}

impl CostBudgetTracker {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub(crate) fn run_usage(&self, run_id: &str) -> RunUsageSnapshot {
        let inner = self
            .inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let activity = inner.runs.get(run_id);
        RunUsageSnapshot {
            usage_reports: activity.map_or(0, |usage| usage.usage_reports),
            completed_usage_reports: activity.map_or(0, |usage| usage.completed_usage_reports),
            unpriced_usage: activity.is_some_and(|usage| usage.unpriced_usage),
            total_tokens: activity.map_or(0, |usage| usage.total_tokens),
            cost_usd: activity.map_or(0.0, |usage| usage.cost_usd),
            model_requests: activity.map_or(0, |usage| usage.total_requests),
            tool_calls: activity.map_or(0, |usage| usage.tool_calls),
        }
    }

    /// Set (or replace) the budget limit for a `(scope, scope_id)`.
    pub async fn set_limit(&self, scope: BudgetScope, scope_id: &str, limit: BudgetLimit) {
        self.inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .limits
            .insert((scope, scope_id.to_string()), limit);
    }

    /// Record `cost_usd` of spend against a `(scope, scope_id)` and return the
    /// resulting status. Emits a `cost.budget.*` tracing event on threshold
    /// crossings so downstream (UI/alerts) can react; also updates the
    /// `uar_cost_budget_spent_usd` gauge.
    pub async fn record(&self, scope: BudgetScope, scope_id: &str, cost_usd: f64) -> BudgetStatus {
        let key = (scope, scope_id.to_string());
        let mut guard = self
            .inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let spent = {
            let e = guard.spend.entry(key.clone()).or_insert(0.0);
            *e += cost_usd.max(0.0);
            *e
        };
        let limit = guard.limits.get(&key).copied().unwrap_or_default();
        drop(guard);

        #[cfg(feature = "telemetry")]
        metrics::gauge!(
            "uar_cost_budget_spent_usd",
            "scope" => scope.as_str(),
        )
        .set(spent);

        let status = if spent >= limit.limit_usd {
            tracing::warn!(
                name: "cost.budget.exceeded",
                scope = scope.as_str(), scope_id, spent_usd = spent, limit_usd = limit.limit_usd,
                "cost budget exceeded"
            );
            BudgetStatus::Exceeded {
                spent_usd: spent,
                limit_usd: limit.limit_usd,
            }
        } else if limit.limit_usd.is_finite() && spent >= limit.warn_at * limit.limit_usd {
            tracing::warn!(
                name: "cost.budget.warning",
                scope = scope.as_str(), scope_id, spent_usd = spent, limit_usd = limit.limit_usd,
                "cost budget warning threshold crossed"
            );
            BudgetStatus::Warning {
                spent_usd: spent,
                limit_usd: limit.limit_usd,
            }
        } else {
            BudgetStatus::Ok
        };
        status
    }

    /// Current accumulated spend for a `(scope, scope_id)`.
    pub async fn spent(&self, scope: BudgetScope, scope_id: &str) -> f64 {
        self.inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .spend
            .get(&(scope, scope_id.to_string()))
            .copied()
            .unwrap_or(0.0)
    }

    /// Inspect a scope without charging it again at run completion.
    pub async fn status(&self, scope: BudgetScope, scope_id: &str) -> BudgetStatus {
        let key = (scope, scope_id.to_string());
        let inner = self
            .inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        budget_status(
            inner.spend.get(&key).copied().unwrap_or(0.0),
            inner.limits.get(&key).copied().unwrap_or_default(),
        )
    }

    fn check_limits(
        inner: &Inner,
        scopes: &[(BudgetScope, String)],
        limits: &ThreadBudgets,
    ) -> anyhow::Result<()> {
        for key in scopes {
            let (reserved_tokens, reserved_cost) = reserved_scope(inner, key);
            let spent = inner.spend.get(key).copied().unwrap_or(0.0) + reserved_cost;
            let limit = inner.limits.get(key).copied().unwrap_or_default();
            if budget_status(spent, limit).is_exceeded() {
                anyhow::bail!("{} cost budget exceeded", key.0.as_str());
            }
            let token_limit = match key.0 {
                BudgetScope::Run => limits.max_tokens_per_turn,
                BudgetScope::Session => limits.max_tokens_per_session,
                _ => None,
            };
            if token_limit.is_some_and(|limit| {
                inner
                    .tokens
                    .get(key)
                    .copied()
                    .unwrap_or(0)
                    .saturating_add(reserved_tokens)
                    >= limit
            }) {
                anyhow::bail!("{} token budget exceeded", key.0.as_str());
            }
            if key.0 == BudgetScope::Session
                && limits
                    .max_cost_per_session_usd
                    .is_some_and(|limit| spent >= limit)
            {
                anyhow::bail!("Session cost budget exceeded");
            }
        }
        Ok(())
    }

    fn admit(
        &self,
        scopes: &[(BudgetScope, String)],
        usage_id: &str,
        limits: &ThreadBudgets,
        model: Option<&str>,
        remote: Option<&RemoteUsageGrantBinding>,
    ) -> anyhow::Result<()> {
        // A poisoned ledger cannot authorize more paid work. No await may
        // separate checking shared counters from reserving a model attempt.
        let mut inner = self
            .inner
            .lock()
            .map_err(|_| anyhow::anyhow!("Cost budget ledger is unavailable"))?;
        Self::check_limits(&inner, scopes, limits)?;
        check_remote_grant(&inner, remote, model.is_some(), false, 0, 0.0)?;
        if let Some(model) = model {
            let finite_cost_limit = limits.max_cost_per_session_usd.is_some()
                || scopes.iter().any(|key| {
                    inner
                        .limits
                        .get(key)
                        .is_some_and(|limit| limit.limit_usd.is_finite())
                });
            if finite_cost_limit && crate::llm::catalog::estimate_cost(model, 0, 0, 0).is_none() {
                anyhow::bail!("Model has no catalog price for enforcing the active cost budget");
            }
            let now = Instant::now();
            let rate_usage_id = remote.map_or(usage_id, |remote| remote.accounting_id.as_str());
            let (reserved_requests, reserved_rate_tokens, _) =
                reserved_activity(&inner, rate_usage_id);
            let rate_usage = inner.runs.entry(rate_usage_id.to_owned()).or_default();
            rate_usage.prune(now);
            if let Some(rate) = &limits.rate_limit {
                if rate.requests_per_minute.is_some_and(|limit| {
                    rate_usage
                        .requests
                        .iter()
                        .fold(0_u64, |total, (_, count)| total.saturating_add(*count))
                        .saturating_add(reserved_requests)
                        >= u64::from(limit)
                }) {
                    anyhow::bail!("Root model request rate exceeded");
                }
                let tokens = rate_usage
                    .token_window
                    .iter()
                    .fold(0_u64, |total, (_, tokens)| total.saturating_add(*tokens));
                if rate
                    .tokens_per_minute
                    .is_some_and(|limit| tokens.saturating_add(reserved_rate_tokens) >= limit)
                {
                    anyhow::bail!("Root model token rate exceeded");
                }
            }
            if let Some(remote) = remote {
                let usage = inner.runs.entry(usage_id.to_owned()).or_default();
                usage.total_requests = usage.total_requests.saturating_add(1);
                let usage = inner.runs.entry(remote.accounting_id.clone()).or_default();
                usage.total_requests = usage.total_requests.saturating_add(1);
                usage.requests.push_back((now, 1));
            } else {
                let usage = inner.runs.entry(usage_id.to_owned()).or_default();
                usage.total_requests = usage.total_requests.saturating_add(1);
                usage.requests.push_back((now, 1));
            }
        }
        Ok(())
    }

    fn admit_tool(
        &self,
        scopes: &[(BudgetScope, String)],
        usage_id: &str,
        limits: &ThreadBudgets,
        remote: Option<&RemoteUsageGrantBinding>,
    ) -> anyhow::Result<()> {
        let mut inner = self
            .inner
            .lock()
            .map_err(|_| anyhow::anyhow!("Cost budget ledger is unavailable"))?;
        Self::check_limits(&inner, scopes, limits)?;
        check_remote_grant(&inner, remote, false, true, 0, 0.0)?;
        let (_, _, reserved_tools) = reserved_activity(&inner, usage_id);
        let usage = inner.runs.entry(usage_id.to_owned()).or_default();
        if limits.max_tool_calls_per_turn.is_some_and(|limit| {
            usage.tool_calls.saturating_add(reserved_tools) >= u64::from(limit)
        }) {
            anyhow::bail!("Root tool-call budget exceeded");
        }
        // Count authorized attempts even if execution subsequently fails. A
        // failed tool must not open an unlimited retry allowance.
        usage.tool_calls = usage.tool_calls.saturating_add(1);
        if let Some(remote) = remote {
            let usage = inner.runs.entry(remote.accounting_id.clone()).or_default();
            usage.tool_calls = usage.tool_calls.saturating_add(1);
        }
        Ok(())
    }

    fn record_usage_report(
        &self,
        usage_id: &str,
        remote: Option<&RemoteUsageGrantBinding>,
        priced: bool,
        completed: bool,
    ) -> anyhow::Result<()> {
        let mut inner = self
            .inner
            .lock()
            .map_err(|_| anyhow::anyhow!("Cost budget ledger is unavailable"))?;
        for id in std::iter::once(usage_id).chain(remote.map(|remote| remote.accounting_id.as_str())) {
            let usage = inner.runs.entry(id.to_owned()).or_default();
            if completed {
                usage.completed_usage_reports = usage.completed_usage_reports.saturating_add(1);
            } else {
                usage.usage_reports = usage.usage_reports.saturating_add(1);
            }
            usage.unpriced_usage |= !priced;
        }
        Ok(())
    }

    fn record_call(
        &self,
        scopes: &[(BudgetScope, String)],
        usage_id: &str,
        remote: Option<&RemoteUsageGrantBinding>,
        cost: f64,
        tokens: u64,
    ) -> anyhow::Result<()> {
        let updates = {
            let mut inner = self
                .inner
                .lock()
                .map_err(|_| anyhow::anyhow!("Cost budget ledger is unavailable"))?;
            let remote_status = check_remote_grant(&inner, remote, false, false, tokens, cost)
                .map_err(|_| {
                    if let Some(remote) = remote {
                        let usage = inner.runs.get(&remote.accounting_id);
                        let observed_tokens = usage.map_or(0, |usage| usage.total_tokens);
                        let observed_cost = usage.map_or(0.0, |usage| usage.cost_usd);
                        tracing::warn!(
                            name: "cost.remote_grant.rejected",
                            observed_tokens,
                            additional_tokens = tokens,
                            projected_tokens = observed_tokens.saturating_add(tokens),
                            max_total_tokens = ?remote.grant.max_total_tokens,
                            observed_cost_usd = observed_cost,
                            additional_cost_usd = cost,
                            projected_cost_usd = observed_cost + cost,
                            max_total_cost_usd = ?remote.grant.max_total_cost_usd,
                            model_requests = usage.map_or(0, |usage| usage.total_requests),
                            max_total_model_requests = ?remote.grant.max_total_model_requests,
                            tool_calls = usage.map_or(0, |usage| usage.tool_calls),
                            max_total_tool_calls = ?remote.grant.max_total_tool_calls,
                            elapsed_seconds = remote.started_at.elapsed().as_secs(),
                            expires_after_seconds = ?remote.grant.expires_after_seconds,
                            "Host cumulative usage grant rejected model usage"
                        );
                    }
                    anyhow::Error::new(crate::llm::provider_error::ProviderError::new(
                        None,
                        crate::llm::provider_error::ProviderErrorKind::BudgetExceeded,
                        None,
                        "Host cumulative usage grant rejected model usage",
                    ))
                });
            let activity = inner.runs.entry(usage_id.to_owned()).or_default();
            activity.total_tokens = activity.total_tokens.saturating_add(tokens);
            activity.cost_usd = (activity.cost_usd + cost).max(0.0);
            if tokens > 0 && remote.is_none() {
                let now = Instant::now();
                let usage = inner.runs.entry(usage_id.to_owned()).or_default();
                usage.prune(now);
                usage.token_window.push_back((now, tokens));
            }
            if let Some(remote) = remote {
                let usage = inner.runs.entry(remote.accounting_id.clone()).or_default();
                usage.total_tokens = usage.total_tokens.saturating_add(tokens);
                usage.cost_usd = (usage.cost_usd + cost).max(0.0);
                if tokens > 0 {
                    let now = Instant::now();
                    usage.prune(now);
                    usage.token_window.push_back((now, tokens));
                }
            }
            let updates = scopes
                .iter()
                .map(|key| {
                    let total_tokens = inner.tokens.entry(key.clone()).or_default();
                    *total_tokens = total_tokens.saturating_add(tokens);
                    let spent = inner.spend.entry(key.clone()).or_default();
                    *spent = (*spent + cost).max(0.0);
                    let spent = *spent;
                    let limit = inner.limits.get(key).copied().unwrap_or_default();
                    (key, spent, budget_status(spent, limit))
                })
                .collect::<Vec<_>>();
            (updates, remote_status)
        };
        for ((scope, scope_id), spent, status) in updates.0 {
            #[cfg(feature = "telemetry")]
            metrics::gauge!("uar_cost_budget_spent_usd", "scope" => scope.as_str()).set(spent);
            if let BudgetStatus::Warning { limit_usd, .. }
            | BudgetStatus::Exceeded { limit_usd, .. } = status
            {
                tracing::warn!(
                    scope = scope.as_str(),
                    scope_id,
                    spent_usd = spent,
                    limit_usd,
                    exceeded = status.is_exceeded(),
                    "Model call reached a cost budget threshold"
                );
            }
        }
        updates.1
    }
}

fn check_remote_grant(
    inner: &Inner,
    remote: Option<&RemoteUsageGrantBinding>,
    model_attempt: bool,
    tool_attempt: bool,
    additional_tokens: u64,
    additional_cost: f64,
) -> anyhow::Result<()> {
    let Some(remote) = remote else {
        return Ok(());
    };
    remote.grant.validate()?;
    if let Some(seconds) = remote.grant.expires_after_seconds {
        let deadline = remote
            .started_at
            .checked_add(Duration::from_secs(seconds))
            .ok_or_else(|| {
                anyhow::anyhow!("Remote usage grant timeout exceeds the runtime clock range")
            })?;
        anyhow::ensure!(Instant::now() < deadline, "Remote usage grant expired");
    }
    let usage = inner.runs.get(&remote.accounting_id);
    let tokens = usage.map_or(0, |usage| usage.total_tokens);
    let cost = usage.map_or(0.0, |usage| usage.cost_usd);
    let requests = usage.map_or(0, |usage| usage.total_requests);
    let tools = usage.map_or(0, |usage| usage.tool_calls);
    if let Some(limit) = remote.grant.max_total_tokens {
        anyhow::ensure!(
            tokens.saturating_add(additional_tokens) <= limit,
            "Remote cumulative token grant exceeded"
        );
        if model_attempt {
            anyhow::ensure!(tokens < limit, "Remote cumulative token grant exhausted");
        }
    }
    if let Some(limit) = remote.grant.max_total_cost_usd {
        anyhow::ensure!(
            cost + additional_cost <= limit + f64::EPSILON,
            "Remote cumulative cost grant exceeded"
        );
        if model_attempt {
            anyhow::ensure!(cost < limit, "Remote cumulative cost grant exhausted");
        }
    }
    if let Some(limit) = remote.grant.max_total_model_requests {
        anyhow::ensure!(
            requests.saturating_add(u64::from(model_attempt)) <= limit,
            "Remote cumulative model-request grant exceeded"
        );
    }
    if let Some(limit) = remote.grant.max_total_tool_calls {
        anyhow::ensure!(
            tools.saturating_add(u64::from(tool_attempt)) <= limit,
            "Remote cumulative tool-call grant exceeded"
        );
    }
    Ok(())
}

fn reserved_scope(inner: &Inner, key: &(BudgetScope, String)) -> (u64, f64) {
    inner
        .remote_reservations
        .values()
        .filter(|reservation| reservation.scopes.contains(key))
        .fold((0_u64, 0.0_f64), |(tokens, cost), reservation| {
            (
                tokens.saturating_add(
                    reservation
                        .grant
                        .max_total_tokens
                        .unwrap_or(0)
                        .saturating_sub(reservation.used.total_tokens),
                ),
                cost + (reservation.grant.max_total_cost_usd.unwrap_or(0.0)
                    - reservation.used.cost_usd)
                    .max(0.0),
            )
        })
}

fn reserved_activity(inner: &Inner, usage_id: &str) -> (u64, u64, u64) {
    inner
        .remote_reservations
        .values()
        .filter(|reservation| reservation.usage_id == usage_id)
        .fold(
            (0_u64, 0_u64, 0_u64),
            |(requests, tokens, tools), reservation| {
                (
                    requests
                        .saturating_add(reservation.grant.max_total_model_requests.unwrap_or(0)),
                    tokens.saturating_add(reservation.grant.max_total_tokens.unwrap_or(0)),
                    tools.saturating_add(
                        reservation
                            .grant
                            .max_total_tool_calls
                            .unwrap_or(0)
                            .saturating_sub(reservation.used.tool_calls),
                    ),
                )
            },
        )
}

fn lower_remaining(current: Option<u64>, used: u64, reserved: u64) -> Option<u64> {
    current.map(|limit| limit.saturating_sub(used.saturating_add(reserved)))
}

fn lower_option(left: Option<u64>, right: Option<u64>) -> Option<u64> {
    match (left, right) {
        (Some(left), Some(right)) => Some(left.min(right)),
        (left, right) => left.or(right),
    }
}

fn budget_status(spent: f64, limit: BudgetLimit) -> BudgetStatus {
    if spent >= limit.limit_usd {
        BudgetStatus::Exceeded {
            spent_usd: spent,
            limit_usd: limit.limit_usd,
        }
    } else if limit.limit_usd.is_finite() && spent >= limit.warn_at * limit.limit_usd {
        BudgetStatus::Warning {
            spent_usd: spent,
            limit_usd: limit.limit_usd,
        }
    } else {
        BudgetStatus::Ok
    }
}

/// Host-selected accounting identities. Descendants retain these root scopes;
/// changing models must not create a fresh budget or change the payer.
#[derive(Debug, Clone)]
pub(crate) struct ModelCallBudget {
    tracker: CostBudgetTracker,
    scopes: Vec<(BudgetScope, String)>,
    cancellation: tokio_util::sync::CancellationToken,
    limits: ThreadBudgets,
    usage_id: String,
    remote: Option<RemoteUsageGrantBinding>,
    started_at: Instant,
    deadline: Option<Instant>,
    durable: Option<DurableBudget>,
}

impl ModelCallBudget {
    /// Capture counters without recording another accounting charge.
    pub(crate) fn snapshot_client_handoff(&self) -> anyhow::Result<ClientToolBudgetSnapshot> {
        let durable = self.durable.as_ref().ok_or_else(|| anyhow::anyhow!("client_budget_durable_accounting_required"))?;
        let now = Instant::now();
        let wall_ms = chrono::Utc::now().timestamp_millis();
        let inner = self.tracker.inner.lock().map_err(|_| anyhow::anyhow!("Cost budget ledger is unavailable"))?;
        anyhow::ensure!(!inner.remote_reservations.values().any(|reservation|
            reservation.usage_id == self.usage_id
                || self.remote.as_ref().is_some_and(|remote| reservation.usage_id == remote.accounting_id)),
            "client_budget_remote_liability_unresolved");
        let scopes = self.scopes.iter().map(|(scope, id)| {
            let key = (*scope, id.clone());
            ClientToolScopeSnapshot { scope: *scope, id: id.clone(),
                tokens: inner.tokens.get(&key).copied().unwrap_or(0),
                cost_usd: inner.spend.get(&key).copied().unwrap_or(0.0) }
        }).collect();
        let deadline_at_ms = self.deadline.map(|deadline| {
            let remaining = i64::try_from(deadline.saturating_duration_since(now).as_millis())?;
            wall_ms.checked_add(remaining).ok_or_else(|| anyhow::anyhow!("client_budget_clock_range"))
        }).transpose()?;
        let remote = self.remote.as_ref().map(|remote| -> anyhow::Result<ClientToolRemoteSnapshot> {
            Ok(ClientToolRemoteSnapshot { accounting_id: remote.accounting_id.clone(),
                elapsed_ms: u64::try_from(now.saturating_duration_since(remote.started_at).as_millis())?,
                activity: handoff_activity(inner.runs.get(&remote.accounting_id), now, wall_ms)? })
        }).transpose()?;
        Ok(ClientToolBudgetSnapshot { version: 1, owner: durable.owner.clone(),
            source_usage_id: self.usage_id.clone(), captured_at_ms: wall_ms,
            elapsed_root_ms: u64::try_from(now.saturating_duration_since(self.started_at).as_millis())?,
            deadline_at_ms, limits: self.limits.clone(), scopes,
            activity: handoff_activity(inner.runs.get(&self.usage_id), now, wall_ms)?, remote })
    }

    /// Restore a host-owned snapshot under freshly resolved policy and authority.
    /// Existing durable scope liabilities are rebound, never replayed as charges.
    pub(crate) async fn restore_client_handoff(
        mut self, snapshot: &ClientToolBudgetSnapshot,
        owner: &crate::uar::runtime::actor::messages::ActorOwner,
    ) -> anyhow::Result<Self> {
        let durable = self.durable.as_ref().ok_or_else(|| anyhow::anyhow!("client_budget_durable_accounting_required"))?;
        let store = Arc::clone(&durable.store);
        anyhow::ensure!(snapshot.version == 1 && !snapshot.source_usage_id.is_empty() && durable.owner == snapshot.owner
            && owner.session_owner_key()? == snapshot.owner, "client_budget_owner_mismatch");
        snapshot.limits.validate()?;
        anyhow::ensure!(snapshot.scopes.len() == self.scopes.len()
            && snapshot.scopes.iter().map(|scope| (scope.scope, scope.id.as_str()))
                .collect::<std::collections::HashSet<_>>().len() == snapshot.scopes.len()
            && snapshot.scopes.iter().filter(|scope| scope.scope == BudgetScope::Run).count() == 1
            && snapshot.scopes.iter().all(|scope| !scope.id.is_empty() && scope.cost_usd.is_finite() && scope.cost_usd >= 0.0
                && (scope.scope == BudgetScope::Run || self.scopes.contains(&(scope.scope, scope.id.clone())))),
            "client_budget_scope_mismatch");
        anyhow::ensure!(snapshot.remote.as_ref().map(|v| &v.accounting_id)
            == self.remote.as_ref().map(|v| &v.accounting_id), "client_budget_remote_scope_mismatch");
        let now = Instant::now();
        let wall_ms = chrono::Utc::now().timestamp_millis();
        let away_ms = u64::try_from(wall_ms.checked_sub(snapshot.captured_at_ms)
            .ok_or_else(|| anyhow::anyhow!("client_budget_clock_range"))?)
            .map_err(|_| anyhow::anyhow!("client_budget_clock_reversed"))?;
        let elapsed = snapshot.elapsed_root_ms.checked_add(away_ms)
            .ok_or_else(|| anyhow::anyhow!("client_budget_clock_range"))?;
        self.started_at = now.checked_sub(Duration::from_millis(elapsed))
            .ok_or_else(|| anyhow::anyhow!("client_budget_clock_range"))?;
        self.limits = self.limits.intersect(&snapshot.limits);
        let saved_deadline = snapshot.deadline_at_ms.map(|deadline| -> anyhow::Result<Instant> {
            let remaining = u64::try_from(deadline.checked_sub(wall_ms)
                .ok_or_else(|| anyhow::anyhow!("client_budget_clock_range"))?)
                .map_err(|_| anyhow::anyhow!("client_budget_deadline_exceeded"))?;
            anyhow::ensure!(remaining > 0, "client_budget_deadline_exceeded");
            now.checked_add(Duration::from_millis(remaining)).ok_or_else(|| anyhow::anyhow!("client_budget_clock_range"))
        }).transpose()?;
        self.deadline = [self.deadline, saved_deadline, Self::deadline(self.started_at, &self.limits)?]
            .into_iter().flatten().min();
        if let (Some(current), Some(saved)) = (&mut self.remote, &snapshot.remote) {
            let elapsed = saved.elapsed_ms.checked_add(away_ms).ok_or_else(|| anyhow::anyhow!("client_budget_clock_range"))?;
            current.started_at = now.checked_sub(Duration::from_millis(elapsed))
                .ok_or_else(|| anyhow::anyhow!("client_budget_clock_range"))?;
        }
        {
            let mut inner = self.tracker.inner.lock().map_err(|_| anyhow::anyhow!("Cost budget ledger is unavailable"))?;
            for scope in &snapshot.scopes {
                let key = (scope.scope, scope.id.clone());
                // Current policy's run-specific ceiling follows the retained
                // economic scope; persistent scope limits remain authoritative.
                if scope.scope == BudgetScope::Run
                    && let Some(current_id) = self.scopes.iter().find(|(kind, _)| *kind == BudgetScope::Run)
                    && let Some(current_limit) = inner.limits.get(current_id).copied() {
                    let retained = inner.limits.entry(key.clone()).or_insert(current_limit);
                    retained.limit_usd = retained.limit_usd.min(current_limit.limit_usd);
                    retained.warn_at = retained.warn_at.min(current_limit.warn_at);
                }
                let tokens = inner.tokens.entry(key.clone()).or_default();
                *tokens = (*tokens).max(scope.tokens);
                let cost = inner.spend.entry(key).or_default();
                *cost = (*cost).max(scope.cost_usd);
            }
            restore_handoff_activity(inner.runs.entry(self.usage_id.clone()).or_default(), &snapshot.activity, now, wall_ms)?;
            if let Some(remote) = &snapshot.remote {
                restore_handoff_activity(inner.runs.entry(remote.accounting_id.clone()).or_default(), &remote.activity, now, wall_ms)?;
            }
        }
        self.scopes = snapshot.scopes.iter().map(|scope| (scope.scope, scope.id.clone())).collect();
        self = self.with_durable_accounting(Some(store), Some(owner)).await?;
        self.admit()?;
        Ok(self)
    }

    pub(crate) fn for_run(
        tracker: CostBudgetTracker,
        run_id: String,
        session_id: String,
        agent_id: String,
        cancellation: tokio_util::sync::CancellationToken,
        limits: ThreadBudgets,
        remote: Option<RemoteUsageGrantBinding>,
        started_at: Instant,
    ) -> anyhow::Result<Self> {
        let deadline = Self::deadline(started_at, &limits)?;
        let usage_id = run_id.clone();
        Ok(Self {
            tracker,
            cancellation,
            limits,
            usage_id,
            remote,
            started_at,
            deadline,
            durable: None,
            scopes: vec![
                (BudgetScope::Run, run_id),
                (BudgetScope::Session, session_id),
                (BudgetScope::Agent, agent_id),
                (BudgetScope::Global, "global".into()),
            ],
        })
    }

    fn deadline(started_at: Instant, limits: &ThreadBudgets) -> anyhow::Result<Option<Instant>> {
        limits
            .timeout_seconds
            .map(|seconds| {
                started_at
                    .checked_add(Duration::from_secs(seconds))
                    .ok_or_else(|| {
                        anyhow::anyhow!("Budget timeout exceeds the runtime clock range")
                    })
            })
            .transpose()
    }

    pub(crate) fn narrowed(&self, requested: &ThreadBudgets) -> anyhow::Result<Self> {
        let limits = self.limits.intersect(requested);
        let deadline = match (Self::deadline(self.started_at, &limits)?, self.deadline) {
            (Some(local), Some(existing)) => Some(local.min(existing)),
            (local, existing) => local.or(existing),
        };
        let narrowed = Self {
            limits,
            deadline,
            ..self.clone()
        };
        narrowed.admit()?;
        Ok(narrowed)
    }

    /// Reserve the source root's remaining finite capacity for one remote
    /// child. Other local/remote work sees the reservation immediately. The
    /// returned contract grant is a balance, not the original ceiling.
    fn reserve_remote_local(
        &self,
        requested: &ThreadBudgets,
    ) -> anyhow::Result<RemoteBudgetReservation> {
        self.admit()?;
        let requested = self.limits.intersect(requested);
        let mut inner = self
            .tracker
            .inner
            .lock()
            .map_err(|_| anyhow::anyhow!("Cost budget ledger is unavailable"))?;
        CostBudgetTracker::check_limits(&inner, &self.scopes, &requested)?;

        let run_key = &self.scopes[0];
        let session_key = &self.scopes[1];
        let (reserved_run_tokens, _) = reserved_scope(&inner, run_key);
        let (reserved_session_tokens, _) = reserved_scope(&inner, session_key);
        let run_tokens = inner.tokens.get(run_key).copied().unwrap_or(0);
        let session_tokens = inner.tokens.get(session_key).copied().unwrap_or(0);
        let token_capacity = lower_option(
            lower_remaining(
                requested.max_tokens_per_turn,
                run_tokens,
                reserved_run_tokens,
            ),
            lower_remaining(
                requested.max_tokens_per_session,
                session_tokens,
                reserved_session_tokens,
            ),
        );

        let mut cost_capacity = requested.max_cost_per_session_usd.map(|limit| {
            let (_, reserved) = reserved_scope(&inner, session_key);
            (limit - inner.spend.get(session_key).copied().unwrap_or(0.0) - reserved).max(0.0)
        });
        for key in &self.scopes {
            if let Some(limit) = inner
                .limits
                .get(key)
                .filter(|limit| limit.limit_usd.is_finite())
            {
                let (_, reserved) = reserved_scope(&inner, key);
                let remaining =
                    (limit.limit_usd - inner.spend.get(key).copied().unwrap_or(0.0) - reserved)
                        .max(0.0);
                cost_capacity =
                    Some(cost_capacity.map_or(remaining, |current| current.min(remaining)));
            }
        }

        let now = Instant::now();
        let (reserved_requests, reserved_rate_tokens, reserved_tools) =
            reserved_activity(&inner, &self.usage_id);
        let usage = inner.runs.entry(self.usage_id.clone()).or_default();
        usage.prune(now);
        let active_requests = usage
            .requests
            .iter()
            .fold(0_u64, |total, (_, count)| total.saturating_add(*count));
        let active_rate_tokens = usage
            .token_window
            .iter()
            .fold(0_u64, |total, (_, tokens)| total.saturating_add(*tokens));
        let request_capacity = requested
            .rate_limit
            .as_ref()
            .and_then(|rate| rate.requests_per_minute)
            .map(|limit| {
                u64::from(limit).saturating_sub(active_requests.saturating_add(reserved_requests))
            });
        let rate_token_capacity = requested
            .rate_limit
            .as_ref()
            .and_then(|rate| rate.tokens_per_minute)
            .map(|limit| {
                limit.saturating_sub(active_rate_tokens.saturating_add(reserved_rate_tokens))
            });
        let token_capacity = lower_option(token_capacity, rate_token_capacity);
        let tool_capacity = requested.max_tool_calls_per_turn.map(|limit| {
            u64::from(limit).saturating_sub(usage.tool_calls.saturating_add(reserved_tools))
        });
        let timeout_seconds = match self.deadline {
            Some(deadline) => {
                let remaining = deadline.saturating_duration_since(now).as_secs();
                anyhow::ensure!(remaining > 0, "Root budget timeout exceeded");
                Some(
                    requested
                        .timeout_seconds
                        .map_or(remaining, |limit| limit.min(remaining)),
                )
            }
            None => requested.timeout_seconds,
        };
        let grant = UarUsageGrant {
            max_total_tokens: token_capacity,
            max_total_cost_usd: cost_capacity,
            max_total_model_requests: request_capacity,
            max_total_tool_calls: tool_capacity,
            expires_after_seconds: timeout_seconds,
        };
        grant.validate()?;
        let id = uuid::Uuid::new_v4().to_string();
        inner.remote_reservations.insert(
            id.clone(),
            RemoteReservationState {
                scopes: self.scopes.clone(),
                usage_id: self.usage_id.clone(),
                grant: grant.clone(),
                used: Default::default(),
            },
        );
        Ok(RemoteBudgetReservation {
            tracker: self.tracker.clone(),
            id,
            grant,
            released: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            durable: None,
            unresolved: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        })
    }

    pub(crate) fn execution_deadline(&self) -> Option<Instant> {
        self.deadline
    }

    pub(crate) fn admit(&self) -> anyhow::Result<()> {
        if self.cancellation.is_cancelled() {
            return Err(BudgetFailureOrigin::RootCancellation.into());
        }
        if self
            .deadline
            .is_some_and(|deadline| Instant::now() >= deadline)
        {
            return Err(BudgetFailureOrigin::RootDeadline.into());
        }
        self.tracker.admit(
            &self.scopes,
            &self.usage_id,
            &self.limits,
            None,
            self.remote.as_ref(),
        ).map_err(|error| error.context(BudgetFailureOrigin::Admission))
    }

    pub(crate) fn admit_tool(&self) -> anyhow::Result<()> {
        self.admit()?;
        self.tracker.admit_tool(
            &self.scopes,
            &self.usage_id,
            &self.limits,
            self.remote.as_ref(),
        ).map_err(|error| error.context(BudgetFailureOrigin::Admission))
    }

    /// Called only by the trusted host before an authorized tool attempt.
    pub(crate) async fn admit_tool_durable(&self) -> anyhow::Result<()> {
        self.admit_tool()?;
        self.admit_remote_activity(crate::uar::persistence::accounting::RemoteGrantActivity::Tool).await
    }

    async fn admit_remote_activity(&self, activity: crate::uar::persistence::accounting::RemoteGrantActivity) -> anyhow::Result<()> {
        let Some(remote) = &self.remote else { return Ok(()); };
        let durable = self.durable.as_ref().ok_or(AccountingError::Unsupported)?;
        durable.store.admit_remote_grant(&durable.owner, &remote.accounting_id, &remote.grant, activity).await?;
        Ok(())
    }

    async fn expired(&self) {
        match self.deadline {
            Some(deadline) => tokio::time::sleep_until(deadline.into()).await,
            None => std::future::pending::<()>().await,
        }
    }

    pub(crate) fn bind(
        &self,
        model: String,
        inner: Arc<dyn crate::llm::LlmDriver>,
    ) -> Arc<dyn crate::llm::LlmDriver> {
        Arc::new(BudgetedModelDriver {
            inner,
            model,
            budget: self.clone(),
        })
    }
}

impl RemoteBudgetReservation {
    pub(crate) fn grant(&self) -> &UarUsageGrant {
        &self.grant
    }

    pub(crate) fn is_released(&self) -> bool {
        self.released.load(std::sync::atomic::Ordering::Acquire)
    }

    pub(crate) fn check_active(&self) -> anyhow::Result<()> {
        anyhow::ensure!(
            !self.released.load(std::sync::atomic::Ordering::Acquire),
            "Remote budget reservation has been released"
        );
        let inner = self
            .tracker
            .inner
            .lock()
            .map_err(|_| anyhow::anyhow!("Cost budget ledger is unavailable"))?;
        anyhow::ensure!(
            inner.remote_reservations.contains_key(&self.id),
            "Remote budget reservation is unavailable"
        );
        Ok(())
    }

    /// A peer without a usable receipt retains unknown durable liability.
    /// Conservatively occupy finite live-tracker capacity without claiming this
    /// is a confirmed provider charge or authorizing durable release.
    pub(crate) async fn charge_reserved_capacity(&self) -> anyhow::Result<()> {
        self.check_active()?;
        let state = self
            .tracker
            .inner
            .lock()
            .map_err(|_| anyhow::anyhow!("Cost budget ledger is unavailable"))?
            .remote_reservations
            .get(&self.id)
            .ok_or_else(|| anyhow::anyhow!("Remote budget reservation is unavailable"))?
            .clone();
        self.unresolved.store(true, std::sync::atomic::Ordering::Release);
        self.record_cumulative_local(UarUsageReceipt {
            total_tokens: state.used.total_tokens.max(
                state
                    .grant
                    .max_total_tokens
                    .unwrap_or(state.used.total_tokens),
            ),
            cost_usd: state.used.cost_usd.max(
                state
                    .grant
                    .max_total_cost_usd
                    .unwrap_or(state.used.cost_usd),
            ),
            model_requests: state.used.model_requests.max(
                state
                    .grant
                    .max_total_model_requests
                    .unwrap_or(state.used.model_requests),
            ),
            tool_calls: state.used.tool_calls.max(
                state
                    .grant
                    .max_total_tool_calls
                    .unwrap_or(state.used.tool_calls),
            ),
        })
    }

    /// Charge a monotonic cumulative peer receipt while retaining all unused
    /// capacity for this child. A malformed or over-budget receipt does not
    /// release the reservation.
    fn record_cumulative_local(&self, receipt: UarUsageReceipt) -> anyhow::Result<()> {
        receipt.validate()?;
        self.check_active()?;
        let mut inner = self
            .tracker
            .inner
            .lock()
            .map_err(|_| anyhow::anyhow!("Cost budget ledger is unavailable"))?;
        let current = inner
            .remote_reservations
            .get(&self.id)
            .ok_or_else(|| anyhow::anyhow!("Remote budget reservation is unavailable"))?
            .clone();
        anyhow::ensure!(
            receipt.total_tokens >= current.used.total_tokens
                && receipt.cost_usd + f64::EPSILON >= current.used.cost_usd
                && receipt.model_requests >= current.used.model_requests
                && receipt.tool_calls >= current.used.tool_calls,
            "Remote usage receipt is not cumulative"
        );
        let grant_status = (|| {
            if let Some(limit) = current.grant.max_total_tokens {
                anyhow::ensure!(
                    receipt.total_tokens <= limit,
                    "Remote token grant was exceeded"
                );
            }
            if let Some(limit) = current.grant.max_total_cost_usd {
                anyhow::ensure!(
                    receipt.cost_usd <= limit + f64::EPSILON,
                    "Remote cost grant was exceeded"
                );
            }
            if let Some(limit) = current.grant.max_total_tool_calls {
                anyhow::ensure!(
                    receipt.tool_calls <= limit,
                    "Remote tool-call grant was exceeded"
                );
            }
            if let Some(limit) = current.grant.max_total_model_requests {
                anyhow::ensure!(
                    receipt.model_requests <= limit,
                    "Remote model-request grant was exceeded"
                );
            }
            Ok::<(), anyhow::Error>(())
        })();

        let delta_tokens = receipt.total_tokens - current.used.total_tokens;
        let delta_cost = receipt.cost_usd - current.used.cost_usd;
        let delta_requests = receipt.model_requests - current.used.model_requests;
        let delta_tools = receipt.tool_calls - current.used.tool_calls;
        let usage = inner.runs.entry(current.usage_id.clone()).or_default();
        usage.total_tokens = usage.total_tokens.saturating_add(delta_tokens);
        usage.cost_usd = (usage.cost_usd + delta_cost).max(0.0);
        usage.total_requests = usage.total_requests.saturating_add(delta_requests);
        usage.tool_calls = usage.tool_calls.saturating_add(delta_tools);
        for key in &current.scopes {
            *inner.tokens.entry(key.clone()).or_default() = inner
                .tokens
                .get(key)
                .copied()
                .unwrap_or(0)
                .saturating_add(delta_tokens);
            *inner.spend.entry(key.clone()).or_default() =
                (inner.spend.get(key).copied().unwrap_or(0.0) + delta_cost).max(0.0);
        }
        inner
            .remote_reservations
            .get_mut(&self.id)
            .ok_or_else(|| anyhow::anyhow!("Remote budget reservation disappeared"))?
            .used = receipt;
        grant_status
    }

    /// Release only when the host proves no dispatch occurred or the exact
    /// remote task confirms terminal cleanup.
    fn release_confirmed_local(&self) -> anyhow::Result<()> {
        if self.is_released() {
            return Ok(());
        }
        let mut inner = self
            .tracker
            .inner
            .lock()
            .map_err(|_| anyhow::anyhow!("Cost budget ledger is unavailable"))?;
        if self.is_released() {
            return Ok(());
        }
        let removed = inner
            .remote_reservations
            .remove(&self.id)
            .ok_or_else(|| anyhow::anyhow!("Remote budget reservation is unavailable"))?;
        let now = Instant::now();
        let usage = inner.runs.entry(removed.usage_id).or_default();
        usage.prune(now);
        // The source has no trustworthy peer-side timestamps. Keep the full
        // reservation while work is live, then conservatively attribute actual
        // traffic at confirmed settlement so capacity cannot widen on close.
        if removed.used.model_requests > 0 {
            usage.requests.push_back((now, removed.used.model_requests));
        }
        if removed.used.total_tokens > 0 {
            usage
                .token_window
                .push_back((now, removed.used.total_tokens));
        }
        self.released
            .store(true, std::sync::atomic::Ordering::Release);
        Ok(())
    }
}

struct BudgetedModelDriver {
    inner: Arc<dyn crate::llm::LlmDriver>,
    model: String,
    budget: ModelCallBudget,
}

#[async_trait::async_trait]
impl crate::llm::LlmDriver for BudgetedModelDriver {
    fn with_bound_model(&self, model: &str) -> anyhow::Result<Arc<dyn crate::llm::LlmDriver>> {
        Ok(self
            .budget
            .bind(model.to_string(), self.inner.with_bound_model(model)?))
    }

    async fn stream(
        &self,
        mut request: crate::llm::LlmRequest,
    ) -> anyhow::Result<crate::llm::ExternalDriverStream> {
        use futures::StreamExt;
        self.budget.narrow_team_output_cap(&mut request).await
            .map_err(|error| error.context(BudgetFailureOrigin::Admission))?;
        self.budget.admit()?;
        self.budget.tracker.admit(
            &self.budget.scopes,
            &self.budget.usage_id,
            &self.budget.limits,
            Some(&self.model),
            self.budget.remote.as_ref(),
        ).map_err(|error| error.context(BudgetFailureOrigin::Admission))?;
        let durable_call = self.budget.begin_call(&self.model, &request).await
            .map_err(|error| error.context(BudgetFailureOrigin::Admission))?;
        let mut stream = tokio::select! {
            biased;
            _ = self.budget.cancellation.cancelled() => return Err(BudgetFailureOrigin::RootCancellation.into()),
            () = self.budget.expired() => return Err(BudgetFailureOrigin::RootDeadline.into()),
            result = self.inner.stream(request) => result?,
        };
        let budget = self.budget.clone();
        let model = self.model.clone();
        Ok(Box::pin(async_stream::stream! {
            let mut charged = 0.0_f64;
            let mut charged_tokens = 0_u64;
            let mut last_price = None;
            let mut reported_usage = false;
            let mut confirmed_usage = false;
            let mut stream_failed = false;
            loop {
                let next = tokio::select! {
                    biased;
                    _ = budget.cancellation.cancelled() => {
                        yield Err(BudgetFailureOrigin::RootCancellation.into());
                        break;
                    }
                    () = budget.expired() => {
                        yield Err(BudgetFailureOrigin::RootDeadline.into());
                        break;
                    }
                    next = stream.next() => next,
                };
                let Some(event) = next else {
                    break;
                };
                stream_failed |= event.is_err() || matches!(&event, Ok(crate::normalized::NormalizedEvent::Error { .. }));
                if let Ok(crate::normalized::NormalizedEvent::Usage {
                    prompt_tokens, completion_tokens, cached_tokens, cache_creation_tokens, ..
                }) = &event {
                    let price = crate::llm::catalog::estimate_cost_with_cache_write(
                        &model, u64::from(*prompt_tokens), u64::from(*completion_tokens),
                        u64::from(cached_tokens.unwrap_or(0)),
                        u64::from(cache_creation_tokens.unwrap_or(0)),
                    );
                    if !reported_usage {
                        if let Err(error) = budget.tracker.record_usage_report(&budget.usage_id, budget.remote.as_ref(), price.is_some(), false) {
                            yield Err(error.context(BudgetFailureOrigin::UsageAccounting));
                            break;
                        }
                        reported_usage = true;
                    }
                    let cost = price.unwrap_or(charged);
                    // Replace this request's cumulative estimate, not add it
                    // again. Later cache counts may reduce that estimate.
                    let delta = cost - charged;
                    // Count usage even for models without a catalog price.
                    // Repeated cumulative events are not new tokens; a later
                    // smaller report cannot refund already observed usage.
                    let tokens = u64::from(*prompt_tokens) + u64::from(*completion_tokens);
                    let new_tokens = tokens.saturating_sub(charged_tokens);
                    if let Some(call) = &durable_call {
                        if let Err(error) = call.usage(price, charged_tokens.max(tokens), false).await {
                            tracing::warn!(%error, call_id = %call.identity.call_id, "Durable usage remains pending");
                            yield Ok(crate::normalized::NormalizedEvent::Custom { source: "uar.accounting".into(), event_name: "unresolved".into(),
                                payload: serde_json::json!({"callId": call.identity.call_id, "state": "pending"}) });
                        }
                    }
                    if delta != 0.0 || new_tokens > 0 {
                        if let Err(error) = budget.tracker.record_call(
                            &budget.scopes, &budget.usage_id, budget.remote.as_ref(), delta, new_tokens,
                        ) {
                            yield Err(error.context(BudgetFailureOrigin::UsageAccounting));
                            break;
                        }
                    }
                    last_price = price;
                    charged = cost;
                    charged_tokens = charged_tokens.max(tokens);

                }
                // A partial cumulative report followed by cancellation is not
                // a terminal provider counter and cannot release a reservation.
                if matches!(&event, Ok(crate::normalized::NormalizedEvent::Done)) && reported_usage && !confirmed_usage && !stream_failed {
                    let mut settled = last_price.is_some();
                    if let Some(call) = &durable_call {
                        if let Err(error) = call.usage(last_price, charged_tokens, true).await {
                            settled = false;
                            tracing::warn!(%error, call_id = %call.identity.call_id, "Execution completed with unresolved durable accounting");
                            // Retain a replayable host-observed receipt rather than
                            // treating failed persistence as a refund. The existing
                            // trusted-host reconciliation route owns later replay.
                            yield Ok(crate::normalized::NormalizedEvent::Custom { source: "uar.accounting".into(), event_name: "unresolved".into(),
                                payload: serde_json::json!({"callId": call.identity.call_id, "state": "pending",
                                    "runId": call.identity.run_id, "attempt": call.identity.attempt,
                                    "terminalUsage": {"costUsd": last_price, "tokens": charged_tokens,
                                        "evidence": call.terminal_evidence()}}) });
                        }
                    }
                    if settled {
                        if let Err(error) = budget.tracker.record_usage_report(&budget.usage_id, budget.remote.as_ref(), true, true) { yield Err(error.context(BudgetFailureOrigin::UsageAccounting)); break; }
                    }
                    confirmed_usage = true;
                }
                yield event;
            }
        }))
    }
}

#[cfg(test)]
mod tests {
    use futures::StreamExt;

    use super::*;
    use crate::llm::{LlmRequest, mock_driver::MockLlmDriver};
    use crate::normalized::NormalizedEvent;

    fn request() -> LlmRequest {
        LlmRequest {
            messages: vec![serde_json::json!({"role": "user", "content": "delegate"})],
            tools: Vec::new(),
            output_token_cap: None,
            tool_choice: None,
            parallel_tool_calls: None,
            cache_strategy: None,
            thinking_config: None,
            anthropic_system: None,
            extra_params: None,
            budget_contract: None,
        }
    }

    #[tokio::test]
    async fn accumulates_and_crosses_thresholds() {
        let t = CostBudgetTracker::new();
        t.set_limit(
            BudgetScope::Task,
            "t1",
            BudgetLimit {
                limit_usd: 1.0,
                warn_at: 0.8,
            },
        )
        .await;

        assert_eq!(
            t.record(BudgetScope::Task, "t1", 0.5).await,
            BudgetStatus::Ok
        );
        assert!(matches!(
            t.record(BudgetScope::Task, "t1", 0.35).await,
            BudgetStatus::Warning { .. }
        ));
        assert!(t.record(BudgetScope::Task, "t1", 0.2).await.is_exceeded());
        assert!((t.spent(BudgetScope::Task, "t1").await - 1.05).abs() < 1e-9);
    }

    #[tokio::test]
    async fn no_limit_never_warns() {
        let t = CostBudgetTracker::new();
        assert_eq!(
            t.record(BudgetScope::Global, "g", 1000.0).await,
            BudgetStatus::Ok
        );
    }

    #[tokio::test]
    async fn child_usage_charges_root_and_exhaustion_refuses_spawns_and_model_calls() {
        let tracker = CostBudgetTracker::new();
        let root_budget = ModelCallBudget::for_run(
            tracker.clone(),
            "root-run".to_owned(),
            "root-session".to_owned(),
            "root-agent".to_owned(),
            tokio_util::sync::CancellationToken::new(),
            ThreadBudgets {
                max_tokens_per_turn: Some(5),
                ..ThreadBudgets::default()
            },
            None,
            Instant::now(),
        )
        .expect("root budget is valid");
        let child_budget = root_budget
            .narrowed(&ThreadBudgets::default())
            .expect("child starts under the root budget");
        let provider = Arc::new(MockLlmDriver::new(vec![vec![
            NormalizedEvent::Usage {
                prompt_tokens: 3,
                completion_tokens: 2,
                total_tokens: 5,
                cached_tokens: None,
                cache_creation_tokens: None,
            },
            NormalizedEvent::Done,
        ]]));
        let child_driver =
            child_budget.bind("peer-provider/peer-model".to_owned(), provider.clone());

        let mut stream = child_driver
            .stream(request())
            .await
            .expect("first child model call is admitted");
        while let Some(event) = stream.next().await {
            event.expect("child model event succeeds");
        }

        let usage = tracker.run_usage("root-run");
        assert_eq!(usage.total_tokens, 5);
        assert_eq!(usage.model_requests, 1);

        let spawn_error = root_budget
            .narrowed(&ThreadBudgets::default())
            .expect_err("an exhausted root refuses another child spawn");
        assert!(
            spawn_error
                .to_string()
                .contains("run token budget exceeded")
        );

        let model_error = match child_driver.stream(request()).await {
            Ok(_) => panic!("an exhausted root must refuse another child model call"),
            Err(error) => error,
        };
        assert!(
            model_error
                .to_string()
                .contains("run token budget exceeded")
        );
        assert_eq!(
            provider.call_count(),
            1,
            "the refused call must not reach the provider"
        );
    }
}

/// Host capability captured once; descendants inherit the same economic scopes.
#[derive(Debug, Clone)]
struct DurableBudget {
    store: Arc<dyn PersistenceLayer>,
    owner: String,
    scopes: Vec<ScopeBinding>,
    source_reservation: Option<CallIdentity>,
}
#[derive(Debug, Clone)]
struct DurableCall { store: Arc<dyn PersistenceLayer>, identity: CallIdentity }
impl DurableCall {
    fn terminal_evidence(&self) -> String {
        format!("host-confirmed-terminal:{}", self.identity.key())
    }

    async fn usage(&self, cost: Option<f64>, tokens: u64, terminal: bool) -> anyhow::Result<()> {
        let recorded = self.store.record_accounting_usage(&CallUsage { identity: self.identity.clone(),
            cost_usd: cost, tokens, terminal,
            evidence: terminal.then(|| self.terminal_evidence()) }).await?;
        if terminal && recorded.state != crate::uar::persistence::accounting::AccountingState::Settled {
            return Err(AccountingError::Unresolved.into());
        }
        Ok(())
    }
}
impl CostBudgetTracker {
    pub(crate) fn accounting_owner(&self, run: &str) -> Option<String> {
        self.inner.lock().ok()?.accounting_owners.get(run).cloned()
    }
}
impl ModelCallBudget {
    /// Preserve explicit untracked, unlimited development execution; a configured
    /// financial/token envelope or remote grant cannot use that compatibility path.
    pub(crate) fn has_configured_limits(&self) -> bool {
        self.remote.is_some() || self.limits.max_cost_per_session_usd.is_some()
            || self.limits.max_tokens_per_turn.is_some() || self.limits.max_tokens_per_session.is_some()
            || self.tracker.inner.lock().map_or(true, |inner| self.scopes.iter().any(|key|
                inner.limits.get(key).is_some_and(|limit| limit.limit_usd.is_finite())))
    }

    /// Capture persistence and the complete middleware-verified principal in the host.
    pub(crate) async fn with_durable_accounting(mut self, store: Option<Arc<dyn PersistenceLayer>>,
        owner: Option<&crate::uar::runtime::actor::messages::ActorOwner>) -> anyhow::Result<Self> {
        let store = store.ok_or(AccountingError::Unsupported)?;
        match store.durable_store_eligibility() {
            crate::uar::persistence::DurableStoreEligibility::RemoteNotAttested => {
                anyhow::bail!("remote_surreal_durability_not_attested");
            }
            crate::uar::persistence::DurableStoreEligibility::UnsupportedBackend => {
                anyhow::bail!("durable_storage_unsupported");
            }
            crate::uar::persistence::DurableStoreEligibility::LocalDurable
            | crate::uar::persistence::DurableStoreEligibility::RemoteAttested => {}
        }
        anyhow::ensure!(store.supports_durable_accounting(), "durable_accounting_unsupported");
        let owner = owner.and_then(|owner| owner.credential_principal())
            .ok_or_else(|| anyhow::anyhow!("accounting_verified_principal_required"))?.storage_key();
        let configured = self.tracker.inner.lock()
            .map_err(|_| anyhow::anyhow!("Cost budget ledger is unavailable"))?.limits.clone();
        let mut scopes = Vec::new();
        // Only the exact pending reservation in this same durable store proves
        // shared economic authority. A different deployment must enforce its
        // own house budget; team grants also retain normal economic scopes.
        let mut shared_source = None;
        if let Some(remote) = &self.remote {
            let state = store.admit_remote_grant(&owner, &remote.accounting_id, &remote.grant,
                crate::uar::persistence::accounting::RemoteGrantActivity::Inspect).await?;
            if let Some(expires_at) = state.expires_at {
                let remaining = (expires_at - chrono::Utc::now()).to_std()
                    .map_err(|_| anyhow::anyhow!("remote_grant_expired"))?;
                let deadline = Instant::now().checked_add(remaining)
                    .ok_or_else(|| anyhow::anyhow!("remote_grant_deadline_invalid"))?;
                self.deadline = Some(self.deadline.map_or(deadline, |current| current.min(deadline)));
            }
            let id = ScopeIdentity { owner: owner.clone(), scope: "remote_destination".into(), scope_id: remote.accounting_id.clone() };
            let limits = ScopeLimits { cost_usd: remote.grant.max_total_cost_usd, tokens: remote.grant.max_total_tokens };
            let row = store.ensure_accounting_scope(&id,&limits).await?;
            scopes.push(ScopeBinding { identity: id, generation: row.generation });
            if let Some(digest) = remote.accounting_id.strip_prefix("a2a:") {
                if let Some(source) = store.get_accounting_delegation(digest).await? {
                    anyhow::ensure!(source.state == crate::uar::persistence::accounting::AccountingState::Pending
                        && source.admission.model == "remote-delegation"
                        && source.admission.reserved_cost_usd == remote.grant.max_total_cost_usd
                        && source.admission.reserved_tokens == remote.grant.max_total_tokens,
                        "shared source accounting reservation is not live or does not match the grant");
                    shared_source = Some(source.admission.identity);
                }
            }
        }
        if shared_source.is_none() {
            for (kind,id) in &self.scopes {
                let identity = ScopeIdentity { owner: if *kind == BudgetScope::Global { "deployment".into() } else { owner.clone() },
                    scope: kind.as_str().into(), scope_id: id.clone() };
                let mut limits = ScopeLimits { cost_usd: configured.get(&(*kind,id.clone())).map(|v| v.limit_usd).filter(|v| v.is_finite()), tokens: None };
                if *kind == BudgetScope::Run { limits.tokens = self.limits.max_tokens_per_turn; }
                if *kind == BudgetScope::Session {
                    limits.tokens = self.limits.max_tokens_per_session;
                    if let Some(cost) = self.limits.max_cost_per_session_usd { limits.cost_usd = Some(limits.cost_usd.map_or(cost,|v| v.min(cost))); }
                }
                let row = store.ensure_accounting_scope(&identity,&limits).await?;
                scopes.push(ScopeBinding { identity, generation: row.generation });
            }
        }
        self.tracker.inner.lock().map_err(|_| anyhow::anyhow!("Cost budget ledger is unavailable"))?
            .accounting_owners.insert(self.usage_id.clone(),owner.clone());
        self.durable = Some(DurableBudget { store, owner, scopes, source_reservation: shared_source });
        Ok(self)
    }

    async fn begin_call(&self, model: &str, request: &crate::llm::LlmRequest) -> anyhow::Result<Option<DurableCall>> {
        self.admit_remote_activity(crate::uar::persistence::accounting::RemoteGrantActivity::Model).await?;
        let Some(durable) = &self.durable else { return Ok(None); };
        let (input, output, reservation_source) = if request.budget_contract.is_some() {
            let (input,output) = crate::llm::liter_driver::request_budget_bound(&request.budget_contract.as_ref().ok_or(AccountingError::Unresolved)?.destination_model,request)?;
            (Some(input),Some(output), "final_wire_contract")
        } else if let Some(remote) = self.remote.as_ref()
            .filter(|remote| remote.accounting_id.starts_with("team-budget:")) {
            let total = remote.grant.max_total_tokens
                .ok_or_else(|| anyhow::anyhow!("team token allocation is missing"))?;
            let (input, allocated_output) = team_token_allocation(total)?;
            let output = u64::from(request.output_token_cap
                .ok_or_else(|| anyhow::anyhow!("team request has no output token cap"))?);
            anyhow::ensure!(output <= allocated_output,
                "team request output token cap exceeds its host allocation");
            // The host chooses this finite split; it is not a measured provider
            // input count and therefore does not qualify a settings-only route.
            (Some(input), Some(output), "team_host_input_allowance_and_output_cap")
        } else {
            let info = model.split_once('/').and_then(|(provider,model)| crate::llm::catalog::ModelCatalog::global().model(provider,model));
            (info.map(|m| m.limits.context_window).filter(|v| *v > 0), info.map(|m| m.limits.max_output).filter(|v| *v > 0), "catalog_context_and_output")
        };
        let tokens = input.zip(output).and_then(|(input,output)| input.checked_add(output));
        let cost = input.zip(output).and_then(|(input,output)| {
            // Reserve the most expensive input category; cache writes can cost
            // more than uncached input and cannot be silently discounted.
            let plain = crate::llm::catalog::estimate_cost_with_cache_write(model,input,output,0,0)?;
            let writes = crate::llm::catalog::estimate_cost_with_cache_write(model,input,output,0,input)?;
            let reads = crate::llm::catalog::estimate_cost_with_cache_write(model,input,output,input,0)?;
            Some(plain.max(writes).max(reads))
        });
        let identity = CallIdentity { owner: durable.owner.clone(), run_id: self.usage_id.clone(),
            call_id: uuid::Uuid::new_v4().to_string(), attempt: 1 };
        durable.store.admit_accounting_call(&CallAdmission { identity: identity.clone(), model: model.into(),
            scopes: durable.scopes.clone(), reserved_cost_usd: cost, reserved_tokens: tokens,
            source_reservation: durable.source_reservation.clone() }).await.map_err(|error| {
                // Only this closed, field-free error enum may supply a cause.
                // Provider/storage error chains and request content stay private;
                // `storage_error_kind` is a closed label derived from the chain, never its text.
                let cause = error.downcast_ref::<AccountingError>()
                    .map(ToString::to_string)
                    .unwrap_or_else(|| "accounting_admission_unavailable".to_owned());
                let reference = uuid::Uuid::new_v4();
                tracing::error!(
                    run_id = %self.usage_id,
                    stage = "model_call_admission",
                    category = BudgetFailureOrigin::Admission.category(),
                    cause,
                    storage_error_kind = storage_error_kind(&error),
                    storage_error_detail = storage_error_detail(&error),
                    diagnostic_reference = %reference,
                    reservation_source,
                    reserved_tokens = ?tokens,
                    configured_turn_token_limit = ?self.limits.max_tokens_per_turn,
                    configured_session_token_limit = ?self.limits.max_tokens_per_session,
                    "Model call budget admission refused"
                );
                error
            })?;
        Ok(Some(DurableCall { store: durable.store.clone(), identity }))
    }

    /// Keep every team provider dispatch within the remaining durable grant.
    /// Exact final-wire contracts already carry their own counted budget and are
    /// deliberately left unchanged.
    async fn narrow_team_output_cap(
        &self,
        request: &mut crate::llm::LlmRequest,
    ) -> anyhow::Result<()> {
        if request.budget_contract.is_some() {
            return Ok(());
        }
        let Some(remote) = self.remote.as_ref()
            .filter(|remote| remote.accounting_id.starts_with("team-budget:"))
        else {
            return Ok(());
        };
        let total = remote.grant.max_total_tokens
            .ok_or_else(|| anyhow::anyhow!("team token allocation is missing"))?;
        let (input, allocated_output) = team_token_allocation(total)?;
        let configured_output = u64::from(request.output_token_cap
            .ok_or_else(|| anyhow::anyhow!("team request has no output token cap"))?);
        anyhow::ensure!(configured_output <= allocated_output,
            "team request output token cap exceeds its host allocation");
        let durable = self.durable.as_ref()
            .ok_or_else(|| anyhow::anyhow!("team budget durable accounting is required"))?;

        let mut remaining = total;
        for binding in &durable.scopes {
            let scope = durable.store.get_accounting_scope(&binding.identity).await?
                .ok_or(AccountingError::Missing)?;
            if scope.generation != binding.generation {
                return Err(AccountingError::Conflict.into());
            }
            if let Some(limit) = scope.limits.tokens {
                let available = limit.saturating_sub(
                    scope.settled_tokens.saturating_add(scope.reserved_tokens),
                );
                remaining = remaining.min(available);
            }
        }
        let output = configured_output.min(remaining.saturating_sub(input));
        anyhow::ensure!(output > 0, "team token grant has no remaining output allowance");
        request.output_token_cap = Some(u32::try_from(output)?);
        Ok(())
    }

    /// The trusted host calls this; the read-only kernel only exposes the budget handle.
    pub(crate) async fn reserve_remote(&self, requested: &ThreadBudgets) -> anyhow::Result<RemoteBudgetReservation> {
        let mut reservation = self.reserve_remote_local(requested)?;
        if let Some(durable) = &self.durable {
            let result = async {
                // Narrow the process-local offer with authoritative current balances.
                for binding in &durable.scopes {
                    let scope = durable.store.get_accounting_scope(&binding.identity).await?.ok_or(AccountingError::Missing)?;
                    if scope.generation != binding.generation { return Err(AccountingError::Conflict.into()); }
                    if let Some(limit) = scope.limits.cost_usd {
                        let available = (limit-scope.settled_cost_usd-scope.reserved_cost_usd).max(0.0);
                        reservation.grant.max_total_cost_usd = Some(reservation.grant.max_total_cost_usd.map_or(available, |v| v.min(available)));
                    }
                    if let Some(limit) = scope.limits.tokens {
                        let available = limit.saturating_sub(scope.settled_tokens.saturating_add(scope.reserved_tokens));
                        reservation.grant.max_total_tokens = Some(reservation.grant.max_total_tokens.map_or(available, |v| v.min(available)));
                    }
                }
                reservation.grant.validate()?;
                let identity = CallIdentity { owner: durable.owner.clone(), run_id: self.usage_id.clone(), call_id: reservation.id.clone(), attempt: 1 };
                durable.store.admit_accounting_call(&CallAdmission { identity: identity.clone(), model: "remote-delegation".into(),
                    scopes: durable.scopes.clone(), reserved_cost_usd: reservation.grant.max_total_cost_usd,
                    reserved_tokens: reservation.grant.max_total_tokens,
                    source_reservation: durable.source_reservation.clone() }).await?;
                Ok::<_,anyhow::Error>(DurableCall { store: durable.store.clone(), identity })
            }.await;
            match result {
                Ok(call) => {
                    let mut inner = self.tracker.inner.lock().map_err(|_| anyhow::anyhow!("Cost budget ledger is unavailable"))?;
                    inner.remote_reservations.get_mut(&reservation.id).ok_or(AccountingError::Missing)?.grant = reservation.grant.clone();
                    reservation.durable = Some(call);
                }
                Err(error) => { reservation.release_confirmed_local()?; return Err(error); }
            }
        }
        Ok(reservation)
    }
}
impl RemoteBudgetReservation {
    /// Bind before peer dispatch. The existing complete contract digest includes
    /// source, target and child identities and every granted restriction.
    pub(crate) async fn bind_contract(&self, digest: String) -> anyhow::Result<()> {
        if let Some(call) = &self.durable {
            call.store.bind_accounting_delegation(&crate::uar::persistence::accounting::AccountingDelegation {
                contract_digest: digest, source: call.identity.clone(),
            }).await?;
        }
        Ok(())
    }

    pub(crate) async fn record_cumulative(&self, receipt: UarUsageReceipt) -> anyhow::Result<()> {
        let result = async {
            receipt.validate()?;
            if let Some(call) = &self.durable { call.usage(Some(receipt.cost_usd),receipt.total_tokens,false).await?; }
            self.record_cumulative_local(receipt)
        }.await;
        if result.is_err() {
            // A later cleanup path cannot turn a stale/default receipt into a
            // confirmed final cost after any counter or persistence failure.
            self.unresolved.store(true, std::sync::atomic::Ordering::Release);
        }
        result
    }
    pub(crate) async fn release_confirmed(&self) -> anyhow::Result<()> {
        if self.is_released() { return Ok(()); }
        if let Some(call) = &self.durable {
            // Cleanup alone cannot establish the billed usage of an uncertain peer.
            if self.unresolved.load(std::sync::atomic::Ordering::Acquire) { return Err(AccountingError::Unresolved.into()); }
            let used = self.tracker.inner.lock().map_err(|_| anyhow::anyhow!("Cost budget ledger is unavailable"))?
                .remote_reservations.get(&self.id).ok_or(AccountingError::Missing)?.used.clone();
            call.usage(Some(used.cost_usd),used.total_tokens,true).await?;
        }
        self.release_confirmed_local()
    }
}
