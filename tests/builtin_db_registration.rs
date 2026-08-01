//! R1: every discovered builtin skill reaches the database, on every provider.
//!
//! # What this asserts, and why it is equality
//!
//! The acceptance criterion is **row count == the loader's discovered count**,
//! not "some builtin rows exist". A subset test passes while skills are silently
//! dropped, which is the exact failure this change exists to catch: a UI that
//! lists skills from the database would show a partial catalogue and look
//! healthy doing it.
//!
//! # Provider coverage
//!
//! UAR ships three: `postgres`, `surreal`, `memory`.
//!
//! **Correction made while writing this test.** I assumed `memory` was the
//! embedded case. `Cargo.toml` says otherwise: `default = ["minimal"]` ->
//! `surreal-backend`, described as "SurrealKV-backed, fully embedded — no
//! external services required". `in-memory-backend` is off by default and
//! documented "do not use when state must survive process restart".
//!
//! So **`surreal` is the embedded path R1 exists for**, and it is the one that
//! must never be skipped. `memory` is still exercised here because it is the
//! only provider needing no server at all, which makes it the fastest place to
//! assert the equality invariant itself.
//!
//! Providers needing a live server **skip loudly** when their env var is unset
//! rather than passing vacuously — a silent skip lets this protection rot.

use std::sync::Arc;

use universal_agent_runtime::uar::domain::skills::{Skill, SkillOrigin};
use universal_agent_runtime::uar::persistence::PersistenceLayer;
use universal_agent_runtime::uar::runtime::skills::service::SkillService;

/// Builtins as the loader produces them: `origin = Builtin`.
fn builtin(id: &str) -> Skill {
    let mut s = Skill::default();
    s.skill_id = id.to_string();
    s.title = id.to_string();
    s.origin = SkillOrigin::Builtin;
    s.enabled = true;
    s
}

/// THE ASSERTION, on the embedded provider.
///
/// `register_builtins` puts skills in the in-memory registry. This test asks the
/// separate question R1 actually cares about: **did they reach the database?**
#[cfg(feature = "in-memory-backend")]
#[tokio::test]
async fn every_builtin_reaches_the_database_on_the_memory_provider() {
    use universal_agent_runtime::uar::persistence::providers::memory::InMemoryProvider;
    let db: Arc<dyn PersistenceLayer> = Arc::new(InMemoryProvider::new());

    // No VectorMatcher — this is the embedded case. An embedded host has no
    // embedding service, and R1 says "no matter whether embedded or not".
    let service = SkillService::new(Some(Arc::clone(&db)), None);

    let discovered = vec![
        builtin("uhe008-alpha"),
        builtin("uhe008-beta"),
        builtin("uhe008-gamma"),
    ];
    let discovered_count = discovered.len();

    service.register_builtins(discovered).await;

    let rows = db
        .list_skills()
        .await
        .expect("list skills from the database");
    let builtin_rows = rows
        .iter()
        .filter(|s| s.origin == SkillOrigin::Builtin)
        .count();

    assert_eq!(
        builtin_rows, discovered_count,
        "builtin rows in the database ({builtin_rows}) != skills the loader discovered \
         ({discovered_count}). Builtins registered into the in-memory registry but never \
         reached persistence, so any consumer reading the database — the UI, the REST API, \
         a mobile embedded host — sees a partial catalogue while the process looks healthy."
    );
}

/// Equality must not be satisfiable by over-registration either. A provider that
/// wrote duplicates, or leaked a non-builtin as builtin, would break consumers
/// just as badly as dropping one.
#[cfg(feature = "in-memory-backend")]
#[tokio::test]
async fn the_database_holds_exactly_the_discovered_set_no_more() {
    use universal_agent_runtime::uar::persistence::providers::memory::InMemoryProvider;
    let db: Arc<dyn PersistenceLayer> = Arc::new(InMemoryProvider::new());
    let service = SkillService::new(Some(Arc::clone(&db)), None);

    service
        .register_builtins(vec![builtin("uhe008-only-one")])
        .await;

    let rows = db.list_skills().await.expect("list skills");
    let ids: Vec<_> = rows.iter().map(|s| s.skill_id.as_str()).collect();

    assert_eq!(
        ids,
        vec!["uhe008-only-one"],
        "the database must hold exactly the discovered set; got {ids:?}"
    );
}

/// Postgres — **UNBLOCKED and verified.**
///
/// # How the block was resolved
///
/// This was previously BLOCKED: `migrations/20251225000000_init_uar.sql:2`
/// requires `CREATE EXTENSION vector`, and Homebrew's pgvector bottle ships only
/// `postgresql@17`/`@18` while the local server was 16.14. A PG17-built `.so` in
/// a PG16 install is an ABI mismatch, not a fix.
///
/// Resolved by using the **PG18 + pgvector image already proven in
/// `flint-forge`** (`docker/postgres/Dockerfile`), rather than fighting the
/// Homebrew packaging. Verified live: PostgreSQL **18.4**, pgvector **0.8.5**.
///
/// Note that image pins `PGVECTOR_REF=v0.8.5` deliberately — its own comment
/// records that 0.8.0 does not compile on PG18, because pgvector called
/// `vacuum_delay_point()` with no arguments while PG18 changed the signature.
/// Reusing a solved problem beat re-solving it.
///
/// ```bash
/// docker build -t flint-forge/postgres:18 \
///     ~/Projects/prometheus/flint-forge/docker/postgres
/// docker run -d --name uhe008-pg -e POSTGRES_PASSWORD=postgres \
///     -e POSTGRES_DB=uar_test -p 5440:5432 flint-forge/postgres:18
/// DATABASE_URL=postgres://postgres:postgres@127.0.0.1:5440/uar_test \
///     cargo test --features postgres-backend --test builtin_db_registration
/// ```
///
/// Skips **loudly** without `DATABASE_URL` rather than passing vacuously — a
/// silent skip lets this protection rot unnoticed.
#[cfg(feature = "postgres-backend")]
#[tokio::test]
async fn every_builtin_reaches_the_database_on_postgres() {
    use universal_agent_runtime::uar::persistence::providers::postgres::PostgresProvider;

    let Ok(url) = std::env::var("DATABASE_URL") else {
        eprintln!(
            "SKIPPED postgres: DATABASE_URL unset. This test proves builtins reach \
             Postgres; it must not be reported as passing when it never ran. See the \
             doc comment for the flint-forge PG18 image that unblocks it."
        );
        return;
    };

    let provider = PostgresProvider::new(&url)
        .await
        .expect("connect to DATABASE_URL and run migrations");
    let db: Arc<dyn PersistenceLayer> = Arc::new(provider);

    // Clean any prior run so the equality assertion measures THIS run.
    for id in ["uhe008-pg-alpha", "uhe008-pg-beta", "uhe008-pg-gamma"] {
        let _ = db.delete_skill(id).await;
    }

    let service = SkillService::new(Some(Arc::clone(&db)), None);
    let discovered = vec![
        builtin("uhe008-pg-alpha"),
        builtin("uhe008-pg-beta"),
        builtin("uhe008-pg-gamma"),
    ];
    let discovered_count = discovered.len();

    service.register_builtins(discovered).await;

    let rows = db.list_skills().await.expect("list skills from postgres");
    let builtin_rows = rows
        .iter()
        .filter(|s| s.skill_id.starts_with("uhe008-pg-"))
        .count();

    assert_eq!(
        builtin_rows, discovered_count,
        "postgres holds {builtin_rows} of the {discovered_count} builtins the loader \
         discovered. This is the third provider R1 requires; a shortfall here means \
         builtins do not reach a server-backed deployment."
    );
}

/// Surreal — **the embedded path, and it needs no server.**
///
/// This was nearly recorded BLOCKED on "requires a live SurrealDB". It does not:
/// UAR compiles `surrealdb` with `kv-surrealkv`, an in-process engine. Since
/// `surreal-backend` is the default feature and the documented embedded
/// deployment, declaring this unexercisable would have left R1's own platform
/// unproven while reporting a tidy BLOCKED — the failure mode the acceptance
/// criteria warn about.
///
/// Uses a temp-dir `surrealkv://` path, **not** `"memory"`. `normalize_endpoint`
/// maps `"memory"` to `mem://`, but that engine is not compiled in — it fails
/// with `Unsupported scheme: memory`. Only `kv-surrealkv` is enabled, so the
/// embedded store must be file-backed.
#[cfg(feature = "surreal-backend")]
#[tokio::test]
async fn every_builtin_reaches_the_database_on_embedded_surreal() {
    use universal_agent_runtime::uar::persistence::providers::surreal::SurrealDbProvider;

    let dir = std::env::temp_dir().join(format!(
        "uhe008-surreal-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    let endpoint = format!("surrealkv://{}", dir.display());

    let provider = SurrealDbProvider::new(&endpoint, None, None, Some("uhe008"), Some("uhe008"))
        .await
        .expect("embedded SurrealKV must start without a server");
    let db: Arc<dyn PersistenceLayer> = Arc::new(provider);
    let service = SkillService::new(Some(Arc::clone(&db)), None);

    let discovered = vec![
        builtin("uhe008-surreal-alpha"),
        builtin("uhe008-surreal-beta"),
        builtin("uhe008-surreal-gamma"),
    ];
    let discovered_count = discovered.len();

    service.register_builtins(discovered).await;

    let rows = db.list_skills().await.expect("list skills from surreal");
    let builtin_rows = rows
        .iter()
        .filter(|s| s.origin == SkillOrigin::Builtin)
        .count();

    assert_eq!(
        builtin_rows, discovered_count,
        "embedded Surreal holds {builtin_rows} builtin rows but the loader discovered \
         {discovered_count}. This is the default backend and the embedded deployment \
         target, so a shortfall here means R1 fails on the platform it exists for."
    );
}

/// Loading from a provider must NOT write back.
///
/// Found while fixing the persistence gate: `initialize()` and `refresh()` read
/// skills via `list_skills()` and then called `register_all`, which now
/// persists. Because `save_skill` upserts `embedding = EXCLUDED.embedding`, a
/// host with no `VectorMatcher` would overwrite a good stored embedding with an
/// empty one on **every restart** — silently degrading vector search on a
/// database that was previously healthy. The fix for one silent data-loss bug
/// would have introduced another.
///
/// # Three earlier versions of this test were worthless
///
/// 1. Row counts against `InMemoryProvider`, whose `save_skill` takes
///    `_embedding` and **discards it** — it cannot observe a clobber at all.
/// 2. A counting double, but calling `initialize()` on a service with **no
///    providers attached**. `initialize` iterates `self.providers`; with none,
///    `list_skills()` is never called, the load path never runs, and "zero
///    writes" holds *even with the bug present*.
/// 3. Hand-implementing `PersistenceLayer` on a counting double — the trait has
///    **24 required methods**, so it did not compile (E0046).
///
/// This version counts at the **provider** layer, where the trait is small, and
/// asserts both that the load path ran and that it wrote nothing.
#[cfg(feature = "in-memory-backend")]
#[tokio::test]
async fn loading_from_a_provider_issues_no_writes() {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use anyhow::Result;
    use async_trait::async_trait;
    use universal_agent_runtime::uar::persistence::providers::memory::InMemoryProvider;
    use universal_agent_runtime::uar::runtime::skills::storage::{
        SkillStorageProvider, StorageProviderKind,
    };

    /// A provider that HAS skills, so the load path actually executes, and that
    /// counts any write attempted through it.
    #[derive(Debug, Default)]
    struct CountingProvider {
        saves: AtomicUsize,
    }

    #[async_trait]
    impl SkillStorageProvider for CountingProvider {
        fn id(&self) -> &str {
            "uhe008-source"
        }
        fn name(&self) -> &str {
            "uhe008 source"
        }
        fn kind(&self) -> StorageProviderKind {
            StorageProviderKind::Database
        }
        fn is_enabled(&self) -> bool {
            true
        }
        async fn list_skills(&self) -> Result<Vec<Skill>> {
            Ok(vec![builtin("uhe008-from-provider")])
        }
        async fn refresh(&self) -> Result<Vec<Skill>> {
            Ok(vec![builtin("uhe008-from-provider")])
        }
        async fn save_skill(&self, _skill: &Skill) -> Result<()> {
            self.saves.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
        async fn delete_skill(&self, _id: &str) -> Result<()> {
            Ok(())
        }
    }

    let db: Arc<dyn PersistenceLayer> = Arc::new(InMemoryProvider::new());
    let source = Arc::new(CountingProvider::default());

    let mut service = SkillService::new(Some(Arc::clone(&db)), None);
    service.add_provider(Arc::clone(&source) as Arc<dyn SkillStorageProvider>);

    service.initialize().await.expect("initialize");
    let _ = service.refresh().await;

    // The load path must have RUN — otherwise "no writes" is trivially true
    // because nothing was ever read.
    let loaded = service.registry().read().await.len();
    assert!(
        loaded >= 1,
        "the provider's skill never reached the registry, so this test proved \
         nothing about the load path"
    );

    // Reading must not have persisted anything back to the DATABASE.
    let persisted = db.list_skills().await.expect("list skills").len();
    assert_eq!(
        persisted, 0,
        "the load path persisted {persisted} skill(s). Reading must not write back: \
         `save_skill` upserts `embedding = EXCLUDED.embedding`, so a host without an \
         embedder would wipe stored embeddings on every restart."
    );

    // Nor back through the provider it came from.
    let provider_writes = source.saves.load(Ordering::SeqCst);
    assert_eq!(
        provider_writes, 0,
        "the load path wrote {provider_writes} time(s) back through the source provider"
    );
}
