//! Authored real-store scenarios; execute only at the full expanded Dot boundary.
//! Child processes give local KV a joined process-exit/reopen boundary. No mocks.
#![cfg(all(feature = "server", feature = "surreal-backend"))]
use std::{path::Path, process::{Command, Child}, sync::Arc, time::{Duration, Instant}};
use serde_json::json;
use universal_agent_runtime::{config::{MemoryConfig, SecurityConfig}, mcp::registry::NativeTool, uar::{
    memory::{MemoryService, MemoryScope, MemoryType, controls::{CapturePolicyUpdate, Retention}},
    security::{claims::UserContext, middleware::authenticate_bearer_context},
    runtime::{actor::messages::ActorOwner, native_skill::NativeExecutionContext},
    tools::memory::{MemoryPolicyTool, MemoryRecallTool},
}};

async fn owner() -> UserContext {
    let security: SecurityConfig = serde_json::from_value(json!({"jwt_required":true,"jwt_secret":"dot13-fixture-only","jwt_issuer":"dot13",
        "trusted_host_principals":[{"issuer":"dot13","subject":"owner","tenant_id":"test","host_id":"fixture"}]})).unwrap();
    let _ = authenticate_bearer_context(&security, "invalid").await;
    let token = jsonwebtoken::encode(&jsonwebtoken::Header::default(), &json!({"iss":"dot13","sub":"owner","tenant_id":"test",
        "uar_credential_kind":"issuer","exp":chrono::Utc::now().timestamp()+3600}), &jsonwebtoken::EncodingKey::from_secret(b"dot13-fixture-only")).unwrap();
    authenticate_bearer_context(&security, &token).await.unwrap()
}
fn id(memory: &surreal_memory::Memory) -> String {
    let record = memory.id.as_ref().unwrap();
    let surrealdb::types::RecordIdKey::String(key) = &record.key else { panic!("string record key required") };
    format!("memory:{key}")
}
fn enable(retention: Retention) -> CapturePolicyUpdate { CapturePolicyUpdate { enabled:true, retention:Some(retention) } }
fn messages(marker: &str) -> Vec<serde_json::Value> { vec![json!({"role":"user","content":marker})] }
fn ready(directory: &Path) {
    let started = Instant::now();
    while !directory.join("ready").exists() {
        assert!(started.elapsed() < Duration::from_secs(180), "capture process failed to admit work");
        std::thread::sleep(Duration::from_millis(20));
    }
}
fn child(directory: &Path, action: &str) -> Child {
    Command::new(std::env::current_exe().unwrap()).args(["--exact","durable_controls","--nocapture"])
        .env("UAR_DOT13_CHILD", action).env("UAR_DOT13_DIRECTORY", directory)
        .spawn().unwrap()
}
fn join(mut process: Child) { assert!(process.wait().unwrap().success()); }

async fn scenario(directory: &Path, action: &str) {
    let config: MemoryConfig = serde_json::from_slice(&std::fs::read(directory.join("config.json")).unwrap()).unwrap();
    let service = Arc::new(MemoryService::new(config).await.unwrap());
    let user = owner().await;
    let storage = service.admin_storage(&user).unwrap();
    let database = storage.as_any().downcast_ref::<surreal_memory::SurrealStorage>().unwrap().db().unwrap();
    match action {
        "seed" => {
            assert!(!service.capture_policy(&user).await.unwrap().enabled);
            assert!(service.auto_capture(messages("no consent"), &user, None, None).await.unwrap().is_empty());
            for input in [json!({"enabled":true}), json!({"enabled":true,"retention":{"kind":"days","days":8}}),
                json!({"enabled":true,"retention":{"kind":"until_deleted","days":7}})] {
                match serde_json::from_value::<CapturePolicyUpdate>(input) {
                    Ok(update) => assert!(service.set_capture_policy(&user, update).await.is_err()),
                    Err(_) => {},
                }
            }
            let explicit = service.add("explicit stays without capture consent", MemoryScope::User, MemoryType::Semantic,
                &user, None, None, vec![], None, 0.5).await.unwrap();
            service.set_capture_policy(&user, enable(Retention::UntilDeleted)).await.unwrap();
            let captured = service.auto_capture(messages("automatic private capture"), &user, None, Some("session")).await.unwrap();
            let capture_id = id(&captured[0]);
            let first_anchor = captured[0].metadata.as_ref().unwrap()["uar_capture"]["first_capture"].clone();
            let retried = service.auto_capture(messages("automatic private capture"), &user, None, Some("session")).await.unwrap();
            assert_eq!(retried[0].id, captured[0].id);
            assert_eq!(retried[0].metadata.as_ref().unwrap()["uar_capture"]["first_capture"], first_anchor);
            let queued = service.capture_admission(&user).await.unwrap().unwrap();
            service.set_capture_policy(&user, CapturePolicyUpdate { enabled:false, retention:None }).await.unwrap();
            service.set_capture_policy(&user, enable(Retention::Days { days:30 })).await.unwrap();
            assert!(service.auto_capture_admitted(messages("stale admitted queue"), &user, None, None, &queued).await.unwrap().is_empty());
            // Fixture-only clock placement avoids sleeping seven days. Production
            // shortening and every subsequent read still use the real database.
            database.query("UPDATE type::record('uar_memory_control',$id) SET first_capture=time::now()-8d, expires_at=NONE")
                .bind(("id", capture_id.clone())).await.unwrap().check().unwrap();
            service.set_capture_policy(&user, enable(Retention::Days { days:7 })).await.unwrap();
            service.set_capture_policy(&user, enable(Retention::UntilDeleted)).await.unwrap();
            assert!(service.get(&capture_id, &user).await.unwrap().is_none());
            assert!(service.history(&capture_id, &user).await.is_err());
            let deleted = service.add("erase this content and its earlier history", MemoryScope::User, MemoryType::Semantic,
                &user, None, None, vec![], None, 0.5).await.unwrap();
            let deleted_id = id(&deleted);
            service.update(&deleted_id, "new content also deleted".into(), &user).await.unwrap();
            // The Dot12 import embeds old history in metadata: the tombstone
            // must remove it without putting that content into a deletion audit.
            database.query("UPDATE $id SET metadata=$metadata")
                .bind(("id", deleted.id.unwrap())).bind(("metadata", json!({"uar_migration":{"history":[{"old_content":"legacy erased"}]}})))
                .await.unwrap().check().unwrap();
            service.delete(&deleted_id, &user).await.unwrap();
            assert!(service.get(&deleted_id, &user).await.unwrap().is_none());
            let history: Vec<surreal_memory::MemoryHistory> = database.query("SELECT * FROM memory_history WHERE memory_id=$id")
                .bind(("id", surrealdb::types::RecordId::parse_simple(&deleted_id).unwrap())).await.unwrap().take(0).unwrap();
            assert!(history.is_empty());
            assert_eq!(service.memory_control_status(&user, &deleted_id).await.unwrap().unwrap()["purged"], false);
            std::fs::write(directory.join("ids.json"), serde_json::to_vec(&json!({"explicit":id(&explicit),"capture":capture_id,"deleted":deleted_id})).unwrap()).unwrap();
            // Process ends with pending physical purge. Reopen must finish it.
        },
        "reopen" => {
            let ids: serde_json::Value = serde_json::from_slice(&std::fs::read(directory.join("ids.json")).unwrap()).unwrap();
            for key in ["capture","deleted"] {
                let id = ids[key].as_str().unwrap();
                assert!(service.get(id, &user).await.unwrap().is_none());
                assert!(service.history(id, &user).await.is_err());
                assert_eq!(service.memory_control_status(&user, id).await.unwrap().unwrap()["purged"], true);
                assert!(storage.get_memory(id).await.unwrap().is_none());
            }
            assert!(service.get(ids["explicit"].as_str().unwrap(), &user).await.unwrap().is_some());
            assert!(service.auto_capture(messages("automatic private capture"), &user, None, Some("session")).await.unwrap().is_empty());
            let execution = NativeExecutionContext { verified_owner:Some(ActorOwner::from_verified_context(&user).unwrap()), ..Default::default() };
            let policy = MemoryPolicyTool::new(Some(Arc::clone(&service))).call_with_context(json!({}), &execution).await.unwrap();
            assert_eq!(policy["enabled"], true);
            let recalled = MemoryRecallTool::new(Some(Arc::clone(&service))).call_with_context(json!({"query":"automatic private capture"}), &execution).await.unwrap();
            assert!(!recalled.to_string().contains("automatic private capture"));
            for days in [7,30,90,365] {
                service.set_capture_policy(&user, enable(Retention::Days { days })).await.unwrap();
                let memory = service.auto_capture(messages(&format!("boundary {days}")), &user, None, None).await.unwrap().remove(0);
                let memory_id = id(&memory);
                let metadata = &memory.metadata.as_ref().unwrap()["uar_capture"];
                let first = chrono::DateTime::parse_from_rfc3339(metadata["first_capture"].as_str().unwrap()).unwrap();
                let expiry = chrono::DateTime::parse_from_rfc3339(metadata["effective_expires_at"].as_str().unwrap()).unwrap();
                assert_eq!((expiry-first).num_seconds(), i64::from(days)*86400);
                database.query("UPDATE type::record('uar_memory_control',$id) SET expires_at=time::now()")
                    .bind(("id", memory_id.clone())).await.unwrap().check().unwrap();
                assert!(service.get(&memory_id, &user).await.unwrap().is_none());
                assert!(service.list(&user, None, None).await.unwrap().iter().all(|row| id(row) != memory_id));
            }
            service.purge_memories(&user).await.unwrap();
            service.purge_memories(&user).await.unwrap();
        },
        "queue" => {
            service.set_capture_policy(&user, enable(Retention::UntilDeleted)).await.unwrap();
            let admission = service.capture_admission(&user).await.unwrap().unwrap();
            std::fs::write(directory.join("ready"), b"admitted").unwrap();
            let started=Instant::now();
            while !directory.join("release").exists() {
                assert!(started.elapsed() < Duration::from_secs(180));
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
            assert!(service.auto_capture_admitted(messages("cross process stale queue"), &user, None, None, &admission).await.unwrap().is_empty());
        },
        "revoke" => {
            service.set_capture_policy(&user, CapturePolicyUpdate { enabled:false, retention:None }).await.unwrap();
            service.set_capture_policy(&user, enable(Retention::Days { days:7 })).await.unwrap();
            std::fs::write(directory.join("release"), b"revoked and reenabled").unwrap();
        },
        _ => panic!("unknown fixture action"),
    }
}

#[test]
fn durable_controls() {
    if let Ok(action) = std::env::var("UAR_DOT13_CHILD") {
        let directory = std::path::PathBuf::from(std::env::var("UAR_DOT13_DIRECTORY").unwrap());
        tokio::runtime::Runtime::new().unwrap().block_on(scenario(&directory, &action));
        return;
    }
    let local = tempfile::tempdir().unwrap();
    std::fs::write(local.path().join("config.json"), serde_json::to_vec(&json!({
        "db_path":local.path().join("database"),"namespace":"dot13_local","database":"memory"
    })).unwrap()).unwrap();
    join(child(local.path(), "seed"));
    join(child(local.path(), "reopen"));

    let shared = tempfile::tempdir().unwrap();
    std::fs::write(shared.path().join("config.json"), serde_json::to_vec(&json!({
        "surreal_endpoint":std::env::var("UAR_DOT_MEMORY_SURREAL_URL").expect("isolated durable shared SurrealDB required"),
        "surreal_user":std::env::var("UAR_DOT_MEMORY_SURREAL_USER").ok(),"surreal_pass":std::env::var("UAR_DOT_MEMORY_SURREAL_PASS").ok(),
        "namespace":format!("dot13_{}",uuid::Uuid::new_v4().simple()),"database":"memory"
    })).unwrap()).unwrap();
    join(child(shared.path(), "seed"));
    join(child(shared.path(), "reopen"));
    let waiting = child(shared.path(), "queue");
    ready(shared.path());
    join(child(shared.path(), "revoke"));
    join(waiting);
}
