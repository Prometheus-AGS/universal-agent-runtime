//! Host-created credential namespace bindings. General principal keys stay unchanged.
use crate::uar::security::claims::CredentialPrincipal;
use super::policy::{CredentialPolicyError, CredentialTenancyPolicy};

/// Verified principal paired with its separately provisioned credential namespace.
/// No request or stored row can deserialize or construct this capability.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CredentialOwnerBinding {
    principal: CredentialPrincipal,
    authority: String,
    key: String,
}
impl CredentialOwnerBinding {
    pub(super) fn bind(principal: &CredentialPrincipal, policy: &CredentialTenancyPolicy) -> Result<Self, CredentialPolicyError> {
        let authority = match principal.issuer_kind() {
            "issuer" => principal.issuer().to_owned(),
            "local-jwt" | "local-host" => {
                let installation = policy.installation_id().ok_or(CredentialPolicyError::InstallationIdRequired)?;
                let kind = if principal.issuer_kind() == "local-jwt" { "jwt" } else { "host" };
                format!("urn:uar:installation:{installation}:{kind}")
            }
            _ => return Err(CredentialPolicyError::IdentityRequired),
        };
        let key = encode_owner(principal.issuer_kind(), &authority, principal.tenant_id(), principal.subject());
        Ok(Self { principal: principal.clone(), authority, key })
    }
    pub fn principal(&self) -> &CredentialPrincipal { &self.principal }
    pub fn storage_key(&self) -> String { self.key.clone() }
    pub fn issuer(&self) -> &str { &self.authority }
    pub fn subject(&self) -> &str { self.principal.subject() }
    pub fn tenant_id(&self) -> Option<&str> { self.principal.tenant_id() }
    pub(super) fn fixed_local_key(&self) -> Option<String> {
        matches!(self.principal.issuer_kind(), "local-jwt" | "local-host").then(|| self.principal.storage_key())
    }
}

pub(super) fn encode_owner(kind: &str, issuer: &str, tenant: Option<&str>, subject: &str) -> String {
    let tenant = tenant.map_or_else(|| "local".to_owned(), |value| format!("t:{}:{value}", value.len()));
    format!("v1:k:{kind}:i:{}:{issuer}:{tenant}:s:{}:{subject}", issuer.len(), subject.len())
}

/// Strictly decode framing without minting an identity capability.
pub(super) fn decode_owner(key: &str) -> Option<(&str, &str, Option<&str>, &str)> {
    fn framed(input: &str) -> Option<(&str, &str)> {
        let (length, rest) = input.split_once(':')?;
        let length: usize = length.parse().ok()?;
        if length.to_string() != input.split_once(':')?.0 { return None; }
        Some((rest.get(..length)?, rest.get(length..)?))
    }
    let (kind, rest) = key.strip_prefix("v1:k:")?.split_once(":i:")?;
    if !matches!(kind, "issuer" | "local-jwt" | "local-host") { return None; }
    let (issuer, rest) = framed(rest)?;
    let rest = rest.strip_prefix(':')?;
    let (tenant, rest) = if let Some(rest) = rest.strip_prefix("local:s:") { (None, rest) }
        else { let (tenant, rest) = framed(rest.strip_prefix("t:")?)?; (Some(tenant), rest.strip_prefix(":s:")?) };
    let (subject, tail) = framed(rest)?;
    if !tail.is_empty() || issuer.trim().is_empty() || subject.trim().is_empty() || tenant.is_some_and(|v| v.trim().is_empty()) { return None; }
    Some((kind, issuer, tenant, subject))
}

pub(super) fn is_fixed_local(key: &str) -> bool {
    matches!(decode_owner(key), Some(("local-jwt", "urn:uar:local:jwt", _, _)) | Some(("local-host", "urn:uar:local:host", _, _)))
}

pub(super) fn is_bound_owner(key: &str) -> bool {
    let Some((kind, issuer, _, _)) = decode_owner(key) else { return false; };
    if kind == "issuer" { return true; }
    let suffix = if kind == "local-jwt" { ":jwt" } else { ":host" };
    issuer.strip_prefix("urn:uar:installation:").and_then(|v| v.strip_suffix(suffix))
        .is_some_and(super::policy::canonical_uuid)
}
