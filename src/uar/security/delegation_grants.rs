//! Process-local, host-issued credentials for bounded UAR delegation.
//!
//! The launch credential stays with the host. Grants carry the captured verified
//! principal and never confer host authentication or administrative authority.

use std::{
    collections::HashMap,
    sync::Mutex,
    time::{Duration, Instant},
};

use axum::{extract::Request, http::Method};
use chrono::{DateTime, Utc};
use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::claims::UserContext;

/// Fixed maximum lifetime; a new runtime authority invalidates every old grant.
pub const GRANT_TTL_SECONDS: u64 = 900;
/// Capability of the managed-local transport, not the external JWT service.
pub const DELEGATION_GRANTS_CAPABILITY: &str = "scoped_delegation_grants_v1";

/// Independently selectable rights; no wildcard or administrative operation exists.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DelegationOperation {
    Discovery,
    ModelRead,
    ModelCompletion,
    FullHarnessDelegation,
}

/// A host explicitly names workspaces and operations; identity comes from auth.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IssueGrantRequest {
    pub workspace_ids: Vec<String>,
    pub operations: Vec<DelegationOperation>,
}

/// Returned only to the authenticated issuing host. Never persist or log this DTO.
#[derive(Serialize)]
pub struct IssuedGrant {
    pub id: String,
    pub token: String,
    pub token_type: &'static str,
    pub expires_at: DateTime<Utc>,
    pub expires_in: u64,
    pub principal: String,
    pub workspace_ids: Vec<String>,
    pub operations: Vec<DelegationOperation>,
    pub instance_id: String,
    pub runtime_epoch: String,
}

struct GrantRecord {
    token: Option<SecretString>,
    principal: UserContext,
    workspace_ids: Vec<String>,
    operations: Vec<DelegationOperation>,
    expires: Instant,
    expires_at: DateTime<Utc>,
}

/// Verified request identity established by the outer sidecar guard only.
#[derive(Clone, Debug)]
pub struct DelegationAuthenticated(pub(crate) UserContext, pub(crate) String);

/// Live authority view without its bearer credential. Never accepted from JSON.
pub(crate) struct VerifiedDelegationGrant {
    pub principal: UserContext,
    pub expires_at: DateTime<Utc>,
}

/// Private native-memory authority. It deliberately has no serialization surface.
pub struct DelegationGrantAuthority {
    instance_id: String,
    runtime_epoch: String,
    records: Mutex<HashMap<String, GrantRecord>>,
}

impl std::fmt::Debug for DelegationGrantAuthority {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("DelegationGrantAuthority(***redacted***)")
    }
}

impl DelegationGrantAuthority {
    /// Create an authority for one service instance and full-harness epoch.
    pub fn new(instance_id: String, runtime_epoch: String) -> Self {
        Self {
            instance_id,
            runtime_epoch,
            records: Mutex::new(HashMap::new()),
        }
    }

    /// Issue a fresh 256-bit credential for the captured authenticated principal.
    ///
    /// # Errors
    /// Returns a stable code when identity or explicit scope is invalid.
    pub fn issue(
        &self,
        principal: &UserContext,
        request: IssueGrantRequest,
    ) -> Result<IssuedGrant, &'static str> {
        if principal.user_id.trim().is_empty()
            || principal.user_id.eq_ignore_ascii_case("anonymous")
        {
            return Err("principal_required");
        }
        if request.workspace_ids.is_empty()
            || request.workspace_ids.iter().any(|id| {
                id.trim().is_empty()
                    || id != id.trim()
                    || id == "*"
                    || id.chars().any(char::is_control)
            })
            || request.operations.is_empty()
        {
            return Err("grant_scope_invalid");
        }
        let mut bytes = [0_u8; 32];
        rand::fill(&mut bytes);
        let token = bytes
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        let id = Uuid::new_v4().to_string();
        let expires_at = Utc::now() + chrono::Duration::seconds(GRANT_TTL_SECONDS as i64);
        // Delegate the captured principal without its launch/service host proof.
        let mut delegated_principal = principal.clone();
        delegated_principal.host_authority = None;
        let mut records = self
            .records
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        records.retain(|_, record| record.expires > Instant::now());
        records.insert(
            id.clone(),
            GrantRecord {
                token: Some(SecretString::from(token.clone())),
                principal: delegated_principal,
                workspace_ids: request.workspace_ids.clone(),
                operations: request.operations.clone(),
                expires: Instant::now() + Duration::from_secs(GRANT_TTL_SECONDS),
                expires_at,
            },
        );
        Ok(IssuedGrant {
            id,
            token,
            token_type: "Bearer",
            expires_at,
            expires_in: GRANT_TTL_SECONDS,
            principal: principal.user_id.clone(),
            workspace_ids: request.workspace_ids,
            operations: request.operations,
            instance_id: self.instance_id.clone(),
            runtime_epoch: self.runtime_epoch.clone(),
        })
    }

    /// Revoke a grant; repeated revocation is harmless.
    pub fn revoke(&self, id: &str) {
        self.records
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .remove(id);
    }

    pub(crate) fn full_harness_grant(
        &self,
        id: &str,
        workspace: &str,
        epoch: &str,
    ) -> Option<VerifiedDelegationGrant> {
        if epoch != self.runtime_epoch {
            return None;
        }
        let mut records = self.records.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        records.retain(|_, record| record.expires > Instant::now());
        let record = records.get(id)?;
        if !record.operations.contains(&DelegationOperation::FullHarnessDelegation)
            || !record.workspace_ids.iter().any(|id| id == workspace)
        {
            return None;
        }
        Some(VerifiedDelegationGrant {
            principal: record.principal.clone(),
            expires_at: record.expires_at,
        })
    }

    pub(crate) fn authenticate(
        &self,
        supplied: &str,
        request: &Request,
    ) -> Option<DelegationAuthenticated> {
        let mut records = self
            .records
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        records.retain(|_, record| record.expires > Instant::now());
        records
            .iter()
            .find(|(_, record)| {
                crate::config::secret_value_matches(&record.token, Some(supplied))
                    && record.allows(request)
            })
            .map(|(id, record)| DelegationAuthenticated(record.principal.clone(), id.clone()))
    }
}

impl GrantRecord {
    fn workspace_allowed(&self, request: &Request) -> bool {
        let mut values = request.headers().get_all("x-uar-workspace-id").iter();
        let (Some(value), None) = (values.next(), values.next()) else {
            return false;
        };
        value
            .to_str()
            .ok()
            .is_some_and(|value| self.workspace_ids.iter().any(|id| id == value))
    }

    fn allows(&self, request: &Request) -> bool {
        let method = request.method();
        let path = request.uri().path();
        if self.operations.contains(&DelegationOperation::Discovery)
            && ((method == Method::GET
                && matches!(
                    path,
                    "/api/uar/capabilities"
                        | "/api/openapi.json"
                        | "/api/uar/full-harness/v1/capabilities"
                        | "/readyz"
                        | "/healthz"
                ))
                || (method == Method::POST && path == "/api/uar/compatibility"))
        {
            return true;
        }
        if self.operations.contains(&DelegationOperation::ModelRead)
            && method == Method::GET
            && (matches!(path, "/api/models" | "/v1/models" | "/api/providers" | "/api/uar/providers")
                || single_segment(path, "/v1/models/"))
        {
            return true;
        }
        if self
            .operations
            .contains(&DelegationOperation::ModelCompletion)
            && method == Method::POST
            && path == "/api/chat/completion"
            && self.workspace_allowed(request)
        {
            return true;
        }
        if !self
            .operations
            .contains(&DelegationOperation::FullHarnessDelegation)
            || !self.workspace_allowed(request)
        {
            return false;
        }
        let Some(path) = path.strip_prefix("/api/uar/full-harness/v1/") else {
            return false;
        };
        let segments = path.split('/').collect::<Vec<_>>();
        match segments.as_slice() {
            ["tasks"] => method == Method::POST,
            ["tasks" | "admissions", id] if !id.is_empty() => method == Method::GET,
            ["tasks", id, "stream"] if !id.is_empty() => method == Method::GET,
            ["tasks", id, "tool-approval" | "cancel" | "detach" | "steer"] if !id.is_empty() => {
                method == Method::POST
            }
            _ => false,
        }
    }
}

fn single_segment(path: &str, prefix: &str) -> bool {
    path.strip_prefix(prefix)
        .is_some_and(|suffix| !suffix.is_empty() && !suffix.contains('/'))
}
