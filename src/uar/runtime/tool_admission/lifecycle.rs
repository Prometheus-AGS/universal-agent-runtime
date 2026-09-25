use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::{Mutex, OnceCell};
use uuid::Uuid;

use crate::uar::persistence::PersistenceLayer;
use crate::uar::persistence::providers::memory::InMemoryProvider;
use crate::uar::persistence::tool_admission::{
    ToolAdmissionEvidence, ToolAdmissionEvidenceState,
};

use super::{
    AdmittedToolInvocation, HostAdmissionPreparation, HostToolAdmissionPort,
    PreparedToolInvocation, ToolAdmissionContext, ToolAdmissionRuntime,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdmissionCancellationReason {
    Cancelled,
    Invalidated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdmissionCancellationOutcome {
    Cancelled,
    AlreadyClaimed,
    AlreadyTerminal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdmissionTerminalOutcome {
    Succeeded,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LiveState {
    AwaitingApproval,
    Claimed,
    ClaimedUnknown,
    Terminal,
    Cancelled,
}

#[derive(Debug)]
pub(super) struct AdmissionLifecycle {
    persistence: Arc<dyn PersistenceLayer>,
    reconciled: OnceCell<()>,
    invocations: Mutex<HashMap<String, Arc<Mutex<LiveState>>>>,
}

impl AdmissionLifecycle {
    pub(super) fn new(persistence: Option<Arc<dyn PersistenceLayer>>) -> Self {
        Self {
            persistence: persistence
                .unwrap_or_else(|| Arc::new(InMemoryProvider::new()) as Arc<dyn PersistenceLayer>),
            reconciled: OnceCell::new(),
            invocations: Mutex::new(HashMap::new()),
        }
    }

    pub(super) fn ephemeral() -> Self {
        Self::new(None)
    }

    async fn cell(&self, invocation_id: &str, initial: LiveState) -> Arc<Mutex<LiveState>> {
        let mut invocations = self.invocations.lock().await;
        Arc::clone(
            invocations
                .entry(invocation_id.to_owned())
                .or_insert_with(|| Arc::new(Mutex::new(initial))),
        )
    }

    pub(super) async fn reconcile(&self, context: &ToolAdmissionContext) -> anyhow::Result<()> {
        self.reconciled
            .get_or_try_init(|| async {
                let history = self
                    .persistence
                    .list_tool_admission_evidence(&context.owner_id)
                    .await?;
                let mut latest = HashMap::<String, ToolAdmissionEvidence>::new();
                for evidence in history {
                    latest.insert(evidence.invocation_id.clone(), evidence);
                }
                for evidence in latest.into_values().filter(|evidence| {
                    !evidence.state.is_terminal()
                        && (evidence.runtime_epoch != context.runtime_epoch
                            || (evidence.root_run_id == context.root_run_id
                                && evidence.host_epoch != context.host.host_epoch))
                }) {
                    let state = match evidence.state {
                        ToolAdmissionEvidenceState::AwaitingApproval => {
                            ToolAdmissionEvidenceState::Interrupted
                        }
                        ToolAdmissionEvidenceState::ClaimIntent => {
                            ToolAdmissionEvidenceState::OutcomeUnknown
                        }
                        _ => continue,
                    };
                    self.persistence
                        .save_tool_admission_evidence(&evidence.next(state))
                        .await?;
                }
                Ok::<(), anyhow::Error>(())
            })
            .await?;
        Ok(())
    }

    pub(super) async fn record_prepared(
        &self,
        invocation: &PreparedToolInvocation,
        preparation: &HostAdmissionPreparation,
    ) -> anyhow::Result<()> {
        let cell = self
            .cell(&invocation.invocation_id, LiveState::AwaitingApproval)
            .await;
        let state = cell.lock().await;
        anyhow::ensure!(
            *state == LiveState::AwaitingApproval,
            "Tool invocation is no longer pending"
        );
        let evidence_state = if preparation.host_disposition
            == super::HostAdmissionDisposition::Deny
        {
            ToolAdmissionEvidenceState::Denied
        } else {
            ToolAdmissionEvidenceState::AwaitingApproval
        };
        self.persistence
            .save_tool_admission_evidence(&ToolAdmissionEvidence::new(
                invocation,
                &preparation.admission_id,
                evidence_state,
            ))
            .await?;
        if evidence_state.is_terminal() {
            drop(state);
            *cell.lock().await = LiveState::Terminal;
        }
        Ok(())
    }

    pub(super) async fn reject(
        &self,
        invocation: &PreparedToolInvocation,
        admission_id: &str,
    ) -> anyhow::Result<()> {
        let cell = self
            .cell(&invocation.invocation_id, LiveState::AwaitingApproval)
            .await;
        let mut state = cell.lock().await;
        if *state == LiveState::Terminal {
            return Ok(());
        }
        anyhow::ensure!(
            *state == LiveState::AwaitingApproval,
            "Tool invocation cannot be denied from its current state"
        );
        self.persistence
            .save_tool_admission_evidence(&ToolAdmissionEvidence::new(
                invocation,
                admission_id,
                ToolAdmissionEvidenceState::Denied,
            ))
            .await?;
        *state = LiveState::Terminal;
        Ok(())
    }

    pub(super) async fn claim(&self, admitted: &AdmittedToolInvocation) -> anyhow::Result<()> {
        let invocation = admitted.prepared.as_ref();
        let cell = self
            .cell(&invocation.invocation_id, LiveState::AwaitingApproval)
            .await;
        let mut state = cell.lock().await;
        anyhow::ensure!(
            *state == LiveState::AwaitingApproval,
            "Tool invocation cannot be claimed from its current state"
        );
        self.persistence
            .save_tool_admission_evidence(&ToolAdmissionEvidence::new(
                invocation,
                &admitted.host_receipt.admission_id,
                ToolAdmissionEvidenceState::ClaimIntent,
            ))
            .await?;
        *state = LiveState::Claimed;
        Ok(())
    }

    pub(super) async fn finish(
        &self,
        admitted: &AdmittedToolInvocation,
        succeeded: bool,
    ) -> anyhow::Result<()> {
        let invocation = admitted.prepared.as_ref();
        let cell = self.cell(&invocation.invocation_id, LiveState::Claimed).await;
        let mut state = cell.lock().await;
        anyhow::ensure!(
            matches!(*state, LiveState::Claimed | LiveState::ClaimedUnknown),
            "Tool invocation was not claimed"
        );
        self.persistence
            .save_tool_admission_evidence(&ToolAdmissionEvidence::new(
                invocation,
                &admitted.host_receipt.admission_id,
                if succeeded {
                    ToolAdmissionEvidenceState::Succeeded
                } else {
                    ToolAdmissionEvidenceState::Failed
                },
            ))
            .await?;
        *state = LiveState::Terminal;
        Ok(())
    }

    pub(super) async fn cancel(
        &self,
        host: &dyn HostToolAdmissionPort,
        invocation: &PreparedToolInvocation,
        admission_id: &str,
        reason: AdmissionCancellationReason,
    ) -> anyhow::Result<AdmissionCancellationOutcome> {
        let cell = self
            .cell(&invocation.invocation_id, LiveState::AwaitingApproval)
            .await;
        let mut state = cell.lock().await;
        match *state {
            LiveState::Claimed => {
                let outcome = host.cancel(invocation, admission_id, reason).await?;
                let evidence_state = match outcome {
                    AdmissionCancellationOutcome::Cancelled => match reason {
                        AdmissionCancellationReason::Cancelled => {
                            ToolAdmissionEvidenceState::Cancelled
                        }
                        AdmissionCancellationReason::Invalidated => {
                            ToolAdmissionEvidenceState::Invalidated
                        }
                    },
                    AdmissionCancellationOutcome::AlreadyClaimed => {
                        ToolAdmissionEvidenceState::OutcomeUnknown
                    }
                    AdmissionCancellationOutcome::AlreadyTerminal => {
                        ToolAdmissionEvidenceState::OutcomeUnknown
                    }
                };
                self.persistence
                    .save_tool_admission_evidence(&ToolAdmissionEvidence::new(
                        invocation,
                        admission_id,
                        evidence_state,
                    ))
                    .await?;
                *state = if matches!(
                    outcome,
                    AdmissionCancellationOutcome::AlreadyClaimed
                        | AdmissionCancellationOutcome::AlreadyTerminal
                ) {
                    LiveState::ClaimedUnknown
                } else {
                    LiveState::Cancelled
                };
                return Ok(outcome);
            }
            LiveState::ClaimedUnknown => {
                return Ok(AdmissionCancellationOutcome::AlreadyClaimed);
            }
            LiveState::Terminal | LiveState::Cancelled => {
                return Ok(AdmissionCancellationOutcome::AlreadyTerminal);
            }
            LiveState::AwaitingApproval => {}
        }
        let outcome = host.cancel(invocation, admission_id, reason).await?;
        let evidence_state = match outcome {
            AdmissionCancellationOutcome::Cancelled => match reason {
                AdmissionCancellationReason::Cancelled => {
                    ToolAdmissionEvidenceState::Cancelled
                }
                AdmissionCancellationReason::Invalidated => {
                    ToolAdmissionEvidenceState::Invalidated
                }
            },
            AdmissionCancellationOutcome::AlreadyClaimed
            | AdmissionCancellationOutcome::AlreadyTerminal => {
                ToolAdmissionEvidenceState::OutcomeUnknown
            }
        };
        self.persistence
            .save_tool_admission_evidence(&ToolAdmissionEvidence::new(
                invocation,
                admission_id,
                evidence_state,
            ))
            .await?;
        *state = if outcome == AdmissionCancellationOutcome::Cancelled {
            LiveState::Cancelled
        } else {
            LiveState::ClaimedUnknown
        };
        Ok(outcome)
    }

    pub(super) fn watch_cancellation(
        self: &Arc<Self>,
        host: Arc<dyn HostToolAdmissionPort>,
        invocation: Arc<PreparedToolInvocation>,
        admission_id: String,
        cancellation: tokio_util::sync::CancellationToken,
    ) {
        let lifecycle = Arc::clone(self);
        tokio::spawn(async move {
            cancellation.cancelled().await;
            if lifecycle
                .cancel(
                    host.as_ref(),
                    invocation.as_ref(),
                    &admission_id,
                    AdmissionCancellationReason::Cancelled,
                )
                .await
                .is_err()
            {
                tracing::error!(
                    invocation_id = %invocation.invocation_id,
                    code = "tool_admission_cancellation_record_failed",
                    "Tool admission cancellation could not be recorded"
                );
            }
        });
    }
}

impl ToolAdmissionEvidence {
    fn new(
        invocation: &PreparedToolInvocation,
        admission_id: &str,
        state: ToolAdmissionEvidenceState,
    ) -> Self {
        Self {
            schema_version: Self::SCHEMA_VERSION,
            evidence_id: Uuid::new_v4().to_string(),
            owner_id: invocation.owner_id.clone(),
            root_run_id: invocation.root_run_id.clone(),
            run_id: invocation.executing_run_id.clone(),
            invocation_id: invocation.invocation_id.clone(),
            admission_id: admission_id.to_owned(),
            tool_name: invocation.provider_tool_name.clone(),
            runtime_epoch: invocation.runtime_epoch.clone(),
            host_epoch: invocation.host_epoch.clone(),
            state,
            occurred_at: chrono::Utc::now(),
        }
    }

    fn next(&self, state: ToolAdmissionEvidenceState) -> Self {
        Self {
            evidence_id: Uuid::new_v4().to_string(),
            state,
            occurred_at: chrono::Utc::now(),
            ..self.clone()
        }
    }
}

impl ToolAdmissionRuntime {
    /// Persist claim intent before any native, MCP, or delegated side effect.
    pub async fn claim(&self, admitted: &AdmittedToolInvocation) -> anyhow::Result<()> {
        self.lifecycle.claim(admitted).await
    }

    /// Persist the terminal receipt after execution. A missing receipt leaves a
    /// durable claim that reconciles to outcome-unknown after restart.
    pub async fn finish(
        &self,
        admitted: &AdmittedToolInvocation,
        succeeded: bool,
    ) -> anyhow::Result<()> {
        self.lifecycle.finish(admitted, succeeded).await?;
        self.host
            .finish(
                admitted.prepared.as_ref(),
                &admitted.host_receipt.admission_id,
                if succeeded {
                    AdmissionTerminalOutcome::Succeeded
                } else {
                    AdmissionTerminalOutcome::Failed
                },
            )
            .await
    }

    /// Cancel an exact unclaimed invocation in both UAR and the paired host.
    pub async fn cancel(
        &self,
        invocation: &PreparedToolInvocation,
        admission_id: &str,
        reason: AdmissionCancellationReason,
    ) -> anyhow::Result<AdmissionCancellationOutcome> {
        self.lifecycle
            .cancel(self.host.as_ref(), invocation, admission_id, reason)
            .await
    }

    pub(super) async fn reject(
        &self,
        invocation: &PreparedToolInvocation,
        admission_id: &str,
    ) -> anyhow::Result<()> {
        self.lifecycle.reject(invocation, admission_id).await
    }
}

impl Drop for ToolAdmissionRuntime {
    fn drop(&mut self) {
        self.cancellation.cancel();
    }
}
