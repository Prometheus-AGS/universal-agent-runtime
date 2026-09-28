//! C09.2 durable task ownership and operator state changes. No turn is admitted here.

use chrono::Utc;

use crate::uar::domain::team_planning::{
    AssignTeamReviewerRequest, AssignTeamTaskRequest, AssignmentAuthority, TeamInstance,
    TeamPlanningCommandReceipt, TransitionTeamTaskRequest,
};

use super::{
    CollaborationCatalogService, CollaborationError, MAX_CAS_ATTEMPTS, check_receipt, command_key,
    request_digest, scoped_team, team_key, validate_id, validate_scope,
};

enum TaskCommand {
    Claim(AssignTeamTaskRequest),
    Reassign(AssignTeamTaskRequest),
    Reviewer(AssignTeamReviewerRequest),
    State(TransitionTeamTaskRequest),
}

impl TaskCommand {
    fn metadata(&self) -> (&str, &str, u64, u64) {
        match self {
            Self::Claim(request) => (
                "claim-task",
                &request.command_id,
                request.expected_team_revision,
                request.expected_task_revision,
            ),
            Self::Reassign(request) => (
                "reassign-task",
                &request.command_id,
                request.expected_team_revision,
                request.expected_task_revision,
            ),
            Self::Reviewer(request) => (
                "assign-reviewer",
                &request.command_id,
                request.expected_team_revision,
                request.expected_task_revision,
            ),
            Self::State(request) => (
                "transition-task",
                &request.command_id,
                request.expected_team_revision,
                request.expected_task_revision,
            ),
        }
    }

    fn digest(&self) -> Result<String, CollaborationError> {
        Ok(match self {
            Self::Claim(request) | Self::Reassign(request) => request_digest(request)?,
            Self::Reviewer(request) => request_digest(request)?,
            Self::State(request) => request_digest(request)?,
        })
    }

    fn apply(&self, team: &mut TeamInstance, task_index: usize) -> Result<(), CollaborationError> {
        let task = &team.tasks[task_index];
        if matches!(task.status.as_str(), "succeeded" | "failed" | "cancelled") {
            return Err(CollaborationError::Conflict(
                "terminal task state is immutable".to_owned(),
            ));
        }
        match self {
            Self::Claim(request) | Self::Reassign(request) => {
                validate_id(&request.member_id)?;
                if task.status != "ready" {
                    return Err(CollaborationError::Conflict(
                        "only a ready task can be assigned".to_owned(),
                    ));
                }
                let member = team
                    .members
                    .iter()
                    .find(|member| member.id == request.member_id)
                    .ok_or_else(|| CollaborationError::NotFound(request.member_id.clone()))?;
                if member.role != task.role {
                    return Err(CollaborationError::Invalid(
                        "assignee does not occupy the task role".to_owned(),
                    ));
                }
                if task.reviewer_member_id.as_deref() == Some(request.member_id.as_str()) {
                    return Err(CollaborationError::Invalid(
                        "assignee and reviewer must be different members".to_owned(),
                    ));
                }
                let is_claim = matches!(self, Self::Claim(_));
                if is_claim != task.assignee_member_id.is_none() {
                    return Err(CollaborationError::Conflict(if is_claim {
                        "task is already claimed".to_owned()
                    } else {
                        "task has no assignee to reassign".to_owned()
                    }));
                }
                if task.assignee_member_id.as_deref() == Some(request.member_id.as_str()) {
                    return Err(CollaborationError::Conflict(
                        "task is already assigned to this member".to_owned(),
                    ));
                }
                let binding_id = team.binding.id.clone();
                let binding_revision = team.binding.revision;
                let workspace_id = team.workspace_id.clone();
                let task = &mut team.tasks[task_index];
                task.assignee_member_id = Some(request.member_id.clone());
                task.ownership_epoch = next_counter(task.ownership_epoch, "ownership epoch")?;
                task.assignment_authority = Some(AssignmentAuthority {
                    binding_id,
                    binding_revision,
                    workspace_id,
                    role: task.role.clone(),
                    member_id: request.member_id.clone(),
                    ownership_epoch: task.ownership_epoch,
                    can_execute: false,
                    can_use_tools: false,
                });
            }
            Self::Reviewer(request) => {
                if !matches!(task.status.as_str(), "queued" | "ready") {
                    return Err(CollaborationError::Conflict(
                        "reviewer can only be assigned before execution".to_owned(),
                    ));
                }
                if let Some(member_id) = &request.member_id {
                    validate_id(member_id)?;
                    if !team.members.iter().any(|member| member.id == *member_id) {
                        return Err(CollaborationError::NotFound(member_id.clone()));
                    }
                    if task.assignee_member_id.as_deref() == Some(member_id.as_str()) {
                        return Err(CollaborationError::Invalid(
                            "reviewer and assignee must be different members".to_owned(),
                        ));
                    }
                }
                if task.reviewer_member_id.as_deref() == request.member_id.as_deref() {
                    return Err(CollaborationError::Conflict(
                        "reviewer assignment is unchanged".to_owned(),
                    ));
                }
                let task = &mut team.tasks[task_index];
                task.reviewer_member_id = request.member_id.clone();
                task.reviewer_epoch = next_counter(task.reviewer_epoch, "reviewer epoch")?;
            }
            Self::State(request) => {
                if request.reason.trim().is_empty() {
                    return Err(CollaborationError::Invalid(
                        "state transition requires a reason".to_owned(),
                    ));
                }
                let allowed = match (task.status.as_str(), request.status.as_str()) {
                    ("queued", "ready") => task.depends_on.iter().all(|dependency| {
                        team.tasks
                            .iter()
                            .any(|other| other.id == *dependency && other.status == "succeeded")
                    }),
                    ("queued" | "ready", "failed" | "cancelled") => true,
                    _ => false,
                };
                if !allowed {
                    return Err(CollaborationError::Conflict(
                        "task state transition is not allowed or dependencies remain unresolved"
                            .to_owned(),
                    ));
                }
                let task = &mut team.tasks[task_index];
                task.status = request.status.clone();
                task.state_reason = Some(request.reason.trim().to_owned());
                if matches!(request.status.as_str(), "failed" | "cancelled") {
                    task.ownership_epoch = next_counter(task.ownership_epoch, "ownership epoch")?;
                    task.assignment_authority = None;
                }
            }
        }
        Ok(())
    }
}

fn next_counter(current: u64, label: &str) -> Result<u64, CollaborationError> {
    current
        .checked_add(1)
        .ok_or_else(|| CollaborationError::Conflict(format!("{label} is exhausted")))
}

impl CollaborationCatalogService {
    pub async fn claim_team_task(
        &self,
        owner_id: &str,
        workspace_id: &str,
        team_id: &str,
        task_id: &str,
        request: AssignTeamTaskRequest,
    ) -> Result<TeamInstance, CollaborationError> {
        self.mutate_team_task(
            owner_id,
            workspace_id,
            team_id,
            task_id,
            TaskCommand::Claim(request),
        )
        .await
    }

    pub async fn reassign_team_task(
        &self,
        owner_id: &str,
        workspace_id: &str,
        team_id: &str,
        task_id: &str,
        request: AssignTeamTaskRequest,
    ) -> Result<TeamInstance, CollaborationError> {
        self.mutate_team_task(
            owner_id,
            workspace_id,
            team_id,
            task_id,
            TaskCommand::Reassign(request),
        )
        .await
    }

    pub async fn assign_team_reviewer(
        &self,
        owner_id: &str,
        workspace_id: &str,
        team_id: &str,
        task_id: &str,
        request: AssignTeamReviewerRequest,
    ) -> Result<TeamInstance, CollaborationError> {
        self.mutate_team_task(
            owner_id,
            workspace_id,
            team_id,
            task_id,
            TaskCommand::Reviewer(request),
        )
        .await
    }

    pub async fn transition_team_task(
        &self,
        owner_id: &str,
        workspace_id: &str,
        team_id: &str,
        task_id: &str,
        request: TransitionTeamTaskRequest,
    ) -> Result<TeamInstance, CollaborationError> {
        self.mutate_team_task(
            owner_id,
            workspace_id,
            team_id,
            task_id,
            TaskCommand::State(request),
        )
        .await
    }

    async fn mutate_team_task(
        &self,
        owner_id: &str,
        workspace_id: &str,
        team_id: &str,
        task_id: &str,
        command: TaskCommand,
    ) -> Result<TeamInstance, CollaborationError> {
        validate_scope(owner_id, workspace_id)?;
        validate_id(team_id)?;
        validate_id(task_id)?;
        let (operation, command_id, expected_team_revision, expected_task_revision) =
            command.metadata();
        validate_id(command_id)?;
        let digest = command.digest()?;
        let receipt_key = command_key(owner_id, workspace_id, command_id);
        let key = team_key(owner_id, workspace_id, team_id);

        for _ in 0..MAX_CAS_ATTEMPTS {
            let current = self.load_state().await?;
            if let Some(receipt) = current.team_command_receipts.get(&receipt_key) {
                check_receipt(receipt, operation, &digest)?;
                if receipt.team_id != team_id || receipt.task_id.as_deref() != Some(task_id) {
                    return Err(CollaborationError::Conflict(
                        "commandId belongs to another team task".to_owned(),
                    ));
                }
                return scoped_team(&current, owner_id, workspace_id, team_id);
            }
            let mut team = scoped_team(&current, owner_id, workspace_id, team_id)?;
            if team.revision != expected_team_revision {
                return Err(CollaborationError::Conflict(format!(
                    "expectedTeamRevision {expected_team_revision} does not match {}",
                    team.revision
                )));
            }
            let task_index = team
                .tasks
                .iter()
                .position(|task| task.id == task_id)
                .ok_or_else(|| CollaborationError::NotFound(task_id.to_owned()))?;
            if team.tasks[task_index].revision != expected_task_revision {
                return Err(CollaborationError::Conflict(format!(
                    "expectedTaskRevision {expected_task_revision} does not match {}",
                    team.tasks[task_index].revision
                )));
            }
            command.apply(&mut team, task_index)?;
            let now = Utc::now();
            team.tasks[task_index].revision =
                next_counter(team.tasks[task_index].revision, "task revision")?;
            team.tasks[task_index].updated_at = now;
            team.revision = next_counter(team.revision, "team revision")?;
            team.updated_at = now;
            let mut next = current.clone();
            next.generation = next_counter(current.generation, "catalog generation")?;
            next.team_instances.insert(key.clone(), team.clone());
            next.team_command_receipts.insert(
                receipt_key.clone(),
                TeamPlanningCommandReceipt {
                    owner_id: owner_id.to_owned(),
                    workspace_id: workspace_id.to_owned(),
                    command_id: command_id.to_owned(),
                    request_digest: digest.clone(),
                    operation: operation.to_owned(),
                    team_id: team_id.to_owned(),
                    task_id: Some(task_id.to_owned()),
                    committed_at: now,
                },
            );
            if self.cas(current.generation, &next).await? {
                return Ok(team);
            }
        }
        Err(CollaborationError::Conflict(
            "team task board changed during mutation".to_owned(),
        ))
    }
}
