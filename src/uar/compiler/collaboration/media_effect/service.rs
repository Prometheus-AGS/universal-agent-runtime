use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

use super::MediaDispatchClaim;
use super::super::{CollaborationCatalogService, CollaborationError, service::MAX_CAS_ATTEMPTS, validation};
use crate::uar::domain::{
    collaboration::CollaborationCatalogState,
    media_effect::{
        ConfirmMediaGrantRequest, DelegateMediaGrantRequest, DispatchMediaEffectRequest,
        MediaAction, MediaAuthorityScope, MediaCommandReceipt, MediaCommandRequest,
        MediaConfirmation, MediaDelegation, MediaEffect, MediaEffectStatus, MediaEffectView,
        MediaGrant, MediaOutcome, MediaOutcomeRequest, MediaPermission, MediaReceipt, MediaScope,
        MediaTarget, PrepareMediaEffectRequest,
    },
};

fn invalid(code: &str) -> CollaborationError {
    CollaborationError::Invalid(code.into())
}

fn conflict(code: &str) -> CollaborationError {
    CollaborationError::Conflict(code.into())
}

fn missing() -> CollaborationError {
    CollaborationError::NotFound("MEDIA_RESOURCE_NOT_FOUND".into())
}

fn digest<T: Serialize>(value: &T) -> Result<String, CollaborationError> {
    validation::request_digest(value).map_err(|_| invalid("MEDIA_REQUEST_INVALID"))
}

fn validate_id(value: &str) -> Result<(), CollaborationError> {
    if value.len() > 128 {
        return Err(invalid("MEDIA_ID_INVALID"));
    }
    validation::validate_id(value).map_err(|_| invalid("MEDIA_ID_INVALID"))
}

fn validate_scope(scope: &MediaScope) -> Result<(), CollaborationError> {
    validate_id(&scope.session_id)?;
    validate_id(&scope.purpose)?;
    match &scope.target {
        MediaTarget::Room(id) | MediaTarget::Recipient(id) => validate_id(id),
    }
}

fn validate_permission(
    scope: &MediaScope,
    permission: &MediaPermission,
) -> Result<(), CollaborationError> {
    validate_id(&permission.capability)?;
    let room_action = matches!(
        permission.action,
        MediaAction::Join | MediaAction::Publish | MediaAction::SendContent | MediaAction::EndMedia
    );
    if room_action != matches!(&scope.target, MediaTarget::Room(_)) {
        return Err(invalid("MEDIA_TARGET_KIND_MISMATCH"));
    }
    Ok(())
}

fn require_future(expiry: DateTime<Utc>, now: DateTime<Utc>) -> Result<(), CollaborationError> {
    if expiry <= now {
        return Err(conflict("MEDIA_AUTHORITY_EXPIRED"));
    }
    Ok(())
}

fn grant<'a>(
    state: &'a CollaborationCatalogState,
    ctx: &MediaAuthorityScope,
    id: &str,
) -> Result<(&'a MediaGrant, &'a MediaConfirmation), CollaborationError> {
    let grant = state.media_grants.get(id).ok_or_else(missing)?;
    let confirmation = state
        .media_confirmations
        .get(&grant.confirmation_id)
        .filter(|confirmation| confirmation.authority == *ctx)
        .ok_or_else(missing)?;
    Ok((grant, confirmation))
}

fn effect<'a>(
    state: &'a CollaborationCatalogState,
    ctx: &MediaAuthorityScope,
    id: &str,
) -> Result<&'a MediaEffect, CollaborationError> {
    state
        .media_effects
        .get(id)
        .filter(|effect| effect.authority == *ctx)
        .ok_or_else(missing)
}

fn effect_view(
    state: &CollaborationCatalogState,
    ctx: &MediaAuthorityScope,
    id: &str,
) -> Result<MediaEffectView, CollaborationError> {
    Ok(MediaEffectView {
        effect: effect(state, ctx, id)?.clone(),
        receipts: state.media_receipts.get(id).cloned().unwrap_or_default(),
    })
}

fn live_grant<'a>(
    state: &'a CollaborationCatalogState,
    ctx: &MediaAuthorityScope,
    id: &str,
    now: DateTime<Utc>,
) -> Result<&'a MediaConfirmation, CollaborationError> {
    let (grant, confirmation) = grant(state, ctx, id)?;
    if grant.revoked_at.is_some() {
        return Err(conflict("MEDIA_GRANT_REVOKED"));
    }
    require_future(confirmation.expires_at, now)?;
    Ok(confirmation)
}

fn authorize_effect<'a>(
    state: &'a CollaborationCatalogState,
    ctx: &MediaAuthorityScope,
    effect: &MediaEffect,
    now: DateTime<Utc>,
) -> Result<&'a MediaConfirmation, CollaborationError> {
    let confirmation = live_grant(state, ctx, &effect.grant_id, now)?;
    if confirmation.scope != effect.scope || !confirmation.permissions.contains(&effect.permission) {
        return Err(conflict("MEDIA_SCOPE_DENIED"));
    }
    match effect.delegation_id.as_deref() {
        None if effect.actor_id == confirmation.coordinator_id => Ok(confirmation),
        Some(id) => {
            let delegation = state.media_delegations.get(id).ok_or_else(missing)?;
            if delegation.grant_id != effect.grant_id
                || delegation.worker_id != effect.actor_id
                || !delegation.permissions.contains(&effect.permission)
            {
                return Err(conflict("MEDIA_DELEGATION_DENIED"));
            }
            require_future(delegation.expires_at, now)?;
            Ok(confirmation)
        }
        None => Err(conflict("MEDIA_ACTOR_DENIED")),
    }
}

fn validate_content(
    effect: &MediaEffect,
    content: Option<&str>,
) -> Result<(), CollaborationError> {
    match (effect.content_digest.as_deref(), content) {
        (None, None) => Ok(()),
        (Some(_), None) => Err(conflict("MEDIA_CONTENT_RESUBMISSION_REQUIRED")),
        (Some(expected), Some(content)) if content.len() <= 64 * 1024 => {
            let actual = validation::digest(content.as_bytes());
            if expected == actual {
                Ok(())
            } else {
                Err(conflict("MEDIA_CONTENT_DIGEST_MISMATCH"))
            }
        }
        _ => Err(invalid("MEDIA_CONTENT_INVALID")),
    }
}

fn transition(
    state: &mut CollaborationCatalogState,
    mut effect: MediaEffect,
    disposition: MediaEffectStatus,
    evidence_ref: Option<String>,
    now: DateTime<Utc>,
) -> MediaEffect {
    effect.status = disposition;
    let receipt = MediaReceipt {
        effect_id: effect.id.clone(),
        actor_id: effect.actor_id.clone(),
        grant_id: effect.grant_id.clone(),
        confirmation_id: effect.confirmation_id.clone(),
        attempt_id: effect.attempt_id.clone(),
        disposition,
        evidence_ref,
        recorded_at: now,
    };
    state.media_receipts.entry(effect.id.clone()).or_default().push(receipt);
    state.media_effects.insert(effect.id.clone(), effect.clone());
    effect
}

impl CollaborationCatalogService {
    async fn update_media<T>(
        &self,
        update: impl Fn(&mut CollaborationCatalogState, DateTime<Utc>) -> Result<(T, bool), CollaborationError>,
    ) -> Result<T, CollaborationError> {
        if self.storage_descriptor().backend == "memory" {
            return Err(CollaborationError::Storage("MEDIA_DURABLE_STORAGE_REQUIRED".into()));
        }
        for _ in 0..MAX_CAS_ATTEMPTS {
            let mut state = self.load_state().await?;
            let previous_generation = state.generation;
            let (result, changed) = update(&mut state, Utc::now())?;
            if !changed {
                return Ok(result);
            }
            state.generation += 1;
            if self.cas(previous_generation, &state).await? {
                return Ok(result);
            }
        }
        Err(conflict("MEDIA_CONCURRENT_CHANGE"))
    }

    async fn media_command<T>(
        &self,
        ctx: &MediaAuthorityScope,
        command_id: &str,
        request: &impl Serialize,
        apply: impl Fn(&mut CollaborationCatalogState, DateTime<Utc>) -> Result<String, CollaborationError>,
        read: impl Fn(&CollaborationCatalogState, &str) -> Result<T, CollaborationError>,
    ) -> Result<T, CollaborationError> {
        validate_id(command_id)?;
        let key = digest(&(ctx, command_id))?;
        let request_digest = digest(request)?;
        self.update_media(|state, now| {
            if let Some(receipt) = state.media_commands.get(&key) {
                if receipt.request_digest != request_digest {
                    return Err(conflict("MEDIA_COMMAND_CONFLICT"));
                }
                return Ok((read(state, &receipt.resource_id)?, false));
            }
            let resource_id = apply(state, now)?;
            let result = read(state, &resource_id)?;
            state.media_commands.insert(key.clone(), MediaCommandReceipt {
                request_digest: request_digest.clone(),
                resource_id,
            });
            Ok((result, true))
        }).await
    }

    pub(crate) async fn confirm_media_grant(
        &self,
        ctx: &MediaAuthorityScope,
        request: ConfirmMediaGrantRequest,
    ) -> Result<MediaGrant, CollaborationError> {
        validate_id(&request.coordinator_id)?;
        validate_scope(&request.scope)?;
        if request.permissions.is_empty() {
            return Err(invalid("MEDIA_PERMISSIONS_REQUIRED"));
        }
        for permission in &request.permissions {
            validate_permission(&request.scope, permission)?;
        }
        let grant_id = Uuid::new_v4().to_string();
        let confirmation_id = Uuid::new_v4().to_string();
        self.media_command(ctx, &request.command_id, &("confirm", &request), |state, now| {
            require_future(request.expires_at, now)?;
            state.media_confirmations.insert(confirmation_id.clone(), MediaConfirmation {
                id: confirmation_id.clone(),
                authority: ctx.clone(),
                coordinator_id: request.coordinator_id.clone(),
                scope: request.scope.clone(),
                permissions: request.permissions.clone(),
                expires_at: request.expires_at,
                confirmed_at: now,
            });
            state.media_grants.insert(grant_id.clone(), MediaGrant {
                id: grant_id.clone(),
                confirmation_id: confirmation_id.clone(),
                revoked_at: None,
            });
            Ok(grant_id.clone())
        }, |state, id| Ok(grant(state, ctx, id)?.0.clone())).await
    }

    pub(crate) async fn revoke_media_grant(
        &self,
        ctx: &MediaAuthorityScope,
        id: &str,
        request: MediaCommandRequest,
    ) -> Result<MediaGrant, CollaborationError> {
        self.media_command(ctx, &request.command_id, &("revoke", id), |state, now| {
            grant(state, ctx, id)?;
            state.media_grants.get_mut(id).ok_or_else(missing)?.revoked_at.get_or_insert(now);
            Ok(id.to_owned())
        }, |state, id| Ok(grant(state, ctx, id)?.0.clone())).await
    }

    pub(crate) async fn delegate_media_grant(
        &self,
        ctx: &MediaAuthorityScope,
        request: DelegateMediaGrantRequest,
    ) -> Result<MediaDelegation, CollaborationError> {
        validate_id(&request.worker_id)?;
        let id = Uuid::new_v4().to_string();
        self.media_command(ctx, &request.command_id, &("delegate", &request), |state, now| {
            let confirmation = live_grant(state, ctx, &request.grant_id, now)?;
            require_future(request.expires_at, now)?;
            if request.worker_id == confirmation.coordinator_id
                || request.expires_at > confirmation.expires_at
                || request.permissions.is_empty()
                || !request.permissions.is_subset(&confirmation.permissions)
            {
                return Err(conflict("MEDIA_DELEGATION_DENIED"));
            }
            state.media_delegations.insert(id.clone(), MediaDelegation {
                id: id.clone(),
                grant_id: request.grant_id.clone(),
                worker_id: request.worker_id.clone(),
                permissions: request.permissions.clone(),
                expires_at: request.expires_at,
            });
            Ok(id.clone())
        }, |state, id| {
            let delegation = state.media_delegations.get(id).ok_or_else(missing)?;
            grant(state, ctx, &delegation.grant_id)?;
            Ok(delegation.clone())
        }).await
    }

    pub(crate) async fn prepare_media_effect(
        &self,
        ctx: &MediaAuthorityScope,
        request: PrepareMediaEffectRequest,
    ) -> Result<MediaEffect, CollaborationError> {
        validate_id(&request.actor_id)?;
        validate_scope(&request.scope)?;
        validate_permission(&request.scope, &request.permission)?;
        match (request.permission.action, request.content_digest.as_deref()) {
            (MediaAction::SendContent, Some(value)) => {
                validation::validate_digest(value)
                    .map_err(|_| invalid("MEDIA_CONTENT_DIGEST_INVALID"))?;
            }
            (MediaAction::SendContent, None) | (_, Some(_)) => {
                return Err(invalid("MEDIA_CONTENT_DIGEST_INVALID"));
            }
            (_, None) => {}
        }
        let id = Uuid::new_v4().to_string();
        self.media_command(ctx, &request.command_id, &("prepare", &request), |state, now| {
            let (grant, _) = grant(state, ctx, &request.grant_id)?;
            let candidate = MediaEffect {
                id: id.clone(),
                authority: ctx.clone(),
                actor_id: request.actor_id.clone(),
                grant_id: request.grant_id.clone(),
                confirmation_id: grant.confirmation_id.clone(),
                delegation_id: request.delegation_id.clone(),
                scope: request.scope.clone(),
                permission: request.permission.clone(),
                content_digest: request.content_digest.clone(),
                status: MediaEffectStatus::Prepared,
                attempt_id: None,
                created_at: now,
            };
            authorize_effect(state, ctx, &candidate, now)?;
            state.media_effects.insert(id.clone(), candidate);
            Ok(id.clone())
        }, |state, id| Ok(effect(state, ctx, id)?.clone())).await
    }

    pub(crate) async fn get_media_effect(
        &self,
        ctx: &MediaAuthorityScope,
        id: &str,
    ) -> Result<MediaEffectView, CollaborationError> {
        effect_view(&self.load_state().await?, ctx, id)
    }

    pub(crate) async fn claim_media_effect(
        &self,
        ctx: &MediaAuthorityScope,
        id: &str,
        request: DispatchMediaEffectRequest,
    ) -> Result<MediaDispatchClaim, CollaborationError> {
        let attempt_id = Uuid::new_v4().to_string();
        let claimed = self.update_media(|state, now| {
            let selected = effect(state, ctx, id)?;
            if selected.status != MediaEffectStatus::Prepared {
                return Err(conflict("MEDIA_EFFECT_NOT_DISPATCHABLE"));
            }
            authorize_effect(state, ctx, selected, now)?;
            validate_content(selected, request.content.as_deref())?;
            let mut selected = selected.clone();
            selected.attempt_id = Some(attempt_id.clone());
            // After the claim commits, UAR cannot know whether the peer received it.
            // Even a lost claim response must never create a second attempt.
            let claimed = transition(state, selected, MediaEffectStatus::Unknown, None, now);
            Ok((claimed, true))
        }).await?;
        Ok(MediaDispatchClaim { effect: claimed, content: request.content })
    }

    pub(crate) async fn cancel_media_effect(
        &self,
        ctx: &MediaAuthorityScope,
        id: &str,
        request: MediaCommandRequest,
    ) -> Result<MediaEffectView, CollaborationError> {
        self.media_command(ctx, &request.command_id, &("cancel", id), |state, now| {
            let selected = effect(state, ctx, id)?;
            match selected.status {
                MediaEffectStatus::Prepared => {
                    transition(state, selected.clone(), MediaEffectStatus::Cancelled, None, now);
                }
                MediaEffectStatus::Cancelled => {}
                _ => return Err(conflict("MEDIA_EFFECT_ALREADY_CLAIMED")),
            }
            Ok(id.to_owned())
        }, |state, id| effect_view(state, ctx, id)).await
    }

    pub(crate) async fn record_media_outcome(
        &self,
        ctx: &MediaAuthorityScope,
        id: &str,
        request: MediaOutcomeRequest,
    ) -> Result<MediaEffectView, CollaborationError> {
        self.settle_media_effect(ctx, id, request, false).await
    }

    pub(crate) async fn reconcile_media_effect(
        &self,
        ctx: &MediaAuthorityScope,
        id: &str,
        request: MediaOutcomeRequest,
    ) -> Result<MediaEffectView, CollaborationError> {
        self.settle_media_effect(ctx, id, request, true).await
    }

    async fn settle_media_effect(
        &self,
        ctx: &MediaAuthorityScope,
        id: &str,
        request: MediaOutcomeRequest,
        reconcile: bool,
    ) -> Result<MediaEffectView, CollaborationError> {
        if reconcile && request.evidence_ref.is_none() {
            return Err(invalid("MEDIA_RECONCILIATION_EVIDENCE_REQUIRED"));
        }
        if request.evidence_ref.as_ref().is_some_and(|reference| Uuid::parse_str(reference).is_err()) {
            return Err(invalid("MEDIA_EVIDENCE_REFERENCE_INVALID"));
        }
        self.media_command(ctx, &request.command_id, &("settle", reconcile, id, &request), |state, now| {
            let selected = effect(state, ctx, id)?;
            if selected.status != MediaEffectStatus::Unknown
                || selected.attempt_id.as_deref() != Some(request.attempt_id.as_str())
            {
                return Err(conflict("MEDIA_ATTEMPT_NOT_UNRESOLVED"));
            }
            let disposition = match request.outcome {
                MediaOutcome::Applied => MediaEffectStatus::Applied,
                MediaOutcome::NotApplied => MediaEffectStatus::NotApplied,
            };
            transition(state, selected.clone(), disposition, request.evidence_ref.clone(), now);
            Ok(id.to_owned())
        }, |state, id| effect_view(state, ctx, id)).await
    }
}
