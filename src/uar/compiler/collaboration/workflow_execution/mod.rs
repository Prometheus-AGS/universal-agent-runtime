//! Catalog-owned workflow progression; C09 remains the execution and usage owner.
mod compiler;
mod progression;
mod start;
pub(crate) use progression::{admission_guard, link_attempt};
mod control;
use super::{CollaborationCatalogService, CollaborationError, service::MAX_CAS_ATTEMPTS};
use crate::uar::domain::{
    collaboration::{CollaborationCatalogState, CollaborationKind},
    team_execution::TeamExecutionAttempt,
    team_planning::TeamInstance,
    workflow_execution::*,
};

pub(super) fn key(owner: &str, workspace: &str, id: &str) -> String {
    format!("{owner}\u{1f}{workspace}\u{1f}{id}")
}
fn conflict(message: &str) -> CollaborationError {
    CollaborationError::Conflict(message.into())
}
fn run<'a>(
    state: &'a CollaborationCatalogState,
    owner: &str,
    workspace: &str,
    id: &str,
) -> Result<&'a WorkflowRun, CollaborationError> {
    super::service::validate_owner(owner)?;
    super::validation::validate_id(workspace)?;
    state
        .workflow_runs
        .get(&key(owner, workspace, id))
        .ok_or_else(|| CollaborationError::NotFound(id.into()))
}
fn authority<'a>(
    state: &'a CollaborationCatalogState,
    run: &WorkflowRun,
) -> Result<&'a TeamInstance, CollaborationError> {
    let team = state
        .team_instances
        .get(&key(&run.owner_id, &run.workspace_id, &run.team_id))
        .ok_or_else(|| conflict("WORKFLOW_TEAM_UNAVAILABLE"))?;
    let binding = state
        .bindings
        .get(&super::bindings::binding_key(
            &run.owner_id,
            &run.workspace_id,
            &run.binding.id,
        ))
        .ok_or_else(|| conflict("WORKFLOW_BINDING_UNAVAILABLE"))?;
    super::grants::validate_binding_grants(
        &run.owner_id,
        &run.workspace_id,
        &binding.document,
        state,
    )?;
    if team.status == "cancelled"
        || team.definition != run.team_definition
        || team.binding.id != run.binding.id
        || team.binding.revision != run.binding.revision
        || binding.revision != run.binding.revision
        || binding.package != run.package
    {
        return Err(conflict("WORKFLOW_AUTHORITY_CHANGED"));
    }
    for step in &run.steps {
        if !team.members.iter().any(|member| {
            member.id == step.member_id
                && member.revision == step.member_revision
                && member.definition == step.member_definition
                && !matches!(member.status.as_str(), "revoked" | "stopped")
        }) {
            return Err(conflict("WORKFLOW_MEMBER_AUTHORITY_CHANGED"));
        }
        let task = team
            .tasks
            .iter()
            .find(|t| t.id == step.task_id)
            .ok_or_else(|| conflict("WORKFLOW_TASK_UNAVAILABLE"))?;
        if task.assignee_member_id.as_deref() != Some(step.member_id.as_str())
            || task.ownership_epoch != 1
        {
            return Err(conflict("WORKFLOW_TASK_AUTHORITY_CHANGED"));
        }
    }
    Ok(team)
}

pub(crate) fn workflow_for_attempt<'a>(
    state: &'a CollaborationCatalogState,
    attempt: &TeamExecutionAttempt,
) -> Option<&'a WorkflowRun> {
    state.workflow_runs.values().find(|r| {
        r.owner_id == attempt.owner_id
            && r.workspace_id == attempt.workspace_id
            && r.team_id == attempt.team_id
            && r.steps.iter().any(|s| s.task_id == attempt.task_id)
    })
}

impl CollaborationCatalogService {
    pub async fn list_workflow_definitions(
        &self,
    ) -> Result<Vec<WorkflowDefinitionSummary>, CollaborationError> {
        Ok(self
            .load_state()
            .await?
            .definitions
            .values()
            .filter(|r| r.kind == CollaborationKind::WorkflowDefinition)
            .map(|record| {
                let result = compiler::compile(record);
                WorkflowDefinitionSummary {
                    identity: record.identity.clone(),
                    package: record.package.clone(),
                    title: record.document["title"].as_str().unwrap_or_default().into(),
                    input: record.document["input"].clone(),
                    output: record.document["output"].clone(),
                    steps: result.as_ref().map(|p| p.steps.clone()).unwrap_or_default(),
                    supported: result.is_ok(),
                    diagnostics: result
                        .err()
                        .map(|e| {
                            vec![WorkflowDiagnostic {
                                field: "/".into(),
                                code: "WORKFLOW_REQUIRED_UNSUPPORTED".into(),
                                message: e.to_string(),
                            }]
                        })
                        .unwrap_or_default(),
                }
            })
            .collect())
    }
    pub async fn list_owner_workflow_runs(
        &self,
        owner: &str,
    ) -> Result<Vec<WorkflowRun>, CollaborationError> {
        Ok(self
            .load_state()
            .await?
            .workflow_runs
            .into_values()
            .filter(|r| r.owner_id == owner)
            .collect())
    }
    pub async fn list_workflow_runs(
        &self,
        owner: &str,
        workspace: &str,
    ) -> Result<Vec<WorkflowRun>, CollaborationError> {
        super::service::validate_owner(owner)?;
        super::validation::validate_id(workspace)?;
        let state = self.load_state().await?;
        state
            .workflow_runs
            .values()
            .filter(|r| r.owner_id == owner && r.workspace_id == workspace)
            .map(|r| view(&state, r))
            .collect()
    }
    pub async fn get_workflow_run(
        &self,
        owner: &str,
        workspace: &str,
        id: &str,
    ) -> Result<WorkflowRun, CollaborationError> {
        let state = self.load_state().await?;
        view(&state, run(&state, owner, workspace, id)?)
    }
    pub async fn workflow_attempt(
        &self,
        attempt: &TeamExecutionAttempt,
    ) -> Result<Option<WorkflowRun>, CollaborationError> {
        let state = self.load_state().await?;
        let Some(run) = workflow_for_attempt(&state, attempt) else {
            return Ok(None);
        };
        authority(&state, run)?;
        if !crate::uar::api::capabilities::workflow_execution_enabled() {
            return Err(conflict("WORKFLOW_CAPABILITY_UNSUPPORTED"));
        }
        if !matches!(run.status.as_str(), "ready" | "running") {
            return Err(conflict("WORKFLOW_NOT_EXECUTABLE"));
        }
        Ok(Some(run.clone()))
    }
}

fn view(
    state: &CollaborationCatalogState,
    run: &WorkflowRun,
) -> Result<WorkflowRun, CollaborationError> {
    let mut result = run.clone();
    result.accounting = WorkflowAccounting::default();
    for step in &run.steps {
        if let Some(attempt) = step.attempt_id.as_ref().and_then(|id| {
            state.team_execution_attempts.values().find(|a| {
                a.id == *id
                    && a.owner_id == run.owner_id
                    && a.workspace_id == run.workspace_id
                    && a.team_id == run.team_id
            })
        }) {
            let amount = attempt.usage.as_ref().unwrap_or(&attempt.reservation);
            let target = if attempt.usage.is_some() {
                &mut result.accounting.committed
            } else {
                result
                    .accounting
                    .unresolved_attempt_ids
                    .push(attempt.id.clone());
                &mut result.accounting.reserved
            };
            target.tokens = target
                .tokens
                .checked_add(amount.tokens)
                .ok_or_else(|| conflict("workflow accounting overflow"))?;
            target.cost_microunits = target
                .cost_microunits
                .checked_add(amount.cost_microunits)
                .ok_or_else(|| conflict("workflow accounting overflow"))?;
            target.elapsed_seconds = target
                .elapsed_seconds
                .checked_add(amount.elapsed_seconds)
                .ok_or_else(|| conflict("workflow accounting overflow"))?;
        }
    }
    Ok(result)
}
