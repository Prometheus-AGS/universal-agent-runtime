# Design

## Context

Issue #426 upgrades the current 0.1.0 export at ebb7c3ce02f7b925bc2e1b45c87ce8abf402b1f0. The operator approved newest inspected upstream main 4b747350c6883497813bed33623a9937ad7a5894 (manifest 0.3.0), 70 commits after the issue's original target. The selected source includes at-depth admission, capacity error context and stray-permit release hooks. This is an exact source snapshot; no published-release claim is made.

## Decisions

Use the upstream core dispatcher, not native WorkerPool. UAR retains ResourceScope ownership of the dispatcher, blocking workers and real joins. Native WorkerPool imposes execution budgets and bounded thread-exit waits; adopting it would widen the lifecycle migration.

The pinned facade does not re-export dispatch/semaphore modules. UAR therefore takes a direct local path dependency on the core crate from the same vendored workspace, alongside the facade's tokio-runtime feature. A thin production admission component wraps that dispatcher. Pool admission snapshots/watch receivers expose active/waiting descriptors and active result receipts for operational observation; observing or dropping a receipt does not transfer worker ownership.

One job costs one CPU-kind concurrency slot, not measured CPU utilization. Active capacity is worker_count (zero selects CPU count). max_queue_depth bounds only waiting descriptors; server configuration remains 100. Normal priority and zero head skips establish local FIFO. Existing durable discovery order remains authoritative outside this queue; no global upload FIFO guarantee is introduced.

Park only durable descriptors and prepare source bytes after a grant. Track queued/running durable attempt keys to avoid duplicate dispatch. Saturation defers discovery without changing HTTP acceptance or creating another durable queue. Return each grant through completion only after actual execution/join and durable settlement. Stray return paths wake the scope-owned dispatcher; shutdown never admits replacement work.

Migrate executor failures to TaskError and retrieval/admission errors to SchedulerError. Catch worker panic at the actual join boundary and preserve durable failed-attempt CAS. Result watch channels communicate completed outcomes; dropped callers do not own workers. No default execution budget is added. Existing explicit deadlines remain cooperative and cannot force non-yielding work to stop.

## Ownership and order

Lead: OpenSpec/KBD records, root Cargo.toml/Cargo.lock, server composition comments, changelog and memory. Vendor writer: vendor/git/prometheus-parking-lot-rs and vendor/git/README.md. Adapter writer: ingestion_worker and optional ingestion_admission module/mod declaration. Acceptance writer: ingestion integration tests and v1.3.0 delivery notes. All writers preserve the shared checkout and each other's changes. versions.toml remains read-only; provenance records the Parking Lot pin.

Finish vendor and production adapter, author all acceptance fixtures/docs, then bump root UAR to 1.3.0 and update its lockfile identity. Independently versioned JS/Tauri packages retain existing versions. Refresh the complete implementation inventory before any verification. Keep UAR's toolchain; a genuine incompatibility is a bounded blocker, not authorization to change pins.

## Verification boundary

AGENTS.md operator verification lock is CLOSED. No test, check, build, formatter, lint, compiler probe, independent code review or OpenSpec validation may run during implementation. Source inspection and authored tests provide no runtime PASS. After whole-phase implementation completion, reconcile canonical state and present exact commands for explicit operator approval. Changes after inventory close the gate again. No GitHub Actions test changes, external publication or paid jobs are included.

## Risks and rollback

A prematurely dropped grant can falsely free capacity while a nested operation still executes. Keep it in the scope-owned join path. Queue descriptors must not become a new durability or retry authority. Reverting the coherent vendor/adapter/manifest/lockfile change restores the old integration; do not revert durable publication records or independently changed user files.
