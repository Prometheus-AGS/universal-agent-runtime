//! Bounded, caller-owned signing-key refresh for one configured issuer and URL.

use std::{
    collections::HashMap,
    sync::{Arc, OnceLock},
    time::Duration,
};

use jsonwebtoken::{DecodingKey, jwk::JwkSet};
use super::{JwksKey, jwk_algorithm};
use tokio::{
    sync::{Mutex, RwLock},
    time::{Instant, timeout_at},
};

const REFRESH_TARGET: Duration = Duration::from_secs(60);
const HARD_AGE: Duration = Duration::from_secs(300);
const FETCH_TIMEOUT: Duration = Duration::from_secs(5);
const RETRY_MINIMUM: Duration = Duration::from_secs(5);

/// Sanitized failures at the signing-key trust boundary.
#[derive(Clone, Copy, Debug, thiserror::Error)]
pub(crate) enum CacheError {
    #[error("JWKS does not contain the requested signing key")]
    UnknownKeyId,
    #[error("JWKS refresh is unavailable")]
    Unavailable,
    #[error("JWKS signing keys exceeded their maximum age")]
    Stale,
    #[error("JWKS refresh deadline expired")]
    Timeout,
    #[error("JWKS contains invalid signing-key material")]
    InvalidKeySet,
}

struct Snapshot {
    keys: HashMap<String, JwksKey>,
    refreshed_at: Instant,
}

struct CacheState {
    snapshot: Option<Snapshot>,
    last_attempt: Option<Instant>,
    last_error: CacheError,
}

impl Default for CacheState {
    fn default() -> Self {
        Self {
            snapshot: None,
            last_attempt: None,
            last_error: CacheError::Unavailable,
        }
    }
}

impl CacheState {
    fn key(&self, kid: &str, unavailable: CacheError) -> Result<JwksKey, CacheError> {
        let snapshot = self.snapshot.as_ref().ok_or(unavailable)?;
        if snapshot.refreshed_at.elapsed() >= HARD_AGE {
            return Err(CacheError::Stale);
        }
        snapshot.keys.get(kid).cloned().ok_or(unavailable)
    }

    fn refresh_due(&self, kid: &str) -> bool {
        self.snapshot.as_ref().is_none_or(|snapshot| {
            snapshot.refreshed_at.elapsed() >= REFRESH_TARGET || !snapshot.keys.contains_key(kid)
        })
    }
}

/// Shared refresh lane and last complete key set for a configured issuer/URL pair.
pub(super) struct JwksCache {
    url: String,
    client: reqwest::Client,
    state: RwLock<CacheState>,
    // Only this asynchronous gate spans network awaits; state guards never do.
    refresh: Mutex<()>,
}

type CacheIdentity = (Option<String>, String);
static JWKS_CACHES: OnceLock<RwLock<HashMap<CacheIdentity, Arc<JwksCache>>>> = OnceLock::new();
static JWKS_CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

/// Reuse one cache for an exact configured issuer and URL, including local `None`.
///
/// # Examples
/// The verifier passes its configured issuer and URL, never token claim values.
/// # Errors
/// This constructor performs no network request.
/// # Panics
/// Retains reqwest's existing default-client initialization behavior.
pub(super) async fn cache_for_issuer(issuer: Option<&str>, url: &str) -> Arc<JwksCache> {
    let caches = JWKS_CACHES.get_or_init(|| RwLock::new(HashMap::new()));
    let identity = (issuer.map(str::to_owned), url.to_owned());
    if let Some(cache) = caches.read().await.get(&identity).cloned() {
        return cache;
    }
    Arc::clone(caches.write().await.entry(identity).or_insert_with(|| {
        Arc::new(JwksCache {
            url: url.to_owned(),
            client: JWKS_CLIENT.get_or_init(reqwest::Client::new).clone(),
            state: RwLock::new(CacheState::default()),
            refresh: Mutex::new(()),
        })
    }))
}

impl JwksCache {
    /// Resolve a key with one five-second budget for lookup, waiting and fetch.
    ///
    /// # Examples
    /// The verifier calls `cache.key(&kid).await` before JWT signature validation.
    /// # Errors
    /// Rejects unknown keys and unavailable keys at or beyond the hard age.
    /// A failed refresh can reuse a known key only while its snapshot is younger.
    /// # Panics
    /// Requires a Tokio runtime with time enabled.
    pub(super) async fn key(&self, kid: &str) -> Result<JwksKey, CacheError> {
        let deadline = Instant::now() + FETCH_TIMEOUT;
        match timeout_at(deadline, self.key_before(kid, deadline)).await {
            Ok(result) if Instant::now() < deadline => result,
            _ => {
                tracing::error!(reason = "deadline expired", "JWKS refresh failed");
                // Do not extend the budget to acquire a lock after timeout.
                self.state
                    .try_read()
                    .map_err(|_| CacheError::Timeout)?
                    .key(kid, CacheError::Timeout)
            }
        }
    }

    async fn key_before(&self, kid: &str, deadline: Instant) -> Result<JwksKey, CacheError> {
        let observed_attempt = {
            let state = self.state.read().await;
            if !state.refresh_due(kid) {
                return state.key(kid, CacheError::UnknownKeyId);
            }
            state.last_attempt
        };

        let _refresh = match self.refresh.try_lock() {
            Ok(guard) => guard,
            Err(_) => {
                // Join this flight; an unknown kid must not queue another fetch.
                let _joined = self.refresh.lock().await;
                let state = self.state.read().await;
                return state.key(kid, state.last_error);
            }
        };

        {
            let mut state = self.state.write().await;
            if state.last_attempt != observed_attempt || !state.refresh_due(kid) {
                return state.key(kid, state.last_error);
            }
            if state
                .last_attempt
                .is_some_and(|attempt| attempt.elapsed() < RETRY_MINIMUM)
            {
                return state.key(kid, state.last_error);
            }
            if Instant::now() >= deadline {
                return Err(CacheError::Timeout);
            }
            // Record before I/O so caller cancellation also consumes the retry interval.
            state.last_attempt = Some(Instant::now());
            state.last_error = CacheError::Unavailable;
        }

        let result = self.fetch(deadline).await;
        if let Err(error) = result.as_ref() {
            tracing::error!(reason = %error, "JWKS refresh failed");
        }
        let mut state = self.state.write().await;
        // Parsing/conversion can finish without yielding; refuse late publication too.
        if Instant::now() >= deadline {
            return Err(CacheError::Timeout);
        }
        match result {
            Ok(keys) => {
                state.snapshot = Some(Snapshot {
                    keys,
                    refreshed_at: Instant::now(),
                });
                state.last_error = CacheError::UnknownKeyId;
            }
            Err(error) => state.last_error = error,
        }
        state.key(kid, state.last_error)
    }

    async fn fetch(&self, deadline: Instant) -> Result<HashMap<String, JwksKey>, CacheError> {
        let response = self
            .client
            .get(&self.url)
            .timeout(deadline.saturating_duration_since(Instant::now()))
            .send()
            .await
            .and_then(reqwest::Response::error_for_status)
            .map_err(|_| CacheError::Unavailable)?;
        let jwks = response
            .json::<JwkSet>()
            .await
            .map_err(|_| CacheError::InvalidKeySet)?;
        let mut keys = HashMap::new();
        for jwk in jwks.keys {
            if Instant::now() >= deadline {
                return Err(CacheError::Timeout);
            }
            let Some(kid) = jwk.common.key_id.as_ref() else {
                continue;
            };
            let Some(algorithm) = jwk_algorithm(&jwk) else {
                continue;
            };
            let key = DecodingKey::from_jwk(&jwk).map_err(|_| CacheError::InvalidKeySet)?;
            keys.insert(kid.clone(), JwksKey { key, algorithm });
        }
        Ok(keys)
    }

    #[cfg(test)]
    pub(super) async fn refreshed_at(&self) -> Option<Instant> {
        self.state
            .read()
            .await
            .snapshot
            .as_ref()
            .map(|snapshot| snapshot.refreshed_at)
    }
}
