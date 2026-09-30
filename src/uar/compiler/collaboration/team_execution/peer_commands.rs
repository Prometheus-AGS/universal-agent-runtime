//! Atomic model peer commands. Sender identity is never decoded from arguments.
use super::{
    CollaborationCatalogService, CollaborationError, MAX_CAS_ATTEMPTS, key, peer::*, team, team_key,
};
use crate::uar::domain::{
    team_execution::*,
    team_planning::{AssignmentAuthority, TeamTask},
    team_wait::*,
};
use chrono::Utc;
use serde_json::{Value, json};
use uuid::Uuid;

impl CollaborationCatalogService {
    pub async fn team_roster(
        &self,
        expected: &TeamExecutionAttempt,
        cursor: Option<String>,
        limit: u64,
    ) -> Result<Value, CollaborationError> {
        let state = self.load_state().await?;
        self.require_execution_owner(&state, false)?;
        current_actor(&state, expected)?;
        if !(1..=50).contains(&limit) {
            return Err(denied("TEAM_SCOPE_DENIED"));
        }
        let (roster, _, revision) = authorized_roster(&state, expected)?;
        let offset = if let Some(cursor) = cursor {
            let pieces: Vec<_> = cursor.split(':').collect();
            if pieces.len() != 3 || pieces[0] != expected.id || pieces[1] != revision.to_string() {
                return Err(denied("TEAM_REVISION_CONFLICT"));
            }
            pieces[2]
                .parse::<usize>()
                .map_err(|_| denied("TEAM_SCOPE_DENIED"))?
        } else {
            0
        };
        if offset > roster.len() {
            return Err(denied("TEAM_SCOPE_DENIED"));
        }
        let end = (offset + limit as usize).min(roster.len());
        let mut result = json!({"members":roster[offset..end],"authorizationRevision":revision,"teamRevision":team(&state,&expected.owner_id,&expected.workspace_id,&expected.team_id)?.revision});
        if end < roster.len() {
            result["nextCursor"] = json!(format!("{}:{revision}:{end}", expected.id));
        }
        Ok(result)
    }
    pub async fn team_send(
        &self,
        expected: &TeamExecutionAttempt,
        request: TeamSendRequest,
    ) -> Result<Value, CollaborationError> {
        super::super::validation::validate_id(&request.command_id)?;
        let digest = super::super::validation::canonical_digest(&serde_json::to_value(&request)?)?;
        for _ in 0..MAX_CAS_ATTEMPTS {
            let current = self.load_state().await?;
            self.require_execution_owner(&current, false)?;
            current_actor(&current, expected)?;
            require_edge(
                &current,
                expected,
                &request.recipient.member_id,
                "queue-only",
            )?;
            validate_payload(&current, expected, &request.payload)?;
            if let Some(id) = &request.recipient.task_id {
                let t = team(
                    &current,
                    &expected.owner_id,
                    &expected.workspace_id,
                    &expected.team_id,
                )?
                .tasks
                .iter()
                .find(|t| &t.id == id)
                .ok_or_else(|| denied("TEAM_SCOPE_DENIED"))?;
                if t.assignee_member_id.as_deref() != Some(request.recipient.member_id.as_str()) {
                    return Err(denied("TEAM_SCOPE_DENIED"));
                }
            }
            if let Some(receipt) = replay(
                &current,
                expected,
                &request.command_id,
                &digest,
                "team_send",
            )? {
                return peer_result(&current, receipt);
            }
            let mut next = current.clone();
            let mut receipt = command_receipt(expected, &request.command_id, &digest, "team_send");
            let message = peer_message(
                expected,
                &request.recipient.member_id,
                request.recipient.task_id.clone(),
                "queue-only",
                request.payload.clone(),
            );
            receipt.message_id = Some(message.message_id.clone());
            insert_message(&mut next, message);
            next.team_peer_commands
                .insert(command_key(expected, &request.command_id), receipt.clone());
            next.generation += 1;
            if self.cas(current.generation, &next).await? {
                if receipt.operation == "team_delegate" {
                    self.team_execution_notify.notify_one();
                }
                return peer_result(&next, &receipt);
            }
        }
        Err(denied("TEAM_REVISION_CONFLICT"))
    }
    pub async fn team_delegate(
        &self,
        expected: &TeamExecutionAttempt,
        request: TeamDelegateRequest,
    ) -> Result<Value, CollaborationError> {
        super::super::validation::validate_id(&request.command_id)?;
        super::super::validation::validate_id(&request.task.task_id)?;
        if serde_json::to_vec(&request.task.input)?.len() > 32768
            || serde_json::to_vec(&request.task.output_contract)?.len() > 32768
            || request.task.depends_on.len() > 16
        {
            return Err(denied("TEAM_CONTEXT_REQUIRED_TOO_LARGE"));
        }
        jsonschema::validator_for(&request.task.output_contract)
            .map_err(|_| denied("TEAM_CONTEXT_REQUIRED_UNSUPPORTED"))?;
        let digest = super::super::validation::canonical_digest(&serde_json::to_value(&request)?)?;
        for _ in 0..MAX_CAS_ATTEMPTS {
            let current = self.load_state().await?;
            self.require_execution_owner(&current, false)?;
            current_actor(&current, expected)?;
            require_edge(
                &current,
                expected,
                &request.recipient_member_id,
                "trigger-turn",
            )?;
            validate_payload(&current, expected, &request.payload)?;
            if let Some(receipt) = replay(
                &current,
                expected,
                &request.command_id,
                &digest,
                "team_delegate",
            )? {
                return peer_result(&current, receipt);
            }
            let mut selected = team(
                &current,
                &expected.owner_id,
                &expected.workspace_id,
                &expected.team_id,
            )?
            .clone();
            let doc = team_document(&current, expected)?;
            let sender = selected
                .members
                .iter()
                .find(|m| m.id == expected.member_id)
                .ok_or_else(|| denied("TEAM_SCOPE_DENIED"))?;
            if doc["coordinatorRole"].as_str() != Some(sender.role.as_str())
                || doc["taskAcceptance"]["mode"].as_str() != Some("coordinator-within-binding")
            {
                return Err(denied("TEAM_SCOPE_DENIED"));
            }
            if selected.revision != request.expected_team_revision {
                return Err(denied("TEAM_REVISION_CONFLICT"));
            }
            if selected.tasks.iter().any(|t| t.id == request.task.task_id) {
                return Err(denied("TEAM_COMMAND_CONFLICT"));
            }
            let member = selected
                .members
                .iter()
                .find(|m| m.id == request.recipient_member_id && m.role == request.task.role)
                .cloned()
                .ok_or_else(|| denied("TEAM_SCOPE_DENIED"))?;
            let binding = current
                .bindings
                .get(&super::super::bindings::binding_key(
                    &expected.owner_id,
                    &expected.workspace_id,
                    &selected.binding.id,
                ))
                .ok_or_else(|| denied("TEAM_SCOPE_DENIED"))?;
            let pending = selected
                .tasks
                .iter()
                .filter(|t| !matches!(t.status.as_str(), "succeeded" | "failed" | "cancelled"))
                .count() as u64;
            let cap = doc["limits"]["maxPendingTasks"].as_u64().unwrap_or(0).min(
                binding.document["effectiveLimits"]["maxPendingTasks"]
                    .as_u64()
                    .unwrap_or(u64::MAX),
            );
            if pending >= cap {
                return Err(denied("TEAM_PENDING_LIMIT"));
            }
            let mut seen = std::collections::BTreeSet::new();
            for dependency in &request.task.depends_on {
                if dependency == &request.task.task_id
                    || !seen.insert(dependency)
                    || !selected.tasks.iter().any(|t| &t.id == dependency)
                {
                    return Err(denied("TEAM_WAIT_CYCLE"));
                }
            }
            reserve(&current, expected, &request.reservation)?;
            let now = Utc::now();
            let ready = request.task.depends_on.iter().all(|id| {
                selected
                    .tasks
                    .iter()
                    .any(|t| &t.id == id && t.status == "succeeded")
            });
            selected.tasks.push(TeamTask {
                id: request.task.task_id.clone(),
                title: request.task.role.clone(),
                role: request.task.role.clone(),
                input: request.task.input.clone(),
                output_contract: request.task.output_contract.clone(),
                output: None,
                depends_on: request.task.depends_on.clone(),
                status: if ready { "ready" } else { "queued" }.into(),
                revision: 1,
                assignee_member_id: Some(member.id.clone()),
                ownership_epoch: 1,
                assignment_authority: Some(AssignmentAuthority {
                    binding_id: selected.binding.id.clone(),
                    binding_revision: selected.binding.revision,
                    workspace_id: selected.workspace_id.clone(),
                    role: member.role.clone(),
                    member_id: member.id.clone(),
                    ownership_epoch: 1,
                    can_execute: false,
                    can_use_tools: false,
                }),
                reviewer_member_id: None,
                reviewer_epoch: 0,
                state_reason: None,
                created_at: now,
                updated_at: now,
            });
            selected.revision += 1;
            selected.updated_at = now;
            let root_id = Uuid::new_v4().to_string();
            let attempt = TeamExecutionAttempt {
                id: Uuid::new_v4().to_string(),
                run_id: Uuid::new_v4().to_string(),
                root_id: root_id.clone(),
                approval_scope_id: root_id,
                queue_sequence: current.generation + 1,
                continuation_of_wait_id: None,
                owner_id: expected.owner_id.clone(),
                workspace_id: expected.workspace_id.clone(),
                team_id: expected.team_id.clone(),
                task_id: request.task.task_id.clone(),
                member_id: member.id.clone(),
                member_revision: member.revision,
                ownership_epoch: 1,
                binding_revision: selected.binding.revision,
                execution_epoch: 1,
                execution_fence: Some(self.require_execution_owner(&current, false)?),
                effective_models: self
                    .capture_team_models(&current, &selected, &member)
                    .await?,
                effect_disposition: "confirmed".into(),
                accounting_state: "reserved-unknown".into(),
                status: "queued".into(),
                execution_outcome: None,
                reservation: request.reservation.clone(),
                context_artifact_ids: request.payload.artifact_ids.clone(),
                usage: None,
                usage_revision: 0,
                output: None,
                state_reason: None,
                diagnostic: None,
                created_at: now,
                updated_at: now,
            };
            let mut next = current.clone();
            next.team_instances.insert(
                team_key(
                    &expected.owner_id,
                    &expected.workspace_id,
                    &expected.team_id,
                ),
                selected,
            );
            next.team_execution_attempts.insert(
                key(
                    &attempt.owner_id,
                    &attempt.workspace_id,
                    &attempt.team_id,
                    &attempt.id,
                ),
                attempt.clone(),
            );
            let mut receipt =
                command_receipt(expected, &request.command_id, &digest, "team_delegate");
            let message = peer_message(
                expected,
                &member.id,
                Some(request.task.task_id.clone()),
                "trigger-turn",
                request.payload.clone(),
            );
            receipt.message_id = Some(message.message_id.clone());
            receipt.task_id = Some(request.task.task_id.clone());
            receipt.attempt_id = Some(attempt.id.clone());
            insert_message(&mut next, message);
            next.team_peer_commands
                .insert(command_key(expected, &request.command_id), receipt.clone());
            next.generation += 1;
            if self.cas(current.generation, &next).await? {
                return peer_result(&next, &receipt);
            }
        }
        Err(denied("TEAM_REVISION_CONFLICT"))
    }
}
fn peer_message(
    a: &TeamExecutionAttempt,
    recipient: &str,
    task: Option<String>,
    mode: &str,
    payload: AttributedPayload,
) -> TeamPeerMessage {
    TeamPeerMessage {
        message_id: Uuid::new_v4().to_string(),
        owner_id: a.owner_id.clone(),
        workspace_id: a.workspace_id.clone(),
        team_id: a.team_id.clone(),
        sender_member_id: a.member_id.clone(),
        sender_attempt_id: a.id.clone(),
        recipient_member_id: recipient.into(),
        recipient_task_id: task,
        mode: mode.into(),
        payload,
        accepted_at: Utc::now(),
    }
}
fn insert_message(
    state: &mut crate::uar::domain::collaboration::CollaborationCatalogState,
    message: TeamPeerMessage,
) {
    state.team_message_deliveries.insert(
        message.message_id.clone(),
        MessageDelivery {
            message_id: message.message_id.clone(),
            recipient_member_id: message.recipient_member_id.clone(),
            recipient_task_id: message.recipient_task_id.clone(),
            status: "accepted".into(),
            selected_attempt_id: None,
            selected_at: None,
            consumed_at: None,
            rejection_code: None,
        },
    );
    state
        .team_peer_messages
        .insert(message.message_id.clone(), message);
}
fn peer_result(
    state: &crate::uar::domain::collaboration::CollaborationCatalogState,
    receipt: &TeamPeerCommandReceipt,
) -> Result<Value, CollaborationError> {
    let id = receipt
        .message_id
        .as_ref()
        .ok_or_else(|| denied("TEAM_COMMAND_CONFLICT"))?;
    let delivery = state
        .team_message_deliveries
        .get(id)
        .ok_or_else(|| denied("TEAM_COMMAND_CONFLICT"))?;
    let mut result = json!({"receipt":receipt,"delivery":delivery});
    if let Some(id) = &receipt.attempt_id {
        result["queuedAttemptId"] = json!(id);
    }
    Ok(result)
}
