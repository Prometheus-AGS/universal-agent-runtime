use super::{super::*, support::*};

#[tokio::test]
async fn management_requires_owner_scope_or_exact_scoped_administrator() {
    let mut config = remote_config();
    let storage = Arc::new(InMemoryApiKeyStorage::new());
    let service = ApiKeyService::new(storage.clone(), "test-secret")
        .with_security_config(&config)
        .unwrap();
    let owner = verified_remote_caller(&config).await;
    let created = service
        .create_key(
            &owner,
            CreateKeyRequest {
                name: "owner-only".to_owned(),
                roles: None,
                expires_in_secs: None,
            },
        )
        .await
        .unwrap();
    let other = context_for(
        &config,
        "other-owner",
        Some("tenant-a"),
        vec!["user".to_owned()],
    )
    .await;
    let other_tenant = context_for(
        &config,
        "key-owner",
        Some("tenant-b"),
        vec!["user".to_owned()],
    )
    .await;
    let role_admin = context_for(
        &config,
        "manager",
        Some("tenant-a"),
        vec!["admin".to_owned()],
    )
    .await;
    for denied in [&other, &other_tenant, &role_admin] {
        assert!(service.list_keys(denied).await.unwrap().is_empty());
        assert!(
            !service
                .revoke_key(denied, &created.metadata.id)
                .await
                .unwrap()
        );
        assert!(
            !storage
                .get_by_id(&created.metadata.id)
                .await
                .unwrap()
                .unwrap()
                .revoked
        );
        assert!(
            service
                .validate_key(&created.raw_key)
                .await
                .unwrap()
                .is_some()
        );
    }
    config
        .api_key_admin_principals
        .push(crate::config::ApiKeyAdminPrincipal {
            issuer: "key-issuer".to_owned(),
            subject: "manager".to_owned(),
            tenant_id: "tenant-a".to_owned(),
        });
    config
        .workspace_authorities
        .push(crate::config::WorkspaceAuthority {
            issuer: "key-issuer".to_owned(),
            subject: "manager".to_owned(),
            tenant_id: "tenant-a".to_owned(),
            workspace_id: "workspace-a".to_owned(),
        });
    let service = ApiKeyService::new(storage.clone(), "test-secret")
        .with_security_config(&config)
        .unwrap();
    let manager = context_for(
        &config,
        "manager",
        Some("tenant-a"),
        vec!["user".to_owned()],
    )
    .await;
    assert_eq!(service.list_keys(&owner).await.unwrap().len(), 1);
    assert_eq!(service.list_keys(&manager).await.unwrap().len(), 1);
    assert!(
        service
            .revoke_key(&manager, &created.metadata.id)
            .await
            .unwrap()
    );
    assert!(
        service
            .validate_key(&created.raw_key)
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        service
            .exchange_for_jwt(&created.raw_key)
            .await
            .unwrap()
            .is_none()
    );
}

#[tokio::test]
async fn named_synthetic_and_tampered_contexts_have_no_key_authority() {
    let service = make_service();
    let mut synthetic = caller("named-user").await;
    synthetic.authority = None;
    assert!(
        service
            .create_key(
                &synthetic,
                CreateKeyRequest {
                    name: "denied".to_owned(),
                    roles: None,
                    expires_in_secs: None
                }
            )
            .await
            .is_err()
    );
    assert!(service.list_keys(&synthetic).await.is_err());
    assert!(
        service
            .revoke_key(&synthetic, "undisclosed-id")
            .await
            .is_err()
    );
    let mut tampered = caller("verified-user").await;
    tampered.claims.roles = Some(vec!["admin".to_owned()]);
    assert!(service.list_keys(&tampered).await.is_err());
    let anonymous = caller("anonymous").await;
    assert!(service.list_keys(&anonymous).await.is_err());
}

#[tokio::test]
async fn exchange_lifetime_is_bounded_and_revocation_window_is_explicit() {
    let config = remote_config();
    let storage = Arc::new(InMemoryApiKeyStorage::new());
    let service = ApiKeyService::new(storage.clone(), "test-secret")
        .with_security_config(&config)
        .unwrap()
        .with_ttl(7200);
    let owner = verified_remote_caller(&config).await;
    let created = service
        .create_key(
            &owner,
            CreateKeyRequest {
                name: "short-lived".to_owned(),
                roles: None,
                expires_in_secs: Some(90),
            },
        )
        .await
        .unwrap();
    let issued = service
        .exchange_with_expiry(&created.raw_key)
        .await
        .unwrap()
        .unwrap();
    let principal = crate::uar::security::verifier::verify_token(&config, &issued.token)
        .await
        .unwrap();
    assert!(issued.expires_in > 0 && issued.expires_in <= 90);
    assert!(principal.claims.exp as i64 <= created.metadata.expires_at.unwrap());
    assert!(
        service
            .revoke_key(&owner, &created.metadata.id)
            .await
            .unwrap()
    );
    assert!(
        service
            .validate_key(&created.raw_key)
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        service
            .exchange_for_jwt(&created.raw_key)
            .await
            .unwrap()
            .is_none()
    );
    // Revocation does not retroactively revoke already-issued self-contained JWTs.
    assert!(
        crate::uar::security::verifier::verify_token(&config, &issued.token)
            .await
            .is_ok()
    );
    let unlimited = service
        .create_key(
            &owner,
            CreateKeyRequest {
                name: "one-hour-cap".to_owned(),
                roles: None,
                expires_in_secs: None,
            },
        )
        .await
        .unwrap();
    assert!(
        service
            .exchange_with_expiry(&unlimited.raw_key)
            .await
            .unwrap()
            .unwrap()
            .expires_in
            <= 3600
    );
    let mut expired = storage
        .get_by_id(&unlimited.metadata.id)
        .await
        .unwrap()
        .unwrap();
    expired.expires_at = Some(now_unix());
    storage.insert(expired).await.unwrap();
    assert!(
        service
            .validate_key(&unlimited.raw_key)
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        service
            .exchange_for_jwt(&unlimited.raw_key)
            .await
            .unwrap()
            .is_none()
    );
}

#[tokio::test]
async fn delegated_and_legacy_credentials_cannot_inherit_cross_owner_admin_policy() {
    let mut config = remote_config();
    config
        .api_key_admin_principals
        .push(crate::config::ApiKeyAdminPrincipal {
            issuer: "key-issuer".to_owned(),
            subject: "manager".to_owned(),
            tenant_id: "tenant-a".to_owned(),
        });
    let storage = Arc::new(InMemoryApiKeyStorage::new());
    let service = ApiKeyService::new(storage.clone(), "test-secret")
        .with_security_config(&config)
        .unwrap();
    let owner = verified_remote_caller(&config).await;
    let owned = service
        .create_key(
            &owner,
            CreateKeyRequest {
                name: "foreign".to_owned(),
                roles: None,
                expires_in_secs: None,
            },
        )
        .await
        .unwrap();
    let manager = context_for(
        &config,
        "manager",
        Some("tenant-a"),
        vec!["user".to_owned()],
    )
    .await;
    let delegated = service
        .create_key(
            &manager,
            CreateKeyRequest {
                name: "manager-key".to_owned(),
                roles: None,
                expires_in_secs: None,
            },
        )
        .await
        .unwrap();
    let direct = service
        .validate_key(&delegated.raw_key)
        .await
        .unwrap()
        .unwrap();
    let token = service
        .exchange_for_jwt(&delegated.raw_key)
        .await
        .unwrap()
        .unwrap();
    let principal = crate::uar::security::verifier::verify_token(&config, &token)
        .await
        .unwrap();
    let exchanged = UserContext {
        host_authority: principal.host_authority,
        authority: principal.authority,
        user_id: principal.subject,
        tenant_id: principal.tenant_id,
        claims: principal.claims,
    };
    let legacy = context_with_kind(
        &config,
        "manager",
        Some("tenant-a"),
        vec!["user".to_owned()],
        None,
    )
    .await;
    let unknown = context_with_kind(
        &config,
        "manager",
        Some("tenant-a"),
        vec!["user".to_owned()],
        Some("unknown"),
    )
    .await;
    for denied in [&direct, &exchanged, &legacy, &unknown] {
        let visible = service.list_keys(denied).await.unwrap();
        assert_eq!(visible.len(), 1, "own key remains visible");
        assert_eq!(visible[0].id, delegated.metadata.id);
        assert!(
            !service
                .revoke_key(denied, &owned.metadata.id)
                .await
                .unwrap()
        );
        assert!(
            !storage
                .get_by_id(&owned.metadata.id)
                .await
                .unwrap()
                .unwrap()
                .revoked
        );
    }
    assert_eq!(service.list_keys(&manager).await.unwrap().len(), 2);
    assert!(
        service
            .revoke_key(&manager, &owned.metadata.id)
            .await
            .unwrap()
    );
}
