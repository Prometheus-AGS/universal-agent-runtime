use serde::{Deserialize, Serialize};

#[cfg(feature = "server")]
use super::verifier::VerifiedTenantClaim;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserClaims {
    pub sub: String, // User ID (Subject)
    pub name: Option<String>,
    pub roles: Option<Vec<String>>,
    #[serde(default)]
    pub tenant_id: Option<String>,
    /// Authenticated UAR issuer identity for governed instance-to-instance A2A.
    /// Ordinary user tokens omit it and cannot present a delegation contract.
    #[serde(default)]
    pub uar_instance_id: Option<String>,
    pub exp: usize, // Expiration time (UNIX timestamp)
}

/// Tenant identity established only after credential verification.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct TenantId(String);

impl TenantId {
    #[cfg(feature = "server")]
    pub(in crate::uar::security) fn from_verified_claim(claim: VerifiedTenantClaim<'_>) -> Self {
        Self(claim.into_value().to_owned())
    }

    pub(in crate::uar::security) fn from_verified_key(claim: super::api_keys::VerifiedKeyTenantClaim<'_>) -> Self {
        Self(claim.value().to_owned())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    #[cfg(test)]
    pub(crate) fn for_test(value: &str) -> Self {
        Self(value.to_owned())
    }
}

/// Signed credential provenance; absence never implies issuer privilege.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::uar::security) enum CredentialKind { Unclassified, Issuer, ApiKey }

impl CredentialKind {
    #[cfg(feature = "server")]
    pub(in crate::uar::security) fn from_verified_marker(marker: Option<&str>) -> Self {
        match marker { Some("issuer") => Self::Issuer, Some("api_key") => Self::ApiKey, _ => Self::Unclassified }
    }
}

/// Identity proof produced inside credential verification, never deserialized.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerifiedIdentity {
    credential_kind: CredentialKind,
    issuer: Option<String>,
    subject: String,
    tenant_id: Option<TenantId>,
    roles: Option<Vec<String>>,
}

impl VerifiedIdentity {
    pub(in crate::uar::security) fn new(issuer: Option<&str>, claims: &UserClaims, tenant_id: Option<TenantId>, credential_kind: CredentialKind) -> Self {
        Self { credential_kind, issuer: issuer.map(str::to_owned), subject: claims.sub.clone(), tenant_id, roles: claims.roles.clone() }
    }

    pub(in crate::uar::security) fn is_issuer_credential(&self) -> bool { self.credential_kind == CredentialKind::Issuer }

    pub fn issuer(&self) -> Option<&str> { self.issuer.as_deref() }
    pub fn subject(&self) -> &str { &self.subject }
    pub fn tenant_id(&self) -> Option<&str> { self.tenant_id.as_ref().map(TenantId::as_str) }
    pub fn roles(&self) -> &[String] { self.roles.as_deref().unwrap_or_default() }

    fn matches(&self, context: &UserContext) -> bool {
        self.subject == context.user_id && self.subject == context.claims.sub
            && self.tenant_id == context.tenant_id
            && self.tenant_id() == context.claims.tenant_id.as_deref()
            && self.roles == context.claims.roles
    }
}

/// Host capability bound to authenticated launch or an exact configured service.
/// Private fields and no deserialization prevent caller-supplied host assertions.
#[derive(Clone, Debug)]
pub struct HostAuthority {
    host_id: String,
    subject: String,
    tenant_id: Option<TenantId>,
    roles: Option<Vec<String>>,
    identity: Option<VerifiedIdentity>,
}

impl HostAuthority {
    #[cfg(feature = "server")]
    pub(in crate::uar::security) fn for_service(identity: &VerifiedIdentity, config: &crate::config::SecurityConfig) -> Option<Self> {
        if !identity.is_issuer_credential() { return None; }
        let entry = config.trusted_host_principals.iter().find(|entry| {
            Some(entry.issuer.as_str()) == identity.issuer()
                && entry.subject == identity.subject()
                && Some(entry.tenant_id.as_str()) == identity.tenant_id()
        })?;
        Some(Self { host_id: entry.host_id.clone(), subject: identity.subject.clone(),
            tenant_id: identity.tenant_id.clone(), roles: identity.roles.clone(), identity: Some(identity.clone()) })
    }

    #[cfg(feature = "server")]
    pub(in crate::uar::security) fn for_launch(_proof: &super::sidecar_guard::HostAuthenticated, context: &UserContext) -> Self {
        Self { host_id: "sidecar-launch-host".to_owned(), subject: context.user_id.clone(),
            tenant_id: None, roles: context.claims.roles.clone(), identity: None }
    }

    fn matches(&self, context: &UserContext) -> bool {
        self.subject == context.user_id && self.subject == context.claims.sub
            && self.tenant_id == context.tenant_id
            && self.tenant_id.as_ref().map(TenantId::as_str) == context.claims.tenant_id.as_deref()
            && self.roles == context.claims.roles
            && match &self.identity {
                Some(identity) => context.verified_identity() == Some(identity),
                None => context.authority.is_none(),
            }
    }
}

#[derive(Clone, Debug)]
pub struct UserContext {
    /// Separate installation/service proof; ordinary identity and roles cannot create it.
    pub host_authority: Option<HostAuthority>,
    /// None for anonymous, synthetic and installation contexts. Not a role claim.
    pub authority: Option<VerifiedIdentity>,
    pub user_id: String,
    pub tenant_id: Option<TenantId>,
    pub claims: UserClaims,
}

impl UserContext {
    pub fn trusted_host_id(&self) -> Option<&str> {
        self.host_authority.as_ref().filter(|proof| proof.matches(self)).map(|proof| proof.host_id.as_str())
    }

    pub fn verified_identity(&self) -> Option<&VerifiedIdentity> {
        self.authority.as_ref().filter(|authority| authority.matches(self))
    }
}
