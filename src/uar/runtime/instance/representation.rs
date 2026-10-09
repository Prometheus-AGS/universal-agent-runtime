//! Trusted private representation attachment over the existing instance CAS.

use std::sync::Arc;

use crate::uar::{
    domain::collaboration::{RepresentationGrant, RepresentationGrantRef},
    runtime::actor::messages::ActorOwner,
};

use super::{
    AgentInstanceController, AgentInstanceError, AgentInstanceView, InstanceLifecycle,
    controller::append_event,
};

impl AgentInstanceController {
    /// Attach an already committed grant to its exact durable grantee. A failed
    /// attachment leaves the catalog receipt intact so the same command retries.
    pub(crate) async fn attach_representation_grant(
        self: &Arc<Self>,
        owner: &ActorOwner,
        workspace: &str,
        grant: &RepresentationGrant,
    ) -> Result<AgentInstanceView, AgentInstanceError> {
        if grant.issuer_principal_id != owner.user_id() {
            return Err(AgentInstanceError::Invalid("representation issuer changed"));
        }
        let id = grant.grantee_agent_instance_id.as_str();
        let reference = RepresentationGrantRef {
            grant_id: grant.grant_id.clone(),
            revision: grant.revision,
            constraint_digest: grant.constraint_digest.clone(),
        };
        // Never roll back attachment when an old idempotent catalog command is
        // retried after a newer revision has already been saved.
        let current = self.catalog.get_representation_grant(
            &owner.presentation_owner_key(), workspace, &grant.grant_id,
        ).await.map_err(|_| AgentInstanceError::Conflict("representation grant is unavailable"))?;
        if current.revision != reference.revision || current.constraint_digest != reference.constraint_digest {
            return Err(AgentInstanceError::Conflict("representation grant revision changed"));
        }
        let mut attached = false;
        let record = self.update(owner, workspace, id, |next| {
            attached = false;
            if let Some(existing) = next.representation_grants.iter_mut()
                .find(|existing| existing.grant_id == reference.grant_id)
            {
                if existing == &reference { return Ok(false); }
                if existing.revision >= reference.revision {
                    return Err(AgentInstanceError::Conflict("representation grant revision changed"));
                }
                *existing = reference.clone();
            } else {
                next.representation_grants.push(reference.clone());
            }
            next.representation_revision = next.representation_revision.checked_add(1)
                .ok_or(AgentInstanceError::Conflict("representation authority revision exhausted"))?;
            // Fence idle actors now. Running roots retain their epoch for exact
            // settlement, but their authority revision is no longer admitted.
            if next.active_attempt.is_none() && next.lifecycle == InstanceLifecycle::Active {
                next.lifecycle = InstanceLifecycle::Dormant;
                next.epoch = next.epoch.checked_add(1)
                    .ok_or(AgentInstanceError::Conflict("activation epoch exhausted"))?;
            }
            append_event(next, "representation_authority_changed", None, None, None)?;
            attached = true;
            Ok(true)
        }).await?;
        if !attached { return Ok(AgentInstanceView::from(&record)); }
        if let Some(active) = &record.active_attempt {
            self.manager.cancel_run_for_user(owner.user_id(), &active.root_run_id).await;
        }
        self.stop_actor(owner, workspace, id).await?;
        Ok(AgentInstanceView::from(&record))
    }
}
