//! Dot12 real-store production-boundary contract. Authoring is not acceptance.
//! Run only after the complete expanded Dot implementation inventory is recorded.
#![cfg(all(feature = "server", feature = "surreal-backend"))]

use std::sync::Arc;
use axum::{Router, extract::Request, middleware::Next, response::{IntoResponse, Response}};
use serde_json::json;
use universal_agent_runtime::{
    config::{MemoryConfig, SecurityConfig},
    mcp::registry::NativeTool,
    uar::{
        memory::{MemoryScope, MemoryType, service::MemoryService, mcp_server::memory_mcp_router},
        runtime::{actor::messages::ActorOwner, native_skill::NativeExecutionContext},
        security::{claims::UserContext, middleware::authenticate_bearer_context},
        tools::memory::{MemorySaveTool, MemoryDeleteTool, MemoryHistoryTool, MemoryRecallTool},
    },
};

const SECRET: &str = "dot12-integration-only-signing-secret";

fn security(issuer: &str) -> SecurityConfig {
    serde_json::from_value(json!({
        "jwt_required": true, "jwt_secret": SECRET, "jwt_issuer": issuer,
        "trusted_host_principals": if issuer == "issuer-a" {
            json!([{"issuer":"issuer-a","subject":"operator","tenant_id":"operations","host_id":"dot12-host"}])
        } else { json!([]) }
    })).unwrap()
}

async fn identity(issuer: &str, tenant: &str, subject: &str, host: bool) -> (String, UserContext) {
    let config = security(issuer);
    // Enter the production verifier before signing to install its crypto provider.
    let _ = authenticate_bearer_context(&config, "invalid").await;
    let mut claims = json!({"iss":issuer,"sub":subject,"tenant_id":tenant,"roles":["user"],"exp":chrono::Utc::now().timestamp()+3600});
    if host { claims["uar_credential_kind"] = json!("issuer"); }
    let token = jsonwebtoken::encode(&jsonwebtoken::Header::default(), &claims,
        &jsonwebtoken::EncodingKey::from_secret(SECRET.as_bytes())).unwrap();
    let context = authenticate_bearer_context(&config, &token).await.unwrap();
    (token, context)
}

async fn ingress(mut request: Request, next: Next) -> Response {
    let Some(token) = request.headers().get("authorization").and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer ")) else {
        return axum::http::StatusCode::UNAUTHORIZED.into_response();
    };
    let context = match authenticate_bearer_context(&security("issuer-a"), token).await {
        Ok(context) => context,
        Err(status) => return status.into_response(),
    };
    request.extensions_mut().insert(context);
    next.run(request).await
}

fn id(memory: &surreal_memory::Memory) -> String {
    let record = memory.id.as_ref().unwrap();
    let surrealdb::types::RecordIdKey::String(key) = &record.key else { panic!("fixture needs string memory key") };
    format!("memory:{key}")
}

fn execution(context: &UserContext) -> NativeExecutionContext {
    NativeExecutionContext { verified_owner: Some(ActorOwner::from_verified_context(context).unwrap()), session_id: Some("same-session".into()), ..Default::default() }
}

async fn exercise(service: Arc<MemoryService>, other_runtime: Arc<MemoryService>) {
    let (alice_token, alice) = identity("issuer-a", "tenant-a", "same", false).await;
    let (bob_token, bob) = identity("issuer-a", "tenant-b", "same", false).await;
    let (_, other_issuer) = identity("issuer-b", "tenant-a", "same", false).await;
    let (_, administrator) = identity("issuer-a", "operations", "operator", true).await;
    let marker = format!("alice-private-{}", uuid::Uuid::new_v4());
    let own = service.add(&marker, MemoryScope::Global, MemoryType::Semantic, &alice,
        Some("same-agent"), Some("same-session"), vec![], None, 0.8).await.unwrap();
    let own_id = id(&own);
    let mut synthetic = alice.clone();
    synthetic.authority = None;
    synthetic.host_authority = None;
    assert!(service.list(&synthetic, None, None).await.is_err());
    assert_eq!(other_runtime.get(&own_id, &alice).await.unwrap().unwrap().content, marker);
    for foreign in [&bob, &other_issuer] {
        assert!(other_runtime.get(&own_id, foreign).await.unwrap().is_none());
        assert!(other_runtime.list(foreign, None, None).await.unwrap().is_empty());
        assert!(other_runtime.search(&marker, foreign, None, None, 10, None).await.unwrap().is_empty());
        assert!(other_runtime.hybrid_search(&marker, foreign, None, None, 10, None, None).await.unwrap().is_empty());
        assert!(other_runtime.history(&own_id, foreign).await.is_err());
        assert!(other_runtime.update(&own_id, "foreign overwrite".into(), foreign).await.is_err());
        assert!(other_runtime.delete(&own_id, foreign).await.is_err());
    }
    let native = MemorySaveTool::new(Some(Arc::clone(&service)));
    assert!(native.call(json!({"content":"unbound write","user_id":"same"})).await.is_err());
    assert!(native.call_with_context(json!({"content":"owner spoof","user_id":"other"}), &execution(&alice)).await.is_err());
    let saved = native.call_with_context(json!({"content":"native owner-bound record"}), &execution(&alice)).await.unwrap();
    assert!(!saved["memory_id"].as_str().unwrap().is_empty());
    let native_delete = MemoryDeleteTool::new(Some(Arc::clone(&service)));
    assert!(native_delete.call_with_context(json!({"memory_id":own_id}), &execution(&bob)).await.is_err());
    let native_history = MemoryHistoryTool::new(Some(Arc::clone(&service)));
    assert!(native_history.call_with_context(json!({"memory_id":own_id}), &execution(&other_issuer)).await.is_err());
    let native_recall = MemoryRecallTool::new(Some(Arc::clone(&service)));
    assert_eq!(native_recall.call_with_context(json!({"query":marker}), &execution(&bob)).await.unwrap(), json!([]));
    service.set_capture_policy(&alice, universal_agent_runtime::uar::memory::controls::CapturePolicyUpdate { enabled: true, retention: Some(universal_agent_runtime::uar::memory::controls::Retention::UntilDeleted) }).await.unwrap();
    let capture = service.auto_capture(vec![json!({"role":"user","content":"remember my personal violet notebook"})], &alice, Some("same-agent"), Some("same-session")).await.unwrap();
    assert!(!capture.is_empty());
    assert!(service.list(&bob, None, None).await.unwrap().is_empty());
    assert!(service.add("forged metadata", MemoryScope::User, MemoryType::Semantic, &alice, None, None,
        vec![], Some(json!({"uar_migration":{"history":[]}})), 0.5).await.is_err());
    let context = universal_agent_runtime::uar::memory::context_builder::build_context_with_hits(
        &service, &marker, &alice, None, None, "default").await;
    assert!(!context.hits.is_empty());
    assert!(context.block.contains("untrusted recalled data"));
    let foreign_context = universal_agent_runtime::uar::memory::context_builder::build_context_with_hits(
        &service, &marker, &bob, None, None, "default").await;
    assert!(foreign_context.hits.is_empty());
    assert!(service.admin_storage(&alice).is_err());
    assert!(service.admin_storage(&administrator).is_ok());

    // Seed quarantined legacy data using a separate SDK session on the same
    // production engine. This also works for embedded SurrealKV without a second lock.
    let storage = service.admin_storage(&administrator).unwrap();
    let storage = storage.as_any().downcast_ref::<surreal_memory::SurrealStorage>().unwrap();
    let legacy = storage.db().unwrap();
    legacy.use_db(service.config().database.clone()).await.unwrap();
    surreal_memory::storage::migrations::run_migrations(&legacy).await.unwrap();
    let legacy_id = format!("legacy_{}", uuid::Uuid::new_v4().simple());
    legacy.query("CREATE type::record('memory', $id) SET content = 'quarantined private record', user_id = 'same', scope = 'user', memory_type = 'semantic', categories = [], importance = 0.5, access_count = 0, version = 1, created_at = time::now(), updated_at = time::now()")
        .bind(("id", legacy_id.clone())).await.unwrap().check().unwrap();
    assert!(service.get(&format!("memory:{legacy_id}"), &alice).await.unwrap().is_none());
    legacy.query("CREATE memory_history SET memory_id = type::record('memory', $id), version = 1, old_content = NONE, new_content = 'quarantined private record', changed_at = time::now(), change_type = 'created'")
        .bind(("id", legacy_id.clone())).await.unwrap().check().unwrap();
    let encoded_subject_id = format!("{legacy_id}_encoded");
    legacy.query("CREATE type::record('memory', $id) SET content = 'encoded-looking subject is still legacy', user_id = $owner, scope = 'user', memory_type = 'semantic', categories = [], importance = 0.5, access_count = 0, version = 1, created_at = time::now(), updated_at = time::now()")
        .bind(("id", encoded_subject_id.clone())).bind(("owner", service.for_context(&alice).unwrap().owner_key()))
        .await.unwrap().check().unwrap();
    assert!(service.import_legacy_memory(&administrator, &encoded_subject_id, &alice, "not-a-mapping").await.is_err());
    let imported = service.import_legacy_memory(&administrator, &legacy_id, &alice, "operator-proof-a").await.unwrap();
    assert_eq!(imported.content, "quarantined private record");
    assert!(service.history(&id(&imported), &alice).await.unwrap().len() >= 2);
    assert!(service.history(&id(&imported), &bob).await.is_err());
    let replay = service.import_legacy_memory(&administrator, &legacy_id, &alice, "operator-proof-a").await.unwrap();
    assert_eq!(replay.id, imported.id);
    assert!(service.import_legacy_memory(&administrator, &legacy_id, &bob, "conflicting-proof").await.is_err());
    assert!(service.import_legacy_memory(&alice, &legacy_id, &alice, "not-an-admin").await.is_err());
    assert!(service.get(&id(&imported), &bob).await.unwrap().is_none());

    // Use the actual MCP router and transport. Session replay is checked before
    // dispatch, not merely in memory tool functions.
    let app = Router::new().nest("/memory", memory_mcp_router(Arc::clone(&service)))
        .layer(axum::middleware::from_fn(ingress));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap(); });
    let endpoint = format!("http://{address}/memory/");
    let client = reqwest::Client::new();
    let initialized = client.post(&endpoint).bearer_auth(&alice_token)
        .header("accept", "application/json, text/event-stream")
        .json(&json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"dot12-contract","version":"1"}}}))
        .send().await.unwrap();
    assert!(initialized.status().is_success());
    let session = initialized.headers().get("mcp-session-id").unwrap().to_str().unwrap().to_owned();
    let _ = initialized.text().await.unwrap();
    for method in [reqwest::Method::GET, reqwest::Method::DELETE, reqwest::Method::POST] {
        let denied = client.request(method, &endpoint).bearer_auth(&bob_token)
            .header("mcp-session-id", &session).header("accept", "application/json, text/event-stream")
            .json(&json!({"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"memory_get","arguments":{"id":own_id}}}))
            .send().await.unwrap();
        assert_eq!(denied.status(), reqwest::StatusCode::FORBIDDEN);
    }
    let own_read = client.post(&endpoint).bearer_auth(&alice_token)
        .header("mcp-session-id", &session).header("mcp-protocol-version", "2025-03-26")
        .header("accept", "application/json, text/event-stream")
        .json(&json!({"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"memory_get","arguments":{"id":own_id}}}))
        .send().await.unwrap();
    assert!(own_read.status().is_success());
    assert!(own_read.text().await.unwrap().contains(&marker));
    server.abort();
    let _ = server.await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn local_surreal_kv_owner_boundary() {
    let directory = tempfile::tempdir().unwrap();
    let mut config = MemoryConfig::default();
    config.db_path = directory.path().join("memory").display().to_string();
    config.namespace = "dot12_local".into();
    config.database = "memory".into();
    let service = Arc::new(MemoryService::new(config).await.unwrap());
    exercise(Arc::clone(&service), service).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn shared_surreal_two_runtime_owner_boundary() {
    let mut config = MemoryConfig::default();
    config.surreal_endpoint = Some(std::env::var("UAR_DOT_MEMORY_SURREAL_URL").expect("explicit isolated durable SurrealDB required"));
    config.surreal_user = std::env::var("UAR_DOT_MEMORY_SURREAL_USER").ok();
    config.surreal_pass = std::env::var("UAR_DOT_MEMORY_SURREAL_PASS").ok();
    config.namespace = format!("dot12_{}", uuid::Uuid::new_v4().simple());
    config.database = "memory".into();
    let first = Arc::new(MemoryService::new(config.clone()).await.unwrap());
    let second = Arc::new(MemoryService::new(config).await.unwrap());
    exercise(first, second).await;
}
