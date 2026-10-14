# Dot08 Execute handoff — 2026-10-12

## Delta from plan and production inventory

Implemented durable admission and fenced publication rather than attaching a retry loop to the old per-chunk writes. No external queue, object store, lease, or dependency was added. Pure extraction/embedding may repeat after process loss or on another runtime; publication is the fenced, atomic effect. The source implementation inventory below is complete for this slice, but this is not a verification or phase-acceptance claim. Canonical KBD tasks and OpenSpec checkboxes were not changed by this teammate.

- Admission: immutable bytes, owner-scoped document and job commit in one supported database transaction before the API emits 202. In-memory/unattested providers fail closed; no-worker acceptance retains discoverable work.
- Discovery: startup plus five-second database scans independent of wake notifications; bounded local dispatch through the existing WorkerPool. Startup reconciliation requires reupload for legacy missing-source documents.
- Processing: immutable source read fenced by current job state; stable chunk identities per generation and index; preparation has no writes. Retry changes generation. Duplicate preparation is safe at the publication boundary.
- Publication: complete chunks, embedding fingerprint, document chunk count/Indexed state and terminal job commit atomically. PostgreSQL row locks and Surreal conditional transactions fence deletion/concurrent publication.
- Failure/recovery: conditional durable failures, up to five attempts with 30s × attempt delay, explicit failed state, authenticated job metadata and retry routes. Processing jobs survive crash without a lease expiry. Shutdown cancels dispatch even while waiting on storage and signals existing worker stages.
- Deletion: source/job/document/chunks revoked atomically; stale preparation cannot recreate them. KB deletion includes all jobs. PostgreSQL uses existing composite foreign keys plus new job foreign keys; Surreal uses one transaction.
- Migration/operator documentation: supported profiles, attestation, retention, migration and rollback behavior documented. Both migrations mark legacy missing-byte pending/processing records failed before workers are available.

## Files changed

- `src/uar/api/knowledge.rs`: durable upload acceptance, owner-scoped job GET and retry POST, optional wake notification, unsupported/error response mapping.
- `src/uar/rag/ingestion_worker.rs`: durable discovery, claim/recovery dispatch, cancellation, completion bookkeeping, CAS failure state and atomic publication call.
- `src/uar/rag/ingest.rs`: `prepare_ingestion`, complete generation preparation and deterministic owner/document/generation/index chunk IDs; existing directory and direct text ingestion APIs retained.
- `src/uar/persistence/mod.rs`: durable capability and fallible admission/discovery/source/transition/publication/reconciliation methods, unsupported defaults.
- `src/uar/persistence/ingestion_records/mod.rs` (new): typed job lifecycle, publication and errors. Source bytes are deliberately absent from public job metadata.
- `src/uar/persistence/providers/postgres.rs`: narrow new-method delegation and atomic document deletion via FK cascades.
- `src/uar/persistence/providers/surreal.rs`: narrow delegation and transactional KB/document deletion.
- `src/uar/persistence/providers/ingestion_records/postgres.rs` (new): actual PostgreSQL transactions and reads.
- `src/uar/persistence/providers/ingestion_records/surreal.rs` (new): actual Surreal transactions/reads and schema initialization. Transaction error mapping inspects every result using the existing provider pattern, so cancelled statements do not conceal a deliberate refusal.
- `migrations/20261012000000_durable_ingestion.sql` (new): job/source table, owner-scoped cascading FKs, discovery index, legacy reconciliation.
- `migrations/surrealdb/durable_ingestion.surql` (new): additive schema/index definitions, legacy reconciliation.
- `tests/durable_ingestion.rs` (new, authored only): real database and in-process production HTTP router admission; joined child processes end after acceptance and claim before another process reads/recover/publishes. Covers source and owner isolation, duplicate and stale generations, deletion, retry, rejected missing-KB admission and KB cascade. Reuses `tests/support/api_key_process.rs` Database support. Does not invoke a worker or hosted provider.
- `docs/operations/durable-ingestion.md` (new): operator/API and rollback contract, caveats and pending evidence.

No edits to domain/knowledge.rs or providers/memory.rs were needed: existing document response/status remains compatible; unsupported trait defaults prevent memory acceptance. No edits to server.rs, manager.rs, surreal_host.rs, memory modules, Cargo manifests/pins, workflows or workflow scripts by this teammate.

## Root wiring and ownership

The root already applied the two requested composition changes:

1. `server.rs`: wrap successful `IngestionWorkerPool::new(...)` in Arc, call `pool.start_durable_dispatch()`, return the Arc. Method returns `()` and is once-only per pool.
2. `surreal_host.rs::migrate`: `super::ingestion_records::initialize(db).await?;` makes schema setup apply to both network/new construction and existing embedded host handles.

Existing synchronous `shutdown()` remains the shutdown entry point. It cancels the dispatcher and workers and uses the existing pool shutdown. A joined OS exit is still the durable embedded database release boundary; no same-process lock-release claim is made.

Shared writer authority for persistence/mod.rs and providers/{surreal,postgres,memory}.rs was explicitly released to the root before this handoff. Root can advance the source ledger and assign the next persistence slice. Other unrelated dirty edits were preserved.

## Evidence and deferred work

Commands executed were source reads/searches and file writes only. Context7 retrieved the official SurrealDB transaction documentation; implementation follows existing repository conditional-transaction and SQLx transaction patterns. No Cargo, compiler, test, build, lint, format, policy validator, probe, or review gate ran. There is no pass receipt.

Authored target: `tests/durable_ingestion.rs`. At the full expanded Dot implementation boundary the lead can include `cargo test --locked --no-default-features --features server-full,postgres-backend --test durable_ingestion -- --nocapture --test-threads=1` in the serialized production gate. It requires isolated durable `UAR_API_KEY_TEST_SURREAL_URL` (optional USER/PASS) and `UAR_API_KEY_TEST_POSTGRES_URL` infrastructure via the reused Database helper. This command has not been executed.

Compilation, SQL execution/schema compatibility, actual worker extraction/embedding, queue saturation, commit outage, two-runtime concurrent preparation/publication and customer artifact remain unverified. The bounded fixture covers crash-before-publication storage semantics; the final production gate must supply the actual worker/outage/concurrency/customer-build evidence required by the complete phase plan. Existing earlier receipts do not certify these files.

## Guard provenance and uncomfortable tradeoff

All new capability, owner/scope, generation and state checks protect the reported accepted-upload loss and the real publication/deletion trust boundary. Retry delay/attempt limits constrain repeated processing failures while preserving bytes and visible failure state. Completion notification ensures abandoned local work remains eligible for rediscovery. No unrelated speculative hardening was introduced.

The uncomfortable thing: durable chunks can be exactly-once visible while hosted extraction/embedding is executed more than once and charged more than once. The implementation deliberately makes no exactly-once provider-call claim. Large source/publication transactions may exceed operational database limits and fail without an acceptance/publication success. Source bytes remain stored until document/KB deletion; no silent retention policy was invented.
