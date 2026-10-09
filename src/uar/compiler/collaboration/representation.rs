//! Current private grants consumed by execution, not persona-derived authority.
use super::{CollaborationCatalogService, CollaborationError};
use crate::uar::domain::collaboration::{
    RepresentationGrant, RepresentationGrantRef, RepresentationGrantStatus,
};

impl CollaborationCatalogService {
    pub(crate) async fn execution_representation_grants(
        &self,
        owner: &str,
        workspace: &str,
        references: &[RepresentationGrantRef],
    ) -> Result<Vec<RepresentationGrant>, CollaborationError> {
        super::service::validate_owner(owner)?;
        super::validation::validate_id(workspace)?;
        let state = self.load_state().await?;
        let now = chrono::Utc::now();
        references
            .iter()
            .map(|reference| {
                let key = format!("{owner}\u{1f}{workspace}\u{1f}{}", reference.grant_id);
                let grant = state.representation_grants.get(&key).ok_or_else(|| {
                    CollaborationError::Conflict("REPRESENTATION_GRANT_UNAVAILABLE".into())
                })?;
                if grant.revision != reference.revision
                    || grant.constraint_digest != reference.constraint_digest
                    || grant.status != RepresentationGrantStatus::Active
                    || grant.revocation.is_some()
                    || grant.not_before > now
                    || grant.expires_at <= now
                {
                    return Err(CollaborationError::Conflict(
                        "REPRESENTATION_GRANT_NOT_CURRENT".into(),
                    ));
                }
                Ok(grant.clone())
            })
            .collect()
    }
}
