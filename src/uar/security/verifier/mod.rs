use std::sync::Arc;

mod jwks_cache;
use jwks_cache::{CacheError, JwksCache, cache_for_issuer};

use async_trait::async_trait;
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode_header};

use super::{
    claims::{CredentialKind, HostAuthority, TenantId, UserClaims, VerifiedIdentity},
    jwt::{self, JwtError},
};
use crate::config::SecurityConfig;
use secrecy::ExposeSecret;

/// Proof that a tenant claim was read inside a successful verifier path.
pub(in crate::uar::security) struct VerifiedTenantClaim<'a>(&'a str);

impl<'a> VerifiedTenantClaim<'a> {
    fn new(value: &'a str) -> Self {
        Self(value)
    }

    pub(in crate::uar::security) fn into_value(self) -> &'a str {
        self.0
    }
}

/// Authenticated material accepted by the runtime's verification boundary.
#[allow(dead_code)]
pub(crate) enum Presented {
    Jwks(String),
    /// Reserved for PID P4; SD-JWT VP verification is intentionally not implemented here.
    SdJwtVp,
    /// Reserved for PID P4; DID authentication is intentionally not implemented here.
    DidAuth,
}

/// Identity produced only after the presented material has been authenticated.
pub(crate) struct Principal {
    credential_kind: CredentialKind,
    pub(crate) host_authority: Option<HostAuthority>,
    pub(crate) authority: Option<VerifiedIdentity>,
    pub(crate) subject: String,
    pub(crate) tenant_id: Option<TenantId>,
    pub(crate) claims: UserClaims,
}

#[derive(Clone, serde::Deserialize)]
struct VerifiedJwtClaims {
    #[serde(flatten)]
    user: UserClaims,
    #[serde(default)]
    uar_credential_kind: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum VerificationError {
    #[error("jsonwebtoken process provider conflicts with UAR's RustCrypto selection")]
    ProviderConflict,
    #[error(transparent)]
    Token(#[from] jsonwebtoken::errors::Error),
    #[error("JWT is missing the required kid header")]
    MissingKeyId,
    #[error("JWKS verification requires RS256, but the token declares {0:?}")]
    UnsupportedAlgorithm(Algorithm),
    #[error(transparent)]
    Jwks(#[from] CacheError),
    #[error("{0}")]
    IdentityPolicy(&'static str),
    #[error("verified remote subject and tenant required")]
    MissingRemoteIdentity,
    #[error("the presented authentication method is not implemented")]
    UnsupportedPresentation,
}

impl From<JwtError> for VerificationError {
    fn from(error: JwtError) -> Self {
        match error {
            JwtError::ProviderConflict => Self::ProviderConflict,
            JwtError::Token(error) => Self::Token(error),
        }
    }
}

#[async_trait]
pub(crate) trait TokenVerifier {
    async fn verify(&self, presented: Presented) -> Result<Principal, VerificationError>;
}

pub(crate) struct SharedSecretVerifier<'a> {
    secret: &'a str,
    issuer: Option<&'a str>,
    audience: Option<&'a str>,
    validate_nbf: bool,
}

impl<'a> SharedSecretVerifier<'a> {
    pub(crate) fn new(
        secret: &'a str,
        issuer: Option<&'a str>,
        audience: Option<&'a str>,
        validate_nbf: bool,
    ) -> Self {
        Self {
            secret,
            issuer,
            audience,
            validate_nbf,
        }
    }
}

fn claim_validation(
    algorithm: Algorithm,
    issuer: Option<&str>,
    audience: Option<&str>,
    validate_nbf: bool,
) -> Validation {
    let mut validation = Validation::new(algorithm);
    validation.leeway = 60;
    validation.required_spec_claims.insert("exp".to_owned());
    validation.validate_nbf = validate_nbf;
    validation.validate_aud = audience.is_some();
    if let Some(issuer) = issuer {
        validation.set_issuer(&[issuer]);
        validation.required_spec_claims.insert("iss".to_owned());
    }
    if let Some(audience) = audience {
        validation.set_audience(&[audience]);
        validation.required_spec_claims.insert("aud".to_owned());
    }
    validation
}

#[async_trait]
impl TokenVerifier for SharedSecretVerifier<'_> {
    async fn verify(&self, presented: Presented) -> Result<Principal, VerificationError> {
        let Presented::Jwks(token) = presented else {
            return Err(VerificationError::UnsupportedPresentation);
        };

        let key = DecodingKey::from_secret(self.secret.as_bytes());
        let validation = claim_validation(
            Algorithm::HS256,
            self.issuer,
            self.audience,
            self.validate_nbf,
        );
        let token_data = jwt::decode::<VerifiedJwtClaims>(token, &key, &validation)?;
        let credential_kind = CredentialKind::from_verified_marker(token_data.claims.uar_credential_kind.as_deref());
        let claims = token_data.claims.user;

        Ok(Principal {
            credential_kind,
            host_authority: None,
            authority: None,
            subject: claims.sub.clone(),
            tenant_id: claims
                .tenant_id
                .as_deref()
                .map(|value| TenantId::from_verified_claim(VerifiedTenantClaim::new(value))),
            claims,
        })
    }
}

pub(crate) struct JwksVerifier {
    issuer: Option<String>,
    audience: Option<String>,
    validate_nbf: bool,
    cache: Arc<JwksCache>,
}

impl JwksVerifier {
    pub(crate) async fn new(
        url: &str,
        issuer: Option<&str>,
        audience: Option<&str>,
        validate_nbf: bool,
    ) -> Self {
        Self {
            issuer: issuer.map(str::to_owned),
            audience: audience.map(str::to_owned),
            validate_nbf,
            cache: cache_for_issuer(issuer, url).await,
        }
    }

    fn validation(&self) -> Validation {
        claim_validation(
            Algorithm::RS256,
            self.issuer.as_deref(),
            self.audience.as_deref(),
            self.validate_nbf,
        )
    }

    #[cfg(test)]
    async fn refreshed_at(&self) -> Option<tokio::time::Instant> {
        self.cache.refreshed_at().await
    }
}

#[async_trait]
impl TokenVerifier for JwksVerifier {
    async fn verify(&self, presented: Presented) -> Result<Principal, VerificationError> {
        let Presented::Jwks(token) = presented else {
            return Err(VerificationError::UnsupportedPresentation);
        };

        jwt::ensure_rustcrypto_provider()?;
        let header = decode_header(&token)?;
        if header.alg != Algorithm::RS256 {
            return Err(VerificationError::UnsupportedAlgorithm(header.alg));
        }
        let kid = header.kid.ok_or(VerificationError::MissingKeyId)?;

        let key = self.cache.key(&kid).await?;

        let token_data = jwt::decode::<VerifiedJwtClaims>(token, &key, &self.validation())?;
        let credential_kind = CredentialKind::from_verified_marker(token_data.claims.uar_credential_kind.as_deref());
        let claims = token_data.claims.user;
        Ok(Principal {
            credential_kind,
            host_authority: None,
            authority: None,
            subject: claims.sub.clone(),
            tenant_id: claims
                .tenant_id
                .as_deref()
                .map(|value| TenantId::from_verified_claim(VerifiedTenantClaim::new(value))),
            claims,
        })
    }
}

pub(crate) async fn verify_token(
    config: &SecurityConfig,
    token: &str,
) -> Result<Principal, VerificationError> {
    config.validate_identity_policy().map_err(VerificationError::IdentityPolicy)?;
    let presented = Presented::Jwks(token.to_owned());
    let mut principal = if let Some(jwks_url) = config.jwks_url.as_deref() {
        // One total budget also covers the asynchronous issuer-cache registry.
        let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(5);
        let result = tokio::time::timeout_at(deadline, async {
            JwksVerifier::new(
                jwks_url,
                config.jwt_issuer.as_deref(),
                config.jwt_audience.as_deref(),
                config.jwt_validate_nbf,
            ).await.verify(presented).await
        }).await.map_err(|_| VerificationError::Jwks(CacheError::Timeout))?;
        if tokio::time::Instant::now() >= deadline {
            return Err(VerificationError::Jwks(CacheError::Timeout));
        }
        result
    } else {
        SharedSecretVerifier::new(
            config.jwt_secret.expose_secret(),
            config.jwt_issuer.as_deref(),
            config.jwt_audience.as_deref(),
            config.jwt_validate_nbf,
        )
        .verify(presented)
        .await
    }?;
    if config.is_remote() && (principal.subject.trim().is_empty()
        || principal.subject == "anonymous"
        || principal.tenant_id.as_ref().is_none_or(|tenant| tenant.as_str().trim().is_empty()))
    {
        return Err(VerificationError::MissingRemoteIdentity);
    }
    principal.authority = Some(VerifiedIdentity::new(config.jwt_issuer.as_deref(), &principal.claims, principal.tenant_id.clone(), principal.credential_kind));
    principal.host_authority = principal.authority.as_ref().and_then(|identity| HostAuthority::for_service(identity, config));
    Ok(principal)
}

#[cfg(test)]
pub(crate) mod test_support;

#[cfg(test)]
mod tests;
