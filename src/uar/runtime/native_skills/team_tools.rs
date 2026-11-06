//! Four attempt-bound tools. No model argument can nominate a sender or root.
use crate::uar::{
    compiler::collaboration::CollaborationCatalogService,
    domain::{team_execution::TeamExecutionAttempt, team_wait::*},
    runtime::native_skill::{NativeExecutionContext, NativeSkill, NativeSkillRegistry},
    tools::descriptor::{ApprovalClass, Exposure, ToolEffect, ToolSource},
};
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};

/// Peer controls materialized only for a qualified, bound team attempt.
pub(crate) const TEAM_TOOL_NAMES: [&str; 4] =
    ["team_roster", "team_send", "team_delegate", "team_wait"];

/// Descriptor approval semantics shared with advertisement preflight.
pub(crate) fn approval_required(name: &str) -> bool {
    matches!(name, "team_send" | "team_delegate")
}

#[derive(Clone)]
pub(crate) struct TeamToolBinding {
    pub catalog: Arc<CollaborationCatalogService>,
    pub attempt: TeamExecutionAttempt,
    pub yielded: Arc<Mutex<Option<KernelTeamYield>>>,
}
struct TeamTool {
    binding: TeamToolBinding,
    name: &'static str,
    pending: Mutex<Option<KernelTeamYield>>,
}
#[async_trait::async_trait]
impl NativeSkill for TeamTool {
    fn admission_owner(&self) -> crate::uar::persistence::tool_admission::AdmissionOwner {
        crate::uar::persistence::tool_admission::AdmissionOwner::UarRuntime
    }

    fn name(&self) -> &str {
        self.name
    }
    fn description(&self) -> &str {
        match self.name {
            "team_roster" => {
                "List currently authorized teammates and current teamRevision for optimistic delegation. Identities are host-resolved; prompts and histories are private. Omit cursor or send an empty string for the first page; use nextCursor unchanged for subsequent pages."
            }
            "team_send" => {
                "Queue an attributed message to an authorized peer after required approval. This never starts a model turn. A refused result does not queue a message."
            }
            "team_delegate" => {
                "Delegate a bounded task atomically to an authorized team member after required approval. Supply current teamRevision from team_roster (or initial context); queued work runs through the team controller. A recoverable refused result requires correcting the request; it did not delegate work."
            }
            _ => {
                "Durably wait for all delegated target tasks to end, including failure/cancellation. An accepted wait ends your current turn; the host admits a fresh continuation with ordered outcomes. No later tool call in that batch runs. A recoverable refused result does not yield: correct the targets or refresh the roster before continuing."
            }
        }
    }
    fn parameters_schema(&self) -> Value {
        let id = json!({"type":"string","minLength":1,"maxLength":128});
        let ids = json!({"type":"array","items":id,"uniqueItems":true,"maxItems":16});
        let payload = json!({"type":"object","properties":{"text":{"type":"string","maxLength":8192},"artifactIds":ids},"required":["text","artifactIds"],"additionalProperties":false});
        let reservation = json!({"type":"object","properties":{"tokens":{"type":"integer","minimum":1},"costMicrounits":{"type":"integer","minimum":0},"elapsedSeconds":{"type":"integer","minimum":1}},"required":["tokens","costMicrounits","elapsedSeconds"],"additionalProperties":false});
        match self.name {
            "team_roster" => {
                json!({"type":"object","properties":{"cursor":{"type":"string","maxLength":2048},"limit":{"type":"integer","minimum":1,"maximum":50}},"required":["limit"],"additionalProperties":false})
            }
            "team_send" => {
                json!({"type":"object","properties":{"commandId":id,"recipient":{"type":"object","properties":{"memberId":id,"taskId":id},"required":["memberId"],"additionalProperties":false},"payload":payload},"required":["commandId","recipient","payload"],"additionalProperties":false})
            }
            "team_delegate" => {
                json!({"type":"object","properties":{"commandId":id,"recipientMemberId":id,"expectedTeamRevision":{"type":"integer","minimum":1},"task":{"type":"object","properties":{"taskId":id,"role":id,"input":{},"outputContract":{"type":"object"},"dependsOn":ids},"required":["taskId","role","input","outputContract","dependsOn"],"additionalProperties":false},"payload":payload,"reservation":reservation},"required":["commandId","recipientMemberId","expectedTeamRevision","task","payload","reservation"],"additionalProperties":false})
            }
            _ => {
                json!({"type":"object","properties":{"commandId":id,"targetTaskIds":{"type":"array","items":id,"minItems":1,"maxItems":16,"uniqueItems":true},"predicate":{"const":"all-terminal"},"continuationInput":payload,"continuationReservation":reservation},"required":["commandId","targetTaskIds","predicate","continuationInput","continuationReservation"],"additionalProperties":false})
            }
        }
    }
    fn effect(&self) -> ToolEffect {
        if self.name == "team_roster" {
            ToolEffect::ReadOnly
        } else {
            ToolEffect::ExternalMutation
        }
    }
    fn approval_class(&self) -> ApprovalClass {
        if approval_required(self.name) {
            ApprovalClass::Required
        } else {
            ApprovalClass::NotRequired
        }
    }
    fn exposure(&self) -> Exposure {
        Exposure::ModelOnly
    }
    fn source(&self) -> ToolSource {
        ToolSource::BuiltIn
    }
    async fn execute(&self, _: Value) -> anyhow::Result<Value> {
        anyhow::bail!("TEAM_SCOPE_DENIED")
    }
    async fn execute_with_context(
        &self,
        args: Value,
        context: &NativeExecutionContext,
    ) -> anyhow::Result<Value> {
        anyhow::ensure!(
            context
                .verified_owner
                .as_ref()
                .is_some_and(|o| o.presentation_owner_key() == self.binding.attempt.owner_id)
                && context.session_id.as_deref()
                    == Some(format!("team-attempt:{}", self.binding.attempt.id).as_str()),
            "TEAM_SCOPE_DENIED"
        );
        let service = &self.binding.catalog;
        let a = &self.binding.attempt;
        match self.name {
            "team_roster" => {
                let mut args = args;
                // The model tool accepts an empty first-page placeholder; the
                // canonical request represents that absence by omission only.
                if args.get("cursor").and_then(Value::as_str) == Some("") {
                    if let Some(object) = args.as_object_mut() {
                        object.remove("cursor");
                    }
                }
                let r: crate::uar::domain::team_context::RosterRequest =
                    serde_json::from_value(args)
                        .map_err(|_| anyhow::anyhow!("TEAM_CONTEXT_REQUIRED_UNSUPPORTED"))?;
                control_result(service.team_roster(a, r.cursor, u64::from(r.limit)).await)
            }
            "team_send" => {
                control_result(service.team_send(a, serde_json::from_value(args)?).await)
            }
            "team_delegate" => control_result(
                service
                    .team_delegate(a, serde_json::from_value(args)?)
                    .await,
            ),
            _ => {
                let request: TeamWaitRequest = serde_json::from_value(args)?;
                let result = match service.request_team_wait(a, request.clone()).await {
                    Ok(result) => result,
                    Err(error) => return control_result(Err(error)),
                };
                let wait_id = result["waitId"]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("TEAM_WAIT_INVALIDATED"))?
                    .to_owned();
                *self
                    .pending
                    .lock()
                    .map_err(|_| anyhow::anyhow!("TEAM_EFFECTS_UNCERTAIN"))? =
                    Some(KernelTeamYield {
                        kind: "team-yield".into(),
                        wait_id,
                        yielding_attempt_id: a.id.clone(),
                        durable_receipt_id: request.command_id,
                        remaining_tool_calls: vec![],
                    });
                Ok(result)
            }
        }
    }
    async fn finish_team_yield(
        &self,
        remaining: Vec<UnexecutedTeamToolCall>,
    ) -> anyhow::Result<Option<KernelTeamYield>> {
        let pending = self
            .pending
            .lock()
            .map_err(|_| anyhow::anyhow!("TEAM_EFFECTS_UNCERTAIN"))?
            .clone();
        let Some(mut control) = pending else {
            return Ok(None);
        };
        control.remaining_tool_calls = remaining;
        self.binding
            .catalog
            .record_kernel_team_yield(&self.binding.attempt, control.clone())
            .await?;
        *self
            .binding
            .yielded
            .lock()
            .map_err(|_| anyhow::anyhow!("TEAM_EFFECTS_UNCERTAIN"))? = Some(control.clone());
        Ok(Some(control))
    }
}
pub(crate) async fn register(
    registry: &NativeSkillRegistry,
    binding: TeamToolBinding,
    allowed: &std::collections::HashSet<String>,
) -> anyhow::Result<()> {
    // The same current authority powers roster capabilities and native tool
    // availability. Selection and governance can narrow this set further.
    let eligible = binding
        .catalog
        .eligible_team_controls(&binding.attempt)
        .await
        .map_err(safe_error)?;
    for name in TEAM_TOOL_NAMES {
        if !allowed.contains(name) || !eligible.iter().any(|candidate| candidate == name) {
            continue;
        }
        registry
            .register(TeamTool {
                binding: binding.clone(),
                name,
                pending: Mutex::new(None),
            })
            .await?;
    }
    Ok(())
}

fn control_result(
    result: Result<Value, crate::uar::compiler::collaboration::CollaborationError>,
) -> anyhow::Result<Value> {
    use crate::uar::compiler::collaboration::CollaborationError;
    match result {
        Ok(value) => Ok(value),
        // These exact catalog returns happen without committing the requested
        // command. Storage/authority/identity failures are deliberately excluded.
        Err(CollaborationError::Conflict(code))
            if matches!(
                code.as_str(),
                "TEAM_REVISION_CONFLICT" | "TEAM_WAIT_CYCLE" | "TEAM_PENDING_LIMIT"
            ) =>
        {
            Ok(json!({"status":"refused","recoverable":true,"code":code,
                "effect":"not_committed","action":"refresh_roster_or_correct_request"}))
        }
        Err(error) => Err(safe_error(error)),
    }
}

fn safe_error(error: crate::uar::compiler::collaboration::CollaborationError) -> anyhow::Error {
    use crate::uar::compiler::collaboration::CollaborationError;
    let code = match &error {
        CollaborationError::Conflict(code) | CollaborationError::Invalid(code)
            if code.starts_with("TEAM_") =>
        {
            code.split_whitespace()
                .next()
                .filter(|c| c.bytes().all(|b| b.is_ascii_uppercase() || b == b'_'))
                .unwrap_or("TEAM_SCOPE_DENIED")
        }
        CollaborationError::Storage(_) => "TEAM_EFFECTS_UNCERTAIN",
        _ => "TEAM_SCOPE_DENIED",
    };
    anyhow::anyhow!(serde_json::json!({"code":code,"retryable":false,"action":if code=="TEAM_BUDGET_EXHAUSTED"{"increase-budget"}else{"reconcile"}}).to_string())
}

#[cfg(test)]
mod control_outcome_tests {
    use super::*;
    use crate::uar::compiler::collaboration::CollaborationError;

    #[test]
    fn benign_control_refusal_is_explicitly_not_committed() {
        for code in [
            "TEAM_REVISION_CONFLICT",
            "TEAM_WAIT_CYCLE",
            "TEAM_PENDING_LIMIT",
        ] {
            let result = control_result(Err(CollaborationError::Conflict(code.into()))).unwrap();
            assert_eq!(result["status"], "refused");
            assert_eq!(result["recoverable"], true);
            assert_eq!(result["effect"], "not_committed");
            assert_eq!(result["code"], code);
        }
    }

    #[test]
    fn scope_integrity_and_uncertainty_are_not_recoverable_success() {
        for code in [
            "TEAM_SCOPE_DENIED",
            "TEAM_COMMAND_CONFLICT",
            "TEAM_EFFECTS_UNCERTAIN",
            "TEAM_EXECUTION_EPOCH_STALE",
        ] {
            assert!(control_result(Err(CollaborationError::Conflict(code.into()))).is_err());
        }
        assert!(
            control_result(Err(CollaborationError::Storage(
                "private source detail".into()
            )))
            .unwrap_err()
            .to_string()
            .contains("TEAM_EFFECTS_UNCERTAIN")
        );
    }
}
