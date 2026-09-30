//! C09.1 team planning over the existing atomic collaboration catalog state.

mod claims;

use std::collections::{BTreeMap, BTreeSet};

use chrono::Utc;
use serde_json::Value;
use uuid::Uuid;

use crate::uar::domain::{
    collaboration::{CollaborationCatalogState, CollaborationKind, ImmutableDefinitionRef},
    team_planning::{
        CreateTeamRequest, CreateTeamTaskRequest, TeamBindingRef, TeamDefinitionSummary,
        TeamInstance, TeamMember, TeamMemberSpec, TeamPlanningCommandReceipt, TeamTask,
    },
};

use super::{
    bindings::binding_key,
    service::{CollaborationCatalogService, CollaborationError, MAX_CAS_ATTEMPTS, validate_owner},
    validation::{request_digest, validate_id},
};

impl CollaborationCatalogService {
    pub async fn list_team_definitions(
        &self,
    ) -> Result<Vec<TeamDefinitionSummary>, CollaborationError> {
        self.load_state()
            .await?
            .definitions
            .into_values()
            .filter(|record| record.kind == CollaborationKind::TeamDefinition)
            .map(|record| {
                let members: Vec<TeamMemberSpec> =
                    serde_json::from_value(record.document["members"].clone())?;
                Ok(TeamDefinitionSummary {
                    instructions: record.document.get("instructions").cloned().map(serde_json::from_value).transpose()?,
                    id: record.identity.id,
                    version: record.identity.version,
                    digest: record.identity.digest,
                    title: required_text(&record.document, "title")?.to_owned(),
                    purpose: required_text(&record.document, "purpose")?.to_owned(),
                    package: record.package,
                    members,
                    budget: record.document["budget"].clone(),
                    limits: record.document["limits"].clone(),
                })
            })
            .collect()
    }

    pub async fn create_team_instance(
        &self,
        owner_id: &str,
        workspace_id: &str,
        request: CreateTeamRequest,
    ) -> Result<TeamInstance, CollaborationError> {
        validate_scope(owner_id, workspace_id)?;
        validate_id(&request.deployment_binding_id)?;
        validate_id(&request.team_definition.id)?;
        validate_id(&request.command_id)?;
        let digest = request_digest(&request)?;
        let receipt_key = command_key(owner_id, workspace_id, &request.command_id);

        for _ in 0..MAX_CAS_ATTEMPTS {
            let current = self.load_state().await?;
            if let Some(receipt) = current.team_command_receipts.get(&receipt_key) {
                check_receipt(receipt, "create-team", &digest)?;
                return scoped_team(&current, owner_id, workspace_id, &receipt.team_id);
            }
            let binding = current
                .bindings
                .get(&binding_key(
                    owner_id,
                    workspace_id,
                    &request.deployment_binding_id,
                ))
                .ok_or_else(|| {
                    CollaborationError::NotFound(request.deployment_binding_id.clone())
                })?;
            let definition = exact_team_definition(&current, &request.team_definition)?;
            if binding.package != definition.package {
                return Err(CollaborationError::Conflict(
                    "team definition is outside the selected binding's immutable package"
                        .to_owned(),
                ));
            }
            let package = current
                .packages
                .get(&binding.package.storage_key())
                .ok_or_else(|| {
                    CollaborationError::Conflict(
                        "selected package is no longer installed".to_owned(),
                    )
                })?;
            if !package.manifest["entrypoints"]
                .as_array()
                .is_some_and(|items| {
                    items.iter().any(|item| {
                        serde_json::from_value::<ImmutableDefinitionRef>(item.clone())
                            .is_ok_and(|entrypoint| entrypoint == request.team_definition)
                    })
                })
            {
                return Err(CollaborationError::Invalid(
                    "team definition is not a package entrypoint".to_owned(),
                ));
            }
            let input_schema = &definition.document["input"];
            let validator = jsonschema::validator_for(input_schema).map_err(|error| {
                CollaborationError::Storage(format!(
                    "installed team input schema is invalid: {error}"
                ))
            })?;
            if !validator.is_valid(&request.input) {
                return Err(CollaborationError::Invalid(
                    "team input does not satisfy the selected definition contract".to_owned(),
                ));
            }
            let members = create_members(&definition.document, &request.member_slots)?;
            let now = Utc::now();
            let team = TeamInstance {
                id: Uuid::new_v4().to_string(),
                owner_id: owner_id.to_owned(),
                workspace_id: workspace_id.to_owned(),
                revision: 1,
                status: "inactive".to_owned(),
                definition: request.team_definition.clone(),
                package: binding.package.clone(),
                binding: TeamBindingRef {
                    id: binding.id.clone(),
                    revision: binding.revision,
                },
                input: request.input.clone(),
                members,
                tasks: Vec::new(),
                created_at: now,
                updated_at: now,
            };
            let mut next = current.clone();
            next.generation = current.generation.saturating_add(1);
            next.team_instances
                .insert(team_key(owner_id, workspace_id, &team.id), team.clone());
            next.team_command_receipts.insert(
                receipt_key.clone(),
                TeamPlanningCommandReceipt {
                    owner_id: owner_id.to_owned(),
                    workspace_id: workspace_id.to_owned(),
                    command_id: request.command_id.clone(),
                    request_digest: digest.clone(),
                    operation: "create-team".to_owned(),
                    team_id: team.id.clone(),
                    task_id: None,
                    committed_at: now,
                },
            );
            if self.cas(current.generation, &next).await? {
                return Ok(team);
            }
        }
        Err(CollaborationError::Conflict(
            "team catalog changed during creation".to_owned(),
        ))
    }

    pub async fn list_team_instances(
        &self,
        owner_id: &str,
        workspace_id: &str,
    ) -> Result<Vec<TeamInstance>, CollaborationError> {
        validate_scope(owner_id, workspace_id)?;
        Ok(self
            .load_state()
            .await?
            .team_instances
            .into_values()
            .filter(|team| team.owner_id == owner_id && team.workspace_id == workspace_id)
            .collect())
    }

    pub async fn get_team_instance(
        &self,
        owner_id: &str,
        workspace_id: &str,
        team_id: &str,
    ) -> Result<TeamInstance, CollaborationError> {
        validate_scope(owner_id, workspace_id)?;
        validate_id(team_id)?;
        scoped_team(&self.load_state().await?, owner_id, workspace_id, team_id)
    }

    pub async fn create_team_task(
        &self,
        owner_id: &str,
        workspace_id: &str,
        team_id: &str,
        request: CreateTeamTaskRequest,
    ) -> Result<TeamInstance, CollaborationError> {
        validate_scope(owner_id, workspace_id)?;
        validate_id(team_id)?;
        validate_id(&request.task_id)?;
        validate_id(&request.command_id)?;
        if request.title.trim().is_empty() {
            return Err(CollaborationError::Invalid(
                "task title is required".to_owned(),
            ));
        }
        jsonschema::validator_for(&request.output_contract).map_err(|error| {
            CollaborationError::Invalid(format!("task outputContract is not JSON Schema: {error}"))
        })?;
        let digest = request_digest(&request)?;
        let receipt_key = command_key(owner_id, workspace_id, &request.command_id);
        let key = team_key(owner_id, workspace_id, team_id);

        for _ in 0..MAX_CAS_ATTEMPTS {
            let current = self.load_state().await?;
            if let Some(receipt) = current.team_command_receipts.get(&receipt_key) {
                check_receipt(receipt, "create-task", &digest)?;
                if receipt.team_id != team_id {
                    return Err(CollaborationError::Conflict(
                        "commandId belongs to another team".to_owned(),
                    ));
                }
                return scoped_team(&current, owner_id, workspace_id, team_id);
            }
            let mut team = scoped_team(&current, owner_id, workspace_id, team_id)?;
            if request.expected_team_revision != team.revision {
                return Err(CollaborationError::Conflict(format!(
                    "expectedTeamRevision {} does not match {}",
                    request.expected_team_revision, team.revision
                )));
            }
            if team.tasks.iter().any(|task| task.id == request.task_id) {
                return Err(CollaborationError::Conflict(
                    "taskId already exists".to_owned(),
                ));
            }
            if !team
                .members
                .iter()
                .any(|member| member.role == request.role)
            {
                return Err(CollaborationError::Invalid(
                    "task role has no member slot".to_owned(),
                ));
            }
            let definition = exact_team_definition(&current, &team.definition)?;
            let max_tasks = definition.document["limits"]["maxPendingTasks"]
                .as_u64()
                .ok_or_else(|| {
                    CollaborationError::Storage("installed team task limit is missing".to_owned())
                })?;
            if team.tasks.len() as u64 >= max_tasks {
                return Err(CollaborationError::Conflict(
                    "team task limit reached".to_owned(),
                ));
            }
            let mut unique = BTreeSet::new();
            for dependency in &request.depends_on {
                validate_id(dependency)?;
                if dependency == &request.task_id
                    || !unique.insert(dependency)
                    || !team.tasks.iter().any(|task| task.id == *dependency)
                {
                    return Err(CollaborationError::Invalid(
                        "dependsOn must contain distinct existing tasks in this team".to_owned(),
                    ));
                }
            }
            let now = Utc::now();
            team.tasks.push(TeamTask {
                id: request.task_id.clone(),
                title: request.title.clone(),
                role: request.role.clone(),
                input: request.input.clone(),
                output_contract: request.output_contract.clone(),
                output: None,
                depends_on: request.depends_on.clone(),
                status: if request.depends_on.is_empty() {
                    "ready".to_owned()
                } else {
                    "queued".to_owned()
                },
                revision: 1,
                assignee_member_id: None,
                ownership_epoch: 0,
                assignment_authority: None,
                reviewer_member_id: None,
                reviewer_epoch: 0,
                state_reason: None,
                created_at: now,
                updated_at: now,
            });
            team.revision = team.revision.saturating_add(1);
            team.updated_at = now;
            let mut next = current.clone();
            next.generation = current.generation.saturating_add(1);
            next.team_instances.insert(key.clone(), team.clone());
            next.team_command_receipts.insert(
                receipt_key.clone(),
                TeamPlanningCommandReceipt {
                    owner_id: owner_id.to_owned(),
                    workspace_id: workspace_id.to_owned(),
                    command_id: request.command_id.clone(),
                    request_digest: digest.clone(),
                    operation: "create-task".to_owned(),
                    team_id: team_id.to_owned(),
                    task_id: Some(request.task_id.clone()),
                    committed_at: now,
                },
            );
            if self.cas(current.generation, &next).await? {
                return Ok(team);
            }
        }
        Err(CollaborationError::Conflict(
            "team task board changed during creation".to_owned(),
        ))
    }
}

fn create_members(
    definition: &Value,
    slots: &[crate::uar::domain::team_planning::MemberSlotRequest],
) -> Result<Vec<TeamMember>, CollaborationError> {
    let specs: Vec<TeamMemberSpec> = serde_json::from_value(definition["members"].clone())?;
    let selected: BTreeMap<&str, u32> = slots
        .iter()
        .map(|slot| (slot.role.as_str(), slot.count))
        .collect();
    if selected.len() != slots.len() {
        return Err(CollaborationError::Invalid(
            "memberSlots contains a duplicate role".to_owned(),
        ));
    }
    if selected
        .keys()
        .any(|role| !specs.iter().any(|spec| &spec.role.as_str() == role))
    {
        return Err(CollaborationError::Invalid(
            "memberSlots contains an unknown role".to_owned(),
        ));
    }
    let mut members = Vec::new();
    for spec in specs {
        let count = selected
            .get(spec.role.as_str())
            .copied()
            .unwrap_or(spec.min);
        if count < spec.min || count > spec.max {
            return Err(CollaborationError::Invalid(format!(
                "memberSlots count for '{}' is outside its definition bounds",
                spec.role
            )));
        }
        for ordinal in 1..=count {
            members.push(TeamMember {
                id: Uuid::new_v4().to_string(),
                role: spec.role.clone(),
                kind: spec.kind.clone(),
                ordinal,
                definition: spec.definition.clone(),
                revision: 1,
                status: "inactive".to_owned(),
            });
        }
    }
    let coordinator = required_text(definition, "coordinatorRole")?;
    if !members.iter().any(|member| member.role == coordinator) {
        return Err(CollaborationError::Invalid(
            "coordinatorRole requires a member slot".to_owned(),
        ));
    }
    let max_members = definition["limits"]["maxMembers"].as_u64().ok_or_else(|| {
        CollaborationError::Storage("installed team member limit is missing".to_owned())
    })?;
    if members.len() as u64 > max_members {
        return Err(CollaborationError::Invalid(
            "memberSlots exceeds team maxMembers".to_owned(),
        ));
    }
    Ok(members)
}

fn exact_team_definition<'a>(
    state: &'a CollaborationCatalogState,
    reference: &ImmutableDefinitionRef,
) -> Result<&'a crate::uar::domain::collaboration::CollaborationDefinitionRecord, CollaborationError>
{
    state
        .definitions
        .get(&reference.storage_key())
        .filter(|record| record.kind == CollaborationKind::TeamDefinition)
        .ok_or_else(|| CollaborationError::NotFound(reference.id.clone()))
}

fn scoped_team(
    state: &CollaborationCatalogState,
    owner_id: &str,
    workspace_id: &str,
    team_id: &str,
) -> Result<TeamInstance, CollaborationError> {
    state
        .team_instances
        .get(&team_key(owner_id, workspace_id, team_id))
        .cloned()
        .ok_or_else(|| CollaborationError::NotFound(team_id.to_owned()))
}

fn validate_scope(owner_id: &str, workspace_id: &str) -> Result<(), CollaborationError> {
    validate_owner(owner_id)?;
    validate_id(workspace_id)?;
    Ok(())
}

fn required_text<'a>(document: &'a Value, field: &str) -> Result<&'a str, CollaborationError> {
    document[field]
        .as_str()
        .ok_or_else(|| CollaborationError::Storage(format!("installed team {field} is missing")))
}

fn team_key(owner_id: &str, workspace_id: &str, team_id: &str) -> String {
    format!("{owner_id}\u{1f}{workspace_id}\u{1f}{team_id}")
}

fn command_key(owner_id: &str, workspace_id: &str, command_id: &str) -> String {
    format!("{owner_id}\u{1f}{workspace_id}\u{1f}{command_id}")
}

fn check_receipt(
    receipt: &TeamPlanningCommandReceipt,
    operation: &str,
    digest: &str,
) -> Result<(), CollaborationError> {
    if receipt.operation != operation || receipt.request_digest != digest {
        return Err(CollaborationError::Conflict(
            "commandId was already used for a different team operation or payload".to_owned(),
        ));
    }
    Ok(())
}
