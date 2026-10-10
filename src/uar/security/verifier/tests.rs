use jsonwebtoken::{EncodingKey, Header};

use super::{
    JwksVerifier, Presented, SharedSecretVerifier, TokenVerifier, VerificationError,
    jwks_cache::CacheError,
    jwt,
    test_support::{TestJwksServer, jwk, rotated_jwk, rotated_signed_token, signed_token},
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
