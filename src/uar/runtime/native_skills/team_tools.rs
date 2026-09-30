//! Four attempt-bound tools. No model argument can nominate a sender or root.
use crate::uar::{
    compiler::collaboration::CollaborationCatalogService,
    domain::{team_execution::TeamExecutionAttempt, team_wait::*},
    runtime::native_skill::{NativeExecutionContext, NativeSkill, NativeSkillRegistry},
    tools::descriptor::{ApprovalClass, Exposure, ToolAssemblyError, ToolEffect, ToolSource},
};
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};
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
    fn name(&self) -> &str {
        self.name
    }
    fn description(&self) -> &str {
        match self.name {
            "team_roster" => {
                "List currently authorized teammates and current teamRevision for optimistic delegation. Identities are host-resolved; prompts and histories are private."
            }
            "team_send" => {
                "Queue an attributed message to an authorized peer. This never starts a model turn."
            }
            "team_delegate" => {
                "Delegate a bounded task atomically to an authorized team member. Supply current teamRevision from team_roster (or initial context); queued work runs through the team controller."
            }
            _ => {
                "Durably wait for all delegated target tasks to end, including failure/cancellation. This ends your current turn; the host admits a fresh continuation with ordered outcomes. No later tool call in this batch runs."
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
        if matches!(self.name, "team_roster" | "team_wait") {
            ApprovalClass::NotRequired
        } else {
            ApprovalClass::Required
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
                let r: crate::uar::domain::team_context::RosterRequest =
                    serde_json::from_value(args)
                        .map_err(|_| anyhow::anyhow!("TEAM_CONTEXT_REQUIRED_UNSUPPORTED"))?;
                Ok(service
                    .team_roster(a, r.cursor, u64::from(r.limit))
                    .await
                    .map_err(safe_error)?)
            }
            "team_send" => Ok(service.team_send(a, serde_json::from_value(args)?).await?),
            "team_delegate" => Ok(service
                .team_delegate(a, serde_json::from_value(args)?)
                .await?),
            _ => {
                let request: TeamWaitRequest = serde_json::from_value(args)?;
                let result = service
                    .request_team_wait(a, request.clone())
                    .await
                    .map_err(safe_error)?;
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
) -> Result<(), ToolAssemblyError> {
    for name in ["team_roster", "team_send", "team_delegate", "team_wait"] {
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
