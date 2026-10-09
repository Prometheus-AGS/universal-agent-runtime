//! Authority carried by keys; caller input can only reduce delegated roles.
use super::ApiKeyRecord;
use crate::{
    config::SecurityConfig,
    uar::security::claims::{UserContext, VerifiedIdentity},
};

pub(super) const AUTHORITY_VERSION: u32 = 1;

#[derive(Debug, thiserror::Error)]
pub enum ApiKeyAuthorityError {
    #[error("verified caller identity required")]
    CallerRequired,
    #[error("requested API-key authority is not delegable")]
    DelegationDenied,
    #[error("API key requires reissue with current verified identity metadata")]
    ReissueRequired,
    #[error("API-key exchange is unsupported by the configured verifier")]
    UnsupportedExchange,
    #[error("API-key authority configuration is invalid")]
    InvalidConfiguration,
    #[error("API key lifetime must be a positive representable duration")]
    InvalidLifetime,
}

#[derive(Debug, Clone)]
pub(super) struct KeyPolicy {
    pub(super) issuer: Option<String>,
    pub(super) audience: Option<String>,
    pub(super) remote: bool,
    pub(super) exchange_supported: bool,
    delegable_roles: Vec<String>,
    admin_principals: Vec<crate::config::ApiKeyAdminPrincipal>,
}

impl Default for KeyPolicy {
    fn default() -> Self {
        Self {
            issuer: None,
            audience: None,
            remote: false,
            exchange_supported: true,
            delegable_roles: SecurityConfig::default_api_key_delegable_roles(),
            admin_principals: Vec::new(),
        }
    }
}

impl KeyPolicy {
    pub(super) fn from_config(config: &SecurityConfig) -> Result<Self, ApiKeyAuthorityError> {
        config
            .validate_identity_policy()
            .map_err(|_| ApiKeyAuthorityError::InvalidConfiguration)?;
        Ok(Self {
            issuer: config.jwt_issuer.clone(),
            audience: config.jwt_audience.clone(),
            remote: config.is_remote(),
            exchange_supported: config.jwks_url.is_none(),
            delegable_roles: config.api_key_delegable_roles.clone(),
            admin_principals: config.api_key_admin_principals.clone(),
        })
    }

    pub(super) fn attenuate(
        &self,
        caller: &UserContext,
        requested: Option<Vec<String>>,
    ) -> Result<Vec<String>, ApiKeyAuthorityError> {
        let identity = self.identity(caller)?;
        if caller.user_id.trim().is_empty()
            || caller.user_id == "anonymous"
            || caller.user_id != caller.claims.sub
            || (self.remote
                && caller
                    .tenant_id
                    .as_ref()
                    .is_none_or(|value| value.as_str().trim().is_empty()))
        {
            return Err(ApiKeyAuthorityError::CallerRequired);
        }
        let requested = requested.unwrap_or_else(SecurityConfig::default_api_key_delegable_roles);
        let held = identity.roles();
        if requested.iter().any(|role| {
            matches!(role.as_str(), "host-session" | "admin")
                || !held.contains(role)
                || !self.delegable_roles.contains(role)
        }) {
            return Err(ApiKeyAuthorityError::DelegationDenied);
        }
        Ok(requested)
    }

    pub(super) fn identity<'a>(
        &self,
        caller: &'a UserContext,
    ) -> Result<&'a VerifiedIdentity, ApiKeyAuthorityError> {
        let identity = caller
            .verified_identity()
            .ok_or(ApiKeyAuthorityError::CallerRequired)?;
        if identity.subject().trim().is_empty()
            || identity.subject() == "anonymous"
            || identity.issuer() != self.issuer.as_deref()
            || (self.remote
                && identity
                    .tenant_id()
                    .is_none_or(|value| value.trim().is_empty()))
        {
            return Err(ApiKeyAuthorityError::CallerRequired);
        }
        Ok(identity)
    }

    pub(super) fn can_manage(
        &self,
        caller: &UserContext,
        record: &ApiKeyRecord,
    ) -> Result<bool, ApiKeyAuthorityError> {
        let identity = self.identity(caller)?;
        let same_scope = record.issuer.as_deref() == identity.issuer()
            && record.tenant_id.as_deref() == identity.tenant_id();
        let administrator = identity.is_issuer_credential()
            && self.admin_principals.iter().any(|entry| {
                Some(entry.issuer.as_str()) == identity.issuer()
                    && entry.subject == identity.subject()
                    && Some(entry.tenant_id.as_str()) == identity.tenant_id()
            });
        Ok(same_scope && (record.subject == identity.subject() || administrator))
    }

    pub(super) fn validate_record(
        &self,
        record: &ApiKeyRecord,
    ) -> Result<(), ApiKeyAuthorityError> {
        if record.subject.trim().is_empty()
            || record.subject == "anonymous"
            || record
                .roles
                .iter()
                .any(|role| matches!(role.as_str(), "host-session" | "admin"))
        {
            return Err(ApiKeyAuthorityError::ReissueRequired);
        }
        if self.remote
            && (record.authority_version != AUTHORITY_VERSION
                || record.issuer.as_deref() != self.issuer.as_deref()
                || record
                    .issuer
                    .as_deref()
                    .is_none_or(|value| value.trim().is_empty())
                || record
                    .tenant_id
                    .as_deref()
                    .is_none_or(|value| value.trim().is_empty()))
        {
            return Err(ApiKeyAuthorityError::ReissueRequired);
        }
        // Issuing a JWT for a different configured issuer would promote a stored
        // identity into that issuer. Require reissue instead, including locally.
        if record.issuer != self.issuer {
            return Err(ApiKeyAuthorityError::ReissueRequired);
        }
        Ok(())
    }
}
