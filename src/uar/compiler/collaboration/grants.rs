use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::uar::domain::collaboration::{
    COLLABORATION_PROFILE_DRAFT_2, CollaborationCatalogState, CollaborationCommandReceipt,
    CollaborationKind, RepresentationGrant, RepresentationGrantRef, RepresentationGrantStatus,
};

use super::service::{
    CollaborationCatalogService, CollaborationError, MAX_CAS_ATTEMPTS, receipt_key, validate_owner,
};
use super::validation::{request_digest, validate_digest, validate_id};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GrantCommandRequest {
    pub command_id: String,
    #[serde(default)]
    pub expected_revision: Option<u64>,
    pub grant: Value,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GrantInstallResponse {
    pub grant: RepresentationGrant,
    pub receipt: CollaborationCommandReceipt,
}

impl CollaborationCatalogService {
    pub async fn install_representation_grant(
        &self,
        owner_id: &str,
        workspace_id: &str,
        request: GrantCommandRequest,
    ) -> Result<GrantInstallResponse, CollaborationError> {
        self.install_representation_grant_for_principal(owner_id, owner_id, workspace_id, request)
            .await
    }

    /// Keep the authenticated issuer distinct from its tenant-scoped storage key.
    pub(crate) async fn install_representation_grant_for_principal(
        &self,
        owner_id: &str,
        issuer_principal_id: &str,
        workspace_id: &str,
        request: GrantCommandRequest,
    ) -> Result<GrantInstallResponse, CollaborationError> {
        validate_owner(owner_id)?;
        validate_id(workspace_id)?;
        if request.command_id.trim().is_empty() {
            return Err(CollaborationError::Invalid(
                "commandId must not be empty".to_owned(),
            ));
        }
        let grant = parse_grant(&request.grant)?;
        if grant.issuer_principal_id != issuer_principal_id {
            return Err(CollaborationError::Invalid(
                "REPRESENTATION_ISSUER_PRINCIPAL_MISMATCH".into(),
            ));
        }
        let command_digest = request_digest(&request)?;
        let command_key = receipt_key(owner_id, &request.command_id);
        for _ in 0..MAX_CAS_ATTEMPTS {
            let current = self.load_state().await?;
            if let Some(receipt) = current.command_receipts.get(&command_key) {
                if receipt.operation != "install-representation-grant"
                    || receipt.request_digest != command_digest
                {
                    return Err(CollaborationError::Conflict(
                        "commandId was already used for different grant content".to_owned(),
                    ));
                }
                let revision = receipt.binding_revision.ok_or_else(|| {
                    CollaborationError::Storage("grant receipt revision is missing".to_owned())
                })?;
                let stored = current
                    .representation_grant_history
                    .get(&grant_history_key(
                        owner_id,
                        workspace_id,
                        &receipt.resource_id,
                        revision,
                    ))
                    .cloned()
                    .ok_or_else(|| {
                        CollaborationError::Storage("grant history record is missing".to_owned())
                    })?;
                return Ok(GrantInstallResponse {
                    grant: stored,
                    receipt: receipt.clone(),
                });
            }
            let key = grant_key(owner_id, workspace_id, &grant.grant_id);
            let actual = current
                .representation_grants
                .get(&key)
                .map(|item| item.revision);
            enforce_revision(request.expected_revision, actual, grant.revision)?;
            let mut next = current.clone();
            next.generation = current.generation.saturating_add(1);
            next.representation_grants.insert(key, grant.clone());
            next.representation_grant_history.insert(
                grant_history_key(owner_id, workspace_id, &grant.grant_id, grant.revision),
                grant.clone(),
            );
            let receipt = CollaborationCommandReceipt {
                command_id: request.command_id.clone(),
                owner_id: owner_id.to_owned(),
                request_digest: command_digest.clone(),
                operation: "install-representation-grant".to_owned(),
                resource_id: grant.grant_id.clone(),
                catalog_revision: next.catalog_revision,
                binding_revision: Some(grant.revision),
                committed_at: Utc::now(),
            };
            next.command_receipts
                .insert(command_key.clone(), receipt.clone());
            if self.cas(current.generation, &next).await? {
                return Ok(GrantInstallResponse { grant, receipt });
            }
        }
        Err(CollaborationError::Conflict(
            "catalog changed repeatedly while installing the grant".to_owned(),
        ))
    }

    pub async fn list_representation_grants(
        &self,
        owner_id: &str,
        workspace_id: &str,
    ) -> Result<Vec<RepresentationGrant>, CollaborationError> {
        validate_owner(owner_id)?;
        validate_id(workspace_id)?;
        let prefix = format!("{owner_id}\u{1f}{workspace_id}\u{1f}");
        Ok(self
            .load_state()
            .await?
            .representation_grants
            .into_iter()
            .filter_map(|(key, grant)| key.starts_with(&prefix).then_some(grant))
            .collect())
    }

    pub async fn get_representation_grant(
        &self,
        owner_id: &str,
        workspace_id: &str,
        grant_id: &str,
    ) -> Result<RepresentationGrant, CollaborationError> {
        validate_owner(owner_id)?;
        validate_id(workspace_id)?;
        validate_id(grant_id)?;
        self.load_state()
            .await?
            .representation_grants
            .get(&grant_key(owner_id, workspace_id, grant_id))
            .cloned()
            .ok_or_else(|| CollaborationError::NotFound(grant_id.to_owned()))
    }

    pub async fn get_representation_grant_revision(
        &self,
        owner_id: &str,
        workspace_id: &str,
        grant_id: &str,
        revision: u64,
    ) -> Result<RepresentationGrant, CollaborationError> {
        validate_owner(owner_id)?;
        validate_id(workspace_id)?;
        validate_id(grant_id)?;
        self.load_state()
            .await?
            .representation_grant_history
            .get(&grant_history_key(
                owner_id,
                workspace_id,
                grant_id,
                revision,
            ))
            .cloned()
            .ok_or_else(|| CollaborationError::NotFound(format!("{grant_id}@{revision}")))
    }

    pub async fn list_representation_grant_history(
        &self,
        owner_id: &str,
        workspace_id: &str,
        grant_id: &str,
    ) -> Result<Vec<RepresentationGrant>, CollaborationError> {
        validate_owner(owner_id)?;
        validate_id(workspace_id)?;
        validate_id(grant_id)?;
        let prefix = format!("{}\u{1f}", grant_key(owner_id, workspace_id, grant_id));
        let mut history = self
            .load_state()
            .await?
            .representation_grant_history
            .into_iter()
            .filter_map(|(key, grant)| key.starts_with(&prefix).then_some(grant))
            .collect::<Vec<_>>();
        history.sort_by_key(|grant| grant.revision);
        Ok(history)
    }
}

pub(super) fn validate_binding_grants(
    owner_id: &str,
    workspace_id: &str,
    document: &Value,
    state: &CollaborationCatalogState,
) -> Result<Vec<RepresentationGrantRef>, CollaborationError> {
    let runtime_instance_id = document
        .get("runtimeInstanceId")
        .and_then(Value::as_str)
        .ok_or_else(|| CollaborationError::Invalid("runtimeInstanceId is missing".to_owned()))?;
    document
        .get("representationGrantRefs")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .map(|value| {
            let reference: RepresentationGrantRef =
                serde_json::from_value(value.clone()).map_err(|_| {
                    CollaborationError::Invalid("grant reference is invalid".to_owned())
                })?;
            let grant = state
                .representation_grant_history
                .get(&grant_history_key(
                    owner_id,
                    workspace_id,
                    &reference.grant_id,
                    reference.revision,
                ))
                .ok_or_else(|| CollaborationError::NotFound(reference.grant_id.clone()))?;
            let current = state.representation_grants.get(&grant_key(
                owner_id,
                workspace_id,
                &reference.grant_id,
            ));
            if current.map(|item| item.revision) != Some(reference.revision)
                || grant.constraint_digest != reference.constraint_digest
                || grant.grantee_agent_instance_id != runtime_instance_id
                || grant.status != RepresentationGrantStatus::Active
                || grant.revocation.is_some()
                || grant.not_before > Utc::now()
                || grant.expires_at <= Utc::now()
            {
                return Err(CollaborationError::Conflict(format!(
                    "representation grant '{}' is not current and active for this binding",
                    reference.grant_id
                )));
            }
            Ok(reference)
        })
        .collect()
}

fn parse_grant(value: &Value) -> Result<RepresentationGrant, CollaborationError> {
    let grant: RepresentationGrant = serde_json::from_value(value.clone()).map_err(|_| {
        CollaborationError::Invalid("representation grant shape is invalid".to_owned())
    })?;
    if grant.profile != COLLABORATION_PROFILE_DRAFT_2
        || grant.kind != CollaborationKind::RepresentationGrant
        || grant.export_class != "private-authority-state"
    {
        return Err(CollaborationError::Invalid(
            "unsupported representation grant profile, kind, or exportClass".to_owned(),
        ));
    }
    validate_id(&grant.grant_id)?;
    for id in [
        &grant.issuer_principal_id,
        &grant.subject_principal_id,
        &grant.grantee_agent_instance_id,
        &grant.organization_id,
    ] {
        validate_id(id)?;
    }
    validate_digest(&grant.constraint_digest)?;
    if grant.revision == 0
        || grant.not_before >= grant.expires_at
        || grant.office.trim().is_empty()
        || grant.purpose.trim().is_empty()
        || grant.action_scopes.is_empty()
        || grant.resource_scopes.is_empty()
        || !grant.consent_evidence_ref.starts_with("protected-evidence://")
        || !grant
            .organizational_authority_evidence_ref
            .starts_with("protected-evidence://")
    {
        return Err(CollaborationError::Invalid(
            "grant identity, scopes, protected evidence references, revision, or validity interval are invalid"
                .to_owned(),
        ));
    }
    for values in [
        &grant.audience_scopes,
        &grant.action_scopes,
        &grant.resource_scopes,
        &grant.data_scopes,
        &grant.approval_requirements,
        &grant.disclosure_requirements,
        &grant.offboarding.required_actions,
        &grant.restrictions.forbidden_claims,
        &grant.restrictions.notes,
    ] {
        let unique = values.iter().collect::<std::collections::BTreeSet<_>>();
        if values.iter().any(|value| value.trim().is_empty()) || unique.len() != values.len() {
            return Err(CollaborationError::Invalid(
                "grant scope and requirement entries must be non-empty and unique".to_owned(),
            ));
        }
    }
    Ok(grant)
}

fn enforce_revision(
    expected: Option<u64>,
    actual: Option<u64>,
    declared: u64,
) -> Result<(), CollaborationError> {
    let next = actual.unwrap_or(0).saturating_add(1);
    if expected.unwrap_or(0) != actual.unwrap_or(0) || declared != next {
        return Err(CollaborationError::Conflict(format!(
            "grant expectedRevision or declared revision does not match next revision {next}"
        )));
    }
    Ok(())
}

fn grant_key(owner_id: &str, workspace_id: &str, grant_id: &str) -> String {
    format!("{owner_id}\u{1f}{workspace_id}\u{1f}{grant_id}")
}

fn grant_history_key(owner_id: &str, workspace_id: &str, grant_id: &str, revision: u64) -> String {
    format!(
        "{}\u{1f}{revision}",
        grant_key(owner_id, workspace_id, grant_id)
    )
}
