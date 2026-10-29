//! Trusted host reconstruction of an unresolved call. Stored data grants no authority.

use super::{ApprovalBroker, ApprovalOutcome, RootApprovalChannel};
use crate::uar::persistence::{
    approval_decisions::{ApprovalContinuation, ApprovalDecision, ApprovalRecord, ApprovalState},
    checkpoint_resume::{CheckpointResumeEnvelope, CheckpointResumeError},
};
use crate::uar::runtime::tool_admission::{PreparedToolInvocation, ToolAdmissionRuntime};
use chrono::Utc;
use std::sync::Arc;

pub(crate) const PENDING_APPROVAL_KEY: &str = "_pending_approval";

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ApprovalCheckpoint {
    pub schema_version: u32,
    pub challenge_id: String,
    pub binding_digest: String,
    pub call_index: usize,
    pub call: crate::llm::ToolCall,
}

pub(super) struct ApprovalRecoveryHost {
    pub checkpoint: Arc<crate::uar::runtime::checkpoint_host::CheckpointHost>,
    pub shutdown: tokio_util::sync::CancellationToken,
    pub admission: Arc<ToolAdmissionRuntime>,
}

#[derive(Clone)]
pub(super) struct PendingAdmissionCleanup {
    pub invocation: Arc<PreparedToolInvocation>,
    pub admission_id: String,
}

fn binding_digest(invocation: &PreparedToolInvocation) -> String {
    crate::uar::runtime::checkpoint::raw_digest(&(
        &invocation.catalog_revision,
        &invocation.mounted_server_id,
        &invocation.native_tool_name,
        &invocation.provider_tool_name,
        &invocation.run_policy_revision,
        &invocation.governance_policy_revision,
        &invocation.tool_policy_revision,
        &invocation.resource_revision,
        &invocation.payload_revision,
    ))
}

/// Validate mapping against the protected envelope, never against caller JSON.
pub(crate) fn protected_call(
    record: &ApprovalRecord,
    envelope: &CheckpointResumeEnvelope,
) -> anyhow::Result<ApprovalCheckpoint> {
    envelope.validate()?;
    envelope.require_claimable_execution()?;
    let continuation = record
        .continuation
        .as_ref()
        .ok_or(CheckpointResumeError::Invalid)?;
    let state = envelope.checkpoint()?.try_restore_state()?;
    let marker: ApprovalCheckpoint = serde_json::from_value(
        state
            .data
            .get(PENDING_APPROVAL_KEY)
            .cloned()
            .ok_or(CheckpointResumeError::Invalid)?,
    )?;
    if marker.schema_version != 1
        || record.owner_key != envelope.owner_key
        || record.root_run_id != envelope.source_run_id
        || continuation.checkpoint_id != envelope.checkpoint_id
        || continuation.checkpoint_revision != envelope.checkpoint_revision
        || marker.challenge_id != record.challenge_id
        || marker.binding_digest != continuation.binding_digest
        || marker.call_index != continuation.call_index
        || marker.call.id != record.tool_call_id
        || marker.call.function.name != record.tool_name
    {
        return Err(CheckpointResumeError::Invalid.into());
    }
    let history =
        crate::uar::runtime::checkpoint::history_from_checkpoint(&envelope.checkpoint()?)?;
    let calls = history
        .iter()
        .flat_map(|message| message.tool_calls.iter().flatten())
        .filter(|call| call.id == marker.call.id)
        .collect::<Vec<_>>();
    if calls.len() != 1
        || serde_json::to_value(calls[0])? != serde_json::to_value(&marker.call)?
        || history
            .iter()
            .any(|message| message.tool_call_id.as_deref() == Some(marker.call.id.as_str()))
    {
        return Err(CheckpointResumeError::Invalid.into());
    }
    Ok(marker)
}

impl RootApprovalChannel {
    pub(crate) async fn invalidate_recovered(&self, record: &ApprovalRecord) -> anyhow::Result<()> {
        let next = record.finish(ApprovalState::Invalidated, None);
        if self.lane.ledger.transition(record, &next).await? {
            self.lane.publish_record(&next).await;
        }
        Ok(())
    }
    pub(crate) fn install_recovery_host(
        &self,
        checkpoint: Arc<crate::uar::runtime::checkpoint_host::CheckpointHost>,
        shutdown: tokio_util::sync::CancellationToken,
        admission: Arc<ToolAdmissionRuntime>,
    ) {
        let _ = self.lane.recovery.set(ApprovalRecoveryHost {
            checkpoint,
            shutdown,
            admission,
        });
    }

    pub(crate) async fn request_tool(
        &self,
        request: crate::uar::runtime::tool_admission::ToolApprovalRequest,
        decision_owner: crate::uar::persistence::tool_admission::AdmissionOwner,
        risk_reason: String,
        cancellation: &tokio_util::sync::CancellationToken,
    ) -> ApprovalOutcome {
        let id = uuid::Uuid::new_v4().to_string();
        let invocation = &request.invocation;
        let mut continuation = None;
        if let Some(host) = self.lane.recovery.get()
            && self
                .lane
                .ledger
                .store
                .as_ref()
                .is_some_and(|store| store.supports_durable_checkpoint_resume())
            && let Some(history) = &request.checkpoint_history
        {
            let Some(call) = history
                .iter()
                .flat_map(|message| message.tool_calls.iter().flatten())
                .find(|call| call.id == invocation.model_tool_call_id)
                .cloned()
            else {
                return ApprovalOutcome::ChannelClosed;
            };
            let marker = ApprovalCheckpoint {
                schema_version: 1,
                challenge_id: id.clone(),
                binding_digest: binding_digest(invocation),
                call_index: invocation.call_index,
                call,
            };
            let state = crate::uar::runtime::graph::GraphState {
                data: std::collections::HashMap::from([(
                    PENDING_APPROVAL_KEY.to_owned(),
                    serde_json::json!(marker),
                )]),
                messages: history
                    .iter()
                    .map(|message| serde_json::json!(message))
                    .collect(),
                iteration: 0,
            };
            if let Ok(mut cleanup) = self.lane.cleanup.lock() {
                cleanup.insert(
                    id.clone(),
                    PendingAdmissionCleanup {
                        invocation: Arc::clone(invocation),
                        admission_id: request.admission_id.clone(),
                    },
                );
            } else {
                return ApprovalOutcome::ChannelClosed;
            }
            let envelope = match host
                .checkpoint
                .persist_envelope("pending_tool_approval", &state)
                .await
            {
                Ok(envelope) => envelope,
                Err(_) => return ApprovalOutcome::ChannelClosed,
            };
            if envelope.require_claimable_execution().is_ok() {
                continuation = Some(ApprovalContinuation {
                    checkpoint_id: envelope.checkpoint_id,
                    checkpoint_revision: envelope.checkpoint_revision,
                    binding_digest: marker.binding_digest,
                    call_index: marker.call_index,
                });
            }
        }
        let outcome = self
            .request_with_continuation(
                Some(request.admission_id),
                invocation.admission_owner,
                decision_owner,
                invocation.call_index,
                invocation.model_tool_call_id.clone(),
                invocation.provider_tool_name.clone(),
                request.action_display.to_string(),
                risk_reason,
                cancellation,
                id.clone(),
                continuation,
            )
            .await;
        if !matches!(outcome, ApprovalOutcome::Cancelled) {
            if let Ok(mut cleanup) = self.lane.cleanup.lock() {
                cleanup.remove(&id);
            }
        }
        outcome
    }

    /// Called only after a fresh checkpoint claim and current policy checks.
    pub(crate) async fn approve_recovered(
        &self,
        record: &ApprovalRecord,
        invocation: &PreparedToolInvocation,
    ) -> anyhow::Result<bool> {
        if record.state != ApprovalState::Pending
            || record.expires_at <= Utc::now()
            || record.owner_key != self.lane.owner_key
            || record.tool_call_id != invocation.model_tool_call_id
            || record.tool_name != invocation.provider_tool_name
            || record.continuation.as_ref().is_none_or(|continuation| {
                continuation.binding_digest != binding_digest(invocation)
                    || continuation.call_index != invocation.call_index
            })
        {
            let state = if record.expires_at <= Utc::now() {
                ApprovalState::Expired
            } else {
                ApprovalState::Invalidated
            };
            let next = record.finish(state, None);
            if self.lane.ledger.transition(record, &next).await? {
                self.lane.publish_record(&next).await;
            }
            return Ok(false);
        }
        let next = record.finish(
            ApprovalState::Approved,
            Some(ApprovalDecision {
                decision_id: uuid::Uuid::new_v4().to_string(),
                actor: self.lane.owner_key.clone(),
                approved: true,
                decided_at: Utc::now(),
            }),
        );
        let consumed = self.lane.ledger.transition(record, &next).await?;
        if consumed {
            self.lane.publish_record(&next).await;
        } else if let Some(current) = self
            .lane
            .ledger
            .list(&record.owner_key, &record.root_run_id)
            .await?
            .into_iter()
            .find(|current| current.storage_key() == record.storage_key())
        {
            self.lane.publish_record(&current).await;
        }
        Ok(consumed)
    }
}

impl ApprovalBroker {
    pub(crate) async fn recovery_record(
        &self,
        owner: &str,
        run: &str,
        id: &str,
        workspace: Option<&str>,
    ) -> anyhow::Result<Option<ApprovalRecord>> {
        let records = self.ledger.list(owner, run).await?;
        Ok(records.into_iter().find(|record| {
            record.challenge_id == id
                && record
                    .workspace_id
                    .as_deref()
                    .is_none_or(|id| Some(id) == workspace)
        }))
    }

    pub(crate) async fn finish_recovered(
        &self,
        record: &ApprovalRecord,
        state: ApprovalState,
    ) -> anyhow::Result<bool> {
        self.ledger
            .transition(record, &record.finish(state, None))
            .await
    }

    pub(crate) async fn deny_recovered(&self, record: &ApprovalRecord) -> anyhow::Result<bool> {
        let next = record.finish(
            ApprovalState::Denied,
            Some(ApprovalDecision {
                decision_id: uuid::Uuid::new_v4().to_string(),
                actor: record.owner_key.clone(),
                approved: false,
                decided_at: Utc::now(),
            }),
        );
        self.ledger.transition(record, &next).await
    }
}

#[cfg(test)]
mod identity_tests {
    use super::*;
    use crate::uar::domain::runs::{Run, RunStatus};
    use crate::uar::runtime::{
        checkpoint::{Checkpoint, raw_digest},
        graph::GraphState,
    };
    use serde_json::{Value, json};

    const CALL_ID: &str = " exact-native-call ";

    fn call() -> crate::llm::ToolCall {
        crate::llm::ToolCall {
            id: CALL_ID.into(),
            call_type: "function".into(),
            function: crate::llm::ToolCallFunction {
                name: "native_fixture".into(),
                arguments: "{\"value\":1}".into(),
            },
        }
    }

    fn history() -> Vec<Value> {
        vec![json!({"role":"assistant","content":"","tool_calls":[call()]})]
    }

    fn protected_fixture(messages: Vec<Value>) -> (ApprovalRecord, CheckpointResumeEnvelope) {
        let artifact = crate::uar::defaults::default_agent();
        let authorization = raw_digest(&"identity-policy");
        let marker = ApprovalCheckpoint {
            schema_version: 1,
            challenge_id: "challenge".into(),
            binding_digest: raw_digest(&"identity-binding"),
            call_index: 0,
            call: call(),
        };
        let state = GraphState {
            data: std::collections::HashMap::from([(PENDING_APPROVAL_KEY.into(), json!(marker))]),
            messages,
            iteration: 0,
        };
        let source = Run {
            run_id: "source-run".into(),
            agent_id: artifact.id.clone(),
            conversation_id: Some("session".into()),
            user_id: Some("owner".into()),
            status: RunStatus::Paused,
            context: json!({
                "agent_snapshot":artifact.snapshot("identity-fixture"),
                "host_resources":{}, "checkpoint_execution":{"schema_version":1,"kind":"tool_loop"},
                "checkpoint_authorization_digest":authorization,
            }),
        };
        let checkpoint = Checkpoint::new(
            "source-run",
            "session",
            "pending_tool_approval",
            &state,
            &authorization,
        );
        let envelope = CheckpointResumeEnvelope::new(
            "owner-key".into(),
            "source-epoch".into(),
            &source,
            &checkpoint,
        )
        .unwrap();
        let now = Utc::now();
        let record = ApprovalRecord {
            version: 1,
            issuer_id: "issuer".into(),
            challenge_id: marker.challenge_id,
            owner_key: "owner-key".into(),
            workspace_id: None,
            root_run_id: "source-run".into(),
            admission_id: Some("admission".into()),
            admission_owner: crate::uar::persistence::tool_admission::AdmissionOwner::UarRuntime,
            tool_call_id: CALL_ID.into(),
            tool_name: "native_fixture".into(),
            created_at: now,
            expires_at: now + chrono::Duration::minutes(5),
            state: ApprovalState::Pending,
            decision: None,
            updated_at: now,
            continuation: Some(ApprovalContinuation {
                checkpoint_id: envelope.checkpoint_id.clone(),
                checkpoint_revision: envelope.checkpoint_revision.clone(),
                binding_digest: marker.binding_digest,
                call_index: 0,
            }),
        };
        (record, envelope)
    }

    #[test]
    fn protected_native_identity_rejects_owner_run_and_history_substitution() {
        let (record, envelope) = protected_fixture(history());
        let restored = protected_call(&record, &envelope).unwrap();
        assert_eq!(restored.call.id, CALL_ID);
        assert_eq!(serde_json::to_value(restored.call).unwrap(), json!(call()));
        for mutation in 0..6 {
            let mut changed = record.clone();
            match mutation {
                0 => changed.owner_key = "other-owner".into(),
                1 => changed.root_run_id = "other-run".into(),
                2 => changed.tool_call_id = "other-call".into(),
                3 => changed.challenge_id = "other-challenge".into(),
                4 => {
                    changed.continuation.as_mut().unwrap().binding_digest =
                        raw_digest(&"other-policy")
                }
                _ => changed.continuation.as_mut().unwrap().call_index = 1,
            }
            assert!(protected_call(&changed, &envelope).is_err());
        }
        let mut substituted = history();
        substituted[0]["tool_calls"][0]["function"]["arguments"] = json!("{\"value\":2}");
        let mut duplicated = history();
        duplicated[0]["tool_calls"]
            .as_array_mut()
            .unwrap()
            .push(json!(call()));
        let mut answered = history();
        answered.push(json!({"role":"tool","content":"already completed","tool_call_id":CALL_ID}));
        for messages in [substituted, duplicated, answered] {
            // A self-consistent protected envelope still cannot disagree with
            // the exact pending call or grant replay of a completed result.
            let (record, envelope) = protected_fixture(messages);
            envelope.validate().unwrap();
            assert!(protected_call(&record, &envelope).is_err());
        }
    }
}
