use super::{super::*, support::*};

#[tokio::test]
async fn delegated_roles_must_be_held_and_allowlisted_before_insertion() {
    let storage = Arc::new(InMemoryApiKeyStorage::new());
    let service = ApiKeyService::new(storage.clone(), "test-secret");
    let user = context_for(
        &local_config(),
        "delegator",
        None,
        vec!["user".to_owned(), "reader".to_owned()],
    )
    .await;
    for role in ["reader", "host-session", "admin"] {
        let result = service
            .create_key(
                &user,
                CreateKeyRequest {
                    name: "denied".to_owned(),
                    roles: Some(vec![role.to_owned()]),
                    expires_in_secs: None,
                },
            )
            .await;
        assert!(matches!(
            result.unwrap_err().downcast_ref::<ApiKeyAuthorityError>(),
            Some(ApiKeyAuthorityError::DelegationDenied)
        ));
        assert!(storage.all_active().await.unwrap().is_empty());
    }
    let user = context_for(&local_config(), "delegator", None, vec![]).await;
    assert!(
        service
            .create_key(
                &user,
                CreateKeyRequest {
                    name: "no-default-promotion".to_owned(),
                    roles: None,
                    expires_in_secs: None,
                }
            )
            .await
            .is_err()
    );
    assert!(storage.all_active().await.unwrap().is_empty());
}

#[cfg(feature = "server")]
#[tokio::test]
async fn direct_and_exchanged_keys_retain_verified_attenuated_authority() {
    let config = remote_config();
    let storage = Arc::new(InMemoryApiKeyStorage::new());
    let service = ApiKeyService::new(storage.clone(), "test-secret")
        .with_security_config(&config)
        .unwrap();
    let user = verified_remote_caller(&config).await;
    let created = service
        .create_key(
            &user,
            CreateKeyRequest {
                name: "scoped-key".to_owned(),
                roles: Some(vec!["user".to_owned()]),
                expires_in_secs: None,
            },
        )
        .await
        .unwrap();
    let stored = storage
        .get_by_id(&created.metadata.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(stored.authority_version, 1);
    assert_eq!(stored.issuer.as_deref(), Some("key-issuer"));
    assert_eq!(stored.tenant_id.as_deref(), Some("tenant-a"));
    assert_eq!(stored.roles, vec!["user"]);
    let direct = service
        .validate_key(&created.raw_key)
        .await
        .unwrap()
        .unwrap();
    let exchanged = service
        .exchange_for_jwt(&created.raw_key)
        .await
        .unwrap()
        .unwrap();
    let exchanged = crate::uar::security::verifier::verify_token(&config, &exchanged)
        .await
        .unwrap();
    assert_eq!(direct.user_id, exchanged.subject);
    assert_eq!(direct.tenant_id, exchanged.tenant_id);
    assert_eq!(
        direct.tenant_id.as_ref().map(TenantId::as_str),
        Some("tenant-a")
    );
    assert_eq!(direct.claims.roles, Some(vec!["user".to_owned()]));
    assert_eq!(direct.claims.roles, exchanged.claims.roles);
    assert!(direct.claims.uar_instance_id.is_none());
    assert!(exchanged.claims.uar_instance_id.is_none());

    let mut legacy = stored.clone();
    legacy.authority_version = 0;
    legacy.tenant_id = None;
    storage.insert(legacy).await.unwrap();
    assert!(service.validate_key(&created.raw_key).await.is_err());
    assert!(service.exchange_for_jwt(&created.raw_key).await.is_err());
    let mut wrong_issuer = stored;
    wrong_issuer.issuer = Some("other-issuer".to_owned());
    storage.insert(wrong_issuer).await.unwrap();
    assert!(service.validate_key(&created.raw_key).await.is_err());
    assert!(service.exchange_for_jwt(&created.raw_key).await.is_err());
}

#[cfg(feature = "server")]
#[tokio::test]
async fn external_jwks_exchange_is_explicitly_unsupported() {
    let mut config = remote_config();
    config.jwks_url = Some("https://example.invalid/jwks".to_owned());
    config.jwt_algorithm = Some(crate::config::JwtAlgorithm::RS256);
    let service = ApiKeyService::new(Arc::new(InMemoryApiKeyStorage::new()), "unused")
        .with_security_config(&config)
        .unwrap();
    let error = service
        .exchange_for_jwt("not-submitted-to-any-network")
        .await
        .unwrap_err();
    assert!(matches!(
        error.downcast_ref::<ApiKeyAuthorityError>(),
        Some(ApiKeyAuthorityError::UnsupportedExchange)
    ));
    config.api_key_delegable_roles.push("admin".to_owned());
    assert!(
        ApiKeyService::new(Arc::new(InMemoryApiKeyStorage::new()), "unused")
            .with_security_config(&config)
            .is_err()
    );
}
