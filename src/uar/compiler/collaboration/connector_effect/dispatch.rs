use super::*;
use std::collections::BTreeMap;

fn origin(binding: &ConnectorBinding) -> Result<String, CollaborationError> {
    Ok(match binding.provider {
        ConnectorProvider::Github => "https://api.github.com".into(),
        ConnectorProvider::Notion => "https://api.notion.com".into(),
        ConnectorProvider::Slack => "https://slack.com".into(),
        ConnectorProvider::Jira => binding
            .site
            .clone()
            .ok_or_else(|| bad("CONNECTOR_SITE_REQUIRED"))?,
    })
}
fn headers(provider: ConnectorProvider) -> BTreeMap<String, String> {
    let mut result = BTreeMap::new();
    result.insert("Accept".into(), "application/json".into());
    if provider == ConnectorProvider::Notion {
        result.insert("Notion-Version".into(), "2026-03-11".into());
    }
    result
}
impl CollaborationCatalogService {
    /// Host-only handoff. The caller must have authenticated the sidecar launch token and
    /// resolved the exact opaque credential reference before asking for a dispatch lease.
    pub async fn lease_connector_effect(
        &self,
        owner: &str,
        workspace: &str,
        id: &str,
        broker_credential_ref: &str,
    ) -> Result<PreparedConnectorRequest, CollaborationError> {
        validate_scope(owner, workspace)?;
        for _ in 0..MAX_CAS_ATTEMPTS {
            let current = self.load_state().await?;
            let selected = effect(&current, owner, workspace, id)?;
            if selected.status != "prepared" {
                return Err(conflict("CONNECTOR_EFFECT_NOT_DISPATCHABLE"));
            }
            let grant = binding(&current, owner, workspace, &selected.binding_id)?;
            if grant.revoked
                || grant.revision != selected.binding_revision
                || grant.provider != selected.provider
                || grant.target != selected.target
                || grant.credential_ref != broker_credential_ref
                || !grant.allowed_actions.contains(&selected.action)
                || !selected
                    .egress_labels
                    .is_subset(&grant.allowed_egress_labels)
            {
                return Err(conflict("CONNECTOR_AUTHORITY_CHANGED"));
            }
            let plan = adapters::plan(
                selected.provider,
                selected.action,
                &selected.target,
                &selected.payload,
            )?;
            if selected.provider == ConnectorProvider::Github
                && selected.action == ConnectorAction::Publish
            {
                customer_approval(
                    &current,
                    owner,
                    workspace,
                    grant,
                    selected.decision_ref.as_deref(),
                    &selected.payload_digest,
                    &selected.egress_labels,
                )?;
            }
            let dispatch_id = Uuid::new_v4().to_string();
            let mut next = current.clone();
            let updated = next
                .connector_effects
                .get_mut(&key(owner, workspace, id))
                .ok_or_else(|| conflict("CONNECTOR_EFFECT_UNAVAILABLE"))?;
            updated.status = "dispatched".into();
            updated.dispatch_id = Some(dispatch_id.clone());
            updated.updated_at = Utc::now();
            next.generation += 1;
            if self.cas(current.generation, &next).await? {
                return Ok(PreparedConnectorRequest {
                    effect_id: id.into(),
                    dispatch_id,
                    provider: grant.provider,
                    credential_ref: grant.credential_ref.clone(),
                    origin: origin(grant)?,
                    method: plan.method.into(),
                    path: plan.path,
                    body: plan.body,
                    headers: headers(grant.provider),
                });
            }
        }
        Err(conflict("CONNECTOR_CONCURRENT_CHANGE"))
    }

    /// A dispatched write is never leased again, including after a lost response or restart.
    pub async fn record_connector_outcome(
        &self,
        owner: &str,
        workspace: &str,
        id: &str,
        request: ConnectorOutcomeRequest,
    ) -> Result<ConnectorEffectView, CollaborationError> {
        validate_scope(owner, workspace)?;
        super::super::validation::validate_id(&request.command_id)?;
        if !matches!(
            request.disposition.as_str(),
            "confirmed" | "rejected" | "uncertain"
        ) || (request.disposition == "confirmed"
            && request.external_id.as_deref().is_none_or(str::is_empty))
        {
            return Err(bad("CONNECTOR_OUTCOME_INVALID"));
        }
        if request.result.as_ref().is_some_and(|value| {
            serde_json::to_vec(value).is_ok_and(|bytes| bytes.len() > 64 * 1024)
        }) {
            return Err(bad("CONNECTOR_RESULT_TOO_LARGE"));
        }
        let request_digest = digest(&request)?;
        let command = key(owner, workspace, &format!("outcome:{}", request.command_id));
        for _ in 0..MAX_CAS_ATTEMPTS {
            let current = self.load_state().await?;
            if let Some(receipt) = current.connector_commands.get(&command) {
                if receipt.request_digest != request_digest || receipt.effect_id != id {
                    return Err(conflict("CONNECTOR_COMMAND_CONFLICT"));
                }
                return Ok(effect(&current, owner, workspace, id)?.into());
            }
            let selected = effect(&current, owner, workspace, id)?;
            if request.result.is_some() && selected.action != ConnectorAction::Read {
                return Err(bad("CONNECTOR_RESULT_ONLY_FOR_READ"));
            }
            if selected.status != "dispatched"
                || selected.dispatch_id.as_deref() != Some(request.dispatch_id.as_str())
            {
                return Err(conflict("CONNECTOR_DISPATCH_CHANGED"));
            }
            let mut next = current.clone();
            let updated = next
                .connector_effects
                .get_mut(&key(owner, workspace, id))
                .ok_or_else(|| conflict("CONNECTOR_EFFECT_UNAVAILABLE"))?;
            updated.status = request.disposition.clone();
            updated.receipt = Some(ConnectorEffectReceipt {
                disposition: request.disposition.clone(),
                external_id: request.external_id.clone(),
                evidence_ref: request.evidence_ref.clone(),
                result: request.result.clone(),
                recorded_at: Utc::now(),
            });
            updated.updated_at = Utc::now();
            let view = ConnectorEffectView::from(&*updated);
            link_feedback_outcome(&mut next, owner, workspace, id, &request);
            next.generation += 1;
            next.connector_commands.insert(
                command.clone(),
                ConnectorCommandReceipt {
                    request_digest: request_digest.clone(),
                    effect_id: id.into(),
                },
            );
            if self.cas(current.generation, &next).await? {
                return Ok(view);
            }
        }
        Err(conflict("CONNECTOR_CONCURRENT_CHANGE"))
    }

    /// Reconciliation records independently checked evidence; it never sends the effect again.
    pub async fn reconcile_connector_effect(
        &self,
        owner: &str,
        workspace: &str,
        id: &str,
        request: ConnectorOutcomeRequest,
    ) -> Result<ConnectorEffectView, CollaborationError> {
        validate_scope(owner, workspace)?;
        super::super::validation::validate_id(&request.command_id)?;
        if !matches!(request.disposition.as_str(), "confirmed" | "not_applied")
            || request.evidence_ref.as_deref().is_none_or(str::is_empty)
            || (request.disposition == "confirmed"
                && request.external_id.as_deref().is_none_or(str::is_empty))
        {
            return Err(bad("CONNECTOR_RECONCILIATION_EVIDENCE_REQUIRED"));
        }
        if request.result.is_some() {
            return Err(bad("CONNECTOR_RECONCILIATION_RESULT_UNSUPPORTED"));
        }
        let request_digest = digest(&request)?;
        let command = key(
            owner,
            workspace,
            &format!("reconcile:{}", request.command_id),
        );
        for _ in 0..MAX_CAS_ATTEMPTS {
            let current = self.load_state().await?;
            if let Some(receipt) = current.connector_commands.get(&command) {
                if receipt.request_digest != request_digest || receipt.effect_id != id {
                    return Err(conflict("CONNECTOR_COMMAND_CONFLICT"));
                }
                return Ok(effect(&current, owner, workspace, id)?.into());
            }
            let selected = effect(&current, owner, workspace, id)?;
            if !matches!(selected.status.as_str(), "uncertain" | "dispatched")
                || selected.dispatch_id.as_deref() != Some(request.dispatch_id.as_str())
            {
                return Err(conflict("CONNECTOR_NOT_UNCERTAIN"));
            }
            let mut next = current.clone();
            let updated = next
                .connector_effects
                .get_mut(&key(owner, workspace, id))
                .ok_or_else(|| conflict("CONNECTOR_EFFECT_UNAVAILABLE"))?;
            updated.status = request.disposition.clone();
            updated.receipt = Some(ConnectorEffectReceipt {
                disposition: request.disposition.clone(),
                external_id: request.external_id.clone(),
                evidence_ref: request.evidence_ref.clone(),
                result: None,
                recorded_at: Utc::now(),
            });
            updated.updated_at = Utc::now();
            let view = ConnectorEffectView::from(&*updated);
            link_feedback_outcome(&mut next, owner, workspace, id, &request);
            next.generation += 1;
            next.connector_commands.insert(
                command.clone(),
                ConnectorCommandReceipt {
                    request_digest: request_digest.clone(),
                    effect_id: id.into(),
                },
            );
            if self.cas(current.generation, &next).await? {
                return Ok(view);
            }
        }
        Err(conflict("CONNECTOR_CONCURRENT_CHANGE"))
    }
}

fn link_feedback_outcome(
    state: &mut crate::uar::domain::collaboration::CollaborationCatalogState,
    owner: &str,
    workspace: &str,
    effect_id: &str,
    request: &ConnectorOutcomeRequest,
) {
    for intake in state.feedback_intakes.values_mut().filter(|intake| {
        intake.owner_id == owner
            && intake.workspace_id == workspace
            && intake.connector_effect_id.as_deref() == Some(effect_id)
    }) {
        intake.external_issue_id = request.external_id.clone();
        intake.revision += 1;
        intake.updated_at = Utc::now();
    }
}
