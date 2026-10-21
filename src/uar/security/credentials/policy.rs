//! Independent host credential policy, present even without encrypted storage.
use std::collections::HashSet;
use serde::{Deserialize, Serialize};
use secrecy::SecretString;
use sha2::{Digest, Sha256};
use crate::uar::security::claims::{CredentialPrincipal, UserContext};
use super::{CredentialOwnerBinding, ProviderService, CredentialScope};

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, schemars::JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CredentialTenancyMode { #[default] SingleOwner, Shared }

#[derive(Clone, Copy, Debug, Deserialize, Serialize, schemars::JsonSchema, Hash, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HouseCredentialSource { System, Config }

#[derive(Clone, Debug, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SharedCredentialFallback {
    pub issuer: String,
    pub tenant_id: String,
    pub provider_id: String,
    pub source: HouseCredentialSource,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct ProviderCredentialPolicyConfig {
    pub mode: CredentialTenancyMode,
    pub installation_id: Option<String>,
    pub shared_fallbacks: Vec<SharedCredentialFallback>,
    pub principal_mapping_file: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum CredentialPolicyError {
    #[error("credential_identity_required")] IdentityRequired,
    #[error("credential_installation_id_required")] InstallationIdRequired,
    #[error("credential_migration_required")] MigrationRequired,
    #[error("credential_source_denied")] SourceDenied,
}

/// Immutable startup policy. Reload creates a new revision; captured runs retain it.
#[derive(Clone, Debug)]
pub struct CredentialTenancyPolicy { config: ProviderCredentialPolicyConfig, revision: String, principal_mappings: Vec<super::PrincipalCredentialOwnerMapping> }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CredentialSource { PrincipalSession, PrincipalAgent, PrincipalUser, System, Config, ExplicitRequest, CredentialFree }

/// Host provenance and credential material captured together before constructing a client.
#[derive(Clone)]
pub struct CredentialSourceDecision {
    owner: Option<CredentialOwnerBinding>, provider_id: String, endpoint: String,
    source: CredentialSource, policy_revision: String, api_key: Option<SecretString>,
}
impl std::fmt::Debug for CredentialSourceDecision {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CredentialSourceDecision").field("source", &self.source).field("policy_revision", &self.policy_revision).finish_non_exhaustive()
    }
}
impl CredentialSourceDecision {
    pub fn owner(&self) -> Option<&CredentialOwnerBinding> { self.owner.as_ref() }
    pub fn provider_id(&self) -> &str { &self.provider_id }
    pub fn endpoint(&self) -> &str { &self.endpoint }
    pub fn source(&self) -> CredentialSource { self.source }
    pub fn policy_revision(&self) -> &str { &self.policy_revision }
    pub fn api_key(&self) -> Option<&SecretString> { self.api_key.as_ref() }
}

pub(super) fn canonical_uuid(value: &str) -> bool {
    uuid::Uuid::parse_str(value).is_ok_and(|id| id.hyphenated().to_string() == value)
}
pub(super) fn metadata_digest(value: &impl Serialize) -> anyhow::Result<String> {
    let digest = Sha256::digest(serde_json::to_vec(value)?);
    Ok(digest.iter().map(|byte| format!("{byte:02x}")).collect())
}
impl Default for CredentialTenancyPolicy {
    fn default() -> Self {
        Self { config: ProviderCredentialPolicyConfig::default(), revision: "single-owner-unprovisioned-v1".to_owned(), principal_mappings: Vec::new() }
    }
}
impl CredentialTenancyPolicy {
    pub fn from_config(config: ProviderCredentialPolicyConfig) -> anyhow::Result<Self> {
        if config.installation_id.as_deref().is_some_and(|value| !canonical_uuid(value)) { anyhow::bail!("invalid credential installation ID"); }
        let mut tuples = HashSet::new();
        for entry in &config.shared_fallbacks {
            if [&entry.issuer, &entry.tenant_id, &entry.provider_id].into_iter().any(|value| value.trim().is_empty() || value.contains('*'))
                || !tuples.insert((&entry.issuer, &entry.tenant_id, &entry.provider_id, entry.source)) {
                anyhow::bail!("invalid or duplicate credential source grant");
            }
        }
        let principal_mappings = if let Some(path) = &config.principal_mapping_file {
            let bytes = std::fs::read(path).map_err(|_| anyhow::anyhow!("cannot read principal credential mappings"))?;
            let mappings: Vec<super::PrincipalCredentialOwnerMapping> = serde_json::from_slice(&bytes).map_err(|_| anyhow::anyhow!("invalid principal credential mappings"))?;
            super::PrincipalCredentialOwnerMapping::validate_all(&mappings)?;
            mappings
        } else { Vec::new() };
        let revision = metadata_digest(&(&config, &principal_mappings))?;
        Ok(Self { config, revision, principal_mappings })
    }
    pub fn principal_mappings(&self) -> &[super::PrincipalCredentialOwnerMapping] { &self.principal_mappings }
    pub fn revision(&self) -> &str { &self.revision }
    pub fn is_shared(&self) -> bool { self.config.mode == CredentialTenancyMode::Shared }
    pub fn installation_id(&self) -> Option<&str> { self.config.installation_id.as_deref() }
    pub fn principal_mapping_file(&self) -> Option<&str> { self.config.principal_mapping_file.as_deref() }
    pub fn bind_owner(&self, context: &UserContext) -> Result<CredentialOwnerBinding, CredentialPolicyError> {
        let principal = CredentialPrincipal::from_verified_context(context).map_err(|_| CredentialPolicyError::IdentityRequired)?;
        self.bind_principal(&principal)
    }
    pub fn bind_principal(&self, principal: &CredentialPrincipal) -> Result<CredentialOwnerBinding, CredentialPolicyError> {
        if self.is_shared() && (principal.issuer_kind() != "issuer" || principal.tenant_id().is_none_or(|v| v.trim().is_empty()) || principal.subject().trim().is_empty()) {
            return Err(CredentialPolicyError::IdentityRequired);
        }
        CredentialOwnerBinding::bind(principal, self)
    }
    pub fn validate_principal(&self, principal: Option<&CredentialPrincipal>) -> Result<Option<CredentialOwnerBinding>, CredentialPolicyError> {
        match principal { Some(principal) => self.bind_principal(principal).map(Some), None if self.is_shared() => Err(CredentialPolicyError::IdentityRequired), None => Ok(None) }
    }
    fn allows(&self, owner: Option<&CredentialOwnerBinding>, provider: &str, source: HouseCredentialSource) -> bool {
        if !self.is_shared() { return true; }
        let Some(owner) = owner else { return false; };
        self.config.shared_fallbacks.iter().any(|grant| grant.issuer == owner.principal().issuer()
            && Some(grant.tenant_id.as_str()) == owner.tenant_id() && grant.provider_id == provider && grant.source == source)
    }
    fn decision(&self, owner: Option<CredentialOwnerBinding>, provider: &str, endpoint: &str, source: CredentialSource, api_key: Option<SecretString>) -> CredentialSourceDecision {
        CredentialSourceDecision { owner, provider_id: provider.to_owned(), endpoint: endpoint.to_owned(), source, policy_revision: self.revision.clone(), api_key }
    }
    /// Called only after the existing exact provider/endpoint request grant has been verified by the host.
    pub(crate) fn explicit_request(&self, principal: Option<&CredentialPrincipal>, provider: &str, endpoint: &str, key: Option<&str>) -> anyhow::Result<CredentialSourceDecision> {
        // An explicit connection does not access stored local credential ownership.
        let owner = self.owner_for_capture(principal, false)?;
        Ok(self.decision(owner, provider, endpoint, CredentialSource::ExplicitRequest, key.map(|v| SecretString::from(v.to_owned()))))
    }
    fn owner_for_capture(&self, principal: Option<&CredentialPrincipal>, uses_store: bool) -> Result<Option<CredentialOwnerBinding>, CredentialPolicyError> {
        if !uses_store && !self.is_shared() && self.installation_id().is_none()
            && principal.is_some_and(|p| matches!(p.issuer_kind(), "local-jwt" | "local-host")) {
            Ok(None)
        } else { self.validate_principal(principal) }
    }
    pub async fn resolve_source(&self, service: Option<&ProviderService>, principal: Option<&CredentialPrincipal>, provider_id: &str, endpoint: &str, session_id: Option<&str>, agent_id: Option<&str>, configured_key: Option<&str>) -> anyhow::Result<CredentialSourceDecision> {
        // Single-owner house-only capture does not access a local credential namespace.
        // Stored lookups and all shared captures still require the full binding.
        let owner = self.owner_for_capture(principal, service.is_some())?;
        if let Some(service) = service {
            if let Some(owner) = &owner {
                if let Some(value) = service.resolver().resolve_with_context(owner, provider_id, session_id, agent_id).await? {
                    let source = match value.scope { CredentialScope::Session => CredentialSource::PrincipalSession, CredentialScope::Agent => CredentialSource::PrincipalAgent, _ => CredentialSource::PrincipalUser };
                    return Ok(self.decision(Some(owner.clone()), provider_id, endpoint, source, Some(value.api_key)));
                }
            }
            if self.allows(owner.as_ref(), provider_id, HouseCredentialSource::System) {
                if let Some(value) = service.resolver().resolve_system(provider_id).await? {
                    return Ok(self.decision(owner, provider_id, endpoint, CredentialSource::System, Some(value.api_key)));
                }
            }
        }
        if let Some(key) = configured_key.filter(|key| !key.is_empty()) {
            if !self.allows(owner.as_ref(), provider_id, HouseCredentialSource::Config) { return Err(CredentialPolicyError::SourceDenied.into()); }
            return Ok(self.decision(owner, provider_id, endpoint, CredentialSource::Config, Some(SecretString::from(key.to_owned()))));
        }
        Ok(self.decision(owner, provider_id, endpoint, CredentialSource::CredentialFree, None))
    }
}
