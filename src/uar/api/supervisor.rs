//! Recovery control stays behind authenticated installation/service authority.

use std::sync::Arc;
use axum::{Extension, Json, Router, extract::{Path, State}, http::StatusCode, routing::post};
use serde::Deserialize;
use serde_json::{Value, json};
use tokio_util::sync::CancellationToken;
use crate::uar::{compiler::collaboration::CollaborationCatalogService,
    runtime::supervisor::{RecoveryTarget, SupervisorRecoveryHost}, security::claims::UserContext};

#[derive(Clone)]
pub struct SupervisorApiState {
    pub host: Option<Arc<SupervisorRecoveryHost>>,
    pub catalog: Arc<CollaborationCatalogService>,
    pub cancellation: CancellationToken,
}

impl std::fmt::Debug for SupervisorApiState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SupervisorApiState").finish_non_exhaustive()
    }
}

pub fn router(state: SupervisorApiState) -> Router<crate::AppState> {
    Router::new()
        .route("/api/uar/recovery/supervisor/challenges", post(challenge))
        .route("/api/uar/recovery/supervisor/challenges/{id}/exclude", post(exclude))
        .with_state(Arc::new(state))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ChallengeRequest { runtime_epoch: String, target: RecoveryTarget }

async fn challenge(
    State(state): State<Arc<SupervisorApiState>>,
    Extension(user): Extension<UserContext>,
    Json(request): Json<ChallengeRequest>,
) -> Result<Json<Value>, StatusCode> {
    if user.trusted_host_id().is_none() { return Err(StatusCode::FORBIDDEN); }
    let host = state.host.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let challenge = host.challenge(&request.runtime_epoch, request.target).await
        .map_err(|_| StatusCode::CONFLICT)?;
    Ok(Json(json!({"challenge":challenge})))
}

async fn exclude(
    State(state): State<Arc<SupervisorApiState>>,
    Extension(user): Extension<UserContext>,
    Path(id): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    if user.trusted_host_id().is_none() { return Err(StatusCode::FORBIDDEN); }
    let host = state.host.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let proof = host.exclude(&id, &state.cancellation).await
        .map_err(|_| StatusCode::CONFLICT)?;
    match &proof.challenge().target {
        RecoveryTarget::CheckpointEpoch { .. } => {
            host.commit_checkpoint_exclusion(&proof).await.map_err(|_| StatusCode::CONFLICT)?;
        }
        RecoveryTarget::TeamOwner { .. } => {
            host.commit_team_exclusion(&proof).await.map_err(|_| StatusCode::CONFLICT)?;
            state.catalog.record_supervisor_exclusion(&proof).await.map_err(|_| StatusCode::CONFLICT)?;
        }
    }
    Ok(Json(json!({"challenge_id":id,"exclusion_recorded":true,
        "effects_reconciled":false,"execution_resumed":false})))
}
