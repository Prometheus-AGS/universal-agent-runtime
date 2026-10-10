use super::{super::*, support::*};
use jsonwebtoken::{Algorithm, DecodingKey, Validation};

#[tokio::test]
async fn test_create_and_exchange() {
    let svc = make_service();
    let resp = svc
        .create_key(
            &caller("user-1").await,
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
async fn exchanged_jwt_contains_configured_issuer_and_audience() {
    #[derive(Deserialize)]
    struct RegisteredClaims {
        iss: String,
        aud: String,
    }

    let storage: Arc<dyn ApiKeyStorage> = Arc::new(InMemoryApiKeyStorage::new());
    let svc = ApiKeyService::new(storage, "test-secret").with_registered_claims(
        Some("uar-issuer".to_owned()),
        Some("uar-clients".to_owned()),
    );
    let created = svc
        .create_key(
            &context_for(
                &{
                    let mut c = local_config();
                    c.jwt_issuer = Some("uar-issuer".to_owned());
                    c.jwt_audience = Some("uar-clients".to_owned());
                    c
                },
                "user-claims",
                None,
                vec!["user".to_owned()],
            )
            .await,
            CreateKeyRequest {
                name: "registered-claims".to_owned(),
                roles: None,
                expires_in_secs: None,
            },
        )
        .await
        .expect("API key must be created");
    let token = svc
        .exchange_for_jwt(&created.raw_key)
        .await
        .expect("API key exchange must run")
        .expect("API key exchange must mint a JWT");

    let mut validation = Validation::new(Algorithm::HS256);
    validation.set_issuer(&["uar-issuer"]);
    validation.set_audience(&["uar-clients"]);
    let decoded = jwt::decode::<RegisteredClaims>(
        token,
        &DecodingKey::from_secret(b"test-secret"),
        &validation,
    )
    .expect("the exchanged JWT must satisfy configured registered claims");
    assert_eq!(decoded.claims.iss, "uar-issuer");
    assert_eq!(decoded.claims.aud, "uar-clients");
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
            &caller("user-2").await,
            CreateKeyRequest {
                name: "revoke-me".to_string(),
                roles: None,
                expires_in_secs: None,
            },
        )
        .await
        .unwrap();

    svc.revoke_key(&caller("user-2").await, &resp.metadata.id)
        .await
        .unwrap();
    let jwt = svc.exchange_for_jwt(&resp.raw_key).await.unwrap();
    assert!(jwt.is_none(), "revoked key should not produce JWT");
}

#[tokio::test]
async fn test_list_keys() {
    let svc = make_service();
    svc.create_key(
        &caller("user-3").await,
        CreateKeyRequest {
            name: "k1".to_string(),
            roles: None,
            expires_in_secs: None,
        },
    )
    .await
    .unwrap();
    svc.create_key(
        &caller("user-3").await,
        CreateKeyRequest {
            name: "k2".to_string(),
            roles: None,
            expires_in_secs: None,
        },
    )
    .await
    .unwrap();

    let keys = svc.list_keys(&caller("user-3").await).await.unwrap();
    assert_eq!(keys.len(), 2);
}

#[tokio::test]
async fn test_validate_key_directly() {
    let svc = make_service();
    let resp = svc
        .create_key(
            &caller("user-4").await,
            CreateKeyRequest {
                name: "direct-validate".to_string(),
                roles: Some(vec!["user".to_string()]),
                expires_in_secs: None,
            },
        )
        .await
        .unwrap();

    let claims = svc.validate_key(&resp.raw_key).await.unwrap();
    assert!(claims.is_some());
    let claims = claims.unwrap();
    assert_eq!(claims.user_id, "user-4");
    assert_eq!(claims.claims.roles.unwrap(), vec!["user"]);
}
