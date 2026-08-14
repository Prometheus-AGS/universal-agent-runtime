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

    pub fn as_str(&self) -> &str {
        &self.0
    }

    #[cfg(test)]
    pub(crate) fn for_test(value: &str) -> Self {
        Self(value.to_owned())
    }
}

#[derive(Clone, Debug)]
pub struct UserContext {
    pub user_id: String,
    pub tenant_id: Option<TenantId>,
    pub claims: UserClaims,
}
