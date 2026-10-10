//! Real HTTP selection against a private durable store; no embedding service.

use std::{path::Path, sync::Arc};

use axum::Router;
use serde_json::{Value, json};
use universal_agent_runtime::uar::{
    api::skills::{build_agent_skills_router, build_router},
    domain::skills::{ScopedSkillConfig, Skill, SkillOrigin, SkillScope},
    persistence::{
        PersistenceLayer,
        providers::surreal::{SurrealDbProvider, value_to_json},
    },
    runtime::skills::{service::SkillService, storage::database::DatabaseStorageProvider},
};

const MODE: &str = "UAR_BAUAR_SELECTION_CHILD";
const ROOT: &str = "UAR_BAUAR_SELECTION_ROOT";

fn skill(id: &str) -> Skill {
    Skill {
        skill_id: id.into(),
        title: format!("Preserved {id}"),
        description: "Selection must not re-embed this document".into(),
        prompt_overlay: "Unchanged document bytes: é\nsecond line".into(),
        enabled: true,
        origin: SkillOrigin::Builtin,
        provider_id: "fixture".into(),
        ..Skill::default()
    }
}

async fn raw_records(root: &Path, seed: bool) -> Value {
    let endpoint = format!("surrealkv://{}", root.join("db").display());
    let db = surrealdb::engine::any::connect(&endpoint)
        .await
        .expect("open private KV");
    db.use_ns("selection")
        .use_db("selection")
        .await
        .expect("select private database");
    if seed {
        for id in ["first", "second"] {
            let mut record = serde_json::to_value(skill(id)).expect("serialize fixture skill");
            record["embedding"] = json!([0.125, -0.5, 0.75]);
            record["preserve_unknown"] = json!({"nested": ["unchanged", 17]});
            let _: Option<surrealdb::types::Value> = db
                .upsert(("skills", id))
                .content(record)
                .await
                .expect("seed exact stored record");
        }
    }
    let records: Vec<surrealdb::types::Value> =
        db.select("skills").await.expect("read raw records");
    let mut records = records
        .into_iter()
        .map(|record| value_to_json(record).expect("decode stored row"))
        .collect::<Vec<_>>();
    records.sort_by_key(|record| record["skill_id"].as_str().unwrap().to_string());
    json!(records)
}

async fn api_selection(root: &Path) {
    let endpoint = format!("surrealkv://{}", root.join("db").display());
    let persistence: Arc<dyn PersistenceLayer> = Arc::new(
        SurrealDbProvider::new(&endpoint, None, None, Some("selection"), Some("selection"))
            .await
            .expect("open actual private provider"),
    );
    let mut service = SkillService::new(Some(Arc::clone(&persistence)), None);
    service.add_provider(Arc::new(DatabaseStorageProvider::new(
        "fixture",
        "Private database",
        Arc::clone(&persistence),
    )));
    service
        .initialize()
        .await
        .expect("hydrate unchanged stored skills");
    let service = Arc::new(service);
    let app = Router::new()
        .nest("/api/uar/skills", build_router())
        .nest("/api/uar/agents", build_agent_skills_router())
        .with_state(Arc::clone(&service));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("private listener");
    let base = format!("http://{}", listener.local_addr().unwrap());
    let server = tokio::spawn(async move { axum::serve(listener, app).await });
    let client = reqwest::Client::new();
    let selection = format!("{base}/api/uar/agents/agent-a/skills");
    let response = client
        .put(&selection)
        .json(&json!({"skill_ids": ["first", "future"]}))
        .send()
        .await
        .expect("real selection request");
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let ids: Vec<String> = client
        .get(&selection)
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(ids.contains(&"first".into()) && ids.contains(&"future".into()));
    assert!(!ids.contains(&"second".into()));
    let persisted = persistence.list_skills().await.unwrap();
    assert!(
        !persisted
            .iter()
            .find(|skill| skill.skill_id == "second")
            .unwrap()
            .scoped_config
            .iter()
            .find(|entry| entry.scope == SkillScope::Agent("agent-a".into()))
            .expect("explicit false override")
            .enabled
    );

    // A confirmed durable metadata write must override a previously cached selected ID.
    let first = persisted
        .iter()
        .find(|skill| skill.skill_id == "first")
        .unwrap();
    let disabled = vec![ScopedSkillConfig {
        scope: SkillScope::Agent("agent-a".into()),
        enabled: false,
    }];
    assert!(
        persistence
            .update_skill_selection("first", first.enabled, &disabled)
            .await
            .unwrap()
    );
    let ids: Vec<String> = client
        .get(&selection)
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(
        ids,
        vec!["future".to_string()],
        "durable explicit false beats cached IDs"
    );
    let response = client
        .post(format!("{base}/api/uar/skills/second/toggle"))
        .json(&json!({"enabled": false}))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), reqwest::StatusCode::OK);

    // Only the fixture's added third row is removed: verify no implicit upsert on a missing row.
    let third = skill("removed");
    service.register_builtins(vec![third]).await;
    assert!(
        !persistence
            .update_skill_selection("never-existed", false, &[])
            .await
            .unwrap()
    );
    persistence.delete_skill("removed").await.unwrap();
    let response = client
        .put(&selection)
        .json(&json!({"skill_ids": ["second"]}))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), reqwest::StatusCode::CONFLICT);
    let response = client.get(&selection).send().await.unwrap();
    assert_eq!(response.status(), reqwest::StatusCode::CONFLICT);
    let error: Value = response.json().await.unwrap();
    assert_eq!(error["outcome"], "partial_possible");
    let all = persistence.list_skills().await.unwrap();
    assert_eq!(all.len(), 2);
    server.abort();
    let _ = server.await;
}

#[tokio::test]
async fn real_selection_preserves_stored_document_and_embedding() {
    if let Ok(mode) = std::env::var(MODE) {
        let root = std::path::PathBuf::from(std::env::var(ROOT).expect("private fixture root"));
        match mode.as_str() {
            "seed" => {
                let before = raw_records(&root, true).await;
                std::fs::write(
                    root.join("before.json"),
                    serde_json::to_vec(&before).unwrap(),
                )
                .unwrap();
            }
            "api" => api_selection(&root).await,
            "inspect" => {
                let mut after = raw_records(&root, false).await;
                let mut before: Value =
                    serde_json::from_slice(&std::fs::read(root.join("before.json")).unwrap())
                        .unwrap();
                assert_ne!(before, after, "selection actually changed durable metadata");
                for records in [&mut before, &mut after] {
                    for record in records.as_array_mut().unwrap() {
                        let object = record.as_object_mut().unwrap();
                        object.remove("enabled");
                        object.remove("scoped_config");
                    }
                }
                assert_eq!(
                    before, after,
                    "all non-selection values, including vector and unknown fields, remain exact"
                );
            }
            _ => panic!("unknown fixture phase"),
        }
        return;
    }
    let root = tempfile::tempdir().expect("private KV directory");
    for mode in ["seed", "api", "inspect"] {
        let status = std::process::Command::new(std::env::current_exe().unwrap())
            .arg("--exact")
            .arg("real_selection_preserves_stored_document_and_embedding")
            .arg("--nocapture")
            .env(MODE, mode)
            .env(ROOT, root.path())
            .status()
            .expect("run private KV fixture phase");
        assert!(
            status.success(),
            "private KV selection phase failed: {mode}"
        );
    }
}
