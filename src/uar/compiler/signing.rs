//! Key provider abstraction for descriptor signing.
//!
//! Provides a trait-based abstraction with a local-first Ed25519 implementation.
//! Future KMS providers (Vault, AWS KMS, etc.) implement the same trait.

use std::path::PathBuf;

use async_trait::async_trait;
use ed25519_dalek::{Signer, SigningKey, Verifier};
use tracing::info;

use super::error::CompileError;

/// Abstraction for cryptographic signing operations.
///
/// Implementations handle key storage, signing, and verification.
/// The default [`LocalKeyProvider`] stores keys on-disk; future
/// implementations can integrate with external KMS services.
#[async_trait]
pub trait KeyProvider: Send + Sync + std::fmt::Debug {
    /// Sign the given data and return the signature bytes.
    async fn sign(&self, data: &[u8]) -> Result<Vec<u8>, CompileError>;

    /// Verify a signature against the given data.
    async fn verify(&self, data: &[u8], signature: &[u8]) -> Result<bool, CompileError>;

    /// Return the public key bytes (for embedding in descriptors).
    async fn public_key_bytes(&self) -> Result<Vec<u8>, CompileError>;
}

/// Local Ed25519 key provider.
///
/// Keys are auto-generated on first use and stored in the given directory
/// (defaults to `~/.uar/keys/`). This is fully self-contained — no external
/// services required.
#[derive(Debug)]
pub struct LocalKeyProvider {
    signing_key: SigningKey,
}

impl LocalKeyProvider {
    /// Create a new provider, loading or generating keys at the given path.
    ///
    /// If `key_dir` is `None`, defaults to `~/.uar/keys/`.
    pub fn new(key_dir: Option<PathBuf>) -> Result<Self, CompileError> {
        let key_dir = key_dir.unwrap_or_else(|| dirs_or_default().join("keys"));

        let key_path = key_dir.join("compiler_signing_key.bin");

        let signing_key = if key_path.exists() {
            let bytes = std::fs::read(&key_path)
                .map_err(|e| CompileError::Signing(format!("failed to read signing key: {e}")))?;
            let key_bytes: [u8; 32] = bytes.try_into().map_err(|_| {
                CompileError::Signing("invalid signing key length (expected 32 bytes)".into())
            })?;
            SigningKey::from_bytes(&key_bytes)
        } else {
            info!(path = %key_path.display(), "generating new Ed25519 signing key");
            std::fs::create_dir_all(&key_dir).map_err(|e| {
                CompileError::Signing(format!("failed to create key directory: {e}"))
            })?;
            let mut key_bytes = [0u8; 32];
            rand::fill(&mut key_bytes);
            let key = SigningKey::from_bytes(&key_bytes);
            std::fs::write(&key_path, key.to_bytes())
                .map_err(|e| CompileError::Signing(format!("failed to write signing key: {e}")))?;
            // Write the public key alongside for convenience
            let pub_path = key_dir.join("compiler_signing_key.pub");
            let verifying = key.verifying_key();
            let _ = std::fs::write(pub_path, verifying.to_bytes());
            key
        };

        Ok(Self { signing_key })
    }

    /// Create a provider with an ephemeral in-memory key.
    ///
    /// The key is randomly generated and never persisted to disk.
    /// Useful for testing and as a default when no key directory is configured.
    pub fn ephemeral() -> Self {
        Self {
            signing_key: {
                let mut key_bytes = [0u8; 32];
                rand::fill(&mut key_bytes);
                SigningKey::from_bytes(&key_bytes)
            },
        }
    }
}

#[async_trait]
impl KeyProvider for LocalKeyProvider {
    async fn sign(&self, data: &[u8]) -> Result<Vec<u8>, CompileError> {
        let signature = self.signing_key.sign(data);
        Ok(signature.to_bytes().to_vec())
    }

    async fn verify(&self, data: &[u8], signature: &[u8]) -> Result<bool, CompileError> {
        let sig_bytes: [u8; 64] = signature.try_into().map_err(|_| {
            CompileError::Signing("invalid signature length (expected 64 bytes)".into())
        })?;
        let sig = ed25519_dalek::Signature::from_bytes(&sig_bytes);
        let verifying_key = self.signing_key.verifying_key();
        Ok(verifying_key.verify(data, &sig).is_ok())
    }

    async fn public_key_bytes(&self) -> Result<Vec<u8>, CompileError> {
        Ok(self.signing_key.verifying_key().to_bytes().to_vec())
    }
}

/// Best-effort `~/.uar` directory.
fn dirs_or_default() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".uar")
}

/// A small dependency for home directory resolution.
mod dirs {
    use std::path::PathBuf;

    pub fn home_dir() -> Option<PathBuf> {
        std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .ok()
            .map(PathBuf::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sign_verify_round_trip() {
        let provider = LocalKeyProvider::ephemeral();
        let data = b"hello, world!";
        let sig = provider.sign(data).await.unwrap();
        assert!(provider.verify(data, &sig).await.unwrap());
    }

    #[tokio::test]
    async fn test_verify_rejects_tampered_data() {
        let provider = LocalKeyProvider::ephemeral();
        let data = b"original";
        let sig = provider.sign(data).await.unwrap();
        assert!(!provider.verify(b"tampered", &sig).await.unwrap());
    }

    #[tokio::test]
    async fn test_public_key_is_32_bytes() {
        let provider = LocalKeyProvider::ephemeral();
        let pk = provider.public_key_bytes().await.unwrap();
        assert_eq!(pk.len(), 32);
    }
}
