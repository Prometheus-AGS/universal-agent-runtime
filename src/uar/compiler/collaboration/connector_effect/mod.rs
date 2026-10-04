//! Persist-before-dispatch connector authority. The host owns secret resolution and I/O.
mod adapters;
mod dispatch;
use super::{CollaborationCatalogService, CollaborationError, service::MAX_CAS_ATTEMPTS};
use crate::uar::domain::connector_effect::*;
use chrono::Utc;
use serde_json::to_value;
use uuid::Uuid;

fn key(owner: &str, workspace: &str, id: &str) -> String {
    format!("{owner}\u{1f}{workspace}\u{1f}{id}")
}
fn bad(message: &str) -> CollaborationError {
    CollaborationError::Invalid(message.into())
}
fn conflict(message: &str) -> CollaborationError {
    CollaborationError::Conflict(message.into())
}
fn digest<T: serde::Serialize>(value: &T) -> Result<String, CollaborationError> {
    Ok(super::validation::canonical_digest(&to_value(value)?)?)
}
fn binding<'a>(
    state: &'a crate::uar::domain::collaboration::CollaborationCatalogState,
    owner: &str,
    workspace: &str,
    id: &str,
) -> Result<&'a ConnectorBinding, CollaborationError> {
    state
        .connector_bindings
        .get(&key(owner, workspace, id))
        .ok_or_else(|| CollaborationError::NotFound(id.into()))
}
fn effect<'a>(
    state: &'a crate::uar::domain::collaboration::CollaborationCatalogState,
    owner: &str,
    workspace: &str,
    id: &str,
) -> Result<&'a ConnectorEffect, CollaborationError> {
    state
        .connector_effects
        .get(&key(owner, workspace, id))
        .ok_or_else(|| CollaborationError::NotFound(id.into()))
}
fn validate_scope(owner: &str, workspace: &str) -> Result<(), CollaborationError> {
    super::service::validate_owner(owner)?;
    super::validation::validate_id(workspace)?;
    Ok(())
}
fn validate_credential_ref(value: &str) -> Result<(), CollaborationError> {
    let Some(name) = value.strip_prefix("host://") else {
        return Err(bad("CONNECTOR_HOST_CREDENTIAL_REF_REQUIRED"));
    };
    if name.is_empty()
        || name.len() > 128
        || !name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.'))
    {
        return Err(bad("CONNECTOR_CREDENTIAL_REF_INVALID"));
    }
    Ok(())
}
fn validate_site(
    provider: ConnectorProvider,
    site: Option<&str>,
) -> Result<(), CollaborationError> {
    match (provider, site) {
        (ConnectorProvider::Jira, Some(value)) => {
            let Some(name) = value
                .strip_prefix("https://")
                .and_then(|value| value.strip_suffix(".atlassian.net"))
            else {
                return Err(bad("CONNECTOR_SITE_INVALID"));
            };
            if name.is_empty() || !name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-') {
                return Err(bad("CONNECTOR_SITE_INVALID"));
            }
        }
        (ConnectorProvider::Jira, None) => return Err(bad("CONNECTOR_SITE_REQUIRED")),
        (_, Some(_)) => return Err(bad("CONNECTOR_SITE_UNEXPECTED")),
        (_, None) => {}
    }
    Ok(())
}
impl CollaborationCatalogService {
    pub async fn install_connector_binding(
        &self,
        owner: &str,
        workspace: &str,
        request: ConnectorBindingRequest,
    ) -> Result<ConnectorBinding, CollaborationError> {
        validate_scope(owner, workspace)?;
        super::validation::validate_id(&request.command_id)?;
        super::validation::validate_id(&request.id)?;
        validate_credential_ref(&request.credential_ref)?;
        validate_site(request.provider, request.site.as_deref())?;
        if !adapters::valid_target(request.provider, &request.target)
            || request.allowed_actions.is_empty()
            || request.allowed_egress_labels.is_empty()
            || request
                .allowed_egress_labels
                .iter()
                .any(|label| label.is_empty() || label.len() > 100)
        {
            return Err(bad("CONNECTOR_BINDING_INVALID"));
        }
        let request_digest = digest(&request)?;
        let command = key(owner, workspace, &format!("binding:{}", request.command_id));
        let target = key(owner, workspace, &request.id);
        for _ in 0..MAX_CAS_ATTEMPTS {
            let current = self.load_state().await?;
            if let Some(receipt) = current.connector_commands.get(&command) {
                if receipt.request_digest != request_digest || receipt.effect_id != request.id {
                    return Err(conflict("CONNECTOR_COMMAND_CONFLICT"));
                }
                return Ok(binding(&current, owner, workspace, &request.id)?.clone());
            }
            let previous = current.connector_bindings.get(&target);
            if previous.map(|value| value.revision) != request.expected_revision {
                return Err(conflict("CONNECTOR_BINDING_REVISION_CHANGED"));
            }
            let updated = ConnectorBinding {
                id: request.id.clone(),
                owner_id: owner.into(),
                workspace_id: workspace.into(),
                revision: previous.map_or(1, |value| value.revision + 1),
                provider: request.provider,
                target: request.target.clone(),
                site: request.site.clone(),
                allowed_actions: request.allowed_actions.clone(),
                allowed_egress_labels: request.allowed_egress_labels.clone(),
                credential_ref: request.credential_ref.clone(),
                revoked: request.revoked,
                updated_at: Utc::now(),
            };
            let mut next = current.clone();
            next.generation += 1;
            next.connector_bindings
                .insert(target.clone(), updated.clone());
            next.connector_commands.insert(
                command.clone(),
                ConnectorCommandReceipt {
                    request_digest: request_digest.clone(),
                    effect_id: request.id.clone(),
                },
            );
            if self.cas(current.generation, &next).await? {
                return Ok(updated);
            }
        }
        Err(conflict("CONNECTOR_CONCURRENT_CHANGE"))
    }

    pub async fn list_connector_bindings(
        &self,
        owner: &str,
        workspace: &str,
    ) -> Result<Vec<ConnectorBinding>, CollaborationError> {
        validate_scope(owner, workspace)?;
        Ok(self
            .load_state()
            .await?
            .connector_bindings
            .into_values()
            .filter(|b| b.owner_id == owner && b.workspace_id == workspace)
            .collect())
    }

    pub async fn prepare_connector_effect(
        &self,
        owner: &str,
        workspace: &str,
        request: ConnectorEffectRequest,
    ) -> Result<ConnectorEffectView, CollaborationError> {
        validate_scope(owner, workspace)?;
        super::validation::validate_id(&request.command_id)?;
        super::validation::validate_id(&request.binding_id)?;
        if serde_json::to_vec(&request.payload)?.len() > 64 * 1024 {
            return Err(bad("CONNECTOR_PAYLOAD_TOO_LARGE"));
        }
        let request_digest = digest(&request)?;
        let command = key(owner, workspace, &format!("effect:{}", request.command_id));
        let id = Uuid::new_v4().to_string();
        for _ in 0..MAX_CAS_ATTEMPTS {
            let current = self.load_state().await?;
            if let Some(receipt) = current.connector_commands.get(&command) {
                if receipt.request_digest != request_digest {
                    return Err(conflict("CONNECTOR_COMMAND_CONFLICT"));
                }
                return Ok(effect(&current, owner, workspace, &receipt.effect_id)?.into());
            }
            let grant = binding(&current, owner, workspace, &request.binding_id)?;
            if grant.revoked
                || grant.revision != request.expected_binding_revision
                || !grant.allowed_actions.contains(&request.action)
                || !request
                    .egress_labels
                    .is_subset(&grant.allowed_egress_labels)
                || request.egress_labels.is_empty()
            {
                return Err(conflict("CONNECTOR_AUTHORITY_DENIED"));
            }
            if !matches!(
                request.action,
                ConnectorAction::Read | ConnectorAction::Draft
            ) && request.decision_ref.as_deref().is_none_or(str::is_empty)
            {
                return Err(conflict("CONNECTOR_DECISION_REQUIRED"));
            }
            if !matches!(
                request.action,
                ConnectorAction::Read | ConnectorAction::Draft
            ) {
                let payload_digest = digest(&request.payload)?;
                let workflow_decision = current.workflow_runs.values().any(|run| {
                    run.owner_id == owner
                        && run.workspace_id == workspace
                        && run.decision.as_ref().is_some_and(|decision| {
                            Some(decision.id.as_str()) == request.decision_ref.as_deref()
                                && decision.decision == "accept"
                                && decision.artifact_digest == payload_digest
                        })
                });
                let standing_approval = current.feedback_intakes.values().any(|intake| {
                    intake.owner_id == owner
                        && intake.workspace_id == workspace
                        && intake.duplicate_of.is_none()
                        && request.action == ConnectorAction::Publish
                        && intake.issue_approval.as_ref().is_some_and(|approval| {
                            Some(approval.id.as_str()) == request.decision_ref.as_deref()
                                && approval.sanitized_payload_digest == payload_digest
                                && approval.connector_binding_id.as_deref()
                                    == Some(grant.id.as_str())
                                && approval.egress_label.as_ref().is_some_and(|label| {
                                    request.egress_labels.len() == 1
                                        && request.egress_labels.contains(label)
                                })
                        })
                });
                if !workflow_decision && !standing_approval {
                    return Err(conflict("CONNECTOR_ARTIFACT_DECISION_MISMATCH"));
                }
                if current.connector_effects.values().any(|effect| {
                    effect.owner_id == owner
                        && effect.workspace_id == workspace
                        && effect.decision_ref == request.decision_ref
                }) {
                    return Err(conflict("CONNECTOR_DECISION_ALREADY_USED"));
                }
            }
            adapters::plan(
                grant.provider,
                request.action,
                &grant.target,
                &request.payload,
            )?;
            let now = Utc::now();
            let record = ConnectorEffect {
                id: id.clone(),
                owner_id: owner.into(),
                workspace_id: workspace.into(),
                binding_id: grant.id.clone(),
                binding_revision: grant.revision,
                provider: grant.provider,
                target: grant.target.clone(),
                action: request.action,
                egress_labels: request.egress_labels.clone(),
                decision_ref: request.decision_ref.clone(),
                payload_digest: digest(&request.payload)?,
                payload: request.payload.clone(),
                status: if request.action == ConnectorAction::Draft {
                    "draft"
                } else {
                    "prepared"
                }
                .into(),
                dispatch_id: None,
                receipt: None,
                created_at: now,
                updated_at: now,
            };
            let mut next = current.clone();
            next.generation += 1;
            next.connector_effects
                .insert(key(owner, workspace, &id), record.clone());
            next.connector_commands.insert(
                command.clone(),
                ConnectorCommandReceipt {
                    request_digest: request_digest.clone(),
                    effect_id: id.clone(),
                },
            );
            if self.cas(current.generation, &next).await? {
                return Ok((&record).into());
            }
        }
        Err(conflict("CONNECTOR_CONCURRENT_CHANGE"))
    }

    pub async fn get_connector_effect(
        &self,
        owner: &str,
        workspace: &str,
        id: &str,
    ) -> Result<ConnectorEffectView, CollaborationError> {
        validate_scope(owner, workspace)?;
        Ok(effect(&self.load_state().await?, owner, workspace, id)?.into())
    }

    pub async fn list_connector_effects(
        &self,
        owner: &str,
        workspace: &str,
    ) -> Result<Vec<ConnectorEffectView>, CollaborationError> {
        validate_scope(owner, workspace)?;
        Ok(self
            .load_state()
            .await?
            .connector_effects
            .values()
            .filter(|e| e.owner_id == owner && e.workspace_id == workspace)
            .map(ConnectorEffectView::from)
            .collect())
    }
}
