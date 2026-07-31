//! A builtin skill cannot be deleted, **even by a caller that bypasses
//! `SkillService`**.
//!
//! # Why this test exists in this shape
//!
//! `SkillService::delete_skill_permanent` already refuses builtins and maps to
//! HTTP 409. That guard lives in **one call path**. `DatabaseStorageProvider::
//! delete_skill` passes straight through to `DELETE FROM skills`, so any caller
//! holding the provider — a future maintenance task, a repair script, a
//! refactor that reaches one layer lower — deletes a pack-shipped skill without
//! ever meeting the check.
//!
//! So the test **deliberately does the bypass**. Asserting that the *service*
//! refuses would demonstrate the guard we already had; asserting that the
//! *provider* refuses proves the route is closed at the database, which is the
//! actual requirement.
//!
//! Requires `DATABASE_URL` pointing at a Postgres instance. Without it the test
//! **skips loudly** rather than passing vacuously — a silent skip would let this
//! protection rot unnoticed.

#![cfg(feature = "postgres-backend")]

use serial_test::serial;
use universal_agent_runtime::uar::domain::skills::{Skill, SkillOrigin};
use universal_agent_runtime::uar::persistence::providers::postgres::PostgresProvider;
use universal_agent_runtime::uar::persistence::PersistenceLayer;

fn database_url() -> Option<String> {
    std::env::var("DATABASE_URL").ok()
}

fn skill(id: &str, origin: SkillOrigin) -> Skill {
    let mut s = Skill::default();
    s.skill_id = id.to_string();
    s.title = id.to_string();
    s.origin = origin;
    s.enabled = true;
    s
}

#[tokio::test]
#[serial]
async fn provider_delete_of_a_builtin_is_refused_by_the_database() {
    let Some(url) = database_url() else {
        eprintln!(
            "SKIPPED builtin_delete_guard: DATABASE_URL unset. \
             This test protects pack-shipped skills from deletion; \
             run it with a Postgres instance before trusting that guarantee."
        );
        return;
    };

    // `new()` runs `sqlx::migrate!` itself, so the guard migration is applied
    // by connecting. Re-applying it here would test a hand-rolled setup rather
    // than the migration path real deployments actually take.
    let provider = PostgresProvider::new(&url)
        .await
        .expect("connect to DATABASE_URL");

    let builtin_id = "uhe007-builtin-probe";
    let user_id = "uhe007-user-probe";

    // Clean any leftovers. The builtin needs the trigger disarmed to remove, so
    // do it as raw SQL against the definition, not through the provider.
    let _ = sqlx::query("DELETE FROM skills WHERE skill_id = $1")
        .bind(user_id)
        .execute(provider.get_pool())
        .await;
    let _ = sqlx::query("UPDATE skills SET definition = jsonb_set(definition, '{origin}', '\"user\"') WHERE skill_id = $1")
        .bind(builtin_id)
        .execute(provider.get_pool())
        .await;
    let _ = sqlx::query("DELETE FROM skills WHERE skill_id = $1")
        .bind(builtin_id)
        .execute(provider.get_pool())
        .await;

    provider
        .save_skill(&skill(user_id, SkillOrigin::User), &[])
        .await
        .expect("save user skill");
    provider
        .save_skill(&skill(builtin_id, SkillOrigin::Builtin), &[])
        .await
        .expect("save builtin skill");

    // A user skill deletes normally — proving the guard is targeted, not a
    // blanket refusal that would make the test pass for the wrong reason.
    provider
        .delete_skill(user_id)
        .await
        .expect("user skills must remain deletable");

    // THE ASSERTION. Straight at the provider, no SkillService anywhere.
    let refused = provider.delete_skill(builtin_id).await;
    assert!(
        refused.is_err(),
        "a builtin skill was deleted through the storage provider — \
         the guard is not at the database and the bypass route is open"
    );
    let msg = format!("{:?}", refused.unwrap_err());
    assert!(
        msg.contains("system_skill_immutable"),
        "refusal must name the reason so an operator can act on it; got: {msg}"
    );

    // And it is still there.
    let survivors = provider.list_skills().await.expect("list skills");
    assert!(
        survivors.iter().any(|s| s.skill_id == builtin_id),
        "the builtin must survive the refused delete"
    );

    // Disabling is still allowed: the requirement is "turned off, never
    // deleted". A guard that also blocked disabling would over-shoot.
    let mut disabled = skill(builtin_id, SkillOrigin::Builtin);
    disabled.enabled = false;
    provider
        .save_skill(&disabled, &[])
        .await
        .expect("a builtin must remain disable-able");

    // Cleanup: disarm by rewriting origin, then remove.
    let _ = sqlx::query("UPDATE skills SET definition = jsonb_set(definition, '{origin}', '\"user\"') WHERE skill_id = $1")
        .bind(builtin_id)
        .execute(provider.get_pool())
        .await;
    let _ = sqlx::query("DELETE FROM skills WHERE skill_id = $1")
        .bind(builtin_id)
        .execute(provider.get_pool())
        .await;
}
