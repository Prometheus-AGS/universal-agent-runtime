//! Integration tests for the Skills API.
//!
//! These tests exercise every HTTP endpoint exposed by the skills router using
//! `axum-test` — no external infrastructure required, no LLM calls.
//!
//! Coverage:
//!  - Skills CRUD (list, create, get, update, delete)
//!  - Lifecycle (toggle, refresh)
//!  - Matching (keyword, with agent filter)
//!  - Config (get/set matching config)
//!  - Agent-skill bindings (get, set, add, remove)

use axum_test::TestServer;
use serde_json::{Value, json};
use std::sync::Arc;

use universal_agent_runtime::uar::api::skills::{build_agent_skills_router, build_router};
use universal_agent_runtime::uar::domain::skills::{Skill, SkillOrigin};
use universal_agent_runtime::uar::runtime::skills::service::SkillService;

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

/// Spin up an in-memory skills service + axum-test server.
fn setup() -> TestServer {
    let service = Arc::new(SkillService::new(None, None));

    // Merge the skills router (mounted at /) and agent bindings router (at /agents)
    let app = axum::Router::new()
        .nest("/skills", build_router())
        .nest("/agents", build_agent_skills_router())
        .with_state(service);

    TestServer::new(app)
}

/// Minimal valid skill creation payload.
fn skill_payload(name: &str) -> Value {
    json!({
        "name": name,
        "description": "An integration test skill",
        "version": "1.0.0",
        "triggers": {
            "keywords": ["test", "integration"],
            "semantic": null
        },
        "prompt_overlay": "You are a test assistant.",
        "preferred_tools": [],
        "enabled": true
    })
}

// ---------------------------------------------------------------------------
// GET /skills — list all skills
// ---------------------------------------------------------------------------

#[tokio::test]
async fn list_skills_returns_empty_on_fresh_service() {
    let server = setup();
    let response = server.get("/skills").await;
    response.assert_status_ok();
    let body: Vec<Value> = response.json();
    assert!(body.is_empty(), "fresh service should have no skills");
}

// ---------------------------------------------------------------------------
// POST /skills — create a skill
// ---------------------------------------------------------------------------

#[tokio::test]
async fn create_skill_returns_created_and_round_trips_fields() {
    let server = setup();
    let response = server
        .post("/skills")
        .json(&skill_payload("My Skill"))
        .await;
    response.assert_status(axum::http::StatusCode::CREATED);

    let body: Value = response.json();
    assert_eq!(body["skill_id"], "my-skill");
    assert_eq!(body["title"], "My Skill");
    assert_eq!(body["description"], "An integration test skill");
    assert_eq!(body["version"], "1.0.0");
    assert_eq!(body["enabled"], true);
    assert_eq!(body["provider_id"], "api");
    assert!(
        body["triggers"]["keywords"].as_array().is_some(),
        "triggers.keywords should be present"
    );
}

#[tokio::test]
async fn create_skill_derives_skill_id_from_name() {
    let server = setup();
    let response = server
        .post("/skills")
        .json(&skill_payload("Hello World Skill"))
        .await;
    response.assert_status(axum::http::StatusCode::CREATED);
    let body: Value = response.json();
    assert_eq!(body["skill_id"], "hello-world-skill");
}

// ---------------------------------------------------------------------------
// GET /skills/{id} — get by ID
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_skill_returns_skill_after_create() {
    let server = setup();

    // Create
    server
        .post("/skills")
        .json(&skill_payload("Fetch Me"))
        .await;

    // Fetch
    let response = server.get("/skills/fetch-me").await;
    response.assert_status_ok();
    let body: Value = response.json();
    assert_eq!(body["skill_id"], "fetch-me");
    assert_eq!(body["title"], "Fetch Me");
}

#[tokio::test]
async fn get_skill_returns_404_for_missing_id() {
    let server = setup();
    let response = server.get("/skills/does-not-exist").await;
    response.assert_status(axum::http::StatusCode::NOT_FOUND);
}

// ---------------------------------------------------------------------------
// PUT /skills/{id} — partial update
// ---------------------------------------------------------------------------

#[tokio::test]
async fn update_skill_applies_partial_changes_only() {
    let server = setup();

    // Create
    server
        .post("/skills")
        .json(&skill_payload("Updatable"))
        .await;

    // Update only the description
    let response = server
        .put("/skills/updatable")
        .json(&json!({ "description": "Updated description" }))
        .await;
    response.assert_status_ok();

    let body: Value = response.json();
    assert_eq!(body["description"], "Updated description");
    // Title should be unchanged
    assert_eq!(body["title"], "Updatable");
}

#[tokio::test]
async fn update_skill_returns_404_for_missing_id() {
    let server = setup();
    let response = server
        .put("/skills/ghost")
        .json(&json!({ "description": "noop" }))
        .await;
    response.assert_status(axum::http::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn update_skill_can_enable_and_disable() {
    let server = setup();
    server
        .post("/skills")
        .json(&skill_payload("Toggle Me"))
        .await;

    // Disable via update
    let resp = server
        .put("/skills/toggle-me")
        .json(&json!({ "enabled": false }))
        .await;
    resp.assert_status_ok();
    assert_eq!(resp.json::<Value>()["enabled"], false);

    // Re-enable
    let resp2 = server
        .put("/skills/toggle-me")
        .json(&json!({ "enabled": true }))
        .await;
    resp2.assert_status_ok();
    assert_eq!(resp2.json::<Value>()["enabled"], true);
}

#[tokio::test]
async fn update_builtin_is_refused_while_toggle_remains_available() {
    let service = Arc::new(SkillService::new(None, None));
    let builtin = Skill {
        skill_id: "pack-skill".to_string(),
        title: "Pack skill".to_string(),
        enabled: true,
        origin: SkillOrigin::Builtin,
        provider_id: "builtin".to_string(),
        ..Skill::default()
    };
    service.register_builtins(vec![builtin]).await;
    let app = axum::Router::new()
        .nest("/skills", build_router())
        .with_state(service);
    let server = TestServer::new(app);

    let response = server
        .put("/skills/pack-skill")
        .json(&json!({ "title": "Mutated pack skill" }))
        .await;
    response.assert_status(axum::http::StatusCode::CONFLICT);
    let error: Value = response.json();
    assert_eq!(error["error"], "system_skill_immutable");

    let unchanged = server.get("/skills/pack-skill").await;
    unchanged.assert_status_ok();
    assert_eq!(unchanged.json::<Value>()["title"], "Pack skill");

    let toggled = server
        .post("/skills/pack-skill/toggle")
        .json(&json!({ "enabled": false }))
        .await;
    toggled.assert_status_ok();
    let disabled = server.get("/skills/pack-skill").await;
    disabled.assert_status_ok();
    assert_eq!(disabled.json::<Value>()["enabled"], false);
}

// ---------------------------------------------------------------------------
// DELETE /skills/{id}
// ---------------------------------------------------------------------------

#[tokio::test]
async fn delete_skill_removes_it_and_returns_no_content() {
    let server = setup();
    server
        .post("/skills")
        .json(&skill_payload("Delete Me"))
        .await;

    let delete_resp = server.delete("/skills/delete-me").await;
    delete_resp.assert_status(axum::http::StatusCode::NO_CONTENT);

    // Subsequent GET should 404
    let get_resp = server.get("/skills/delete-me").await;
    get_resp.assert_status(axum::http::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn delete_nonexistent_skill_returns_not_found() {
    let server = setup();
    let response = server.delete("/skills/never-existed").await;
    response.assert_status(axum::http::StatusCode::NOT_FOUND);
}

// ---------------------------------------------------------------------------
// POST /skills/{id}/toggle
// ---------------------------------------------------------------------------

#[tokio::test]
async fn toggle_skill_changes_enabled_state() {
    let server = setup();
    server.post("/skills").json(&skill_payload("Toggle")).await;

    // Disable
    let resp = server
        .post("/skills/toggle/toggle")
        .json(&json!({ "enabled": false }))
        .await;
    resp.assert_status_ok();

    // Verify via list
    let body: Vec<Value> = server.get("/skills").await.json();
    let skill = body.iter().find(|s| s["skill_id"] == "toggle").unwrap();
    assert_eq!(skill["enabled"], false);
}

#[tokio::test]
async fn toggle_missing_skill_returns_not_found() {
    let server = setup();
    let response = server
        .post("/skills/missing/toggle")
        .json(&json!({ "enabled": false }))
        .await;
    response.assert_status(axum::http::StatusCode::NOT_FOUND);
}

// ---------------------------------------------------------------------------
// POST /skills/refresh
// ---------------------------------------------------------------------------

#[tokio::test]
async fn refresh_skills_returns_ok_with_count() {
    let server = setup();
    server.post("/skills").json(&skill_payload("Alpha")).await;
    server.post("/skills").json(&skill_payload("Beta")).await;

    let response = server.post("/skills/refresh").await;
    response.assert_status_ok();

    let body: Value = response.json();
    // No filesystem providers, so count resets to 0 after refresh
    assert!(
        body["count"].as_u64().is_some(),
        "response should include a count field"
    );
    assert!(
        body["skills"].as_array().is_some(),
        "response should include skills array"
    );
}

// ---------------------------------------------------------------------------
// GET /skills/match
// ---------------------------------------------------------------------------

#[tokio::test]
async fn match_skills_returns_skills_matching_keyword() {
    let server = setup();

    // Create a skill with a known keyword
    server
        .post("/skills")
        .json(&json!({
            "name": "Memory Skill",
            "description": "Remembers things",
            "triggers": { "keywords": ["memory", "remember"] },
            "prompt_overlay": "",
            "enabled": true
        }))
        .await;

    let response = server.get("/skills/match?q=memory").await;
    response.assert_status_ok();

    let body: Vec<Value> = response.json();
    assert!(!body.is_empty(), "should match at least one skill");
    assert_eq!(body[0]["skill_id"], "memory-skill");
}

#[tokio::test]
async fn match_skills_returns_empty_for_no_matches() {
    let server = setup();
    server
        .post("/skills")
        .json(&skill_payload("Unrelated"))
        .await;

    let response = server.get("/skills/match?q=zzz-no-match-zzz").await;
    response.assert_status_ok();
    let body: Vec<Value> = response.json();
    assert!(body.is_empty(), "should return no matches");
}

#[tokio::test]
async fn match_skills_respects_agent_filter() {
    let server = setup();

    // Create two skills
    server
        .post("/skills")
        .json(&json!({
            "name": "Agent Skill",
            "description": "Agent specific",
            "triggers": { "keywords": ["agent"] },
            "prompt_overlay": "",
            "enabled": true
        }))
        .await;
    server
        .post("/skills")
        .json(&json!({
            "name": "Other Skill",
            "description": "Not bound to agent",
            "triggers": { "keywords": ["agent"] },
            "prompt_overlay": "",
            "enabled": true
        }))
        .await;

    // Bind only "agent-skill" to "agent-1"
    server
        .put("/agents/agent-1/skills")
        .json(&json!({ "skill_ids": ["agent-skill"] }))
        .await;

    // Match with agent filter — only agent-skill should appear
    let response = server.get("/skills/match?q=agent&agent_id=agent-1").await;
    response.assert_status_ok();

    let body: Vec<Value> = response.json();
    assert_eq!(body.len(), 1, "only bound skill should match");
    assert_eq!(body[0]["skill_id"], "agent-skill");
}

// ---------------------------------------------------------------------------
// GET /skills/config — matching config
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_config_returns_default_config() {
    let server = setup();
    let response = server.get("/skills/config").await;
    response.assert_status_ok();

    let body: Value = response.json();
    assert_eq!(body["algorithm"], "keyword");
    assert!(body["threshold"].as_f64().is_some());
    assert!(body["top_k"].as_u64().is_some());
}

// ---------------------------------------------------------------------------
// PUT /skills/config — set matching config
// ---------------------------------------------------------------------------

#[tokio::test]
async fn set_config_updates_matching_algorithm() {
    let server = setup();

    let response = server
        .put("/skills/config")
        .json(&json!({
            "algorithm": "hybrid",
            "threshold": 0.7,
            "top_k": 5
        }))
        .await;
    response.assert_status_ok();

    // Verify the change persisted
    let get_resp = server.get("/skills/config").await;
    let body: Value = get_resp.json();
    assert_eq!(body["algorithm"], "hybrid");
    assert!((body["threshold"].as_f64().unwrap() - 0.7).abs() < 0.01);
    assert_eq!(body["top_k"], 5);
}

// ---------------------------------------------------------------------------
// Agent-skill binding endpoints
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_agent_skills_returns_empty_for_unknown_agent() {
    let server = setup();
    let response = server.get("/agents/unknown-agent/skills").await;
    response.assert_status_ok();
    let body: Vec<Value> = response.json();
    assert!(body.is_empty());
}

#[tokio::test]
async fn set_agent_skills_replaces_bindings() {
    let server = setup();

    // Set bindings
    let resp = server
        .put("/agents/agent-a/skills")
        .json(&json!({ "skill_ids": ["skill-1", "skill-2"] }))
        .await;
    resp.assert_status_ok();

    // Verify
    let body: Vec<Value> = server.get("/agents/agent-a/skills").await.json();
    assert_eq!(body.len(), 2);
    let ids: Vec<&str> = body.iter().map(|v| v.as_str().unwrap()).collect();
    assert!(ids.contains(&"skill-1"));
    assert!(ids.contains(&"skill-2"));
}

#[tokio::test]
async fn set_agent_skills_overwrites_previous_bindings() {
    let server = setup();

    server
        .put("/agents/agent-b/skills")
        .json(&json!({ "skill_ids": ["old-skill"] }))
        .await;

    // Overwrite
    server
        .put("/agents/agent-b/skills")
        .json(&json!({ "skill_ids": ["new-skill"] }))
        .await;

    let body: Vec<Value> = server.get("/agents/agent-b/skills").await.json();
    assert_eq!(body.len(), 1);
    assert_eq!(body[0].as_str().unwrap(), "new-skill");
}

#[tokio::test]
async fn add_skill_to_agent_appends_to_bindings() {
    let server = setup();

    // Start with one binding
    server
        .put("/agents/agent-c/skills")
        .json(&json!({ "skill_ids": ["skill-a"] }))
        .await;

    // Add another
    let resp = server.post("/agents/agent-c/skills/skill-b").await;
    resp.assert_status(axum::http::StatusCode::CREATED);

    let body: Vec<Value> = server.get("/agents/agent-c/skills").await.json();
    assert_eq!(body.len(), 2);
}

#[tokio::test]
async fn add_skill_to_agent_is_idempotent() {
    let server = setup();

    server.post("/agents/agent-d/skills/skill-x").await;
    server.post("/agents/agent-d/skills/skill-x").await;

    let body: Vec<Value> = server.get("/agents/agent-d/skills").await.json();
    assert_eq!(body.len(), 1, "duplicate adds should not create duplicates");
}

#[tokio::test]
async fn remove_skill_from_agent_deletes_binding() {
    let server = setup();

    server
        .put("/agents/agent-e/skills")
        .json(&json!({ "skill_ids": ["skill-keep", "skill-remove"] }))
        .await;

    let resp = server.delete("/agents/agent-e/skills/skill-remove").await;
    resp.assert_status(axum::http::StatusCode::NO_CONTENT);

    let body: Vec<Value> = server.get("/agents/agent-e/skills").await.json();
    assert_eq!(body.len(), 1);
    assert_eq!(body[0].as_str().unwrap(), "skill-keep");
}

#[tokio::test]
async fn remove_skill_from_agent_with_no_bindings_is_no_op() {
    let server = setup();
    // Should not panic or 500
    let resp = server.delete("/agents/agent-f/skills/any-skill").await;
    resp.assert_status(axum::http::StatusCode::NO_CONTENT);
}

// ---------------------------------------------------------------------------
// End-to-end lifecycle scenario
// ---------------------------------------------------------------------------

#[tokio::test]
async fn full_lifecycle_create_toggle_update_delete() {
    let server = setup();

    // 1. Create
    let create = server
        .post("/skills")
        .json(&skill_payload("Lifecycle"))
        .await;
    create.assert_status(axum::http::StatusCode::CREATED);
    let created: Value = create.json();
    assert_eq!(created["skill_id"], "lifecycle");

    // 2. Verify via list
    let list: Vec<Value> = server.get("/skills").await.json();
    assert_eq!(list.len(), 1);

    // 3. Toggle off
    server
        .post("/skills/lifecycle/toggle")
        .json(&json!({ "enabled": false }))
        .await
        .assert_status_ok();

    // 4. Verify disabled
    let get: Value = server.get("/skills/lifecycle").await.json();
    assert_eq!(get["enabled"], false);

    // 5. Update title and re-enable
    server
        .put("/skills/lifecycle")
        .json(&json!({ "title": "Lifecycle v2", "enabled": true }))
        .await
        .assert_status_ok();

    let updated: Value = server.get("/skills/lifecycle").await.json();
    assert_eq!(updated["title"], "Lifecycle v2");
    assert_eq!(updated["enabled"], true);

    // 6. Delete
    server
        .delete("/skills/lifecycle")
        .await
        .assert_status(axum::http::StatusCode::NO_CONTENT);

    // 7. Confirm gone
    server
        .get("/skills/lifecycle")
        .await
        .assert_status(axum::http::StatusCode::NOT_FOUND);
}
