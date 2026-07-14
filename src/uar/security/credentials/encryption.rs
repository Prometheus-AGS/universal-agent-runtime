//! AES-256-GCM encryption for provider API keys stored at rest.
//!
//! Keys are encrypted with a 32-byte key sourced from the
//! `CREDENTIAL_ENCRYPTION_KEY` environment variable (either 32 raw ASCII bytes
//! or 64 hex characters). The ciphertext is stored as
//! `base64(12-byte-nonce || ciphertext)` so that each encryption produces a
//! unique result even for identical plaintexts.

use aes_gcm::{
    Aes256Gcm, Key, Nonce,
    aead::{Aead, Generate, KeyInit},
};
use anyhow::{Context, Result, bail};
use base64::{Engine, engine::general_purpose::STANDARD as BASE64};

/// Holds the AES-256-GCM cipher derived from `CREDENTIAL_ENCRYPTION_KEY`.
#[derive(Clone)]
pub struct CredentialEncryption {
    cipher: Aes256Gcm,
}

impl std::fmt::Debug for CredentialEncryption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CredentialEncryption")
            .field("cipher", &"<redacted>")
            .finish()
    }
}

impl CredentialEncryption {
    /// Create a new instance directly from a 32-byte key.
    ///
    /// Useful in tests and contexts where the key is already available without
    /// reading an environment variable.
    #[must_use]
    pub fn from_key(key_bytes: &[u8; 32]) -> Self {
        let key = Key::<Aes256Gcm>::from(*key_bytes);
        Self {
            cipher: Aes256Gcm::new(&key),
        }
    }

    /// Create a new instance from the `CREDENTIAL_ENCRYPTION_KEY` env var.
    ///
    /// The env var must be either exactly 32 ASCII bytes or a 64-hex-char
    /// encoding of a 32-byte key. Returns `Ok(None)` if the env var is absent
    /// so the caller can decide whether to hard-fail or degrade gracefully
    /// (single-tenant deployments never set it).
    pub fn from_env() -> Result<Option<Self>> {
        match std::env::var("CREDENTIAL_ENCRYPTION_KEY") {
            Err(_) => Ok(None),
            Ok(raw) => {
                let raw = raw.trim();
                let key_bytes: Vec<u8> =
                    if raw.len() == 64 && raw.chars().all(|c| c.is_ascii_hexdigit()) {
                        (0..64)
                            .step_by(2)
                            .map(|i| u8::from_str_radix(&raw[i..i + 2], 16))
                            .collect::<std::result::Result<_, _>>()
                            .context("CREDENTIAL_ENCRYPTION_KEY hex decode")?
                    } else if raw.len() == 32 {
                        raw.as_bytes().to_vec()
                    } else {
                        bail!(
                            "CREDENTIAL_ENCRYPTION_KEY must be exactly 32 bytes (32 ASCII chars) \
                             or 64 hex characters; got {} chars",
                            raw.len()
                        );
                    };

                let key_bytes: [u8; 32] = key_bytes
                    .try_into()
                    .map_err(|_| anyhow::anyhow!("credential encryption key is not 32 bytes"))?;
                let key = Key::<Aes256Gcm>::from(key_bytes);
                let cipher = Aes256Gcm::new(&key);
                Ok(Some(Self { cipher }))
            }
        }
    }

    /// Encrypt `plaintext` and return `base64(nonce || ciphertext)`.
    pub fn encrypt(&self, plaintext: &str) -> Result<String> {
        let nonce = Nonce::generate();
        let ciphertext = self
            .cipher
            .encrypt(&nonce, plaintext.as_bytes())
            .map_err(|e| anyhow::anyhow!("AES-GCM encrypt error: {e}"))?;

        let mut combined = Vec::with_capacity(nonce.len() + ciphertext.len());
        combined.extend_from_slice(&nonce);
        combined.extend_from_slice(&ciphertext);

        Ok(BASE64.encode(combined))
    }

    /// Decrypt a value produced by [`Self::encrypt`].
    pub fn decrypt(&self, encoded: &str) -> Result<String> {
        let combined = BASE64
            .decode(encoded)
            .context("base64 decode of encrypted credential")?;

        if combined.len() < 12 {
            bail!("encrypted credential too short to contain a nonce");
        }

        let (nonce_bytes, ciphertext) = combined.split_at(12);
        let nonce_bytes: [u8; 12] = nonce_bytes
            .try_into()
            .map_err(|_| anyhow::anyhow!("encrypted credential nonce is not 12 bytes"))?;
        let nonce = Nonce::from(nonce_bytes);

        let plaintext = self
            .cipher
            .decrypt(&nonce, ciphertext)
            .map_err(|e| anyhow::anyhow!("AES-GCM decrypt error: {e}"))?;

        String::from_utf8(plaintext).context("decrypted credential is not valid UTF-8")
    }

    /// Return the last `n` characters of a key for display as a hint.
    #[must_use]
    pub fn key_hint(plaintext: &str, n: usize) -> String {
        let chars: Vec<char> = plaintext.chars().collect();
        let start = chars.len().saturating_sub(n);
        chars[start..].iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_encryption() -> CredentialEncryption {
        CredentialEncryption::from_key(&[b'a'; 32])
    }

    #[test]
    fn round_trip() {
        let enc = make_encryption();
        let original = "sk-proj-abc123XYZ";
        let ciphertext = enc.encrypt(original).expect("encrypt");
        let recovered = enc.decrypt(&ciphertext).expect("decrypt");
        assert_eq!(original, recovered);
    }

    #[test]
    fn same_plaintext_different_ciphertext() {
        let enc = make_encryption();
        let a = enc.encrypt("test_key").expect("encrypt a");
        let b = enc.encrypt("test_key").expect("encrypt b");
        assert_ne!(a, b, "nonces should differ");
    }

    #[test]
    fn wrong_key_fails_cleanly() {
        let enc = CredentialEncryption::from_key(&[b'a'; 32]);
        let other = CredentialEncryption::from_key(&[b'b'; 32]);
        let ciphertext = enc.encrypt("secret").expect("encrypt");
        let result = other.decrypt(&ciphertext);
        assert!(result.is_err(), "decrypt under wrong key must fail");
    }

    #[test]
    fn key_hint_short_key() {
        assert_eq!(CredentialEncryption::key_hint("abc", 4), "abc");
    }

    #[test]
    fn key_hint_normal() {
        assert_eq!(CredentialEncryption::key_hint("sk-abcdefgh1234", 4), "1234");
    }
}
