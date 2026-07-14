//! HashiCorp Vault KV-v2 configuration source (feature-gated).
//!
//! When the `vault` Cargo feature is enabled, `ConfigManager` runs this resolver
//! after loading the merged configuration. Any string field that starts with
//! `vault://` is replaced by the corresponding secret from Vault.
//!
//! Vault connectivity is configured through environment variables:
//! - `VAULT_ADDR` — Vault server URL (required).
//! - `VAULT_TOKEN` — Vault authentication token (required).
//! - `VAULT_MOUNT` — KV-v2 mount point, defaults to `secret`.
//!
//! URL format:
//! - `vault://mount/path` reads the secret and returns the `value` field.
//! - `vault://mount/path#field` reads the secret and returns the named field.

use crate::config::AppConfig;
use crate::config_manager::ConfigError;
use secrecy::ExposeSecret;
use std::env;

/// Resolve every `vault://...` URL in `config`.
pub async fn resolve(config: &mut AppConfig) -> Result<(), ConfigError> {
    let addr = match env::var("VAULT_ADDR") {
        Ok(a) => a,
        Err(_) => return Ok(()), // Vault is not configured; keep literal values.
    };
    let token = env::var("VAULT_TOKEN").unwrap_or_default();
    let mount = env::var("VAULT_MOUNT").unwrap_or_else(|_| "secret".to_string());

    let client = build_client(&addr, &token)
        .map_err(|e| ConfigError::Vault(format!("failed to build Vault client for {addr}: {e}")))?;

    if let Some(resolved) =
        maybe_resolve(config.security.jwt_secret.expose_secret(), &client, &mount).await?
    {
        config.security.jwt_secret = secrecy::SecretString::new(resolved.into());
    }
    if let Some(ref key) = config.llm.api_key {
        if let Some(resolved) = maybe_resolve(key, &client, &mount).await? {
            config.llm.api_key = Some(resolved);
        }
    }
    if let Some(ref pass) = config.persistence.surreal_pass {
        if let Some(resolved) = maybe_resolve(pass, &client, &mount).await? {
            config.persistence.surreal_pass = Some(resolved);
        }
    }

    Ok(())
}

/// Build a `vaultrs` client from the provided address and token.
fn build_client(addr: &str, token: &str) -> Result<vaultrs::client::VaultClient, VaultError> {
    let settings = vaultrs::client::VaultClientSettingsBuilder::default()
        .address(addr)
        .token(token)
        .build()
        .map_err(|e| VaultError::Client(format!("{e}")))?;
    vaultrs::client::VaultClient::new(settings).map_err(|e| VaultError::Client(format!("{e}")))
}

/// If `value` is a `vault://` URL, resolve it; otherwise return `None`.
async fn maybe_resolve(
    value: &str,
    client: &vaultrs::client::VaultClient,
    default_mount: &str,
) -> Result<Option<String>, ConfigError> {
    let url = match parse_vault_url(value, default_mount) {
        Some(u) => u,
        None => return Ok(None),
    };

    let secret: serde_json::Value = vaultrs::kv2::read(client, &url.mount, &url.path)
        .await
        .map_err(|e| ConfigError::Vault(format!("failed to read {value}: {e}")))?;

    let field = url.field.as_deref().unwrap_or("value");
    let resolved = secret
        .get(field)
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            ConfigError::Vault(format!(
                "Vault secret at {value} does not contain a string field '{field}'"
            ))
        })?
        .to_string();

    Ok(Some(resolved))
}

/// Parsed `vault://` reference.
struct VaultUrl {
    mount: String,
    path: String,
    field: Option<String>,
}

/// Parse a `vault://mount/path#field` URL.
fn parse_vault_url(url: &str, default_mount: &str) -> Option<VaultUrl> {
    let url = url.strip_prefix("vault://")?;
    let (path_part, field) = url
        .split_once('#')
        .map(|(p, f)| (p, Some(f.to_string())))
        .unwrap_or((url, None));
    let mut parts = path_part.splitn(2, '/');
    let mount = parts.next().filter(|s| !s.is_empty())?.to_string();
    let path = parts.next().unwrap_or("").to_string();
    if mount.is_empty() || path.is_empty() {
        return None;
    }
    Some(VaultUrl {
        mount: if mount == "_" {
            default_mount.to_string()
        } else {
            mount
        },
        path,
        field,
    })
}

/// Errors specific to Vault interaction.
#[derive(Debug, thiserror::Error)]
enum VaultError {
    #[error("Vault client error: {0}")]
    Client(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_vault_url_with_mount_and_path() {
        let url = parse_vault_url("vault://secret/uar/jwt_secret", "secret").unwrap();
        assert_eq!(url.mount, "secret");
        assert_eq!(url.path, "uar/jwt_secret");
        assert_eq!(url.field, None);
    }

    #[test]
    fn parse_vault_url_with_field() {
        let url = parse_vault_url("vault://secret/uar/creds#api_key", "secret").unwrap();
        assert_eq!(url.mount, "secret");
        assert_eq!(url.path, "uar/creds");
        assert_eq!(url.field, Some("api_key".to_string()));
    }

    #[test]
    fn parse_non_vault_url_returns_none() {
        assert!(parse_vault_url("https://example.com/secret", "secret").is_none());
        assert!(parse_vault_url("plain text", "secret").is_none());
    }
}
