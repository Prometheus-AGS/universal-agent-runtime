//! Real provider contracts for Dot12 cases 41/44. Fixture seeding is setup,
//! never proof of HTTP authorization (covered by owner_isolation_journey.py).
//! Author only until the lead opens the full-phase verification boundary.
#![cfg(all(feature = "server", feature = "surreal-backend"))]
use serde_json::json;
use universal_agent_runtime::{config::SecurityConfig, session::Session,
    uar::{domain::knowledge::{KnowledgeBase, KnowledgeDocument, KnowledgeChunk, KbConfig, DocumentStatus},
    persistence::{PersistenceLayer, providers::surreal::SurrealDbProvider},
    security::middleware::authenticate_bearer_context}};

async fn owner(issuer: &str, tenant: &str) -> String {
    let secret = "dot12-adjacent-fixture-only-secret";
    let config: SecurityConfig = serde_json::from_value(json!({"jwt_required":true,"jwt_secret":secret,"jwt_issuer":issuer})).unwrap();
    let _ = authenticate_bearer_context(&config, "invalid").await;
    let token = jsonwebtoken::encode(&jsonwebtoken::Header::default(),
        &json!({"iss":issuer,"tenant_id":tenant,"sub":"same-subject","roles":["user"],"exp":chrono::Utc::now().timestamp()+3600}),
        &jsonwebtoken::EncodingKey::from_secret(secret.as_bytes())).unwrap();
    authenticate_bearer_context(&config, &token).await.unwrap().session_owner_key().unwrap()
}

async fn exercise(store: &SurrealDbProvider) {
    let owners = [owner("issuer-a", "tenant-a").await,
        owner("issuer-a", "tenant-b").await, owner("issuer-b", "tenant-a").await];
    let session_id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    // Reproduce the old physical namespace, where a subject could itself look
    // exactly like a canonical principal key. This is legacy-data setup, not an
    // authenticated write or a fabricated host execution receipt.
    let collision_id = uuid::Uuid::new_v4().to_string();
    let legacy_key = format!("{}:{}:{}", owners[0].len(), owners[0], collision_id);
    let collision: Session = serde_json::from_value(json!({"id":collision_id,"owner_id":owners[0],
        "messages":[],"created_at":now,"last_activity":now,"system_prompt":null})).unwrap();
    collision.add_user_message("legacy canonical-looking subject private history");
    let mut payload = serde_json::to_value(&collision).unwrap();
    payload.as_object_mut().unwrap().remove("id");
    payload["logical_id"] = collision_id.clone().into();
    store.client().query("CREATE type::record('sessions',$key) CONTENT $payload")
        .bind(("key",legacy_key.clone())).bind(("payload",payload))
        .await.unwrap().check().unwrap();
    let mut before = store.client().query("SELECT VALUE messages FROM type::record('sessions',$key)")
        .bind(("key",legacy_key.clone())).await.unwrap().check().unwrap();
    let legacy_messages: Vec<serde_json::Value> = before.take(0).unwrap();
    assert_eq!(legacy_messages.len(), 1);
    assert!(store.load_session(&owners[0], &collision_id).await.unwrap().is_none());
    assert!(!store.delete_session(&owners[0], &collision_id).await.unwrap());
    let principal: Session = serde_json::from_value(json!({"id":collision_id,"owner_id":owners[0],
        "messages":[],"created_at":now,"last_activity":now,"system_prompt":null})).unwrap();
    principal.add_user_message("verified principal independent history");
    store.save_session(&principal).await.unwrap();
    assert_eq!(store.load_session(&owners[0], &collision_id).await.unwrap().unwrap()
        .messages()[0].content.to_string(), "verified principal independent history");
    assert!(store.delete_session(&owners[0], &collision_id).await.unwrap());
    let mut after = store.client().query("SELECT VALUE messages FROM type::record('sessions',$key)")
        .bind(("key",legacy_key)).await.unwrap().check().unwrap();
    let retained: Vec<serde_json::Value> = after.take(0).unwrap();
    assert_eq!(retained, legacy_messages);
    // Deserialize an actual legacy shape with no owner, then enter the real
    // provider contract. It remains anonymous and is never adopted by subject.
    let legacy: Session = serde_json::from_value(json!({"id":session_id,"messages":[],
        "created_at":now,"last_activity":now,"system_prompt":null})).unwrap();
    legacy.add_user_message("anonymous legacy history");
    store.save_session(&legacy).await.unwrap();
    for owner in &owners {
        assert!(store.load_session(owner, &session_id).await.unwrap().is_none());
        assert!(!store.delete_session(owner, &session_id).await.unwrap());
    }
    for owner in &owners {
        let session: Session = serde_json::from_value(json!({"id":session_id,"owner_id":owner,
            "messages":[],"created_at":now,"last_activity":now,"system_prompt":null})).unwrap();
        session.add_user_message(format!("private:{owner}"));
        store.save_session(&session).await.unwrap();
    }
    for owner in &owners {
        let loaded = store.load_session(owner, &session_id).await.unwrap().unwrap();
        assert_eq!(loaded.owner_id(), owner);
        assert_eq!(loaded.messages()[0].content.to_string(), format!("private:{owner}"));
    }
    assert_eq!(store.load_session("anonymous", &session_id).await.unwrap().unwrap()
        .messages()[0].content.to_string(), "anonymous legacy history");
    // The old cascade deleted unqualified session-scope cost history. A real
    // provider cost entry is fixture setup for retention, not spend authority.
    store.record_cost_entry("session", &session_id, 0.125).await.unwrap();
    let costs = store.list_cost_history("session", &session_id).await.unwrap();
    assert_eq!(costs.len(), 1);
    assert!(store.delete_session(&owners[0], &session_id).await.unwrap());
    let retained_costs = store.list_cost_history("session", &session_id).await.unwrap();
    assert_eq!(retained_costs.len(), costs.len());
    assert_eq!(retained_costs[0].scope, costs[0].scope);
    assert_eq!(retained_costs[0].scope_id, costs[0].scope_id);
    assert_eq!(retained_costs[0].cost_usd, costs[0].cost_usd);
    assert_eq!(retained_costs[0].recorded_at, costs[0].recorded_at);
    for owner in &owners[1..] { assert!(store.load_session(owner, &session_id).await.unwrap().is_some()); }

    // Identical KB, document and chunk IDs exercise the actual durable owner
    // partition, including scope narrowing and deletion on the same records.
    let kb_id = uuid::Uuid::new_v4().to_string();
    let doc_id = uuid::Uuid::new_v4().to_string();
    let chunk_id = uuid::Uuid::new_v4();
    for owner in &owners {
        store.save_knowledge_base(&KnowledgeBase { id: kb_id.clone(), owner_id: owner.clone(),
            name: "same-kb-name".into(), description: None, config: KbConfig::default(),
            created_at: now.clone(), updated_at: now.clone() }).await.unwrap();
        store.save_document(&KnowledgeDocument { id: doc_id.clone(), owner_id: owner.clone(), kb_id: kb_id.clone(),
            filename: format!("private:{owner}"), file_path: None, mime_type: Some("text/plain".into()),
            chunk_count: 1, status: DocumentStatus::Indexed, created_at: now.clone(), updated_at: now.clone() }).await.unwrap();
        store.save_chunk(&KnowledgeChunk { id: chunk_id, owner_id: owner.clone(), kb_id: kb_id.clone(),
            document_id: Some(doc_id.clone()), content: format!("private:{owner}"), metadata: None,
            embedding: vec![1.0, 0.0, 0.0], created_at: now.clone() }).await.unwrap();
    }
    for owner in &owners {
        assert_eq!(store.get_knowledge_base_by_name(owner, "same-kb-name").await.unwrap().unwrap().id, kb_id);
        assert_eq!(store.get_document(owner, &doc_id).await.unwrap().unwrap().filename, format!("private:{owner}"));
        let matches = store.search_knowledge_scoped(owner, &[&kb_id], &[1.0, 0.0, 0.0], 10, 0.0).await.unwrap();
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].chunk.content, format!("private:{owner}"));
        assert!(store.search_knowledge_scoped(owner, &["unresolved-name"], &[1.0, 0.0, 0.0], 10, 0.0).await.unwrap().is_empty());
    }
    store.delete_knowledge_base(&owners[0], &kb_id).await.unwrap();
    assert!(store.get_document(&owners[0], &doc_id).await.unwrap().is_none());
    for owner in &owners[1..] { assert_eq!(store.count_documents(owner, &kb_id).await.unwrap(), 1); }
}

#[tokio::test]
async fn local_durable_adjacent_owner_contract() {
    let directory = tempfile::tempdir().unwrap();
    let endpoint = format!("surrealkv://{}", directory.path().join("owner.db").display());
    let store = SurrealDbProvider::new(&endpoint, None, None, Some("dot-owner"), Some("local")).await.unwrap();
    exercise(&store).await;
}

#[tokio::test]
async fn shared_durable_adjacent_owner_contract() {
    let endpoint = std::env::var("UAR_DOT_MEMORY_SURREAL_URL").expect("isolated durable shared SurrealDB required");
    assert!(endpoint.starts_with("ws://") || endpoint.starts_with("wss://") || endpoint.starts_with("http://") || endpoint.starts_with("https://"));
    let user = std::env::var("UAR_DOT_MEMORY_SURREAL_USER").ok();
    let pass = std::env::var("UAR_DOT_MEMORY_SURREAL_PASS").ok();
    let database = format!("adjacent_{}", uuid::Uuid::new_v4().simple());
    let store = SurrealDbProvider::new(&endpoint, user.as_deref(), pass.as_deref(), Some("dot-owner"), Some(&database)).await.unwrap();
    exercise(&store).await;
}
