use base64::Engine as _;
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use serde_json::{Value, json};

use super::{
    JwksVerifier, Presented, SharedSecretVerifier, TokenVerifier, VerificationError,
    jwks_cache::CacheError,
    jwt,
    test_support::{
        TestJwksServer, access_claims, jwk, now_secs, rotated_jwk, rotated_rsa_signed,
        rotated_signed_token, rsa_header, rsa_signed, signed_token,
    },
};
use crate::uar::security::claims::UserClaims;

#[tokio::test]
async fn verified_tenant_claim_becomes_typed_principal_identity() {
    let claims = UserClaims {
        sub: "user-123".to_owned(),
        name: None,
        roles: None,
        tenant_id: Some("tenant-a".to_owned()),
        uar_instance_id: None,
        exp: usize::MAX,
    };
    let token = jwt::encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(b"tenant-test-secret"),
    )
    .expect("tenant test token must encode");

    let principal = SharedSecretVerifier::new("tenant-test-secret", None, None, true)
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
    let verifier = JwksVerifier::new(&server.url, Some("issuer"), Some("audience"), true).await;

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
    advance_cache_clock(5).await;
    verifier
        .verify(Presented::Jwks(signed_token("key-c", "issuer", "audience")))
        .await
        .expect("rotated kid must verify after one refresh");
    assert_eq!(server.request_count(), 2);
}

#[tokio::test]
async fn unknown_kid_refreshes_once_then_rejects() {
    let server = TestJwksServer::start(vec![jwk("known")]).await;
    let verifier = JwksVerifier::new(&server.url, Some("issuer"), Some("audience"), true).await;

    let error = verifier
        .verify(Presented::Jwks(signed_token(
            "missing", "issuer", "audience",
        )))
        .await
        .err()
        .expect("unknown kid must be rejected");
    assert!(matches!(
        error,
        VerificationError::Jwks(CacheError::UnknownKeyId)
    ));
    assert_eq!(server.request_count(), 1);
}

#[tokio::test]
async fn rejects_wrong_issuer_and_audience() {
    let server = TestJwksServer::start(vec![jwk("claims-key")]).await;
    let verifier = JwksVerifier::new(&server.url, Some("issuer"), Some("audience"), true).await;

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

async fn advance_cache_clock(seconds: u64) {
    // Resume before real loopback I/O so timers do not auto-advance during I/O.
    tokio::time::pause();
    tokio::time::advance(std::time::Duration::from_secs(seconds)).await;
    tokio::time::resume();
}

#[tokio::test]
async fn target_refresh_removes_absent_keys_and_replaces_same_kid_material() {
    let server = TestJwksServer::start(vec![jwk("same"), jwk("removed")]).await;
    let verifier = JwksVerifier::new(&server.url, Some("issuer"), Some("audience"), true).await;
    verifier
        .verify(Presented::Jwks(signed_token("same", "issuer", "audience")))
        .await
        .unwrap();
    server.replace(vec![rotated_jwk("same")]).await;
    advance_cache_clock(60).await;
    verifier
        .verify(Presented::Jwks(rotated_signed_token(
            "same", "issuer", "audience",
        )))
        .await
        .unwrap();
    assert!(
        verifier
            .verify(Presented::Jwks(signed_token("same", "issuer", "audience")))
            .await
            .is_err()
    );
    assert!(
        verifier
            .verify(Presented::Jwks(signed_token(
                "removed", "issuer", "audience"
            )))
            .await
            .is_err()
    );
    assert_eq!(server.request_count(), 2);
}

#[tokio::test]
async fn failed_refresh_never_renews_age_and_hard_age_denies() {
    let server = TestJwksServer::start(vec![jwk("known")]).await;
    let verifier = JwksVerifier::new(&server.url, Some("issuer"), Some("audience"), true).await;
    verifier
        .verify(Presented::Jwks(signed_token("known", "issuer", "audience")))
        .await
        .unwrap();
    let original = verifier.refreshed_at().await;
    server.fail(true);
    advance_cache_clock(60).await;
    verifier
        .verify(Presented::Jwks(signed_token("known", "issuer", "audience")))
        .await
        .unwrap();
    assert_eq!(verifier.refreshed_at().await, original);
    advance_cache_clock(240).await;
    assert!(matches!(
        verifier
            .verify(Presented::Jwks(signed_token("known", "issuer", "audience")))
            .await,
        Err(VerificationError::Jwks(CacheError::Stale))
    ));
    assert_eq!(verifier.refreshed_at().await, original);
    server.fail(false);
    advance_cache_clock(5).await;
    verifier
        .verify(Presented::Jwks(signed_token("known", "issuer", "audience")))
        .await
        .unwrap();
    assert!(verifier.refreshed_at().await > original);
}

#[tokio::test]
async fn concurrent_unknown_keys_share_one_refresh_and_retry_interval() {
    let server = TestJwksServer::start(vec![jwk("known")]).await;
    server.delay(std::time::Duration::from_millis(25)).await;
    let verifier = JwksVerifier::new(&server.url, Some("issuer"), Some("audience"), true).await;
    let (a, b, c) = tokio::join!(
        verifier.verify(Presented::Jwks(signed_token(
            "unknown-a",
            "issuer",
            "audience"
        ))),
        verifier.verify(Presented::Jwks(signed_token(
            "unknown-b",
            "issuer",
            "audience"
        ))),
        verifier.verify(Presented::Jwks(signed_token("known", "issuer", "audience")))
    );
    assert!(a.is_err() && b.is_err() && c.is_ok());
    assert_eq!(server.request_count(), 1);
    assert_eq!(server.max_active(), 1);
    assert!(
        verifier
            .verify(Presented::Jwks(signed_token(
                "unknown-c",
                "issuer",
                "audience"
            )))
            .await
            .is_err()
    );
    assert_eq!(server.request_count(), 1);
    advance_cache_clock(5).await;
    assert!(
        verifier
            .verify(Presented::Jwks(signed_token(
                "unknown-c",
                "issuer",
                "audience"
            )))
            .await
            .is_err()
    );
    assert_eq!(server.request_count(), 2);
}

#[tokio::test]
async fn production_verifier_bounds_construction_and_delayed_fetch() {
    let server = TestJwksServer::start(vec![jwk("delayed")]).await;
    server.delay(std::time::Duration::from_secs(10)).await;
    let config: crate::config::SecurityConfig = serde_json::from_value(serde_json::json!({
        "jwt_required":true, "jwt_secret":"unused", "jwks_url":server.url,
        "jwt_issuer":"issuer", "jwt_audience":"audience", "jwt_validate_nbf":true,
        "settings_mutation_auth_required":false
    }))
    .unwrap();
    let start = tokio::time::Instant::now();
    let result = super::verify_token(&config, &signed_token("delayed", "issuer", "audience")).await;
    assert!(matches!(
        result,
        Err(VerificationError::Jwks(CacheError::Timeout))
    ));
    assert!(start.elapsed() < std::time::Duration::from_secs(6));
    assert_eq!(server.request_count(), 1);
}

// ── #422 frozen validator contract (J01-J09 at the verifier boundary) ──────

const CLAIMS_SECRET: &[u8] = b"claims-secret";

fn hs_token(claims: &Value) -> String {
    jwt::encode(&Header::default(), claims, &EncodingKey::from_secret(CLAIMS_SECRET))
        .expect("claims test token must encode")
}

fn b64url(bytes: &[u8]) -> String {
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}

/// Replace the payload segment while keeping the original header and signature.
fn with_payload(token: &str, claims: &Value) -> String {
    let mut parts = token.split('.');
    let header = parts.next().expect("compact JWT has a header");
    let signature = parts.nth(1).expect("compact JWT has a signature");
    format!("{header}.{}.{signature}", b64url(claims.to_string().as_bytes()))
}

async fn verifier_for(server: &TestJwksServer) -> JwksVerifier {
    JwksVerifier::new(&server.url, Some("issuer"), Some("audience"), true).await
}

/// Whether the shared-secret and the JWKS verifier each accept `claims`
/// (signed for the configured `issuer`/`audience`, JWKS kid `claims-key`).
async fn accepted_on_both_paths(server: &TestJwksServer, claims: &Value) -> (bool, bool) {
    let shared = SharedSecretVerifier::new("claims-secret", Some("issuer"), Some("audience"), true)
        .verify(Presented::Jwks(hs_token(claims)))
        .await
        .is_ok();
    let jwks = verifier_for(server)
        .await
        .verify(Presented::Jwks(rsa_signed(&rsa_header("claims-key"), claims)))
        .await
        .is_ok();
    (shared, jwks)
}

fn with_claim(claim: &str, value: Value) -> Value {
    let mut claims = access_claims("issuer", "audience");
    claims[claim] = value;
    claims
}

fn without_claim(claim: &str) -> Value {
    let mut claims = access_claims("issuer", "audience");
    claims
        .as_object_mut()
        .expect("access claims are a JSON object")
        .remove(claim);
    claims
}

#[tokio::test]
async fn j02_tampered_payload_with_original_signature_is_rejected() {
    let forged = with_claim("sub", json!("admin"));

    let server = TestJwksServer::start(vec![jwk("tamper-key")]).await;
    let verifier = verifier_for(&server).await;
    let original = signed_token("tamper-key", "issuer", "audience");
    verifier
        .verify(Presented::Jwks(original.clone()))
        .await
        .expect("the untampered JWKS token must verify");
    let error = verifier
        .verify(Presented::Jwks(with_payload(&original, &forged)))
        .await
        .err()
        .expect("a JWKS token with an altered payload must be rejected");
    assert!(matches!(error, VerificationError::Token(_)));

    let shared = SharedSecretVerifier::new("claims-secret", Some("issuer"), Some("audience"), true);
    let original = hs_token(&access_claims("issuer", "audience"));
    shared
        .verify(Presented::Jwks(original.clone()))
        .await
        .expect("the untampered shared-secret token must verify");
    let error = shared
        .verify(Presented::Jwks(with_payload(&original, &forged)))
        .await
        .err()
        .expect("a shared-secret token with an altered payload must be rejected");
    assert!(matches!(error, VerificationError::Token(_)));
}

#[tokio::test]
async fn j03_alg_none_is_rejected_on_both_verifiers() {
    let server = TestJwksServer::start(vec![jwk("none-key")]).await;
    let verifier = verifier_for(&server).await;
    let shared = SharedSecretVerifier::new("claims-secret", Some("issuer"), Some("audience"), true);
    let payload = b64url(access_claims("issuer", "audience").to_string().as_bytes());
    for header in [
        r#"{"alg":"none","typ":"JWT"}"#,
        r#"{"alg":"none","typ":"JWT","kid":"none-key"}"#,
        r#"{"alg":"None","kid":"none-key"}"#,
    ] {
        let unsigned = format!("{}.{payload}.", b64url(header.as_bytes()));
        let jwks_error = verifier
            .verify(Presented::Jwks(unsigned.clone()))
            .await
            .err()
            .expect("an unsigned JWKS token must be rejected");
        assert!(matches!(jwks_error, VerificationError::Token(_)));
        let shared_error = shared
            .verify(Presented::Jwks(unsigned))
            .await
            .err()
            .expect("an unsigned shared-secret token must be rejected");
        assert!(matches!(shared_error, VerificationError::Token(_)));
    }
    assert_eq!(server.request_count(), 0, "an unsigned token must not trigger a key fetch");
}

/// ES256/ES384 key modes, RSA/EC key-type substitution and HS256 on the JWKS
/// path are already covered through gRPC ingress by
/// `tests/jwks_es256_verification.rs`; this adds the allowlist edge and the
/// shared-secret path.
#[tokio::test]
async fn j03_disallowed_algorithms_are_rejected_on_both_verifiers() {
    let server = TestJwksServer::start(vec![jwk("rsa-key")]).await;
    let verifier = verifier_for(&server).await;
    let claims = access_claims("issuer", "audience");
    for algorithm in [Algorithm::RS384, Algorithm::PS256] {
        let mut header = Header::new(algorithm);
        header.kid = Some("rsa-key".to_owned());
        let error = verifier
            .verify(Presented::Jwks(rsa_signed(&header, &claims)))
            .await
            .err()
            .expect("an algorithm outside the JWKS allowlist must be rejected");
        assert!(matches!(
            error,
            VerificationError::UnsupportedAlgorithm(found) if found == algorithm
        ));
    }
    assert_eq!(server.request_count(), 0, "a disallowed algorithm must not trigger a key fetch");

    let shared = SharedSecretVerifier::new("claims-secret", Some("issuer"), Some("audience"), true);
    for algorithm in [Algorithm::RS256, Algorithm::PS256] {
        let mut header = Header::new(algorithm);
        header.kid = Some("rsa-key".to_owned());
        let error = shared
            .verify(Presented::Jwks(rsa_signed(&header, &claims)))
            .await
            .err()
            .expect("an asymmetric token must be rejected by the shared-secret verifier");
        assert!(matches!(error, VerificationError::Token(_)));
    }
}

#[tokio::test]
async fn j04_missing_malformed_or_expired_exp_is_rejected() {
    let server = TestJwksServer::start(vec![jwk("claims-key")]).await;
    for claims in [
        without_claim("exp"),
        with_claim("exp", json!("tomorrow")),
        with_claim("exp", json!(now_secs() - 3_600)),
    ] {
        assert_eq!(accepted_on_both_paths(&server, &claims).await, (false, false), "{claims}");
    }
}

#[tokio::test]
async fn j04_missing_or_malformed_required_iss_and_aud_are_rejected() {
    let server = TestJwksServer::start(vec![jwk("claims-key")]).await;
    for claims in [
        without_claim("iss"),
        without_claim("aud"),
        with_claim("iss", json!(42)),
        with_claim("iss", Value::Null),
        with_claim("aud", json!({"not": "an audience"})),
        with_claim("aud", json!(7)),
    ] {
        assert_eq!(accepted_on_both_paths(&server, &claims).await, (false, false), "{claims}");
    }
    assert_eq!(
        accepted_on_both_paths(&server, &access_claims("issuer", "audience")).await,
        (true, true)
    );
}

/// Boundary semantics of the fixed 60-second leeway: `exp` is rejected only
/// when `exp < now - 60`; `nbf` is rejected only when `nbf > now + 60`. The
/// margins (45 s inside, 75 s outside) keep the test independent of the
/// second-granularity clock read during verification.
#[tokio::test]
async fn j05_exp_inside_sixty_second_leeway_is_accepted_and_outside_is_rejected() {
    let server = TestJwksServer::start(vec![jwk("claims-key")]).await;
    let inside = with_claim("exp", json!(now_secs() - 45));
    assert_eq!(accepted_on_both_paths(&server, &inside).await, (true, true));
    let outside = with_claim("exp", json!(now_secs() - 75));
    assert_eq!(accepted_on_both_paths(&server, &outside).await, (false, false));
}

#[tokio::test]
async fn j05_nbf_absent_or_inside_leeway_is_accepted_and_outside_is_rejected() {
    let server = TestJwksServer::start(vec![jwk("claims-key")]).await;
    let absent = access_claims("issuer", "audience");
    assert!(absent.get("nbf").is_none());
    assert_eq!(accepted_on_both_paths(&server, &absent).await, (true, true));
    let inside = with_claim("nbf", json!(now_secs() + 45));
    assert_eq!(accepted_on_both_paths(&server, &inside).await, (true, true));
    let outside = with_claim("nbf", json!(now_secs() + 75));
    assert_eq!(accepted_on_both_paths(&server, &outside).await, (false, false));
}

#[tokio::test]
async fn j06_audience_array_membership_is_accepted_and_non_membership_rejected() {
    let server = TestJwksServer::start(vec![jwk("claims-key")]).await;
    let member = with_claim("aud", json!(["other-audience", "audience"]));
    assert_eq!(accepted_on_both_paths(&server, &member).await, (true, true));
    for claims in [
        with_claim("aud", json!(["other-audience", "another-audience"])),
        with_claim("aud", json!([])),
        with_claim("aud", json!("other-audience")),
        with_claim("iss", json!("other-issuer")),
    ] {
        assert_eq!(accepted_on_both_paths(&server, &claims).await, (false, false), "{claims}");
    }
}

#[tokio::test]
async fn j07_token_jku_and_x5u_headers_never_redirect_key_lookup() {
    let trusted = TestJwksServer::start(vec![jwk("trusted-key")]).await;
    let attacker =
        TestJwksServer::start(vec![rotated_jwk("attacker-key"), rotated_jwk("trusted-key")])
            .await;
    let verifier = verifier_for(&trusted).await;
    let claims = access_claims("issuer", "audience");
    let pointing_at_attacker = |kid: &str| {
        let mut header = rsa_header(kid);
        header.jku = Some(attacker.url.clone());
        header.x5u = Some(attacker.url.clone());
        header
    };

    let error = verifier
        .verify(Presented::Jwks(rotated_rsa_signed(&pointing_at_attacker("attacker-key"), &claims)))
        .await
        .err()
        .expect("a key published only at a token-supplied URL must be unknown");
    assert!(matches!(error, VerificationError::Jwks(CacheError::UnknownKeyId)));
    assert!(!error.is_key_source_unavailable());

    let error = verifier
        .verify(Presented::Jwks(rotated_rsa_signed(&pointing_at_attacker("trusted-key"), &claims)))
        .await
        .err()
        .expect("an unrelated key must never verify a trusted kid");
    assert!(matches!(error, VerificationError::Token(_)));

    verifier
        .verify(Presented::Jwks(rsa_signed(&pointing_at_attacker("trusted-key"), &claims)))
        .await
        .expect("a correctly signed token verifies with the configured key; jku/x5u are ignored");

    assert_eq!(attacker.request_count(), 0, "a token-supplied URL must never be fetched");
    assert_eq!(trusted.request_count(), 1);
}

#[tokio::test]
async fn j07_unavailable_configured_jwks_is_classified_unavailable() {
    let server = TestJwksServer::start(vec![jwk("down-key")]).await;
    server.fail(true);
    let verifier = verifier_for(&server).await;
    let error = verifier
        .verify(Presented::Jwks(signed_token("down-key", "issuer", "audience")))
        .await
        .err()
        .expect("an unavailable JWKS must never verify");
    assert!(matches!(error, VerificationError::Jwks(CacheError::Unavailable)));
    assert!(error.is_key_source_unavailable());
    for cache_error in [CacheError::Stale, CacheError::Timeout, CacheError::InvalidKeySet] {
        assert!(VerificationError::Jwks(cache_error).is_key_source_unavailable());
    }
    assert!(!VerificationError::Jwks(CacheError::UnknownKeyId).is_key_source_unavailable());
}

#[tokio::test]
async fn j08_malformed_compact_jwts_are_rejected() {
    let server = TestJwksServer::start(vec![jwk("malformed-key")]).await;
    let verifier = verifier_for(&server).await;
    let shared = SharedSecretVerifier::new("claims-secret", None, None, true);
    let valid = signed_token("malformed-key", "issuer", "audience");
    let segments: Vec<&str> = valid.split('.').collect();
    for token in [
        String::new(),
        "not-a-jwt".to_owned(),
        format!("{}.{}", segments[0], segments[1]),
        format!("{valid}.extra"),
        "!!!.@@@.###".to_owned(),
        format!("{}.%%%.{}", segments[0], segments[2]),
        format!("{}.{}.", segments[0], segments[1]),
    ] {
        assert!(verifier.verify(Presented::Jwks(token.clone())).await.is_err(), "{token}");
        assert!(shared.verify(Presented::Jwks(token.clone())).await.is_err(), "{token}");
    }
}

#[tokio::test]
async fn j08_explicit_non_access_typ_is_rejected_before_key_lookup() {
    let server = TestJwksServer::start(vec![jwk("typ-key")]).await;
    let verifier = verifier_for(&server).await;
    let shared = SharedSecretVerifier::new("claims-secret", Some("issuer"), Some("audience"), true);
    let claims = access_claims("issuer", "audience");
    for typ in ["JOSE", "dpop+jwt", "logout+jwt", "secevent+jwt", "application/id_token+jwt", ""] {
        let mut header = rsa_header("typ-key");
        header.typ = Some(typ.to_owned());
        let error = verifier
            .verify(Presented::Jwks(rsa_signed(&header, &claims)))
            .await
            .err()
            .expect("a non-access token type must be rejected on the JWKS path");
        assert!(matches!(error, VerificationError::UnsupportedTokenType), "{typ}");

        let header = Header { typ: Some(typ.to_owned()), ..Header::default() };
        let token = jwt::encode(&header, &claims, &EncodingKey::from_secret(CLAIMS_SECRET))
            .expect("typ test token must encode");
        let error = shared
            .verify(Presented::Jwks(token))
            .await
            .err()
            .expect("a non-access token type must be rejected on the shared-secret path");
        assert!(matches!(error, VerificationError::UnsupportedTokenType), "{typ}");
    }
    assert_eq!(server.request_count(), 0, "a wrong-purpose token must not trigger a key fetch");
}

#[tokio::test]
async fn j08_access_token_typ_values_and_absent_typ_are_accepted() {
    let server = TestJwksServer::start(vec![jwk("typ-key")]).await;
    let verifier = verifier_for(&server).await;
    let claims = access_claims("issuer", "audience");
    for typ in [
        None,
        Some("JWT"),
        Some("jwt"),
        Some("application/jwt"),
        Some("at+jwt"),
        Some("AT+JWT"),
        Some("application/at+jwt"),
    ] {
        let mut header = rsa_header("typ-key");
        header.typ = typ.map(str::to_owned);
        verifier
            .verify(Presented::Jwks(rsa_signed(&header, &claims)))
            .await
            .unwrap_or_else(|error| panic!("typ {typ:?} must be accepted: {error}"));
    }
}
