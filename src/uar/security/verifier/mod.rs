use std::{
    collections::HashMap,
    sync::{Arc, OnceLock},
    time::SystemTime,
};

use async_trait::async_trait;
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode_header, jwk::JwkSet};
use tokio::sync::RwLock;

use super::{
    claims::{TenantId, UserClaims},
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
    pub(crate) subject: String,
    pub(crate) tenant_id: Option<TenantId>,
    pub(crate) claims: UserClaims,
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
    #[error("JWKS endpoint {url} could not be read: {source}")]
    JwksFetch {
        url: String,
        #[source]
        source: reqwest::Error,
    },
    #[error("JWKS endpoint {url} does not contain kid {kid}")]
    UnknownKeyId { url: String, kid: String },
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
}

impl<'a> SharedSecretVerifier<'a> {
    pub(crate) fn new(secret: &'a str) -> Self {
        Self { secret }
    }
}

#[async_trait]
impl TokenVerifier for SharedSecretVerifier<'_> {
    async fn verify(&self, presented: Presented) -> Result<Principal, VerificationError> {
        let Presented::Jwks(token) = presented else {
            return Err(VerificationError::UnsupportedPresentation);
        };

        let key = DecodingKey::from_secret(self.secret.as_bytes());
        let token_data = jwt::decode::<UserClaims>(token, &key, &Validation::default())?;
        let claims = token_data.claims;

        Ok(Principal {
            subject: claims.sub.clone(),
            tenant_id: claims
                .tenant_id
                .as_deref()
                .map(|value| TenantId::from_verified_claim(VerifiedTenantClaim::new(value))),
            claims,
        })
    }
}

#[derive(Default)]
struct JwksCache {
    keys: RwLock<HashMap<String, DecodingKey>>,
    refreshed_at: RwLock<Option<SystemTime>>,
}

static JWKS_CACHES: OnceLock<RwLock<HashMap<String, Arc<JwksCache>>>> = OnceLock::new();
static JWKS_CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

async fn cache_for_url(url: &str) -> Arc<JwksCache> {
    let caches = JWKS_CACHES.get_or_init(|| RwLock::new(HashMap::new()));

    if let Some(cache) = caches.read().await.get(url).cloned() {
        return cache;
    }

    Arc::clone(
        caches
            .write()
            .await
            .entry(url.to_owned())
            .or_insert_with(|| Arc::new(JwksCache::default())),
    )
}

pub(crate) struct JwksVerifier {
    url: String,
    issuer: Option<String>,
    audience: Option<String>,
    client: reqwest::Client,
    cache: Arc<JwksCache>,
}

impl JwksVerifier {
    pub(crate) async fn new(url: &str, issuer: Option<&str>, audience: Option<&str>) -> Self {
        Self {
            url: url.to_owned(),
            issuer: issuer.map(str::to_owned),
            audience: audience.map(str::to_owned),
            client: JWKS_CLIENT.get_or_init(reqwest::Client::new).clone(),
            cache: cache_for_url(url).await,
        }
    }

    async fn cached_key(&self, kid: &str) -> Option<DecodingKey> {
        self.cache.keys.read().await.get(kid).cloned()
    }

    async fn refresh(&self) -> Result<(), VerificationError> {
        let response = self
            .client
            .get(&self.url)
            .send()
            .await
            .and_then(reqwest::Response::error_for_status)
            .map_err(|source| self.fetch_error(source))?;
        let jwks = response
            .json::<JwkSet>()
            .await
            .map_err(|source| self.fetch_error(source))?;

        let mut keys = HashMap::new();
        for jwk in jwks.keys {
            let Some(kid) = jwk.common.key_id.clone() else {
                continue;
            };
            keys.insert(kid, DecodingKey::from_jwk(&jwk)?);
        }

        *self.cache.keys.write().await = keys;
        *self.cache.refreshed_at.write().await = Some(SystemTime::now());
        Ok(())
    }

    fn fetch_error(&self, source: reqwest::Error) -> VerificationError {
        tracing::error!(jwks_url = %self.url, error = %source, "JWKS refresh failed");
        VerificationError::JwksFetch {
            url: self.url.clone(),
            source,
        }
    }

    fn validation(&self) -> Validation {
        let mut validation = Validation::new(Algorithm::RS256);
        if let Some(issuer) = self.issuer.as_deref() {
            validation.set_issuer(&[issuer]);
            validation.required_spec_claims.insert("iss".to_owned());
        }
        if let Some(audience) = self.audience.as_deref() {
            validation.set_audience(&[audience]);
            validation.required_spec_claims.insert("aud".to_owned());
        }
        validation
    }

    #[cfg(test)]
    async fn refreshed_at(&self) -> Option<SystemTime> {
        *self.cache.refreshed_at.read().await
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

        let key = if let Some(key) = self.cached_key(&kid).await {
            key
        } else {
            self.refresh().await?;
            self.cached_key(&kid)
                .await
                .ok_or_else(|| VerificationError::UnknownKeyId {
                    url: self.url.clone(),
                    kid: kid.clone(),
                })?
        };

        let token_data = jwt::decode::<UserClaims>(token, &key, &self.validation())?;
        let claims = token_data.claims;
        Ok(Principal {
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
    let presented = Presented::Jwks(token.to_owned());
    if let Some(jwks_url) = config.jwks_url.as_deref() {
        JwksVerifier::new(
            jwks_url,
            config.jwt_issuer.as_deref(),
            config.jwt_audience.as_deref(),
        )
        .await
        .verify(presented)
        .await
    } else {
        SharedSecretVerifier::new(config.jwt_secret.expose_secret())
            .verify(presented)
            .await
    }
}

#[cfg(test)]
pub(crate) mod test_support {
    use std::{
        sync::{
            Arc,
            atomic::{AtomicUsize, Ordering},
        },
        time::{SystemTime, UNIX_EPOCH},
    };

    use axum::{Json, Router, extract::State, routing::get};
    use base64::Engine as _;
    use jsonwebtoken::{
        Algorithm, EncodingKey, Header,
        jwk::{Jwk, JwkSet},
    };
    use serde::Serialize;
    use tokio::{net::TcpListener, sync::RwLock, task::JoinHandle};

    use super::jwt;

    const RSA_PRIVATE_KEY_DER: &str = "MIIEpAIBAAKCAQEAyRE6rHuNR0QbHO3H3Kt2pOKGVhQqGZXInOduQNxXzuKlvQTLUTv4l4sggh5/CYYi/cvI+SXVT9kPWSKXxJXBXd/4LkvcPuUakBoAkfh+eiFVMh2VrUyWyj3MFl0HTVF9KwRXLAcwkREiS3npThHRyIxuy0ZMeZfxVL5arMhw1SRELB8HoGfG/AtH89BIE9jDBHZ9dLelK9a184zAf8LwoPLxvJb3Il5nncqPcSfKDDodMFBIMc4lQzDKL5gvmiXLXB1AGLm8KBjfE8s3L5xqi+yUod+j8MtvIj812dkS4QMiRVN/by2h3ZY8LYVGrqZXZTcgn2ujn8uKjXLZVD5TdQIDAQABAoIBAHREk0I0O9DvECKdWUpAmF3mY7oY9PNQiu44Yaf+AoSuyRpRUGTMIgc3u3eivOE8ALX0BmYUO5JtuRNZDpvt4SAwqCnVUinIf6C+eH/wSurCpapSM0BAHp4aOA7igptyOMgMPYBHNA1e9A7jE0dCxKWMl3DSWNyjQTk4zeRGEAEfbNjHrq6YCtjHSZSLmWiG80hnfnYos9hOr5JnLnyS7ZmFE/5P3XVrxLc/tQ5zum0R4cbrgzHiQP5RgfxGJaEi7XcgherCCOgurJSSbYH29Gz8u5fFbS+Yg8s+OiCss3cs1rSgJ9/eHZuzGEdUZVARH6hVMjSuwvqVTFaE8AgtleECgYEA+uLMn4kNqHlJS2A5uAnCkj90ZxEtNm3E8hAxUrhssktY5XSOAPBlxyf5RuRGIImGtUVIr4HuJSa5TX48n3Vdt9MYCprO/iYl6moNRSPt5qowIIOJmIjY2mqPDfDt/zw+fcDD3lmCJrFlzcnh0uea1CohxEbQnL3cypeLt+WbU6kCgYEAzSp19m1ajieFkqgoB0YTpt/OroDx38vvI5unInJlEeOjQ+oIAQdN2wpxBvTrRorMU6P07mFUbt1j+Co6CbNiw+X8HcCaqYLR5clbJOOWNR36PuzOpQLkfK8woupBxzW9B8gZmY8rB1mbJ+/WTPrEJy6YGmIEBkWylQ2VpW8O4O0CgYEApdbvvfFBlwD9YxbrcGz7MeNCFbMz+MucqQntIKoKJ91ImPxvtc0y6e/Rhnv0oyNlaUOwJVu0yNgNG117w0g4t/+Q38mvVC5xV7/cn7x9UMFk6MkqVir3dYGEqIl/OP1grY2Tq9HtB5iyG9L8NIamQOLMyUqqMUILxdthHyFmiGkCgYEAn9+PjpjGMPHxL0gj8Q8VbzsFtou6b1deIRRA2CHmSltltR1gYVTMwXxQeUhPMmgkMqUXzs4/WijgpthY44hK1TaZEKIuoxrS70nJ4WQLf5a9k1065fDsFZD6yGjdGxvwEmlGMZgTwqV7t1I4X0Ilqhav5hcs5apYL7gnPYPeRz0CgYALHCj/Ji8XSsDoF/MhVhnGdIs2P99NNdmo3R2Pv0CuZbDKMU559LJHUvrKS8WkuWRDuKrz1W/EQKApFjDGpdqToZqriUFQzwy7mR3ayIiogzNtHcvbDHx8oFnGY0OFksX/ye0/XGpy2SFxYRwGU98HPYeBvAQQrVjdkzfy7BmXQQ==";

    #[derive(Serialize)]
    struct TestClaims<'a> {
        sub: &'a str,
        name: Option<&'a str>,
        roles: Option<Vec<&'a str>>,
        exp: usize,
        iss: &'a str,
        aud: &'a str,
    }

    fn encoding_key() -> EncodingKey {
        let der = base64::engine::general_purpose::STANDARD
            .decode(RSA_PRIVATE_KEY_DER)
            .expect("test RSA key must decode");
        EncodingKey::from_rsa_der(&der)
    }

    pub(crate) fn jwk(kid: &str) -> Jwk {
        jwt::ensure_rustcrypto_provider().expect("RustCrypto must initialize for test JWK");
        let mut jwk = Jwk::from_encoding_key(&encoding_key(), Algorithm::RS256)
            .expect("test RSA key must produce a JWK");
        jwk.common.key_id = Some(kid.to_owned());
        jwk
    }

    pub(crate) fn signed_token(kid: &str, issuer: &str, audience: &str) -> String {
        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some(kid.to_owned());
        let claims = TestClaims {
            sub: "user-123",
            name: Some("Test User"),
            roles: Some(vec!["user"]),
            exp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system clock must follow the Unix epoch")
                .as_secs()
                .saturating_add(3600) as usize,
            iss: issuer,
            aud: audience,
        };
        jwt::encode(&header, &claims, &encoding_key()).expect("test token must encode")
    }

    #[derive(Clone)]
    struct TestServerState {
        keys: Arc<RwLock<JwkSet>>,
        requests: Arc<AtomicUsize>,
    }

    async fn serve_jwks(State(state): State<TestServerState>) -> Json<JwkSet> {
        state.requests.fetch_add(1, Ordering::SeqCst);
        Json(state.keys.read().await.clone())
    }

    pub(crate) struct TestJwksServer {
        pub(crate) url: String,
        keys: Arc<RwLock<JwkSet>>,
        requests: Arc<AtomicUsize>,
        task: JoinHandle<()>,
    }

    impl TestJwksServer {
        pub(crate) async fn start(keys: Vec<Jwk>) -> Self {
            let listener = TcpListener::bind("127.0.0.1:0")
                .await
                .expect("test JWKS listener must bind");
            let address = listener
                .local_addr()
                .expect("test JWKS listener must have an address");
            let state = TestServerState {
                keys: Arc::new(RwLock::new(JwkSet { keys })),
                requests: Arc::new(AtomicUsize::new(0)),
            };
            let app = Router::new()
                .route("/jwks", get(serve_jwks))
                .with_state(state.clone());
            let task = tokio::spawn(async move {
                axum::serve(listener, app)
                    .await
                    .expect("test JWKS server must run");
            });

            Self {
                url: format!("http://{address}/jwks"),
                keys: state.keys,
                requests: state.requests,
                task,
            }
        }

        pub(crate) async fn replace(&self, keys: Vec<Jwk>) {
            *self.keys.write().await = JwkSet { keys };
        }

        pub(crate) fn request_count(&self) -> usize {
            self.requests.load(Ordering::SeqCst)
        }
    }

    impl Drop for TestJwksServer {
        fn drop(&mut self) {
            self.task.abort();
        }
    }
}

#[cfg(test)]
mod tests {
    use jsonwebtoken::{EncodingKey, Header};

    use super::{
        JwksVerifier, Presented, SharedSecretVerifier, TokenVerifier, VerificationError, jwt,
        test_support::{TestJwksServer, jwk, signed_token},
    };
    use crate::uar::security::claims::UserClaims;

    #[tokio::test]
    async fn verified_tenant_claim_becomes_typed_principal_identity() {
        let claims = UserClaims {
            sub: "user-123".to_owned(),
            name: None,
            roles: None,
            tenant_id: Some("tenant-a".to_owned()),
            exp: usize::MAX,
        };
        let token = jwt::encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(b"tenant-test-secret"),
        )
        .expect("tenant test token must encode");

        let principal = SharedSecretVerifier::new("tenant-test-secret")
            .verify(Presented::Jwks(token))
            .await
            .expect("verified tenant token must produce a principal");

        assert_eq!(principal.subject, "user-123");
        assert_eq!(
            principal.tenant_id.as_ref().map(|tenant| tenant.as_str()),
            Some("tenant-a")
        );
        assert_ne!(principal.subject, "tenant-a");
    }

    #[tokio::test]
    async fn accepts_two_cached_keys_and_refreshes_after_rotation() {
        let server = TestJwksServer::start(vec![jwk("key-a"), jwk("key-b")]).await;
        let verifier = JwksVerifier::new(&server.url, Some("issuer"), Some("audience")).await;

        let first = verifier
            .verify(Presented::Jwks(signed_token("key-a", "issuer", "audience")))
            .await
            .expect("first JWKS token must verify");
        assert_eq!(first.subject, "user-123");
        assert!(verifier.refreshed_at().await.is_some());
        assert_eq!(server.request_count(), 1);

        verifier
            .verify(Presented::Jwks(signed_token("key-b", "issuer", "audience")))
            .await
            .expect("second cached kid must verify");
        assert_eq!(server.request_count(), 1);

        server.replace(vec![jwk("key-c")]).await;
        verifier
            .verify(Presented::Jwks(signed_token("key-c", "issuer", "audience")))
            .await
            .expect("rotated kid must verify after one refresh");
        assert_eq!(server.request_count(), 2);
    }

    #[tokio::test]
    async fn unknown_kid_refreshes_once_then_rejects() {
        let server = TestJwksServer::start(vec![jwk("known")]).await;
        let verifier = JwksVerifier::new(&server.url, Some("issuer"), Some("audience")).await;

        let error = verifier
            .verify(Presented::Jwks(signed_token(
                "missing", "issuer", "audience",
            )))
            .await
            .err()
            .expect("unknown kid must be rejected");
        assert!(matches!(error, VerificationError::UnknownKeyId { .. }));
        assert_eq!(server.request_count(), 1);
    }

    #[tokio::test]
    async fn rejects_wrong_issuer_and_audience() {
        let server = TestJwksServer::start(vec![jwk("claims-key")]).await;
        let verifier = JwksVerifier::new(&server.url, Some("issuer"), Some("audience")).await;

        let wrong_audience = verifier
            .verify(Presented::Jwks(signed_token(
                "claims-key",
                "issuer",
                "other-audience",
            )))
            .await
            .err()
            .expect("wrong audience must be rejected");
        assert!(matches!(wrong_audience, VerificationError::Token(_)));

        let wrong_issuer = verifier
            .verify(Presented::Jwks(signed_token(
                "claims-key",
                "other-issuer",
                "audience",
            )))
            .await
            .err()
            .expect("wrong issuer must be rejected");
        assert!(matches!(wrong_issuer, VerificationError::Token(_)));
        assert_eq!(server.request_count(), 1);
    }
}
