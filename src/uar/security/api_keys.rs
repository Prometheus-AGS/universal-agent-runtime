//! API Key management — Personal Access Token (PAT) pattern.
//!
//! API keys are long-lived credentials that can be exchanged for short-lived JWTs.
//! This allows agents and external systems to authenticate without managing JWT
//! expiry themselves.
//!
//! ## Flow
//! 1. User creates an API key via `POST /api/uar/auth/keys` (requires JWT)
//! 2. Raw key is shown **once** — caller must store it securely
//! 3. Caller sends `POST /api/uar/auth/exchange` with the raw key → receives a JWT
//! 4. JWT is used for subsequent requests (standard Bearer auth)
//! 5. Middleware also accepts raw API keys directly via `X-API-Key` header

use std::collections::HashMap;
use std::sync::Arc;

use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use async_trait::async_trait;
use jsonwebtoken::{EncodingKey, Header};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::uar::security::{claims::{TenantId, UserClaims, UserContext, VerifiedIdentity}, jwt};
use secrecy::ExposeSecret;

mod policy;
pub use policy::ApiKeyAuthorityError;
use policy::{AUTHORITY_VERSION, KeyPolicy};

/// Proof created only after a stored API key hash and its authority are validated.
pub(in crate::uar::security) struct VerifiedKeyTenantClaim<'a>(&'a str);
impl VerifiedKeyTenantClaim<'_> {
    pub(in crate::uar::security) fn value(&self) -> &str { self.0 }
}

// ─────────────────────────────────────────────────────────────────────────────
// Types
// ─────────────────────────────────────────────────────────────────────────────

/// Stored API key record (hash only — raw key is never persisted).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyRecord {
    /// Unique key ID (prefix shown to user for identification).
    pub id: String,
    /// Argon2 hash of the raw key.
    pub key_hash: String,
    /// Human-readable label.
    pub name: String,
    /// Subject (user ID) this key belongs to.
    pub subject: String,
    #[serde(default)]
    pub authority_version: u32,
    #[serde(default)]
    pub issuer: Option<String>,
    #[serde(default)]
    pub tenant_id: Option<String>,
    /// Roles granted by this key.
    pub roles: Vec<String>,
    /// Creation timestamp (Unix seconds).
    pub created_at: i64,
    /// Expiry timestamp (Unix seconds), or `None` for non-expiring.
    pub expires_at: Option<i64>,
    /// Whether this key has been revoked.
    pub revoked: bool,
}

/// Public metadata returned to callers (no hash).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyMetadata {
    pub id: String,
    pub name: String,
    pub subject: String,
    pub authority_version: u32,
    pub issuer: Option<String>,
    pub tenant_id: Option<String>,
    pub roles: Vec<String>,
    pub created_at: i64,
    pub expires_at: Option<i64>,
    pub revoked: bool,
}

impl From<&ApiKeyRecord> for ApiKeyMetadata {
    fn from(r: &ApiKeyRecord) -> Self {
        Self {
            id: r.id.clone(),
            name: r.name.clone(),
            subject: r.subject.clone(),
            authority_version: r.authority_version,
            issuer: r.issuer.clone(),
            tenant_id: r.tenant_id.clone(),
            roles: r.roles.clone(),
            created_at: r.created_at,
            expires_at: r.expires_at,
            revoked: r.revoked,
        }
    }
}

/// Request body for creating a new API key.
#[derive(Debug, Deserialize)]
pub struct CreateKeyRequest {
    /// Human-readable label for the key.
    pub name: String,
    /// Roles to grant. Defaults to `["user"]`.
    pub roles: Option<Vec<String>>,
    /// Optional expiry in seconds from now.
    pub expires_in_secs: Option<i64>,
}

/// Response returned when a key is created (raw key shown once).
#[derive(Debug, Serialize)]
pub struct ApiKeyResponse {
    /// Key metadata (safe to store/display).
    pub metadata: ApiKeyMetadata,
    /// Raw API key — shown **once**, never stored.
    pub raw_key: String,
}

// ─────────────────────────────────────────────────────────────────────────────
// Storage trait
// ─────────────────────────────────────────────────────────────────────────────

/// Persistence abstraction for API key records.
#[async_trait]
pub trait ApiKeyStorage: Send + Sync + std::fmt::Debug {
    async fn insert(&self, record: ApiKeyRecord) -> anyhow::Result<()>;
    async fn get_by_id(&self, id: &str) -> anyhow::Result<Option<ApiKeyRecord>>;
    async fn list_by_subject(&self, subject: &str) -> anyhow::Result<Vec<ApiKeyRecord>>;
    async fn revoke(&self, id: &str) -> anyhow::Result<bool>;
    /// Return all non-revoked records (for key-scanning during exchange).
    async fn all_active(&self) -> anyhow::Result<Vec<ApiKeyRecord>>;
    async fn all(&self) -> anyhow::Result<Vec<ApiKeyRecord>>;
}

// ─────────────────────────────────────────────────────────────────────────────
// In-memory implementation
// ─────────────────────────────────────────────────────────────────────────────

/// Thread-safe in-memory API key store (suitable for development / testing).
#[derive(Debug, Default)]
pub struct InMemoryApiKeyStorage {
    records: RwLock<HashMap<String, ApiKeyRecord>>,
}

impl InMemoryApiKeyStorage {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl ApiKeyStorage for InMemoryApiKeyStorage {
    async fn insert(&self, record: ApiKeyRecord) -> anyhow::Result<()> {
        self.records.write().await.insert(record.id.clone(), record);
        Ok(())
    }

    async fn get_by_id(&self, id: &str) -> anyhow::Result<Option<ApiKeyRecord>> {
        Ok(self.records.read().await.get(id).cloned())
    }

    async fn list_by_subject(&self, subject: &str) -> anyhow::Result<Vec<ApiKeyRecord>> {
        Ok(self
            .records
            .read()
            .await
            .values()
            .filter(|r| r.subject == subject)
            .cloned()
            .collect())
    }

    async fn revoke(&self, id: &str) -> anyhow::Result<bool> {
        let mut map = self.records.write().await;
        if let Some(record) = map.get_mut(id) {
            record.revoked = true;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn all(&self) -> anyhow::Result<Vec<ApiKeyRecord>> {
        Ok(self.records.read().await.values().cloned().collect())
    }

    async fn all_active(&self) -> anyhow::Result<Vec<ApiKeyRecord>> {
        Ok(self
            .records
            .read()
            .await
            .values()
            .filter(|r| !r.revoked)
            .cloned()
            .collect())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Service
// ─────────────────────────────────────────────────────────────────────────────

/// API key management service.
///
/// Handles creation, validation, exchange, and revocation of API keys.
#[derive(Debug, Clone)]
pub struct ApiKeyService {
    db: Arc<dyn ApiKeyStorage>,
    jwt_secret: secrecy::SecretString,
    policy: KeyPolicy,
    /// JWT TTL in seconds for exchanged tokens.
    jwt_ttl_secs: i64,
}

#[derive(Serialize)]
struct IssuedJwtClaims<'a> {
    uar_credential_kind: &'static str,
    #[serde(flatten)]
    user: &'a UserClaims,
    #[serde(skip_serializing_if = "Option::is_none")]
    iss: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    aud: Option<&'a str>,
}

pub struct ExchangedJwt {
    pub token: String,
    pub expires_in: u64,
}

impl ApiKeyService {
    /// Create a new service with the given storage and JWT secret.
    pub fn new(db: Arc<dyn ApiKeyStorage>, jwt_secret: impl Into<String>) -> Self {
        let jwt_secret: String = jwt_secret.into();
        Self {
            db,
            jwt_secret: jwt_secret.into(),
            policy: KeyPolicy::default(),
            jwt_ttl_secs: 3600, // 1 hour default
        }
    }

    #[cfg(all(test, feature = "server"))]
    pub(crate) fn with_registered_claims(
        mut self,
        issuer: Option<String>,
        audience: Option<String>,
    ) -> Self {
        self.policy.issuer = issuer;
        self.policy.audience = audience;
        self
    }

    pub fn with_security_config(mut self, config: &crate::config::SecurityConfig) -> anyhow::Result<Self> {
        self.policy = KeyPolicy::from_config(config)?;
        Ok(self)
    }

    /// Create a new service with a custom JWT TTL.
    pub fn with_ttl(mut self, ttl_secs: i64) -> Self {
        self.jwt_ttl_secs = ttl_secs;
        self
    }

    /// Create a new API key. Returns the raw key (shown once) + metadata.
    pub async fn create_key(
        &self,
        caller: &UserContext,
        request: CreateKeyRequest,
    ) -> anyhow::Result<ApiKeyResponse> {
        let roles = self.policy.attenuate(caller, request.roles)?;
        let subject = caller.user_id.clone();
        let raw_key = generate_raw_key();
        let key_hash = hash_key(&raw_key)?;
        let now = now_unix();

        let id = format!("uar_{}", &Uuid::new_v4().to_string().replace('-', "")[..16]);
        let expires_at = request.expires_in_secs.map(|seconds| {
            if seconds <= 0 { return Err(ApiKeyAuthorityError::InvalidLifetime); }
            now.checked_add(seconds).ok_or(ApiKeyAuthorityError::InvalidLifetime)
        }).transpose()?;

        let record = ApiKeyRecord {
            id: id.clone(),
            key_hash,
            name: request.name,
            subject: subject.clone(),
            authority_version: AUTHORITY_VERSION,
            issuer: self.policy.issuer.clone(),
            tenant_id: caller.tenant_id.as_ref().map(|tenant| tenant.as_str().to_owned()),
            roles,
            created_at: now,
            expires_at,
            revoked: false,
        };

        let metadata = ApiKeyMetadata::from(&record);
        self.db.insert(record).await?;

        Ok(ApiKeyResponse { metadata, raw_key })
    }

    /// Validate a raw API key and exchange it for a short-lived JWT.
    ///
    /// Returns `None` if the key is invalid, expired, or revoked.
    pub async fn exchange_for_jwt(&self, raw_key: &str) -> anyhow::Result<Option<String>> {
        Ok(self.exchange_with_expiry(raw_key).await?.map(|issued| issued.token))
    }

    pub async fn exchange_with_expiry(&self, raw_key: &str) -> anyhow::Result<Option<ExchangedJwt>> {
        if !self.policy.exchange_supported { return Err(ApiKeyAuthorityError::UnsupportedExchange.into()); }
        let active = self.db.all_active().await?;
        let now = now_unix();

        for record in active {
            // Check expiry
            if let Some(exp) = record.expires_at {
                if now >= exp {
                    continue;
                }
            }

            // Verify hash
            if verify_key(raw_key, &record.key_hash)? {
                self.policy.validate_record(&record)?;
                let Some(exp) = self.token_expiry(&record, now) else { return Ok(None); };
                let claims = UserClaims {
                    sub: record.subject.clone(),
                    name: Some(record.name.clone()),
                    roles: Some(record.roles.clone()),
                    tenant_id: record.tenant_id.clone(),
                    uar_instance_id: None,
                    exp,
                };
                let issued_claims = IssuedJwtClaims {
                    uar_credential_kind: "api_key",
                    user: &claims,
                    iss: record.issuer.as_deref(),
                    aud: self.policy.audience.as_deref(),
                };
                let token = jwt::encode(
                    &Header::default(),
                    &issued_claims,
                    &EncodingKey::from_secret(self.jwt_secret.expose_secret().as_bytes()),
                )
                .map_err(|error| anyhow::anyhow!("issuing API-key exchange JWT: {error}"))?;
                return Ok(Some(ExchangedJwt { token, expires_in: (exp as i64 - now) as u64 }));
            }
        }

        Ok(None)
    }

    /// Revoke an API key by ID.
    pub async fn revoke_key(&self, caller: &UserContext, id: &str) -> anyhow::Result<bool> {
        self.policy.identity(caller)?;
        let Some(record) = self.db.get_by_id(id).await? else { return Ok(false); };
        if !self.policy.can_manage(caller, &record)? { return Ok(false); }
        self.db.revoke(id).await
    }

    /// List all keys for a subject (metadata only, no raw keys).
    pub async fn list_keys(&self, caller: &UserContext) -> anyhow::Result<Vec<ApiKeyMetadata>> {
        self.policy.identity(caller)?;
        let records = self.db.all().await?;
        let mut visible = Vec::new();
        for record in records {
            if self.policy.can_manage(caller, &record)? { visible.push(ApiKeyMetadata::from(&record)); }
        }
        Ok(visible)
    }

    fn token_expiry(&self, record: &ApiKeyRecord, now: i64) -> Option<usize> {
        let ttl = self.jwt_ttl_secs.min(3600);
        if ttl <= 0 { return None; }
        let expiry = now.checked_add(ttl)?.min(record.expires_at.unwrap_or(i64::MAX));
        (expiry > now).then(|| usize::try_from(expiry).ok()).flatten()
    }

    /// Validate a raw API key directly (for middleware use).
    ///
    /// Returns verified user context if valid, or `None` if invalid/expired/revoked.
    pub async fn validate_key(&self, raw_key: &str) -> anyhow::Result<Option<UserContext>> {
        let active = self.db.all_active().await?;
        let now = now_unix();

        for record in active {
            if let Some(exp) = record.expires_at {
                if now >= exp {
                    continue;
                }
            }
            if verify_key(raw_key, &record.key_hash)? {
                self.policy.validate_record(&record)?;
                let Some(exp) = self.token_expiry(&record, now) else { return Ok(None); };
                let claims = UserClaims {
                    sub: record.subject.clone(),
                    name: Some(record.name.clone()),
                    roles: Some(record.roles.clone()),
                    tenant_id: record.tenant_id.clone(),
                    uar_instance_id: None,
                    exp,
                };
                let tenant_id = record.tenant_id.as_deref().map(|tenant| TenantId::from_verified_key(VerifiedKeyTenantClaim(tenant)));
                let authority = Some(VerifiedIdentity::new(record.issuer.as_deref(), &claims, tenant_id.clone(), super::claims::CredentialKind::ApiKey));
                return Ok(Some(UserContext {
                    host_authority: None, authority, user_id: record.subject.clone(),
                    tenant_id,
                    claims,
                }));
            }
        }
        Ok(None)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Generate a cryptographically random API key (32 bytes → 64 hex chars).
fn generate_raw_key() -> String {
    let mut bytes = [0u8; 32];
    rand::fill(&mut bytes);
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Hash a raw key with Argon2id.
fn hash_key(raw_key: &str) -> anyhow::Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(raw_key.as_bytes(), &salt)
        .map_err(|e| anyhow::anyhow!("argon2 hash error: {e}"))?;
    Ok(hash.to_string())
}

/// Verify a raw key against an Argon2 hash.
fn verify_key(raw_key: &str, hash: &str) -> anyhow::Result<bool> {
    let parsed =
        PasswordHash::new(hash).map_err(|e| anyhow::anyhow!("invalid hash format: {e}"))?;
    Ok(Argon2::default()
        .verify_password(raw_key.as_bytes(), &parsed)
        .is_ok())
}

/// Current Unix timestamp in seconds.
fn now_unix() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(all(test, feature = "server"))]
mod tests;
