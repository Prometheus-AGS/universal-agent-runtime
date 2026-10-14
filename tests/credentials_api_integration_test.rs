//! Principal credential acceptance through real runtime processes and durable DBs.
//! Product verification stays local. No ambient .env or provider keys are used.
#![cfg(all(feature = "server", feature = "surreal-backend"))]

#[path = "support/credential_process.rs"]
mod process;

use axum_test::TestServer;
use process::{Database, Node, Provider, TEST_KEY, authenticated, call, token};
use reqwest::Method;
use serde_json::json;
use std::sync::Arc;
use universal_agent_runtime::uar::security::{
    claims::{CredentialPrincipal, UserClaims, UserContext},
    credentials::{
        CredentialEncryption, CredentialRecord, CredentialScope, CredentialStore,
        InMemoryCredentialStore, LegacyCredentialOwnerMapping, PrincipalCredentialStore,
        ProviderService, SurrealCredentialStore,
    },
};

#[tokio::test]
async fn synthetic_user_context_never_confers_credential_ownership() {
    let context = UserContext {
        host_authority: None,
        authority: None,
        user_id: "same".to_owned(),
        tenant_id: None,
        claims: UserClaims {
            sub: "same".to_owned(),
            name: None,
            roles: None,
            tenant_id: None,
            uar_instance_id: None,
            exp: usize::MAX,
        },
    };
    let service = Arc::new(ProviderService::new(
        Arc::new(InMemoryCredentialStore::new()),
        Arc::new(CredentialEncryption::from_key(TEST_KEY)),
    ));
    let app = axum::Router::new()
        .nest(
            "/credentials",
            universal_agent_runtime::uar::api::credentials::build_router(),
        )
        .with_state(Some(service))
        .layer(axum::Extension(context));
    let server = TestServer::new(app);
    server
        .put("/credentials/openai")
        .json(&json!({"api_key":"synthetic-secret"}))
        .await
        .assert_status(axum::http::StatusCode::UNAUTHORIZED);
    server
        .get("/credentials")
        .await
        .assert_status(axum::http::StatusCode::UNAUTHORIZED);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn local_surreal_kv_api_resolution_survives_process_restart() {
    let provider = Provider::start().await;
    let database = Database::local();
    let first = Node::boot(&database, &provider, None, &[], None).await;
    let alice = token(None, None, "alice");
    let owner = call(&first, &alice, Method::GET, "/principal", None, 200).await;
    assert_eq!(owner["issuer"], "urn:uar:local:jwt");
    call(
        &first,
        &alice,
        Method::PUT,
        "/openai",
        Some(json!({"api_key":"credential-local-1111"})),
        200,
    )
    .await;
    let created = call(&first, &alice, Method::GET, "/openai", None, 200).await;
    assert_eq!(created["api_key_hint"], "1111");
    provider
        .prove_run(
            &first,
            &alice,
            "credential-local-1111",
            "local-before-restart",
        )
        .await;
    first.stop().await;
    let second = Node::boot(&database, &provider, None, &[], None).await;
    assert_eq!(
        call(&second, &alice, Method::GET, "", None, 200)
            .await
            .as_array()
            .unwrap()
            .len(),
        1
    );
    provider
        .prove_run(
            &second,
            &alice,
            "credential-local-1111",
            "local-after-restart",
        )
        .await;
    call(
        &second,
        &alice,
        Method::PUT,
        "/openai",
        Some(json!({"api_key":"credential-local-9999"})),
        200,
    )
    .await;
    let rotated = call(&second, &alice, Method::GET, "/openai", None, 200).await;
    assert_eq!(created["created_at"], rotated["created_at"]);
    provider
        .prove_run(&second, &alice, "credential-local-9999", "local-rotated")
        .await;
    call(&second, &alice, Method::DELETE, "/openai", None, 204).await;
    call(&second, &alice, Method::GET, "/openai", None, 404).await;
    provider
        .prove_run(&second, &alice, "credential-test-house", "local-deleted")
        .await;
    second.stop().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires UAR_CREDENTIAL_TEST_SURREAL_URL pointing to a real durable networked SurrealDB"]
async fn shared_surreal_principals_migrations_and_two_runtimes() {
    let provider = Provider::start().await;
    let database = Database::shared().await;
    let subjects = ["same".to_owned()];
    let first = Node::boot(
        &database,
        &provider,
        Some("credential-issuer-a"),
        &subjects,
        None,
    )
    .await;
    let second = Node::boot(
        &database,
        &provider,
        Some("credential-issuer-a"),
        &subjects,
        None,
    )
    .await;
    let other_issuer = Node::boot(
        &database,
        &provider,
        Some("credential-issuer-b"),
        &subjects,
        None,
    )
    .await;
    let a = token(Some("credential-issuer-a"), Some("tenant-a"), "same");
    let b = token(Some("credential-issuer-a"), Some("tenant-b"), "same");
    let c = token(Some("credential-issuer-b"), Some("tenant-a"), "same");
    call(
        &first,
        &a,
        Method::PUT,
        "/openai",
        Some(json!({"api_key":"credential-tenant-a-1111"})),
        200,
    )
    .await;
    for (node, credential) in [(&second, &b), (&other_issuer, &c)] {
        assert!(
            call(node, credential, Method::GET, "", None, 200)
                .await
                .as_array()
                .unwrap()
                .is_empty()
        );
        call(node, credential, Method::GET, "/openai", None, 404).await;
        call(node, credential, Method::DELETE, "/openai", None, 404).await;
        provider
            .prove_run(node, credential, "credential-test-house", "foreign-empty")
            .await;
    }
    call(
        &second,
        &b,
        Method::PUT,
        "/openai",
        Some(json!({"api_key":"credential-tenant-b-2222"})),
        200,
    )
    .await;
    call(
        &other_issuer,
        &c,
        Method::PUT,
        "/openai",
        Some(json!({"api_key":"credential-issuer-b-3333"})),
        200,
    )
    .await;
    provider
        .prove_run(
            &second,
            &a,
            "credential-tenant-a-1111",
            "same-owner-other-runtime",
        )
        .await;
    provider
        .prove_run(&second, &b, "credential-tenant-b-2222", "second-tenant")
        .await;
    provider
        .prove_run(
            &other_issuer,
            &c,
            "credential-issuer-b-3333",
            "second-issuer",
        )
        .await;
    call(
        &first,
        &a,
        Method::PUT,
        "/openai",
        Some(json!({"api_key":"credential-tenant-a-9999"})),
        200,
    )
    .await;
    provider
        .prove_run(
            &second,
            &a,
            "credential-tenant-a-9999",
            "rotation-observed-on-second-runtime",
        )
        .await;
    call(&first, &a, Method::DELETE, "/openai", None, 204).await;
    provider
        .prove_run(
            &second,
            &a,
            "credential-test-house",
            "deletion-observed-on-second-runtime",
        )
        .await;

    let owner = call(&first, &a, Method::GET, "/principal", None, 200).await;
    let owner_key = owner["owner_key"].as_str().unwrap().to_owned();
    let raw_subject = Node::boot(
        &database,
        &provider,
        Some("credential-issuer-a"),
        &[owner_key.clone()],
        None,
    )
    .await;
    let raw = token(Some("credential-issuer-a"), Some("tenant-a"), &owner_key);
    let store = Arc::new(SurrealCredentialStore::new(database.connect().await));
    let encryption = CredentialEncryption::from_key(TEST_KEY);
    let legacy = record(
        CredentialScope::User,
        "same",
        "legacy",
        &encryption,
        "credential-legacy-4444",
    );
    store.put(legacy.clone()).await.unwrap();
    store
        .put(record(
            CredentialScope::User,
            &owner_key,
            "collision",
            &encryption,
            "credential-collision-5555",
        ))
        .await
        .unwrap();
    call(&raw_subject, &raw, Method::GET, "/collision", None, 404).await;
    call(&first, &a, Method::GET, "/collision", None, 404).await;
    call(&first, &a, Method::GET, "/legacy", None, 404).await;
    let request = json!({"scope":"user", "legacy_scope_id":"same", "provider_id":"legacy", "reason":"verified legacy owner evidence"});
    call(
        &first,
        &a,
        Method::POST,
        "/migrate-legacy",
        Some(request.clone()),
        403,
    )
    .await;
    let mapping = LegacyCredentialOwnerMapping {
        scope: CredentialScope::User,
        legacy_scope_id: "same".to_owned(),
        provider_id: "legacy".to_owned(),
        destination_owner_key: owner_key,
        approval_id: "operator-reviewed-evidence-1".to_owned(),
    };
    let mapped = Node::boot(
        &database,
        &provider,
        Some("credential-issuer-a"),
        &subjects,
        Some(vec![mapping]),
    )
    .await;
    call(
        &mapped,
        &b,
        Method::POST,
        "/migrate-legacy",
        Some(request.clone()),
        403,
    )
    .await;
    let receipt = call(
        &mapped,
        &a,
        Method::POST,
        "/migrate-legacy",
        Some(request.clone()),
        200,
    )
    .await;
    assert_eq!(
        receipt,
        call(
            &mapped,
            &a,
            Method::POST,
            "/migrate-legacy",
            Some(request.clone()),
            200
        )
        .await
    );
    let principal = authenticated(Some("credential-issuer-a"), Some("tenant-a"), "same").await;
    let copied = store
        .get_principal(&principal, CredentialScope::User, "same", "legacy")
        .await
        .unwrap()
        .unwrap();
    assert!(
        copied.api_key_encrypted == legacy.api_key_encrypted,
        "migration must preserve ciphertext"
    );
    assert_eq!(copied.created_at, legacy.created_at);
    let service = ProviderService::new(store, Arc::new(encryption));
    let resolved = service
        .resolver()
        .resolve(&principal, "legacy")
        .await
        .unwrap()
        .unwrap();
    use secrecy::ExposeSecret;
    assert!(resolved.api_key.expose_secret() == "credential-legacy-4444");
    call(&mapped, &a, Method::DELETE, "/legacy", None, 204).await;
    call(
        &mapped,
        &a,
        Method::POST,
        "/migrate-legacy",
        Some(request),
        200,
    )
    .await;
    call(&mapped, &a, Method::GET, "/legacy", None, 404).await;
    let conflicting = Node::boot(
        &database,
        &provider,
        Some("credential-issuer-a"),
        &subjects,
        Some(vec![LegacyCredentialOwnerMapping {
            scope: CredentialScope::User,
            legacy_scope_id: "same".to_owned(),
            provider_id: "legacy".to_owned(),
            destination_owner_key:
                call(&second, &b, Method::GET, "/principal", None, 200).await["owner_key"]
                    .as_str()
                    .unwrap()
                    .to_owned(),
            approval_id: "operator-reviewed-evidence-2".to_owned(),
        }]),
    )
    .await;
    call(&conflicting, &b, Method::POST, "/migrate-legacy", Some(json!({"scope":"user","legacy_scope_id":"same","provider_id":"legacy","reason":"conflicting mapping"})), 409).await;
    prove_scope_precedence(
        &service,
        &principal,
        &authenticated(Some("credential-issuer-a"), Some("tenant-b"), "same").await,
    )
    .await;
    prove_destination_conflict(&service, &principal).await;
    for node in [
        first,
        second,
        other_issuer,
        raw_subject,
        mapped,
        conflicting,
    ] {
        node.stop().await;
    }
}

fn record(
    scope: CredentialScope,
    scope_id: &str,
    provider: &str,
    encryption: &CredentialEncryption,
    key: &str,
) -> CredentialRecord {
    let now = chrono::Utc::now();
    CredentialRecord {
        scope,
        scope_id: scope_id.to_owned(),
        provider_id: provider.to_owned(),
        api_key_encrypted: encryption.encrypt(key).unwrap(),
        api_key_hint: CredentialEncryption::key_hint(key, 4),
        created_at: now,
        updated_at: now,
    }
}

async fn prove_scope_precedence(
    service: &ProviderService,
    a: &CredentialPrincipal,
    b: &CredentialPrincipal,
) {
    for (scope, id, key) in [
        (CredentialScope::Session, "shared-session", "session-key"),
        (CredentialScope::Agent, "shared-agent", "agent-key"),
        (CredentialScope::User, a.subject(), "user-key"),
    ] {
        service
            .store()
            .put_principal(
                a,
                record(scope, id, "precedence", service.encryption(), key),
            )
            .await
            .unwrap();
    }
    let resolver = service.resolver();
    assert_eq!(
        resolver
            .resolve_with_context(
                a,
                "precedence",
                Some("shared-session"),
                Some("shared-agent")
            )
            .await
            .unwrap()
            .unwrap()
            .scope,
        CredentialScope::Session
    );
    assert!(
        resolver
            .resolve_with_context(
                b,
                "precedence",
                Some("shared-session"),
                Some("shared-agent")
            )
            .await
            .unwrap()
            .is_none()
    );
    service
        .store()
        .delete_principal(a, CredentialScope::Session, "shared-session", "precedence")
        .await
        .unwrap();
    assert_eq!(
        resolver
            .resolve_with_context(
                a,
                "precedence",
                Some("shared-session"),
                Some("shared-agent")
            )
            .await
            .unwrap()
            .unwrap()
            .scope,
        CredentialScope::Agent
    );
    service
        .store()
        .delete_principal(a, CredentialScope::Agent, "shared-agent", "precedence")
        .await
        .unwrap();
    assert_eq!(
        resolver
            .resolve(a, "precedence")
            .await
            .unwrap()
            .unwrap()
            .scope,
        CredentialScope::User
    );
}

async fn prove_destination_conflict(service: &ProviderService, principal: &CredentialPrincipal) {
    let provider = format!("occupied-{}", uuid::Uuid::new_v4());
    let existing = record(
        CredentialScope::User,
        principal.subject(),
        &provider,
        service.encryption(),
        "existing-approved-secret",
    );
    service
        .store()
        .put_principal(principal, existing.clone())
        .await
        .unwrap();
    service
        .store()
        .put(record(
            CredentialScope::User,
            "unmapped-legacy",
            &provider,
            service.encryption(),
            "different-legacy-secret",
        ))
        .await
        .unwrap();
    let mapped = ProviderService::new(service.store().clone(), service.encryption().clone())
        .with_legacy_owner_mappings(vec![LegacyCredentialOwnerMapping {
            scope: CredentialScope::User,
            legacy_scope_id: "unmapped-legacy".to_owned(),
            provider_id: provider.clone(),
            destination_owner_key: principal.storage_key(),
            approval_id: "occupied-destination-approval".to_owned(),
        }])
        .unwrap();
    let error = mapped
        .migrate_legacy_credential(
            principal,
            CredentialScope::User,
            "unmapped-legacy",
            &provider,
            "operator conflict control",
        )
        .await
        .unwrap_err();
    assert!(matches!(error.downcast_ref::<universal_agent_runtime::uar::security::credentials::CredentialMigrationError>(),
        Some(universal_agent_runtime::uar::security::credentials::CredentialMigrationError::Conflict)));
    let after = service
        .store()
        .get_principal(
            principal,
            CredentialScope::User,
            principal.subject(),
            &provider,
        )
        .await
        .unwrap()
        .unwrap();
    assert!(
        existing.api_key_encrypted == after.api_key_encrypted,
        "conflict overwrote destination"
    );
}

#[cfg(feature = "sqlx")]
#[tokio::test]
#[ignore = "requires UAR_CREDENTIAL_TEST_POSTGRES_URL pointing to an isolated real PostgreSQL database"]
async fn postgres_principal_scope_and_atomic_legacy_mapping() {
    use universal_agent_runtime::uar::security::credentials::PostgresCredentialStore;
    let url = std::env::var("UAR_CREDENTIAL_TEST_POSTGRES_URL")
        .expect("explicit isolated Postgres URL required");
    let pool = sqlx::PgPool::connect(&url).await.unwrap();
    sqlx::raw_sql(include_str!(
        "../migrations/20260703000000_provider_credentials.sql"
    ))
    .execute(&pool)
    .await
    .unwrap();
    sqlx::raw_sql(include_str!(
        "../migrations/20261011000000_principal_credentials.sql"
    ))
    .execute(&pool)
    .await
    .unwrap();
    let a = authenticated(Some("credential-issuer-a"), Some("tenant-a"), "same").await;
    let b = authenticated(Some("credential-issuer-a"), Some("tenant-b"), "same").await;
    let c = authenticated(Some("credential-issuer-b"), Some("tenant-a"), "same").await;
    let provider = format!("postgres-{}", uuid::Uuid::new_v4());
    let encryption = Arc::new(CredentialEncryption::from_key(TEST_KEY));
    let store = Arc::new(PostgresCredentialStore::new(pool.clone()));
    let legacy = record(
        CredentialScope::User,
        "same",
        &provider,
        &encryption,
        "postgres-legacy-7777",
    );
    store.put(legacy.clone()).await.unwrap();
    let service = ProviderService::new(store.clone(), encryption)
        .with_legacy_owner_mappings(vec![LegacyCredentialOwnerMapping {
            scope: CredentialScope::User,
            legacy_scope_id: "same".to_owned(),
            provider_id: provider.clone(),
            destination_owner_key: a.storage_key(),
            approval_id: "postgres-operator-approval".to_owned(),
        }])
        .unwrap();
    assert!(
        service
            .resolver()
            .resolve(&a, &provider)
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        service
            .migrate_legacy_credential(&b, CredentialScope::User, "same", &provider, "bad tenant")
            .await
            .is_err()
    );
    let (first, second) = tokio::join!(
        service.migrate_legacy_credential(
            &a,
            CredentialScope::User,
            "same",
            &provider,
            "owner evidence"
        ),
        service.migrate_legacy_credential(
            &a,
            CredentialScope::User,
            "same",
            &provider,
            "owner evidence"
        )
    );
    assert_eq!(
        serde_json::to_value(first.unwrap()).unwrap(),
        serde_json::to_value(second.unwrap()).unwrap()
    );
    let migrated = store
        .get_principal(&a, CredentialScope::User, "same", &provider)
        .await
        .unwrap()
        .unwrap();
    assert!(migrated.api_key_encrypted == legacy.api_key_encrypted);
    for foreign in [&b, &c] {
        assert!(
            store
                .get_principal(foreign, CredentialScope::User, "same", &provider)
                .await
                .unwrap()
                .is_none()
        );
        assert!(
            store
                .list_principal(foreign, CredentialScope::User, "same")
                .await
                .unwrap()
                .is_empty()
        );
        assert!(
            !store
                .delete_principal(foreign, CredentialScope::User, "same", &provider)
                .await
                .unwrap()
        );
    }
    prove_scope_precedence(&service, &a, &b).await;
    prove_destination_conflict(&service, &a).await;
    store
        .delete_principal(&a, CredentialScope::User, "same", &provider)
        .await
        .unwrap();
    service
        .migrate_legacy_credential(
            &a,
            CredentialScope::User,
            "same",
            &provider,
            "resume completed mapping",
        )
        .await
        .unwrap();
    assert!(
        store
            .get_principal(&a, CredentialScope::User, "same", &provider)
            .await
            .unwrap()
            .is_none()
    );
    pool.close().await;
}
