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

use crate::uar::security::{claims::UserClaims, jwt};

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
    jwt_secret: String,
    /// JWT TTL in seconds for exchanged tokens.
    jwt_ttl_secs: i64,
}

impl ApiKeyService {
    /// Create a new service with the given storage and JWT secret.
    pub fn new(db: Arc<dyn ApiKeyStorage>, jwt_secret: impl Into<String>) -> Self {
        Self {
            db,
            jwt_secret: jwt_secret.into(),
            jwt_ttl_secs: 3600, // 1 hour default
        }
    }

    /// Create a new service with a custom JWT TTL.
    pub fn with_ttl(mut self, ttl_secs: i64) -> Self {
        self.jwt_ttl_secs = ttl_secs;
        self
    }

    /// Create a new API key. Returns the raw key (shown once) + metadata.
    pub async fn create_key(
        &self,
        subject: impl Into<String>,
        request: CreateKeyRequest,
    ) -> anyhow::Result<ApiKeyResponse> {
        let subject = subject.into();
        let raw_key = generate_raw_key();
        let key_hash = hash_key(&raw_key)?;
        let now = now_unix();

        let id = format!("uar_{}", &Uuid::new_v4().to_string().replace('-', "")[..16]);
        let roles = request.roles.unwrap_or_else(|| vec!["user".to_string()]);
        let expires_at = request.expires_in_secs.map(|secs| now + secs);

        let record = ApiKeyRecord {
            id: id.clone(),
            key_hash,
            name: request.name,
            subject: subject.clone(),
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
        let active = self.db.all_active().await?;
        let now = now_unix();

        for record in active {
            // Check expiry
            if let Some(exp) = record.expires_at {
                if now > exp {
                    continue;
                }
            }

            // Verify hash
            if verify_key(raw_key, &record.key_hash)? {
                let exp = (now + self.jwt_ttl_secs) as usize;
                let claims = UserClaims {
                    sub: record.subject.clone(),
                    name: Some(record.name.clone()),
                    roles: Some(record.roles.clone()),
                    exp,
                };
                let token = jwt::encode(
                    &Header::default(),
                    &claims,
                    &EncodingKey::from_secret(self.jwt_secret.as_bytes()),
                )
                .map_err(|error| anyhow::anyhow!("issuing API-key exchange JWT: {error}"))?;
                return Ok(Some(token));
            }
        }

        Ok(None)
    }

    /// Revoke an API key by ID.
    pub async fn revoke_key(&self, id: &str) -> anyhow::Result<bool> {
        self.db.revoke(id).await
    }

    /// List all keys for a subject (metadata only, no raw keys).
    pub async fn list_keys(&self, subject: &str) -> anyhow::Result<Vec<ApiKeyMetadata>> {
        let records = self.db.list_by_subject(subject).await?;
        Ok(records.iter().map(ApiKeyMetadata::from).collect())
    }

    /// Validate a raw API key directly (for middleware use).
    ///
    /// Returns the `UserClaims` if valid, or `None` if invalid/expired/revoked.
    pub async fn validate_key(&self, raw_key: &str) -> anyhow::Result<Option<UserClaims>> {
        let active = self.db.all_active().await?;
        let now = now_unix();

        for record in active {
            if let Some(exp) = record.expires_at {
                if now > exp {
                    continue;
                }
            }
            if verify_key(raw_key, &record.key_hash)? {
                let exp = (now + self.jwt_ttl_secs) as usize;
                return Ok(Some(UserClaims {
                    sub: record.subject.clone(),
                    name: Some(record.name.clone()),
                    roles: Some(record.roles.clone()),
                    exp,
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

#[cfg(test)]
mod tests {
    use super::*;

    fn make_service() -> ApiKeyService {
        let storage: Arc<dyn ApiKeyStorage> = Arc::new(InMemoryApiKeyStorage::new());
        ApiKeyService::new(storage, "test-secret")
    }

    #[tokio::test]
    async fn test_create_and_exchange() {
        let svc = make_service();
        let resp = svc
            .create_key(
                "user-1",
                CreateKeyRequest {
                    name: "my-key".to_string(),
                    roles: None,
                    expires_in_secs: None,
                },
            )
            .await
            .unwrap();

        assert!(!resp.raw_key.is_empty());
        assert_eq!(resp.metadata.subject, "user-1");

        // Exchange for JWT
        let jwt = svc.exchange_for_jwt(&resp.raw_key).await.unwrap();
        assert!(jwt.is_some(), "should produce a JWT");
    }

    #[tokio::test]
    async fn test_invalid_key_rejected() {
        let svc = make_service();
        let jwt = svc.exchange_for_jwt("not-a-real-key").await.unwrap();
        assert!(jwt.is_none());
    }

    #[tokio::test]
    async fn test_revoked_key_rejected() {
        let svc = make_service();
        let resp = svc
            .create_key(
                "user-2",
                CreateKeyRequest {
                    name: "revoke-me".to_string(),
                    roles: None,
                    expires_in_secs: None,
                },
            )
            .await
            .unwrap();

        svc.revoke_key(&resp.metadata.id).await.unwrap();
        let jwt = svc.exchange_for_jwt(&resp.raw_key).await.unwrap();
        assert!(jwt.is_none(), "revoked key should not produce JWT");
    }

    #[tokio::test]
    async fn test_list_keys() {
        let svc = make_service();
        svc.create_key(
            "user-3",
            CreateKeyRequest {
                name: "k1".to_string(),
                roles: None,
                expires_in_secs: None,
            },
        )
        .await
        .unwrap();
        svc.create_key(
            "user-3",
            CreateKeyRequest {
                name: "k2".to_string(),
                roles: None,
                expires_in_secs: None,
            },
        )
        .await
        .unwrap();

        let keys = svc.list_keys("user-3").await.unwrap();
        assert_eq!(keys.len(), 2);
    }

    #[tokio::test]
    async fn test_validate_key_directly() {
        let svc = make_service();
        let resp = svc
            .create_key(
                "user-4",
                CreateKeyRequest {
                    name: "direct-validate".to_string(),
                    roles: Some(vec!["admin".to_string()]),
                    expires_in_secs: None,
                },
            )
            .await
            .unwrap();

        let claims = svc.validate_key(&resp.raw_key).await.unwrap();
        assert!(claims.is_some());
        let claims = claims.unwrap();
        assert_eq!(claims.sub, "user-4");
        assert_eq!(claims.roles.unwrap(), vec!["admin"]);
    }
}
