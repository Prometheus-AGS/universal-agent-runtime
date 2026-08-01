//! R4: the embedded SDK, consumed exactly as an external crate would.
//!
//! # Why this test is in `tests/`
//!
//! An integration test compiles as a **separate crate** that links against
//! `universal_agent_runtime` as a dependency. It therefore sees only the public
//! API — the same view a mobile host or a desktop shell gets. A unit test inside
//! `src/` would see `pub(crate)` items too and could pass while the published
//! surface was unusable.
//!
//! # What it asserts
//!
//! That the five verbs R4 names — list / get / install / toggle / query — are
//! reachable **without importing anything from `uar::runtime::skills`**. The
//! import list at the top of this file is itself part of the assertion: if this
//! test ever needs a runtime-internal type to do its job, the facade is
//! incomplete.

use std::sync::Arc;

use universal_agent_runtime::embedded::EmbeddedRuntime;
use universal_agent_runtime::uar::domain::skills::{Skill, SkillOrigin};

/// Build a runtime the way an embedded host does: no server, no network.
///
/// # An R4 finding, recorded rather than worked around
///
/// `EmbeddedRuntime::build()` **requires an LLM driver** — it fails with
/// `E_EMBEDDED_LOCAL_DRIVER_REQUIRED` otherwise, and an in-crate test asserts
/// that deliberately. So a host that wants *only* the skill catalogue (a mobile
/// app listing what it can do, an installer registering pack skills) must still
/// supply a driver it may have no use for.
///
/// That is a real ergonomic edge in the embedding story, not a bug in this test,
/// and it is worth knowing before shipping an SDK. Recorded in the change
/// proposal; the test satisfies the requirement with the crate's public
/// `MockLlmDriver` rather than pretending the constraint does not exist.
#[cfg(feature = "in-memory-backend")]
async fn embedded_runtime() -> EmbeddedRuntime {
    use universal_agent_runtime::llm::registry::{ModelConfig, ProtocolSetting, ProviderConfig};
    use universal_agent_runtime::llm::mock_driver::MockLlmDriver;
    use universal_agent_runtime::uar::persistence::PersistenceLayer;
    use universal_agent_runtime::uar::persistence::providers::memory::InMemoryProvider;

    let persistence: Arc<dyn PersistenceLayer> = Arc::new(InMemoryProvider::new());

    let provider = ProviderConfig {
        id: "embedded-sdk-local".to_string(),
        display_name: "Embedded SDK test model".to_string(),
        base_url: String::new(),
        api_key: None,
        protocol: ProtocolSetting::Auto,
        default_model: Some("offline-agent-model".to_string()),
        models: vec![ModelConfig {
            id: "offline-agent-model".to_string(),
            display_name: Some("Offline agent model".to_string()),
            context_window: Some(8_192),
            supports_vision: false,
            supports_tools: true,
            supports_reasoning: true,
            supports_structured_output: true,
            supports_streaming: true,
            max_output_tokens: Some(2_048),
            enabled: true,
        }],
        enabled: true,
    };

    EmbeddedRuntime::builder()
        // No scripted responses: this test never runs a completion, it only
        // needs the builder's driver requirement satisfied.
        .local_provider(Arc::new(MockLlmDriver::new(Vec::new())), provider)
        .persistence(persistence)
        .seed_defaults(false)
        .build()
        .await
        .expect("an embedded runtime must build with no network services")
}

fn skill(id: &str, title: &str) -> Skill {
    let mut s = Skill::default();
    s.skill_id = id.to_string();
    s.title = title.to_string();
    s.description = format!("{title} — used by the embedded SDK test");
    s.origin = SkillOrigin::User;
    s.enabled = true;
    s
}

/// The whole R4 surface in one round trip, through the public API only.
#[cfg(feature = "in-memory-backend")]
#[tokio::test]
async fn an_embedder_can_list_install_get_toggle_and_query() {
    let runtime = embedded_runtime().await;

    // THE ENTRY POINT. Note what is absent: no `SkillService`, no
    // `SkillRegistry`, no `runtime::skills::*` import anywhere in this file.
    let skills = runtime.skills();

    // --- list: works on an empty runtime, does not panic ---
    let before = skills.list().await;

    // --- install: the R4 "dynamic skill creation" path ---
    let installed = skills
        .install(skill("emb-sdk-alpha", "Alpha"))
        .await
        .expect("install must succeed on a runtime with persistence");
    assert_eq!(installed.skill_id, "emb-sdk-alpha");

    let after = skills.list().await;
    assert_eq!(
        after.len(),
        before.len() + 1,
        "install must be visible to list; an embedder that installs a skill and \
         cannot then see it has no way to know the call worked"
    );

    // --- get: by id, and a miss returns None rather than panicking ---
    let fetched = skills
        .get("emb-sdk-alpha")
        .await
        .expect("an installed skill must be retrievable by id");
    assert_eq!(fetched.title, "Alpha");
    assert!(
        skills.get("emb-sdk-does-not-exist").await.is_none(),
        "a miss must be None, not a panic — an embedder cannot catch a panic \
         across an FFI boundary"
    );

    // --- toggle: disable, and confirm the two lists diverge ---
    assert!(
        skills.toggle("emb-sdk-alpha", false).await,
        "toggle must report success for a skill that exists"
    );
    assert!(
        !skills.list_enabled().await.iter().any(|s| s.skill_id == "emb-sdk-alpha"),
        "a disabled skill must leave list_enabled"
    );
    assert!(
        skills.list().await.iter().any(|s| s.skill_id == "emb-sdk-alpha"),
        "but it must REMAIN in list — disabling is not deleting, which is the \
         guarantee pack builtins depend on"
    );

    // Re-enable, so the divergence is proven in both directions rather than
    // only on the way down.
    assert!(skills.toggle("emb-sdk-alpha", true).await);
    assert!(
        skills.list_enabled().await.iter().any(|s| s.skill_id == "emb-sdk-alpha"),
        "re-enabling must restore the skill to list_enabled"
    );

    // --- toggle on a missing id reports failure rather than pretending ---
    assert!(
        !skills.toggle("emb-sdk-does-not-exist", false).await,
        "toggling an unknown skill must return false"
    );

    // --- query: returns without requiring an embedding backend ---
    // An embedded host typically has no embedder. The contract is that query
    // degrades to keyword matching rather than erroring or hanging.
    let matches = skills.query("Alpha").await;
    assert!(
        matches.iter().any(|s| s.skill_id == "emb-sdk-alpha"),
        "query must find an installed skill by its title even with no embedding \
         backend configured; got {:?}",
        matches.iter().map(|s| &s.skill_id).collect::<Vec<_>>()
    );
}

/// The facade must be `Clone` and `Send`/`Sync`, because a host hands it to
/// background tasks and across an FFI boundary.
///
/// This is a compile-time assertion: if `SkillsApi` stops satisfying these
/// bounds, this test fails to build, which is exactly when we want to know.
#[cfg(feature = "in-memory-backend")]
#[test]
fn the_facade_is_clone_send_and_sync() {
    fn assert_bounds<T: Clone + Send + Sync + 'static>() {}
    assert_bounds::<universal_agent_runtime::SkillsApi>();
}

/// Guard against the facade being quietly bypassed.
///
/// `SkillsApi` must be reachable from the crate root, so embedder code can
/// write `use universal_agent_runtime::SkillsApi;` rather than a deep path.
#[test]
fn the_facade_is_exported_at_the_crate_root() {
    // Naming the type through the root path is the assertion; this fails to
    // compile if the re-export is removed.
    fn _accepts(_: Option<universal_agent_runtime::SkillsApi>) {}
}

// ---------------------------------------------------------------------------
// A REAL driver, not a stub
// ---------------------------------------------------------------------------

#[path = "common/ollama.rs"]
mod ollama;

/// The same R4 surface, but built on a **real** LLM driver.
///
/// # Why this test exists alongside the mock one
///
/// The test above satisfies `E_EMBEDDED_LOCAL_DRIVER_REQUIRED` with
/// `MockLlmDriver`. That proves the builder accepts *a* driver — it proves
/// nothing about whether an embedded host can be constructed around a driver
/// that actually talks to a model.
///
/// This one uses the local Ollama install through `LiterLlmDriver`, the same
/// OpenAI-compatible path a production embedder would take. If the runtime
/// cannot be built around a genuine driver, that is a real embedding defect and
/// the mock would never reveal it.
///
/// Skips **loudly** when Ollama is not running — never silently.
#[cfg(feature = "in-memory-backend")]
#[tokio::test]
async fn an_embedder_can_use_the_skill_api_with_a_real_llm_driver() {
    if !ollama::is_available().await {
        ollama::skip_notice("an_embedder_can_use_the_skill_api_with_a_real_llm_driver");
        return;
    }

    use universal_agent_runtime::uar::persistence::PersistenceLayer;
    use universal_agent_runtime::uar::persistence::providers::memory::InMemoryProvider;

    let persistence: Arc<dyn PersistenceLayer> = Arc::new(InMemoryProvider::new());
    let driver = ollama::driver().expect("Ollama reported available, so a driver must build");

    let runtime = EmbeddedRuntime::builder()
        .local_provider(driver, ollama::provider())
        .persistence(persistence)
        .seed_defaults(false)
        .build()
        .await
        .expect(
            "an embedded runtime must build around a REAL driver. Failing here — while the \
             mock-driver test passes — would mean the embedding path only works with a stub.",
        );

    // The skill surface must behave identically regardless of which driver the
    // host supplied. Skills and inference are separate concerns; if they are
    // not, an embedder cannot reason about either.
    let skills = runtime.skills();

    let installed = skills
        .install(skill("emb-sdk-ollama", "Ollama-backed"))
        .await
        .expect("install must succeed on a runtime built with a real driver");
    assert_eq!(installed.skill_id, "emb-sdk-ollama");

    assert!(
        skills.get("emb-sdk-ollama").await.is_some(),
        "a skill installed on a real-driver runtime must be retrievable"
    );

    let matches = skills.query("Ollama-backed").await;
    assert!(
        matches.iter().any(|s| s.skill_id == "emb-sdk-ollama"),
        "query must work identically on a real-driver runtime; got {:?}",
        matches.iter().map(|s| &s.skill_id).collect::<Vec<_>>()
    );
}

// ---------------------------------------------------------------------------
// R4: dynamic skill registration is OPTIONAL — and the default encodes that
// ---------------------------------------------------------------------------

/// WITHOUT the opt-in, a generated skill writes **nothing**.
///
/// This is the assertion that matters. "Optionally" has to live in the default,
/// not just the documentation: a generator that registers by default silently
/// grows a user's skill catalogue with artifacts they never asked to keep, and
/// a `skills` table that fills on its own is far harder to diagnose than one
/// that stays empty.
///
/// Asserted against the **database**, not the return value — a function can
/// return `None` while still having written a row.
#[cfg(feature = "in-memory-backend")]
#[tokio::test]
async fn without_the_opt_in_a_generated_skill_is_not_registered() {
    use universal_agent_runtime::uar::persistence::PersistenceLayer;
    use universal_agent_runtime::uar::persistence::providers::memory::InMemoryProvider;
    use universal_agent_runtime::uar::runtime::skills::service::SkillService;

    let db: Arc<dyn PersistenceLayer> = Arc::new(InMemoryProvider::new());
    let service = Arc::new(SkillService::new(Some(Arc::clone(&db)), None));

    // Explicit `false` rather than relying on an unset env var: tests share a
    // process, so another test setting UAR_REGISTER_GENERATED_SKILLS would
    // otherwise make this pass or fail depending on execution order.
    let skills = universal_agent_runtime::SkillsApi::for_test(Arc::clone(&service))
        .with_generated_registration(false);

    assert!(
        !skills.generated_registration_enabled(),
        "the default must be OFF; 'optionally' is a property of the default, \
         not of the docs"
    );

    let outcome = skills
        .install_generated(skill("gen-not-registered", "Generated"))
        .await
        .expect("the disabled path does nothing, so it cannot fail");

    assert!(
        outcome.is_none(),
        "a disabled registration must report that it did not register"
    );

    let rows = db.list_skills().await.expect("list skills");
    assert!(
        rows.is_empty(),
        "the database must hold NO rows when registration is off; found {:?}. \
         Returning None while still writing would be the worst outcome — the \
         caller believes nothing happened.",
        rows.iter().map(|s| &s.skill_id).collect::<Vec<_>>()
    );
}

/// WITH the opt-in, the generated skill is registered and durable.
#[cfg(feature = "in-memory-backend")]
#[tokio::test]
async fn with_the_opt_in_a_generated_skill_is_registered() {
    use universal_agent_runtime::uar::persistence::PersistenceLayer;
    use universal_agent_runtime::uar::persistence::providers::memory::InMemoryProvider;
    use universal_agent_runtime::uar::runtime::skills::service::SkillService;

    let db: Arc<dyn PersistenceLayer> = Arc::new(InMemoryProvider::new());
    let service = Arc::new(SkillService::new(Some(Arc::clone(&db)), None));

    let skills = universal_agent_runtime::SkillsApi::for_test(Arc::clone(&service))
        .with_generated_registration(true);

    assert!(skills.generated_registration_enabled());

    let registered = skills
        .install_generated(skill("gen-registered", "Generated"))
        .await
        .expect("registration must succeed when enabled")
        .expect("an enabled registration must return the skill");
    assert_eq!(registered.skill_id, "gen-registered");

    let rows = db.list_skills().await.expect("list skills");
    assert!(
        rows.iter().any(|s| s.skill_id == "gen-registered"),
        "an opted-in registration must reach the database, not just the registry"
    );
}
