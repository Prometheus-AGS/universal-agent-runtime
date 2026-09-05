//! Integration tests for the per-user provider credential API.
//!
//! Exercises the HTTP endpoints under `/credentials` using `axum-test` — no
//! external infrastructure, no LLM calls. Storage is the in-memory credential
//! store; encryption uses a fixed test key.
//!
//! Coverage (maps to `provider-credentials → Credential CRUD API` scenarios):
//!  - Anonymous request → 401
//!  - Service disabled (no ProviderService) → 503
//!  - Store (PUT) then list (GET) returns masked metadata, never plaintext
//!  - Rotate (second PUT) changes the stored key (hint updates)
//!  - Delete → 204, then 404 on a second delete
//!  - A second user does not see the first user's credentials (isolation)

use std::sync::Arc;

use axum_test::TestServer;
use serde_json::{Value, json};

use universal_agent_runtime::uar::security::claims::{UserClaims, UserContext};
use universal_agent_runtime::uar::security::credentials::{
    CredentialEncryption, InMemoryCredentialStore, ProviderService,
};

const TEST_KEY: &[u8; 32] = b"0123456789abcdef0123456789abcdef";

/// Build a credentials server with the given user identity injected as an
/// `Extension<UserContext>` (simulating the auth middleware). Pass `None` for
/// an anonymous server.
fn server_for(user: Option<&str>, service: Option<Arc<ProviderService>>) -> TestServer {
    let mut app = axum::Router::new()
        .nest(
            "/credentials",
            universal_agent_runtime::uar::api::credentials::build_router(),
        )
        .with_state(service);

    if let Some(uid) = user {
        let ctx = UserContext {
            user_id: uid.to_string(),
            tenant_id: None,
            claims: UserClaims {
                sub: uid.to_string(),
                name: None,
                roles: Some(vec!["user".to_string()]),
                tenant_id: None,
                uar_instance_id: None,
                exp: usize::MAX,
            },
        };
        app = app.layer(axum::Extension(ctx));
    }

    TestServer::new(app)
}

fn test_service() -> Arc<ProviderService> {
    let store = Arc::new(InMemoryCredentialStore::new());
    let enc = Arc::new(CredentialEncryption::from_key(TEST_KEY));
    Arc::new(ProviderService::new(store, enc))
}

// ---------------------------------------------------------------------------

#[tokio::test]
async fn anonymous_request_is_rejected() {
    let server = server_for(None, Some(test_service()));
    let res = server.get("/credentials").await;
    res.assert_status(axum::http::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn service_disabled_returns_503() {
    let server = server_for(Some("alice"), None);
    let res = server.get("/credentials").await;
    res.assert_status(axum::http::StatusCode::SERVICE_UNAVAILABLE);
}

#[tokio::test]
async fn store_then_list_returns_masked_metadata_never_plaintext() {
    let server = server_for(Some("alice"), Some(test_service()));

    let put = server
        .put("/credentials/openai")
        .json(&json!({ "api_key": "sk-secret-TAIL1234" }))
        .await;
    put.assert_status_ok();

    let list = server.get("/credentials").await;
    list.assert_status_ok();
    let body: Value = list.json();
    let arr = body.as_array().expect("array");
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["provider_id"], "openai");
    assert_eq!(arr[0]["api_key_hint"], "1234");

    // The plaintext key must never appear anywhere in the serialized body.
    let raw = list.text();
    assert!(
        !raw.contains("sk-secret-TAIL1234"),
        "plaintext key leaked in response"
    );
    assert!(
        !raw.contains("api_key_encrypted"),
        "ciphertext field leaked in response"
    );
}

#[tokio::test]
async fn rotate_changes_stored_key() {
    let server = server_for(Some("alice"), Some(test_service()));

    server
        .put("/credentials/openai")
        .json(&json!({ "api_key": "sk-first-0001" }))
        .await
        .assert_status_ok();
    server
        .put("/credentials/openai")
        .json(&json!({ "api_key": "sk-second-9999" }))
        .await
        .assert_status_ok();

    let list = server.get("/credentials").await;
    let body: Value = list.json();
    let arr = body.as_array().unwrap();
    assert_eq!(arr.len(), 1, "rotate replaces, not appends");
    assert_eq!(arr[0]["api_key_hint"], "9999", "hint reflects rotated key");
}

#[tokio::test]
async fn delete_then_second_delete_is_not_found() {
    let server = server_for(Some("alice"), Some(test_service()));

    server
        .put("/credentials/openai")
        .json(&json!({ "api_key": "sk-xyz-4321" }))
        .await
        .assert_status_ok();

    server
        .delete("/credentials/openai")
        .await
        .assert_status(axum::http::StatusCode::NO_CONTENT);

    server
        .delete("/credentials/openai")
        .await
        .assert_status(axum::http::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn second_user_does_not_see_first_users_credentials() {
    let service = test_service();

    server_for(Some("alice"), Some(Arc::clone(&service)))
        .put("/credentials/openai")
        .json(&json!({ "api_key": "sk-alice-1111" }))
        .await
        .assert_status_ok();

    // Bob shares the same backing service but must see none of Alice's keys.
    let bob = server_for(Some("bob"), Some(service));
    let list = bob.get("/credentials").await;
    list.assert_status_ok();
    let body: Value = list.json();
    assert_eq!(
        body.as_array().unwrap().len(),
        0,
        "cross-user isolation violated"
    );
}
