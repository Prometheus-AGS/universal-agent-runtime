//! Restart-durable caller-tool handoffs. Stored results confer no execution authority.
use std::collections::{BTreeMap, BTreeSet};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClientToolContinuationState { Pending, Claimed, Completed }

/// Actual delegated producer, bound by the host rather than caller history.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClientToolExecutionSource {
    pub agent_id: String,
    pub run_id: String,
    pub thread_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClientToolContinuation {
    pub owner_key: String,
    pub tenant_id: Option<String>,
    pub agent_id: String,
    pub batch_id: String,
    pub protocol: String,
    pub source_run_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub execution_source: Option<ClientToolExecutionSource>,
    pub session_id: Option<String>,
    /// Protected graph execution state; never reconstructed from caller history.
    pub graph_state: Option<Value>,
    pub budget_snapshot: Option<crate::uar::runtime::cost_budget::ClientToolBudgetSnapshot>,
    /// Restriction retained across native protocol continuation turns.
    pub output_token_cap: Option<u32>,
    /// Complete host transcript, including results hidden from the caller.
    pub canonical_transcript: Value,
    /// Exact caller-visible predecessor binding; never reconstruct hidden results from input.
    pub visible_transcript: Value,
    pub client_schemas: Value,
    pub pending_call_ids: Vec<String>,
    pub state: ClientToolContinuationState,
    /// Monotonic physical cleanup acknowledgment, committed only after the source join.
    #[serde(default)]
    pub source_settled: bool,
    pub result_batch: Option<BTreeMap<String, Value>>,
    pub completed_response: Option<Value>,
    pub completed_events: Vec<Value>,
}

impl ClientToolContinuation {
    pub fn key(owner: &str, tenant: Option<&str>, agent: &str, batch: &str) -> String {
        let identity = serde_json::json!([owner, tenant, agent, batch]).to_string();
        Sha256::digest(identity.as_bytes()).iter().map(|byte| format!("{byte:02x}")).collect()
    }
    pub fn storage_key(&self) -> String {
        Self::key(&self.owner_key, self.tenant_id.as_deref(), &self.agent_id, &self.batch_id)
    }
    pub fn validate(&self) -> anyhow::Result<()> {
        for id in [&self.owner_key, &self.agent_id, &self.batch_id, &self.protocol, &self.source_run_id]
            .into_iter().chain(self.tenant_id.iter()).chain(self.session_id.iter()).chain(self.pending_call_ids.iter()) {
            anyhow::ensure!(!id.is_empty() && id.trim() == id && !id.chars().any(char::is_control), "invalid_client_tool_identity");
        }
        anyhow::ensure!(self.output_token_cap.is_none_or(|cap| cap > 0), "invalid_client_tool_output_token_cap");
        if let Some(source) = &self.execution_source {
            for identity in [&source.agent_id, &source.run_id, &source.thread_id] {
                anyhow::ensure!(!identity.is_empty() && identity.trim() == identity
                    && !identity.chars().any(char::is_control), "invalid_client_tool_execution_source");
            }
            anyhow::ensure!(source.run_id != self.source_run_id && self.graph_state.is_some(),
                "invalid_delegated_client_tool_binding");
        }
        anyhow::ensure!(self.budget_snapshot.is_some(), "client_tool_budget_snapshot_required");
        anyhow::ensure!(self.canonical_transcript.is_array() && self.visible_transcript.is_array() && self.client_schemas.is_object(), "invalid_client_tool_transcript_binding");
        let calls: BTreeSet<_> = self.pending_call_ids.iter().collect();
        anyhow::ensure!(!calls.is_empty() && calls.len() == self.pending_call_ids.len(), "invalid_client_tool_batch");
        if let Some(results) = &self.result_batch {
            anyhow::ensure!(results.keys().collect::<BTreeSet<_>>() == calls, "client_tool_result_batch_mismatch");
        }
        anyhow::ensure!(self.state == ClientToolContinuationState::Pending || self.source_settled, "client_tool_source_not_settled");
        match self.state {
            ClientToolContinuationState::Pending => anyhow::ensure!(self.result_batch.is_none() && self.completed_response.is_none() && self.completed_events.is_empty(), "invalid_client_tool_pending"),
            ClientToolContinuationState::Claimed => anyhow::ensure!(self.result_batch.is_some() && self.completed_response.is_none() && self.completed_events.is_empty(), "invalid_client_tool_claim"),
            ClientToolContinuationState::Completed => anyhow::ensure!(self.result_batch.is_some() && self.completed_response.is_some(), "invalid_client_tool_completion"),
        }
        Ok(())
    }
    pub fn validate_create(&self) -> anyhow::Result<()> {
        self.validate()?;
        anyhow::ensure!(self.state == ClientToolContinuationState::Pending && !self.source_settled, "invalid_client_tool_initial_state");
        Ok(())
    }
    pub fn validate_transition(&self, next: &Self) -> anyhow::Result<()> {
        self.validate()?;
        next.validate()?;
        if self.state == ClientToolContinuationState::Pending && !self.source_settled
            && next.state == ClientToolContinuationState::Pending && next.source_settled {
            let mut acknowledged = self.clone();
            acknowledged.source_settled = true;
            anyhow::ensure!(&acknowledged == next, "client_tool_settlement_binding_conflict");
            return Ok(());
        }
        let mut identity = next.clone();
        identity.state = self.state.clone();
        identity.result_batch = self.result_batch.clone();
        identity.completed_response = self.completed_response.clone();
        identity.completed_events = self.completed_events.clone();
        anyhow::ensure!(&identity == self, "client_tool_continuation_binding_conflict");
        anyhow::ensure!(matches!((&self.state, &next.state),
            (ClientToolContinuationState::Pending, ClientToolContinuationState::Claimed) |
            (ClientToolContinuationState::Claimed, ClientToolContinuationState::Completed)), "client_tool_continuation_state_conflict");
        if self.state == ClientToolContinuationState::Claimed {
            anyhow::ensure!(self.result_batch == next.result_batch, "client_tool_result_batch_conflict");
        }
        Ok(())
    }
}
