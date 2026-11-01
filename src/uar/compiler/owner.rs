//! Verified owner scope for compiler specs, reports and authoring sessions (#387).
//!
//! Authentication alone cannot partition the compiler's stored records. Every
//! stored record carries the storage key of the credential principal that
//! created it, and every read or mutation is filtered through a
//! [`CompilerOwner`] captured at the trusted authentication boundary.

use anyhow::Result;

use crate::uar::security::claims::{CredentialPrincipal, UserContext};

/// Issuer kind of the installation-local `local-host` principal.
const LOCAL_HOST_ISSUER_KIND: &str = "local-host";

/// The verified owner of a compiler request.
///
/// Built only from a [`UserContext`] that carries a verified credential
/// principal; anonymous and synthetic contexts are rejected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilerOwner {
    key: String,
    local_host: bool,
}

impl CompilerOwner {
    /// Capture the owner of a verified request context.
    ///
    /// # Errors
    /// Rejects anonymous, synthetic and inconsistent contexts.
    pub fn from_context(user: &UserContext) -> Result<Self> {
        let principal = CredentialPrincipal::from_verified_context(user)?;
        Ok(Self {
            key: principal.storage_key(),
            local_host: principal.issuer_kind() == LOCAL_HOST_ISSUER_KIND,
        })
    }

    /// Canonical storage key written to the `owner` field of new records.
    #[must_use]
    pub fn key(&self) -> &str {
        &self.key
    }

    /// Whether this owner may also read records written before owner scoping
    /// existed. Only the installation-local `local-host` principal may.
    #[must_use]
    pub fn reads_legacy_records(&self) -> bool {
        self.local_host
    }

    /// Whether a record whose stored owner is `record_owner` belongs to this
    /// owner. An absent or empty owner marks a legacy record.
    #[must_use]
    pub fn owns(&self, record_owner: Option<&str>) -> bool {
        match record_owner.filter(|owner| !owner.is_empty()) {
            Some(owner) => owner == self.key,
            None => self.local_host,
        }
    }

    #[cfg(test)]
    pub(crate) fn for_test(key: &str, local_host: bool) -> Self {
        Self {
            key: key.to_owned(),
            local_host,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn owner_reads_only_its_own_records() {
        let owner = CompilerOwner::for_test("owner-a", false);
        assert!(owner.owns(Some("owner-a")));
        assert!(!owner.owns(Some("owner-b")));
    }

    #[test]
    fn legacy_records_are_readable_only_by_the_local_host_owner() {
        let remote = CompilerOwner::for_test("owner-a", false);
        let local = CompilerOwner::for_test("local", true);
        for legacy in [None, Some("")] {
            assert!(!remote.owns(legacy));
            assert!(local.owns(legacy));
        }
        assert!(!local.owns(Some("owner-a")));
    }
}
