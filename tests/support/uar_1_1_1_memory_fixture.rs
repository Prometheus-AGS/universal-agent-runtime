//! Source-only phase supplement until the lead opens the full integration gate.
//! Real loopback HTTP, production JWT verification, router, policy and memory store.
//! The ingress adapter covers issuer JWTs, not AppState API-key/delegation admission.

use axum::{
    Router,
    extract::{Request, State},
    http::StatusCode,
    middleware::{Next, from_fn_with_state},
    response::Response,
};
use reqwest::Method;
use serde_json::{Value, json};
use std::{sync::Arc, time::Duration};
use universal_agent_runtime::{
    config::SecurityConfig,
    uar::{
        api::credentials::{CredentialApiState, build_router},
        security::{
            credentials::{
                CredentialEncryption, CredentialRecord, CredentialScope, CredentialStore,
                CredentialTenancyPolicy, InMemoryCredentialStore, LegacyCredentialOwnerMapping,
                PrincipalCredentialStore, ProviderService,
            },
            middleware::authenticate_bearer_context,
        },
    },
};

const ISSUER: &str = "phase-memory-issuer";
const SIGNING_KEY: &[u8] = b"phase-memory-synthetic-signing-key";
const SOURCE_SECRET: &str = "phase-memory-private-source-canary";
const ROTATED_SECRET: &str = "phase-memory-private-rotation-canary";

fn bearer(subject: &str, tenant: &str, signing_key: &[u8]) -> String {
    let now = chrono::Utc::now().timestamp();
    let claims = json!({"iss":ISSUER,"aud":"phase-memory-runtime","sub":subject,
        "tenant_id":tenant,"roles":["user"],"uar_credential_kind":"issuer",
        "exp":now+3600,"nbf":now-5});
    jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(signing_key),
    )
    .unwrap()
}

async fn verified_ingress(
    State(config): State<Arc<SecurityConfig>>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let token = request
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .ok_or(StatusCode::UNAUTHORIZED)?;
    let context = authenticate_bearer_context(&config, token).await?;
    request.extensions_mut().insert(context);
    Ok(next.run(request).await)
}

struct HttpCredentials {
    base: String,
    client: reqwest::Client,
    task: Option<tokio::task::JoinHandle<()>>,
}

impl HttpCredentials {
    async fn start(
        security: Arc<SecurityConfig>,
        policy: Arc<CredentialTenancyPolicy>,
        service: Arc<ProviderService>,
    ) -> Self {
        let app = Router::new()
            .nest("/credentials", build_router())
            .with_state(CredentialApiState { policy, service: Some(service) })
            .layer(from_fn_with_state(security, verified_ingress));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base = format!("http://{}/credentials", listener.local_addr().unwrap());
        let task = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        let client = reqwest::Client::builder()
            .no_proxy()
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap();
        Self { base, client, task: Some(task) }
    }

    async fn call(
        &self,
        method: Method,
        path: &str,
        token: Option<&str>,
        condition: Option<&str>,
        body: Option<&Value>,
        status: u16,
    ) -> Value {
        let mut request = self.client.request(method, format!("{}{path}", self.base));
        if let Some(token) = token {
            request = request.bearer_auth(token);
        }
        if let Some(condition) = condition {
            request = request.header("If-Match", condition);
        }
        if let Some(body) = body {
            request = request.json(body);
        }
        let response = request.send().await.unwrap();
        assert_eq!(response.status().as_u16(), status, "credential HTTP status at {path}");
        let bytes = response.bytes().await.unwrap();
        let text = std::str::from_utf8(&bytes).unwrap();
        assert!(!text.contains(SOURCE_SECRET) && !text.contains(ROTATED_SECRET), "HTTP secret leak");
        if bytes.is_empty() { Value::Null } else { serde_json::from_slice(&bytes).unwrap() }
    }

    async fn preview(&self, host: &str, expected_status: &str) -> Value {
        let preview = self.call(Method::GET, "/migration-preview?limit=1", Some(host), None, None, 200).await;
        assert_eq!(preview["consistency"], "page_observation");
        assert_eq!(preview["counts"]["inventory_total"], 1);
        assert_eq!(preview["counts"]["page_total"], 1);
        assert!(preview["next_cursor"].is_null());
        let item = &preview["items"][0];
        assert_eq!(item["status"], expected_status);
        assert_eq!(item["source_kind"], "legacy");
        assert_eq!(item["source_scope_id"], "legacy-memory-owner");
        assert_eq!(item["provider_id"], "openai");
        assert!(item.get("api_key_hint").is_none());
        assert!(item.get("api_key_encrypted").is_none());
        assert!(item.get("reason").is_none());
        item.clone()
    }

    async fn close(mut self) {
        let task = self.task.take().unwrap();
        task.abort();
        assert!(task.await.unwrap_err().is_cancelled());
    }
}

impl Drop for HttpCredentials {
    fn drop(&mut self) {
        if let Some(task) = &self.task { task.abort(); }
    }
}

pub async fn conditional_migration() {
    let security: Arc<SecurityConfig> = Arc::new(serde_json::from_value(json!({
        "jwt_required":true,"jwt_secret":std::str::from_utf8(SIGNING_KEY).unwrap(),
        "jwt_issuer":ISSUER,"jwt_audience":"phase-memory-runtime","jwt_validate_nbf":true,
        "settings_mutation_auth_required":false,
        "trusted_host_principals":[{"issuer":ISSUER,"subject":"migration-host",
            "tenant_id":"tenant-a","host_id":"phase-memory-host"}]
    })).unwrap());
    let policy = Arc::new(CredentialTenancyPolicy::from_config(
        serde_json::from_value(json!({"mode":"shared"})).unwrap(),
    ).unwrap());
    // This scenario also runs alone in a fresh process, before any Node boots.
    // Acquire UAR's guarded provider before the fixture signs its first token.
    super::process::initialize_jwt_verifier(&security).await;
    let owner_token = bearer("memory-owner", "tenant-a", SIGNING_KEY);
    let foreign_token = bearer("memory-owner", "tenant-b", SIGNING_KEY);
    let host_token = bearer("migration-host", "tenant-a", SIGNING_KEY);
    let forged_token = bearer("migration-host", "tenant-a", b"wrong-synthetic-key");
    let owner_context = authenticate_bearer_context(&security, &owner_token).await.unwrap();
    let owner = policy.bind_owner(&owner_context).unwrap();
    let store = Arc::new(InMemoryCredentialStore::new());
    let encryption = Arc::new(CredentialEncryption::from_key(super::process::TEST_KEY));
    let now = chrono::Utc::now();
    store.put(CredentialRecord {
        scope: CredentialScope::User,
        scope_id: "legacy-memory-owner".to_owned(),
        provider_id: "openai".to_owned(),
        api_key_encrypted: encryption.encrypt(SOURCE_SECRET).unwrap(),
        api_key_hint: "source-hint-canary".to_owned(),
        created_at: now,
        updated_at: now,
        row_revision: None,
    }).await.unwrap();
    let first_source = store.get(CredentialScope::User, "legacy-memory-owner", "openai").await.unwrap().unwrap();
    assert!(first_source.row_revision.is_some(), "public memory writes must initialize revisions");
    let service = Arc::new(ProviderService::new(store.clone(), encryption.clone())
        .with_policy(policy.clone()).unwrap()
        .with_legacy_owner_mappings(vec![LegacyCredentialOwnerMapping {
            scope: CredentialScope::User,
            legacy_scope_id: "legacy-memory-owner".to_owned(),
            provider_id: "openai".to_owned(),
            destination_owner_key: owner.storage_key(),
            approval_id: "phase-memory-exact-approval".to_owned(),
        }]).unwrap());
    let http = HttpCredentials::start(security, policy, service).await;
    let request = json!({"scope":"user","legacy_scope_id":"legacy-memory-owner",
        "provider_id":"openai","reason":"phase memory operator-approved copy"});

    http.call(Method::GET, "/migration-preview", None, None, None, 401).await;
    http.call(Method::GET, "/migration-preview", Some(&forged_token), None, None, 401).await;
    let denied = http.call(Method::GET, "/migration-preview", Some(&owner_token), None, None, 403).await;
    assert_eq!(denied["error"]["code"], "credential_preview_denied");
    let identity = http.call(Method::GET, "/principal", Some(&owner_token), None, None, 200).await;
    assert_eq!(identity["owner_key"], owner.storage_key());
    http.call(Method::GET, "/openai", Some(&owner_token), None, None, 404).await;
    let ready = http.preview(&host_token, "ready").await;
    assert_eq!(ready["source_revision"], first_source.row_revision.as_deref().unwrap());
    assert_eq!(ready["destination_owner_key"], owner.storage_key());
    assert_eq!(ready["approval_id"], "phase-memory-exact-approval");
    assert_eq!(ready["destination_exists"], false);
    assert!(!ready.to_string().contains("source-hint-canary"));
    let after_preview = store.get(CredentialScope::User, "legacy-memory-owner", "openai").await.unwrap().unwrap();
    assert_eq!(after_preview.row_revision, first_source.row_revision);
    assert_eq!(after_preview.updated_at, first_source.updated_at);
    assert!(after_preview.api_key_encrypted == first_source.api_key_encrypted);
    let stale_tag = ready["precondition"].as_str().unwrap().to_owned();
    let denied = http.call(Method::POST, "/migrate-legacy", Some(&foreign_token), Some(&stale_tag), Some(&request), 403).await;
    assert_eq!(denied["error"]["code"], "credential_mapping_denied");

    // Identical timestamps and ciphertext still represent a new source revision.
    store.put(first_source.clone()).await.unwrap();
    let source = store.get(CredentialScope::User, "legacy-memory-owner", "openai").await.unwrap().unwrap();
    assert_eq!(source.updated_at, first_source.updated_at);
    assert_ne!(source.row_revision, first_source.row_revision);
    let stale = http.call(Method::POST, "/migrate-legacy", Some(&owner_token), Some(&stale_tag), Some(&request), 412).await;
    assert_eq!(stale["error"]["code"], "credential_migration_precondition_failed");
    let untouched = store.migration_inventory().await.unwrap();
    assert!(untouched.journals.is_empty() && untouched.principal.is_empty());
    let ready = http.preview(&host_token, "ready").await;
    let tag = ready["precondition"].as_str().unwrap().to_owned();
    assert_ne!(tag, stale_tag);

    // Every future performs real HTTP; one lock commits destination and journal.
    let replies = futures::future::join_all((0..8).map(|_| {
        http.call(Method::POST, "/migrate-legacy", Some(&owner_token), Some(&tag), Some(&request), 200)
    })).await;
    let receipt = &replies[0];
    assert!(replies.iter().all(|reply| reply == receipt), "concurrent retries changed immutable receipt");
    assert_eq!(receipt["precondition"], tag);
    assert_eq!(receipt["source_revision"], source.row_revision.as_deref().unwrap());
    let applied = store.migration_inventory().await.unwrap();
    assert_eq!(applied.journals.len(), 1);
    assert_eq!(applied.principal.len(), 1);
    assert_eq!(applied.legacy.len(), 1);
    let journal = serde_json::to_value(&applied.journals[0]).unwrap();
    let destination = store.get_principal(&owner, CredentialScope::User, owner.subject(), "openai").await.unwrap().unwrap();
    assert!(destination.api_key_encrypted == source.api_key_encrypted, "copy changed ciphertext");
    assert_eq!(destination.created_at, source.created_at);
    assert_eq!(destination.updated_at, source.updated_at);
    assert_eq!(receipt["destination_revision"], destination.row_revision.as_deref().unwrap());
    http.preview(&host_token, "already_applied").await;
    http.call(Method::GET, "/openai", Some(&foreign_token), None, None, 404).await;
    http.call(Method::POST, "/migrate-legacy", Some(&foreign_token), Some(&tag), Some(&request), 403).await;

    http.call(Method::PUT, "/openai", Some(&owner_token), None, Some(&json!({"api_key":ROTATED_SECRET})), 200).await;
    let rotated = store.get_principal(&owner, CredentialScope::User, owner.subject(), "openai").await.unwrap().unwrap();
    assert_ne!(rotated.row_revision, destination.row_revision);
    assert_eq!(rotated.created_at, destination.created_at);
    assert!(encryption.decrypt(&rotated.api_key_encrypted).unwrap() == ROTATED_SECRET);
    http.preview(&host_token, "already_applied_destination_changed").await;
    let mut retry = request.clone();
    retry["reason"] = json!("later retry must preserve original audit reason");
    assert_eq!(&http.call(Method::POST, "/migrate-legacy", Some(&owner_token), Some(&tag), Some(&retry), 200).await, receipt);
    let after_retry = store.get_principal(&owner, CredentialScope::User, owner.subject(), "openai").await.unwrap().unwrap();
    assert_eq!(after_retry.row_revision, rotated.row_revision);
    assert!(after_retry.api_key_encrypted == rotated.api_key_encrypted);

    http.call(Method::DELETE, "/openai", Some(&owner_token), None, None, 204).await;
    assert_eq!(&http.call(Method::POST, "/migrate-legacy", Some(&owner_token), Some(&tag), Some(&retry), 200).await, receipt);
    http.call(Method::GET, "/openai", Some(&owner_token), None, None, 404).await;
    http.preview(&host_token, "already_applied_destination_missing").await;
    assert!(store.delete(CredentialScope::User, "legacy-memory-owner", "openai").await.unwrap());
    assert_eq!(&http.call(Method::POST, "/migrate-legacy", Some(&owner_token), Some(&tag), Some(&retry), 200).await, receipt);
    let final_preview = http.preview(&host_token, "already_applied_destination_missing").await;
    assert!(final_preview["source_revision"].is_null());
    assert_eq!(final_preview["precondition"], tag);
    http.call(Method::GET, "/openai", Some(&owner_token), None, None, 404).await;
    http.call(Method::POST, "/migrate-legacy", Some(&owner_token), Some(&stale_tag), Some(&request), 412).await;
    let final_inventory = store.migration_inventory().await.unwrap();
    assert!(final_inventory.principal.is_empty() && final_inventory.legacy.is_empty());
    assert_eq!(final_inventory.journals.len(), 1);
    assert_eq!(serde_json::to_value(&final_inventory.journals[0]).unwrap(), journal);

    // Memory has no persisted historical rows, and its public writers initialize
    // revisions. Durable missing-revision upgrade is covered by the image adapters.
    http.close().await;
}
