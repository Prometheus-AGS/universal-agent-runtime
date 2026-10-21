# Runtime producer lifecycle continuation

## Scope and verification boundary

This source-only continuation belongs to
`uar-outstanding-issues-v1-1-1::function-first-release-certification`.
Verification is closed. No test, build, formatter, lint, typecheck, compiler
probe, review gate, or acceptance script has run.

## Confirmed producer contracts and changes

- `src/uar/rag/ingest.rs` adds `IngestService::watch_with_readiness`. It sends
  `oneshot::Sender<Result<(), String>>` only after native watcher construction
  and recursive registration complete; it sends the registration error before
  returning when either step fails. The existing `watch` API delegates without
  a readiness receiver. Its event wait is stop-biased, so cancellation wins over
  an already queued filesystem event. The watcher task still owns the native
  handle until its actual return.
- `src/uar/rag/ingestion_worker.rs` already keeps every accepted preparation in
  the pool scope and awaits its blocking `JoinHandle`. The blocking runtime
  entry now calls `ResourceScope::run_in_scope` before `executor.execute`, so
  task-local lifecycle authority crosses that otherwise non-inheriting boundary.
  Dropping a mailbox retrieval waiter therefore cannot release preparation.
- `src/uar/runtime/manager.rs` retains the run-history writer in its own scope,
  closes sender admission before draining it, tracks retention and skill
  evolution in the manager scope, and retains checkpoint producers for joined
  shutdown. The raw retained checkpoint producer now re-enters the manager
  scope with `run_in_scope` inside its owned Tokio task.
- `src/uar/runtime/thread/graph_host.rs` retains graph-root worker handles and
  joins them from `GraphRootSupervisor::shutdown`. The worker now re-enters the
  manager scope across `tokio::spawn`; this preserves task-local lifecycle
  authority without changing graph-root join ownership.
- `src/uar/runtime/team_execution/controller.rs` no longer borrows the manager
  scope for its notifier and direct-work admission. It owns a child scope that
  the existing team-execution finalizer closes after cancellation and joined
  worker handles. Shutdown closes child admission before snapshotting jobs,
  cancels and joins local jobs even when this process no longer owns the durable
  catalog claim, and retains catalog/child-scope failure evidence. A dispatch
  claimed just as shutdown begins is settled as a known zero-effect cancellation
  before a model worker is launched.
- `src/uar/runtime/thread/service.rs` keeps its existing retained child-job
  registry and joins, while restoring the current host `ResourceScope` across
  the root job and its separately spawned local-kernel execution when one is
  present. Thread-service callers outside an admitted scope retain the existing
  manual join contract rather than inventing a new scope owner.
- `src/uar/runtime/native_skills/search_tools_gate.rs` now retains every
  accepted controlled-probe filesystem task by stable task ID. Finalization
  joins accepted tasks without treating the probe's business deadline as a
  completion result. Blocking control I/O uses the inherited lifecycle
  `spawn_blocking` helper, so a dropped control waiter cannot detach an
  accepted operation or hide a panic through `is_finished` pruning.

The server composition now creates the ingestion watcher readiness receiver
before task admission and awaits it before marking `ingestion-files` ready.
That integration is owned by the lead in `src/server.rs`; this document records
the `IngestService` contract it consumes.

## Inventory observations

- Run retention is a manager-scope task. It wakes from committed activity or a
  one-shot business due time and observes both root and scope cancellation.
- Skill evolution is admitted to the manager scope. Rejection during admission
  is explicit, and accepted work is included in scope closure.
- Observer delivery has its own scope, receives both controller and scope
  cancellation, and is closed by the observer finalizer before downstream
  persistence finalization.
- Schedule and follow-up controllers do not spawn independent loops. Observer
  delivery invokes their authoritative scan/due methods, so their work shares
  its acknowledged owner.
- The run-history writer ends from sender admission closure, drains accepted
  writes, and joins while the run-history persistence dependency remains
  available.
- Production A2A task execution source inspected for this pass did not add a
  runtime-owned detached producer; the visible `tokio::spawn` occurrences are
  test fixtures.

## Source-only scenarios (not executed)

1. A host shutdown during native ingestion watcher construction waits for the
   watcher task’s registration result or error; readiness cannot precede actual
   OS registration.
2. A queued ingestion filesystem event and cancellation select cancellation
   first. A file ingestion already accepted remains awaited by its watcher task.
3. A caller drops while extraction runs in a blocking thread. The pool scope
   retains the accepted parent and waits for the actual blocking join.
4. A graph root, run checkpoint, or team worker reaches a nested blocking
   helper. The retained Tokio worker restores its owner scope first.
5. Team catalog ownership is lost after local work was admitted. Local worker
   cancellation and joining still happen; the ownership inspection failure is
   retained instead of being treated as a clean close.
6. Shutdown crosses durable team dispatch after claim but before worker launch.
   The attempt records a known cancellation rather than an unowned `running`
   state.

## Outstanding limits

A permanently silent accepted durable operation remains an outstanding
obligation; no elapsed time is converted into release. The separate gRPC SDK
transport-driver limit is recorded in
`docs/operations/lifecycle-grpc-continuation.md`.

The source changes establish ownership and ordering contracts. They do not prove
the historical embedded-store lock, concurrent migration 503, or sidecar stall
root causes.
