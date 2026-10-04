use super::*;
use crate::uar::domain::team_planning::{AssignmentAuthority, TeamTask};
use chrono::Utc;
use serde_json::json;
use uuid::Uuid;

impl CollaborationCatalogService {
    pub async fn start_workflow(
        &self,
        owner: &str,
        workspace: &str,
        request: StartWorkflowRequest,
    ) -> Result<WorkflowRun, CollaborationError> {
        super::super::service::validate_owner(owner)?;
        super::super::validation::validate_id(workspace)?;
        super::super::validation::validate_id(&request.command_id)?;
        let digest = super::super::validation::canonical_digest(&serde_json::to_value(&request)?)?;
        let command = key(owner, workspace, &format!("start:{}", request.command_id));
        let id = Uuid::new_v4().to_string();
        for _ in 0..MAX_CAS_ATTEMPTS {
            let current = self.load_state().await?;
            self.require_execution_owner(&current, false)?;
            if let Some(receipt) = current.workflow_commands.get(&command) {
                if receipt.request_digest != digest || receipt.operation != "start" {
                    return Err(conflict("WORKFLOW_COMMAND_CONFLICT"));
                }
                authority(&current, run(&current, owner, workspace, &receipt.run_id)?)?;
                return Ok(receipt.result.clone());
            }
            let definition = current
                .definitions
                .get(&request.workflow_definition.storage_key())
                .ok_or_else(|| {
                    CollaborationError::NotFound(request.workflow_definition.id.clone())
                })?;
            let plan = compiler::compile(definition)?;
            let package = current
                .packages
                .get(&definition.package.storage_key())
                .ok_or_else(|| conflict("WORKFLOW_PACKAGE_UNAVAILABLE"))?;
            if package.manifest["requiredCapabilities"]
                .as_array()
                .into_iter()
                .flatten()
                .any(|capability| {
                    let capability = capability.as_str().unwrap_or_default();
                    !matches!(
                        capability,
                        WORKFLOW_CAPABILITY
                            | "collaboration_definition_packages_v1"
                            | "collaboration_definition_packages_v2"
                    ) && !(crate::uar::api::capabilities::team_execution_b_enabled()
                        && crate::uar::api::capabilities::TEAM_EXECUTION_B_CAPABILITIES
                            .contains(&capability))
                })
            {
                return Err(conflict("WORKFLOW_PACKAGE_CAPABILITY_UNSUPPORTED"));
            }
            if !package.manifest["requiredCapabilities"]
                .as_array()
                .is_some_and(|a| a.iter().any(|v| v.as_str() == Some(WORKFLOW_CAPABILITY)))
            {
                return Err(conflict("WORKFLOW_PACKAGE_CAPABILITY_REQUIRED"));
            }
            let mut team = current
                .team_instances
                .get(&key(owner, workspace, &request.team_id))
                .cloned()
                .ok_or_else(|| CollaborationError::NotFound(request.team_id.clone()))?;
            if team.revision != request.expected_team_revision
                || team.binding.revision != request.expected_binding_revision
                || team.package != definition.package
                || team.status == "cancelled"
            {
                return Err(conflict("WORKFLOW_TEAM_OR_BINDING_CHANGED"));
            }
            let binding = current
                .bindings
                .get(&super::super::bindings::binding_key(
                    owner,
                    workspace,
                    &team.binding.id,
                ))
                .ok_or_else(|| conflict("WORKFLOW_BINDING_UNAVAILABLE"))?;
            if binding.revision != team.binding.revision {
                return Err(conflict("WORKFLOW_BINDING_CHANGED"));
            }
            let team_definition = current
                .definitions
                .get(&team.definition.storage_key())
                .ok_or_else(|| conflict("WORKFLOW_TEAM_DEFINITION_UNAVAILABLE"))?;
            if !team_definition.document["taskAcceptance"]["allowedWorkflows"]
                .as_array()
                .is_some_and(|items| {
                    items.iter().any(|item| {
                        serde_json::from_value::<
                            crate::uar::domain::collaboration::ImmutableDefinitionRef,
                        >(item.clone())
                        .is_ok_and(|r| r == definition.identity)
                    })
                })
            {
                return Err(conflict("WORKFLOW_NOT_ACCEPTED_BY_TEAM"));
            }
            let max_tasks = team_definition.document["limits"]["maxPendingTasks"]
                .as_u64()
                .unwrap_or(0)
                .min(
                    binding.document["effectiveLimits"]["maxPendingTasks"]
                        .as_u64()
                        .unwrap_or(u64::MAX),
                );
            if team.tasks.len() as u64 + 2 > max_tasks {
                return Err(conflict("WORKFLOW_TEAM_TASK_LIMIT"));
            }
            let activations = current
                .workflow_runs
                .values()
                .filter(|r| {
                    r.owner_id == owner
                        && r.workspace_id == workspace
                        && r.definition == definition.identity
                })
                .count() as u64;
            if activations >= plan.max_activations {
                return Err(conflict("WORKFLOW_ACTIVATION_LIMIT"));
            }
            let validator = jsonschema::validator_for(&plan.input)
                .map_err(|_| conflict("WORKFLOW_INPUT_SCHEMA_INVALID"))?;
            if !validator.is_valid(&request.input) {
                return Err(CollaborationError::Invalid("WORKFLOW_INPUT_INVALID".into()));
            }
            if request.steps.len() != 2 {
                return Err(CollaborationError::Invalid(
                    "WORKFLOW_TWO_STEP_SELECTION_REQUIRED".into(),
                ));
            }
            let now = Utc::now();
            let mut steps = Vec::new();
            for (index, compiled) in plan.steps.iter().enumerate() {
                let selection = &request.steps[index];
                if selection.step_id != compiled.id
                    || selection.reservation.tokens == 0
                    || selection.reservation.elapsed_seconds == 0
                {
                    return Err(conflict("WORKFLOW_STEP_SELECTION_INVALID"));
                }
                let member = team
                    .members
                    .iter()
                    .find(|m| {
                        m.id == selection.member_id
                            && m.role == compiled.role
                            && !matches!(m.status.as_str(), "revoked" | "stopped")
                    })
                    .ok_or_else(|| conflict("WORKFLOW_MEMBER_AUTHORITY_CHANGED"))?;
                let agent = current
                    .definitions
                    .get(&member.definition.storage_key())
                    .ok_or_else(|| conflict("WORKFLOW_MEMBER_DEFINITION_UNAVAILABLE"))?;
                let mut properties = json!({"feedback":plan.input["properties"]["feedback"]});
                let required = if index == 0 {
                    json!(["feedback"])
                } else {
                    properties["classification"] = plan.steps[0].output.clone();
                    json!(["feedback", "classification"])
                };
                if agent.document["input"]
                    != json!({"type":"object","properties":properties,"required":required,"additionalProperties":false})
                {
                    return Err(conflict("WORKFLOW_MEMBER_INPUT_CONTRACT_MISMATCH"));
                }
                if agent.document["output"] != compiled.output {
                    return Err(conflict("WORKFLOW_MEMBER_OUTPUT_CONTRACT_MISMATCH"));
                }
                let task_id = format!("workflow-{id}-{}", compiled.id);
                let depends_on = if index == 0 {
                    vec![]
                } else {
                    vec![format!("workflow-{id}-classify")]
                };
                team.tasks.push(TeamTask {
                    id: task_id.clone(),
                    title: compiled.id.clone(),
                    role: compiled.role.clone(),
                    input: if index == 0 {
                        json!({"feedback":request.input["feedback"]})
                    } else {
                        json!({})
                    },
                    output_contract: compiled.output.clone(),
                    output: None,
                    depends_on,
                    status: if index == 0 { "ready" } else { "queued" }.into(),
                    revision: 1,
                    assignee_member_id: Some(member.id.clone()),
                    ownership_epoch: 1,
                    assignment_authority: Some(AssignmentAuthority {
                        binding_id: team.binding.id.clone(),
                        binding_revision: team.binding.revision,
                        workspace_id: workspace.into(),
                        role: compiled.role.clone(),
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
                steps.push(WorkflowStepState {
                    step_id: compiled.id.clone(),
                    task_id,
                    member_id: member.id.clone(),
                    member_revision: member.revision,
                    member_definition: member.definition.clone(),
                    reservation: selection.reservation.clone(),
                    status: if index == 0 { "ready" } else { "pending" }.into(),
                    attempt_id: None,
                    artifact: None,
                });
            }
            team.revision += 1;
            team.updated_at = now;
            let result = WorkflowRun {
                id: id.clone(),
                owner_id: owner.into(),
                workspace_id: workspace.into(),
                revision: 1,
                status: "ready".into(),
                definition: definition.identity.clone(),
                package: definition.package.clone(),
                definition_snapshot: definition.document.clone(),
                plan,
                team_id: team.id.clone(),
                team_definition: team.definition.clone(),
                binding: team.binding.clone(),
                input: request.input.clone(),
                steps,
                wait: None,
                decision: None,
                state_reason: None,
                accounting: WorkflowAccounting::default(),
                created_at: now,
                updated_at: now,
            };
            let mut next = current.clone();
            next.generation += 1;
            next.team_instances
                .insert(key(owner, workspace, &team.id), team);
            next.workflow_runs
                .insert(key(owner, workspace, &id), result.clone());
            next.workflow_commands.insert(
                command.clone(),
                WorkflowCommandReceipt {
                    request_digest: digest.clone(),
                    operation: "start".into(),
                    run_id: id.clone(),
                    result: result.clone(),
                    completed: true,
                    team_control: None,
                },
            );
            if self.cas(current.generation, &next).await? {
                return Ok(result);
            }
        }
        Err(conflict("WORKFLOW_CONCURRENT_CHANGE"))
    }
}
