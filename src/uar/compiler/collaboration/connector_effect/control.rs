use super::*;

impl CollaborationCatalogService {
    /// Cancel only an intent that has never been handed to the host.
    pub async fn cancel_connector_effect(
        &self,
        owner: &str,
        workspace: &str,
        id: &str,
        request: CancelConnectorEffectRequest,
    ) -> Result<ConnectorEffectView, CollaborationError> {
        validate_scope(owner, workspace)?;
        super::super::validation::validate_id(&request.command_id)?;
        let request_digest = digest(&(id, &request))?;
        let command = key(owner, workspace, &format!("cancel:{}", request.command_id));
        for _ in 0..MAX_CAS_ATTEMPTS {
            let current = self.load_state().await?;
            if let Some(prior) = current.connector_commands.get(&command) {
                if prior.request_digest != request_digest || prior.effect_id != id {
                    return Err(conflict("CONNECTOR_COMMAND_CONFLICT"));
                }
                return Ok(effect(&current, owner, workspace, id)?.into());
            }
            let selected = effect(&current, owner, workspace, id)?;
            if !matches!(selected.status.as_str(), "prepared" | "draft")
                || selected.dispatch_id.is_some()
            {
                return Err(conflict("CONNECTOR_EFFECT_ALREADY_DISPATCHED"));
            }
            let mut next = current.clone();
            let updated = next
                .connector_effects
                .get_mut(&key(owner, workspace, id))
                .ok_or_else(|| conflict("CONNECTOR_EFFECT_UNAVAILABLE"))?;
            updated.status = "cancelled".into();
            updated.updated_at = Utc::now();
            let result = ConnectorEffectView::from(&*updated);
            next.generation += 1;
            next.connector_commands.insert(
                command.clone(),
                ConnectorCommandReceipt {
                    request_digest: request_digest.clone(),
                    effect_id: id.into(),
                },
            );
            if self.cas(current.generation, &next).await? {
                return Ok(result);
            }
        }
        Err(conflict("CONNECTOR_CONCURRENT_CHANGE"))
    }
}
