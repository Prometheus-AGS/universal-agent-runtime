//! New execution placement derived from an immutable deployment receipt.

use crate::uar::domain::collaboration::EffectiveBindingReceipt;

use super::{CollaborationCatalogService, CollaborationError};

impl CollaborationCatalogService {
    /// This projection is for new admissions only. It neither updates the
    /// deployment revision nor rewrites receipts already captured by a run.
    pub(super) fn admit_deployment_receipt(
        &self,
        stored: &EffectiveBindingReceipt,
    ) -> Result<EffectiveBindingReceipt, CollaborationError> {
        let mut admitted = stored.clone();
        if let Some(authority) = &self.service_instance {
            let binding = stored.service_binding.as_ref().ok_or_else(|| {
                CollaborationError::Conflict(
                    "effective binding receipt has no live service binding".to_owned(),
                )
            })?;
            admitted.service_binding = Some(authority.readmit_deployment(binding).map_err(
                |response| {
                    CollaborationError::Conflict(
                        response.diagnostics.into_iter()
                            .map(|diagnostic| diagnostic.message)
                            .collect::<Vec<_>>().join("; "),
                    )
                },
            )?);
        }
        admitted.content_digest =
            super::validation::canonical_digest(&serde_json::to_value(&admitted)?)?;
        Ok(admitted)
    }

    /// Compare the complete trusted projection, including its digest. A caller
    /// cannot change policy, resources, models, grants or endpoint roles by
    /// supplying an otherwise plausible binding reference.
    pub(super) fn validate_receipt_projection(
        &self,
        stored: &EffectiveBindingReceipt,
        receipt: &EffectiveBindingReceipt,
    ) -> Result<(), CollaborationError> {
        let expected = self.admit_deployment_receipt(stored)?;
        if serde_json::to_value(&expected)? != serde_json::to_value(receipt)? {
            return Err(CollaborationError::Conflict(
                "effective binding receipt differs from its admitted deployment projection"
                    .to_owned(),
            ));
        }
        Ok(())
    }
}
