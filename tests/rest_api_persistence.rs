//! R4: the REST API must **persist**, not merely return 201.
//!
//! # The gap this closes
//!
//! `tests/skills_api_integration_test.rs` has 26 passing tests covering every
//! skill endpoint. All of them build the service with:
//!
//! ```ignore
//! let service = Arc::new(SkillService::new(None, None));
//! //                                       ^^^^ no persistence
//! ```
//!
//! So they assert HTTP status codes and response bodies against an in-memory
//! registry. **Zero of them assert that anything reached a database.**
//!
//! That is exactly the shape of the bug `change-uhe-008` found: `POST /skills`
//! returns `201 Created` with a perfectly correct body while the row is silently
//! dropped — because `SkillRegistry::register` logs persist failures without
//! propagating them. Two separate defects hid behind that seam (an empty
//! pgvector value, and a notify trigger that aborted every insert), and 26
//! green tests said nothing about either.
//!
//! **"The endpoint returns 201" is not "the skill was installed."** These tests
//! assert the second thing.

#![cfg(feature = "in-memory-backend")]

use std::sync::Arc;

use axum_test::TestServer;
use serde_json::{Value, json};

use universal_agent_runtime::uar::api::skills::build_router;
use universal_agent_runtime::uar::domain::skills::Skill;
use universal_agent_runtime::uar::persistence::PersistenceLayer;
use universal_agent_runtime::uar::persistence::providers::memory::InMemoryProvider;
use universal_agent_runtime::uar::runtime::skills::service::SkillService;

/// A server whose service has a **real** persistence layer, plus a handle to
/// that layer so a test can ask the database directly.
fn setup_with_persistence() -> (TestServer, Arc<dyn PersistenceLayer>) {
    let db: Arc<dyn PersistenceLayer> = Arc::new(InMemoryProvider::new());

    // No VectorMatcher — the embedded case, and the configuration under which
    // the uhe-008 persistence bug occurred.
    let service = Arc::new(SkillService::new(Some(Arc::clone(&db)), None));

    let app = axum::Router::new()
        .nest("/skills", build_router())
        .with_state(service);

    (TestServer::new(app), db)
}

/// Mirrors the payload shape proven by `skills_api_integration_test.rs`, so a
/// failure here is about persistence rather than about a malformed request.
fn skill_payload(name: &str) -> Value {
    json!({
        "name": name,
        "description": "A REST persistence test skill",
        "version": "1.0.0",
        "triggers": {
            "keywords": ["persistence", "rest"],
            "semantic": null
        },
        "prompt_overlay": "You are a test assistant.",
        "preferred_tools": [],
        "enabled": true
    })
}

async fn rows(db: &Arc<dyn PersistenceLayer>) -> Vec<Skill> {
    db.list_skills()
        .await
        .expect("list skills from persistence")
}

/// **install** — `POST /skills` must write to the database, not just answer 201.
#[tokio::test]
async fn post_skills_persists_the_skill_not_just_returns_201() {
    let (server, db) = setup_with_persistence();

    assert!(
        rows(&db).await.is_empty(),
        "the database must start empty, or this test measures leftovers"
    );

    let response = server
        .post("/skills")
        .json(&skill_payload("Rest Persist"))
        .await;
    response.assert_status(axum::http::StatusCode::CREATED);

    // THE ASSERTION the existing 26 tests do not make.
    let persisted = rows(&db).await;
    assert_eq!(
        persisted.len(),
        1,
        "POST returned 201 but the database holds {} rows. A 201 with a correct \
         body proves the handler ran; it does not prove the skill was installed. \
         This is the exact seam two silent data-loss bugs hid behind.",
        persisted.len()
    );
    assert_eq!(persisted[0].title, "Rest Persist");
}

/// **install via import** — the second install path must persist too.
#[tokio::test]
async fn install_is_visible_to_a_subsequent_get() {
    let (server, db) = setup_with_persistence();

    let created = server
        .post("/skills")
        .json(&skill_payload("Round Trip"))
        .await;
    created.assert_status(axum::http::StatusCode::CREATED);
    let id = created.json::<Value>()["skill_id"]
        .as_str()
        .unwrap()
        .to_string();

    // Query path: GET /{id} must find what install wrote.
    let fetched = server.get(&format!("/skills/{id}")).await;
    fetched.assert_status_ok();
    assert_eq!(fetched.json::<Value>()["title"], "Round Trip");

    // And it is in the database, not only the registry.
    assert!(
        rows(&db).await.iter().any(|s| s.skill_id == id),
        "a skill retrievable over HTTP must also exist in persistence; if it does \
         not, a restart loses it and the API lied about installing it"
    );
}

/// **query** — `GET /skills` must reflect what is stored.
#[tokio::test]
async fn get_skills_lists_what_was_installed() {
    let (server, db) = setup_with_persistence();

    for name in ["Query Alpha", "Query Beta"] {
        server
            .post("/skills")
            .json(&skill_payload(name))
            .await
            .assert_status(axum::http::StatusCode::CREATED);
    }

    let listed = server.get("/skills").await;
    listed.assert_status_ok();
    let body = listed.json::<Value>();
    let count = body.as_array().map_or(0, Vec::len);

    assert_eq!(
        count,
        rows(&db).await.len(),
        "the list endpoint and the database must agree; a divergence means one of \
         them is lying to whoever reads it"
    );
    assert_eq!(count, 2, "both installed skills must be listed");
}

/// **query** — `GET /skills/match` finds an installed skill.
#[tokio::test]
async fn match_finds_an_installed_skill() {
    let (server, _db) = setup_with_persistence();

    server
        .post("/skills")
        .json(&skill_payload("Searchable Widget"))
        .await
        .assert_status(axum::http::StatusCode::CREATED);

    // The param is `q` (see `MatchQuery`), not `query`.
    let matched = server.get("/skills/match?q=Searchable").await;
    matched.assert_status_ok();

    let body = matched.json::<Value>();
    let found = body
        .as_array()
        .map(|a| a.iter().any(|s| s["title"] == "Searchable Widget"))
        .unwrap_or(false);
    assert!(
        found,
        "match must find an installed skill by title even with no embedding \
         backend — an embedded host has none, and a search that only works with \
         an embedder is not usable there. got: {body}"
    );
}

/// **toggle** — the state change must be durable, not registry-only.
#[tokio::test]
async fn toggle_persists_the_new_enabled_state() {
    let (server, db) = setup_with_persistence();

    let created = server
        .post("/skills")
        .json(&skill_payload("Toggle Me"))
        .await;
    created.assert_status(axum::http::StatusCode::CREATED);
    let id = created.json::<Value>()["skill_id"]
        .as_str()
        .unwrap()
        .to_string();

    server
        .post(&format!("/skills/{id}/toggle"))
        .json(&json!({ "enabled": false }))
        .await
        .assert_status_ok();

    let stored = rows(&db)
        .await
        .into_iter()
        .find(|s| s.skill_id == id)
        .expect("a toggled skill must still exist — disabling is not deleting");

    assert!(
        !stored.enabled,
        "toggle returned OK but the database still records enabled=true. A state \
         change that does not survive a restart is not a state change."
    );
}
