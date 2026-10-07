//! Admin-only installed metadata for portable authoring and private binding.

use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Extension, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
};
use serde::Serialize;
use serde_json::json;

use crate::uar::{
    domain::{collaboration::SkillRef, skills::Skill},
    runtime::skills::SkillService,
    security::claims::UserContext,
};

#[derive(Debug)]
pub struct SkillDeploymentCatalogState {
    pub service: Arc<SkillService>,
    pub admin_key: Option<secrecy::SecretString>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillDeploymentCatalogResponse {
    pub schema_version: u32,
    pub entries: Vec<SkillDeploymentCatalogEntry>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillDeploymentCatalogEntry {
    pub skill_id: String,
    pub title: String,
    pub description: String,
    pub enabled: bool,
    pub tombstoned: bool,
    pub availability: SkillDeploymentAvailability,
    pub reasons: Vec<SkillDeploymentUnavailableReason>,
    pub skill_ref: Option<SkillRef>,
    pub private_binding: Option<SkillInstallationBinding>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SkillDeploymentAvailability {
    Available,
    Unavailable,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SkillDeploymentUnavailableReason {
    MissingSkillId,
    MissingVersion,
    MissingArtifactDigest,
    MissingInstalledLocation,
    Disabled,
    Tombstoned,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillInstallationBinding {
    pub installed_location: String,
}

pub fn build_router() -> Router<Arc<SkillDeploymentCatalogState>> {
    Router::new().route("/deployment-catalog", get(get_deployment_catalog))
}

async fn get_deployment_catalog(
    State(state): State<Arc<SkillDeploymentCatalogState>>,
    Extension(user): Extension<UserContext>,
    headers: HeaderMap,
) -> Response {
    if super::super::user_settings::principal_storage_key(&user).is_none() {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "authenticated_owner_required"})),
        )
            .into_response();
    }
    let supplied = headers
        .get("x-uar-admin-key")
        .and_then(|value| value.to_str().ok());
    if !crate::config::secret_value_matches(&state.admin_key, supplied) {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({"error": "admin_key_required"})),
        )
            .into_response();
    }
    let entries = state
        .service
        .registry()
        .read()
        .await
        .list_for_deployment()
        .into_iter()
        .map(SkillDeploymentCatalogEntry::from)
        .collect();
    Json(SkillDeploymentCatalogResponse {
        schema_version: 1,
        entries,
    })
    .into_response()
}

impl From<Skill> for SkillDeploymentCatalogEntry {
    fn from(skill: Skill) -> Self {
        use SkillDeploymentUnavailableReason as Reason;

        let mut reasons = Vec::new();
        if skill.skill_id.trim().is_empty() {
            reasons.push(Reason::MissingSkillId);
        }
        if skill.version.trim().is_empty() {
            reasons.push(Reason::MissingVersion);
        }
        let digest = skill
            .artifact_digest
            .filter(|value| !value.trim().is_empty());
        if digest.is_none() {
            reasons.push(Reason::MissingArtifactDigest);
        }
        // required/config are explicit authoring defaults, not stored settings.
        let skill_ref = if reasons.is_empty() {
            digest.map(|digest| SkillRef {
                id: skill.skill_id.clone(),
                version: skill.version,
                digest,
                required: true,
                config: json!({}),
                entrypoint: skill.entrypoint,
                required_tools: skill.required_tools,
            })
        } else {
            None
        };
        let private_binding = skill
            .installed_location
            .filter(|value| !value.trim().is_empty())
            .map(|installed_location| SkillInstallationBinding { installed_location });
        if private_binding.is_none() {
            reasons.push(Reason::MissingInstalledLocation);
        }
        if !skill.enabled {
            reasons.push(Reason::Disabled);
        }
        if skill.tombstoned {
            reasons.push(Reason::Tombstoned);
        }
        Self {
            skill_id: skill.skill_id,
            title: skill.title,
            description: skill.description,
            enabled: skill.enabled,
            tombstoned: skill.tombstoned,
            availability: if reasons.is_empty() {
                SkillDeploymentAvailability::Available
            } else {
                SkillDeploymentAvailability::Unavailable
            },
            reasons,
            skill_ref,
            private_binding,
        }
    }
}
