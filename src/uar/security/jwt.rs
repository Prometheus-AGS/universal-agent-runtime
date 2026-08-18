//! Process-level JWT provider selection and guarded encode/decode operations.

#[cfg(feature = "server")]
use jsonwebtoken::{DecodingKey, TokenData, Validation};
use jsonwebtoken::{EncodingKey, Header, crypto::rust_crypto};
use serde::Serialize;
#[cfg(feature = "server")]
use serde::de::DeserializeOwned;
use std::sync::OnceLock;

static UAR_PROVIDER: OnceLock<Result<(), ()>> = OnceLock::new();

/// Errors returned before or during a guarded JWT operation.
#[derive(Debug, thiserror::Error)]
pub(crate) enum JwtError {
    /// A process-level provider was installed before UAR acquired the slot.
    #[error("jsonwebtoken process provider was initialized before UAR could install RustCrypto")]
    ProviderConflict,
    /// The selected provider ran and rejected the token or key material.
    #[error(transparent)]
    Token(#[from] jsonwebtoken::errors::Error),
}

/// Install UAR's selected provider once and reuse that UAR-owned result.
pub(crate) fn ensure_rustcrypto_provider() -> Result<(), JwtError> {
    match UAR_PROVIDER.get_or_init(|| {
        rust_crypto::DEFAULT_PROVIDER
            .install_default()
            .map_err(|_already_initialized| ())
    }) {
        Ok(()) => Ok(()),
        Err(()) => Err(JwtError::ProviderConflict),
    }
}

/// Encode only after the process provider is known to match UAR's selection.
pub(crate) fn encode<T: Serialize>(
    header: &Header,
    claims: &T,
    key: &EncodingKey,
) -> Result<String, JwtError> {
    ensure_rustcrypto_provider()?;
    Ok(jsonwebtoken::encode(header, claims, key)?)
}

/// Decode only after the process provider is known to match UAR's selection.
#[cfg(feature = "server")]
pub(crate) fn decode<T: DeserializeOwned>(
    token: impl AsRef<[u8]>,
    key: &DecodingKey,
    validation: &Validation,
) -> Result<TokenData<T>, JwtError> {
    ensure_rustcrypto_provider()?;
    Ok(jsonwebtoken::decode(token, key, validation)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uar_owned_rustcrypto_initialization_is_idempotent() {
        ensure_rustcrypto_provider().expect("initial provider installation should succeed");
        ensure_rustcrypto_provider().expect("repeated provider installation should succeed");
    }
}
