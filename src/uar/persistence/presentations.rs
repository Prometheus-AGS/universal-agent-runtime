//! Shared trusted-host validation for Presentation persistence implementations.

use crate::uar::a2ui::presentations::{Presentation, PresentationDraft};

/// Load only enabled, validated templates in a verified host owner's partition.
/// Unavailable storage produces a closed set and a secret-free policy warning.
pub(crate) async fn eligible_presentations(
    persistence: Option<&std::sync::Arc<dyn super::PersistenceLayer>>,
    owner: Option<&crate::uar::runtime::actor::messages::ActorOwner>,
) -> (std::collections::BTreeSet<String>, Vec<String>) {
    let (records, warnings) = eligible_presentation_records(persistence, owner).await;
    (records.into_keys().collect(), warnings)
}

/// Capture validated contents, not recipes for reloading templates after admission.
pub(crate) async fn eligible_presentation_records(
    persistence: Option<&std::sync::Arc<dyn super::PersistenceLayer>>,
    owner: Option<&crate::uar::runtime::actor::messages::ActorOwner>,
) -> (
    std::collections::BTreeMap<String, Presentation>,
    Vec<String>,
) {
    let Some(owner) = owner else {
        return Default::default();
    };
    let Some(persistence) = persistence else {
        return (
            Default::default(),
            vec!["Presentation storage is unavailable; access is closed".into()],
        );
    };
    let key = owner.presentation_owner_key();
    let records = match persistence.list_presentations(&key).await {
        Ok(records) => records,
        Err(error) => {
            tracing::warn!(%error, "Presentation catalog admission failed");
            return (
                Default::default(),
                vec!["Presentation catalog could not be loaded; access is closed".into()],
            );
        }
    };
    let mut eligible = std::collections::BTreeMap::new();
    let mut warnings = Vec::new();
    for record in records {
        if record.owner_id != key {
            return (
                Default::default(),
                vec!["Presentation catalog owner mismatch; access is closed".into()],
            );
        }
        if !record.content.enabled {
            continue;
        }
        if record.id.trim().is_empty() || record.revision == 0 || record.content.validate().is_err()
        {
            warnings.push(format!(
                "Presentation '{}' is invalid and was excluded",
                record.id
            ));
            continue;
        }
        eligible.insert(record.id.clone(), record);
    }
    (eligible, warnings)
}

/// Portable catalog errors, distinct from database transport failures.
#[derive(Debug, thiserror::Error)]
pub enum PresentationStoreError {
    /// The requested record is not present in the verified owner's partition.
    #[error("Presentation not found")]
    NotFound,
    /// Another writer changed the record after the editor loaded it.
    #[error("Presentation changed; reload it before saving")]
    Conflict,
    /// Content or host-owned record metadata is invalid.
    #[error("Invalid Presentation: {0}")]
    Invalid(String),
}

pub(crate) fn new_record(
    owner_id: &str,
    draft: &PresentationDraft,
) -> Result<Presentation, PresentationStoreError> {
    if owner_id.trim().is_empty() {
        return Err(PresentationStoreError::Invalid(
            "owner must not be blank".into(),
        ));
    }
    draft.validate().map_err(PresentationStoreError::Invalid)?;
    let now = chrono::Utc::now();
    Ok(Presentation {
        id: uuid::Uuid::new_v4().to_string(),
        owner_id: owner_id.to_string(),
        revision: 1,
        content: draft.clone(),
        created_at: now,
        updated_at: now,
    })
}

pub(crate) fn next_record(
    current: &Presentation,
    expected_revision: u64,
    draft: &PresentationDraft,
) -> Result<Presentation, PresentationStoreError> {
    if current.revision != expected_revision {
        return Err(PresentationStoreError::Conflict);
    }
    draft.validate().map_err(PresentationStoreError::Invalid)?;
    // Both durable stores use signed 64-bit integers for concurrency checks.
    let revision = current
        .revision
        .checked_add(1)
        .filter(|revision| *revision <= i64::MAX as u64)
        .ok_or_else(|| PresentationStoreError::Invalid("revision exhausted".into()))?;
    Ok(Presentation {
        revision,
        content: draft.clone(),
        updated_at: chrono::Utc::now(),
        ..current.clone()
    })
}
/// Preserve every unrelated policy field while validating new Presentation intent.
pub(crate) fn global_policy_with_presentations(
    expected: &serde_json::Value,
    selection: &crate::uar::domain::policy::ResourceSelection,
) -> anyhow::Result<serde_json::Value> {
    use crate::uar::domain::policy::{RunPolicy, SelectionMode};
    let _: RunPolicy = serde_json::from_value(expected.clone())?;
    anyhow::ensure!(
        selection.mode == SelectionMode::Selected || selection.ids.is_empty(),
        "Only Selected mode may carry active template IDs"
    );
    let mut next = expected.clone();
    next.as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("Global policy must be an object"))?
        .insert(
            "presentations".to_string(),
            serde_json::to_value(selection)?,
        );
    Ok(next)
}
