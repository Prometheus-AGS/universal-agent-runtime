# Issue #426 acceptance authoring handoff

Phase: `uar-post-lifecycle-verification-readiness`, Execute. Verification **CLOSED**.
This handoff is source authoring, not acceptance evidence.

## Files changed

- `tests/ingestion_admission.rs` (new): real production pool + embedded SurrealKV + local extraction + production IngestService/Chunker/publication, with a controlled embedding collaborator that holds the actual blocking worker using owned channels.
- `docs/releases/v1.3.0-parking-lot.md` (new): exact upstream pin, version1.3.0, resource/queue semantics, ownership, cancellation, uncomfortable uncooperative-worker case and proposed finite gate/environment requirements.
- This handoff. Existing `tests/durable_ingestion.rs` and `tests/lifecycle_owned_resources.rs` preserved.

## Authored scenarios

1. Six durable accepts, one active slot and one local waiting descriptor: all other generations remain Pending/attempt0 with retained source; completion dispatches every job once; repeated notifications cannot duplicate generations/chunks.
2. Two active slots: both real workers enter embeddings while the third remains unclaimed; releasing one grants the third capacity.
3. Held actual ingestion worker across shutdown admission closure, concurrent shutdown observers and canceled first observer: actual embedded-engine finalizer captures durable outcomes and closes the physical store only after worker join; cooperative stop prevents late publication; queued work/source remain recoverable.
4. Controlled embedding error and panic: failure persisted Retryable before successor starts, capacity reused, any unsuccessful lifecycle closure retains its actual failure evidence.

5. Actual active-mailbox receipt: poll legacy-zero-timeout retrieval to Pending, cancel that waiter, externally time out another; active grant/mailbox and queued Pending generation remain retained until actual worker completion wakes the successor.

6. Same public production `IngestionAdmission<ScheduledTask<u64>>` wrapper: reject oversized2vs1 demand explicitly, immediately admit eligible1slot, enforce waiting2 QueueFull, FIFO completion, require a fresh exceptional permit-drop wake then poke without a submission, close returns parked descriptor and rejects new admission.

No sleeps establish completion. Channel receipts hold/release workers; existing `ResourceScope::subscribe()` identities establish worker task joins. Thirty-second fixture waits bound failure reporting only.

## Coordination / unresolved dependencies

- Adapter agent owns `src/uar/rag/ingestion_worker.rs`; acceptance does not modify runtime source.
- Tests expect worker task IDs to include `:task:ingestion-dispatch:` for the retained dispatcher and use existing public pool constructor/submit/start/shutdown APIs.
- Adapter owner agreed operational `admission_snapshot()` and `subscribe_admission()` with `active_jobs`, `waiting_jobs`, and `active_mailboxes` keyed by durable storage key. Exact waiting1 bound and active-receipt dedup assertions authored. Mailbox receipt supports canceled/externally timed-out retrieval coverage. The runtime-free production `IngestionAdmission<T>` wrapper is now authored and covered through the same component used by the real pool. Adapter owner supplied exact SHA-confirmed ScheduledTask/Admission/Dispatched/Grant shapes; oversized/FIFO/bounded-queue/drop-wake/close coverage is authored.
- Tests require adapter panic handling to persist durable failure before successor entry. Panic coverage enters the production worker through an injected EmbeddingBackend, not an upstream-only unit executor.
- Source inspection of the newly authored pool confirms diagnostic fields, mailbox keying and worker task names match the acceptance contract. No unresolved acceptance design decision remains. Final vendor export and dependency/lock resolution remain owned by vendor/lead; compilation is unverified.

## Inspection / verification

Only source reads/searches and authorized test/document writes ran. No Cargo command, formatter, compiler probe, test, lint, build, diff check, independent review gate or external action ran. Compilation and behavior remain unverified. Do not mark this change accepted from source inspection.

Proposed local gate, requiring explicit operator approval after whole-phase inventory:
`cargo test --locked --no-default-features --features server-full --test ingestion_admission --test durable_ingestion --test lifecycle_owned_resources -- --test-threads=1`

The existing durable_ingestion fixture requires `UAR_API_KEY_TEST_SURREAL_URL` and optional `UAR_API_KEY_TEST_SURREAL_USER`/`UAR_API_KEY_TEST_SURREAL_PASS` for an isolated shared store. Conditional PostgreSQL coverage additionally needs its backend feature and isolated `UAR_API_KEY_TEST_POSTGRES_URL`. No new provider credentials are required for ingestion_admission.
