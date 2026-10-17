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
    api_key_id: Option<String>,
    captured_principal: Option<CredentialPrincipal>,
    issuer: Option<String>,
    subject: String,
    tenant_id: Option<TenantId>,
    roles: Option<Vec<String>>,
}

impl VerifiedIdentity {
    pub(in crate::uar::security) fn new(issuer: Option<&str>, claims: &UserClaims, tenant_id: Option<TenantId>, credential_kind: CredentialKind) -> Self {
        Self { credential_kind, api_key_id: None, captured_principal: None, issuer: issuer.map(str::to_owned), subject: claims.sub.clone(), tenant_id, roles: claims.roles.clone() }
    }

    #[cfg(feature = "server")]
    pub(in crate::uar::security) fn for_scoped_delegation(context: &UserContext) -> anyhow::Result<Self> {
        let principal = CredentialPrincipal::from_verified_context(context)?;
        if let Some(identity) = context.verified_identity() {
            return Ok(identity.clone());
        }
        let mut identity = Self::new(None, &context.claims, context.tenant_id.clone(), CredentialKind::Unclassified);
        identity.captured_principal = Some(principal);
        Ok(identity)
    }

    pub(in crate::uar::security) fn with_api_key_id(mut self, id: Option<String>) -> Self {
        self.api_key_id = id;
        self
    }

    /// True only for credentials whose signed provenance identifies an API key.
    pub fn is_api_key_credential(&self) -> bool { self.credential_kind == CredentialKind::ApiKey }

    pub(in crate::uar::security) fn api_key_id(&self) -> Option<&str> { self.api_key_id.as_deref() }

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
    /// Conversation storage uses the entire verified credential identity.
    /// Anonymous compatibility is restricted to the actual anonymous boundary.
    pub fn session_owner_key(&self) -> anyhow::Result<String> {
        if self.user_id == crate::session::ANONYMOUS_SESSION_OWNER
            && self.claims.sub == self.user_id && self.tenant_id.is_none()
            && self.claims.tenant_id.is_none() && self.authority.is_none()
            && self.host_authority.is_none()
        {
            return Ok(crate::session::ANONYMOUS_SESSION_OWNER.to_owned());
        }
        Ok(CredentialPrincipal::from_verified_context(self)?.storage_key())
    }

    pub fn trusted_host_id(&self) -> Option<&str> {
        self.host_authority.as_ref().filter(|proof| proof.matches(self)).map(|proof| proof.host_id.as_str())
    }

    pub fn verified_identity(&self) -> Option<&VerifiedIdentity> {
        self.authority.as_ref().filter(|authority| authority.matches(self))
    }
}

/// Complete credential owner captured at a trusted authentication boundary.
/// This capability is never deserialized from a request or a persisted row.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CredentialPrincipal {
    issuer_kind: &'static str,
    issuer: String,
    tenant: Option<String>,
    subject: String,
}

impl CredentialPrincipal {
    /// Capture verified JWT/API-key identity or an authenticated local launch.
    ///
    /// # Errors
    /// Rejects synthetic, anonymous and inconsistent contexts.
    pub fn from_verified_context(context: &UserContext) -> anyhow::Result<Self> {
        if context.user_id.trim().is_empty() || context.user_id == "anonymous" {
            anyhow::bail!("verified credential principal required");
        }
        if let Some(identity) = context.verified_identity() {
            if let Some(principal) = &identity.captured_principal {
                return Ok(principal.clone());
            }
            let (issuer_kind, issuer) = match identity.issuer() {
                Some(issuer) if !issuer.trim().is_empty() => ("issuer", issuer),
                Some(_) => anyhow::bail!("verified credential issuer required"),
                None => ("local-jwt", "urn:uar:local:jwt"),
            };
            if identity.tenant_id().is_some_and(|tenant| tenant.trim().is_empty()) {
                anyhow::bail!("verified credential tenant required");
            }
            return Ok(Self {
                issuer_kind,
                issuer: issuer.to_owned(),
                tenant: identity.tenant_id().map(str::to_owned),
                subject: identity.subject().to_owned(),
            });
        }
        if context.trusted_host_id().is_some() && context.tenant_id.is_none() {
            return Ok(Self {
                issuer_kind: "local-host",
                issuer: "urn:uar:local:host".to_owned(),
                tenant: None,
                subject: context.user_id.clone(),
            });
        }
        anyhow::bail!("verified credential principal required")
    }

    /// Canonical versioned owner key. Length framing prevents delimiter collisions;
    /// storage must additionally use a namespace disjoint from legacy subjects.
    pub fn storage_key(&self) -> String {
        let tenant = match &self.tenant {
            Some(value) => format!("t:{}:{}", value.len(), value),
            None => "local".to_owned(),
        };
        format!("v1:k:{}:i:{}:{}:{}:s:{}:{}", self.issuer_kind,
            self.issuer.len(), self.issuer, tenant, self.subject.len(), self.subject)
    }

    /// Authentication kind captured at ingress; retained across delegation.
    pub fn issuer_kind(&self) -> &str { self.issuer_kind }

    /// Verified issuer, including the explicit local authentication namespace.
    pub fn issuer(&self) -> &str { &self.issuer }

    /// Verified tenant; absence is the explicit local tenant namespace.
    pub fn tenant_id(&self) -> Option<&str> { self.tenant.as_deref() }

    /// Verified subject.
    pub fn subject(&self) -> &str { &self.subject }

    #[cfg(test)]
    pub(crate) fn for_test(subject: &str) -> Self {
        Self { issuer_kind: "local-jwt", issuer: "urn:uar:local:jwt".to_owned(),
            tenant: None, subject: subject.to_owned() }
    }
}

/// Host-issued delegation for one durable schedule and destination. This is
/// protected catalog state, never an API request or a general identity token.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScheduleOwnerGrant {
    issuer_kind: String,
    issuer: String,
    tenant: Option<String>,
    subject: String,
    owner_key: String,
    schedule_id: String,
    workspace_id: String,
    instance_id: String,
}

/// Separate protected authority for one follow-up rule and conversation target.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct FollowupOwnerGrant(ScheduleOwnerGrant);

impl FollowupOwnerGrant {
    pub(crate) fn capture(owner: &CredentialPrincipal, rule: &str, workspace: &str, instance: &str) -> Self {
        Self(ScheduleOwnerGrant::capture(owner, rule, workspace, instance))
    }
    pub(crate) fn restore(&self, key: &str, rule: &str, workspace: &str, instance: &str) -> anyhow::Result<CredentialPrincipal> {
        self.0.restore(key, rule, workspace, instance)
    }
}

impl ScheduleOwnerGrant {
    pub(crate) fn capture(
        owner: &CredentialPrincipal, schedule_id: &str, workspace: &str, instance: &str,
    ) -> Self {
        Self { issuer_kind: owner.issuer_kind.into(), issuer: owner.issuer.clone(),
            tenant: owner.tenant.clone(), subject: owner.subject.clone(),
            owner_key: owner.storage_key(), schedule_id: schedule_id.into(),
            workspace_id: workspace.into(), instance_id: instance.into() }
    }

    /// Only the schedule host calls this after loading the exact protected
    /// record and checking its claim/revision and current destination binding.
    pub(crate) fn restore(
        &self, owner_key: &str, schedule_id: &str, workspace: &str, instance: &str,
    ) -> anyhow::Result<CredentialPrincipal> {
        anyhow::ensure!(self.owner_key == owner_key && self.schedule_id == schedule_id
            && self.workspace_id == workspace && self.instance_id == instance,
            "schedule delegation scope mismatch");
        let issuer_kind = match self.issuer_kind.as_str() {
            "issuer" => "issuer",
            "local-jwt" if self.issuer == "urn:uar:local:jwt" => "local-jwt",
            "local-host" if self.issuer == "urn:uar:local:host" && self.tenant.is_none() => "local-host",
            _ => anyhow::bail!("schedule delegation issuer is invalid"),
        };
        anyhow::ensure!(!self.issuer.trim().is_empty() && !self.subject.trim().is_empty()
            && self.subject != "anonymous" && self.tenant.as_deref().is_none_or(|t| !t.trim().is_empty()),
            "schedule delegation principal is invalid");
        let principal = CredentialPrincipal { issuer_kind, issuer: self.issuer.clone(),
            tenant: self.tenant.clone(), subject: self.subject.clone() };
        anyhow::ensure!(principal.storage_key() == self.owner_key, "schedule delegation owner mismatch");
        Ok(principal)
    }
}
