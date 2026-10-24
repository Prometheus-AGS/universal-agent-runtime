# Issue 426 — production ingestion adapter handoff

Phase: `uar-post-lifecycle-verification-readiness`, Execute. Verification gate **CLOSED**. No test, compiler, build, lint, formatting, review verification, or external mutation was executed by this writer.

## Owned source changes

- `src/uar/rag/ingestion_admission.rs`: generic production `IngestionAdmission<T: Parkable>` policy over exact upstream runtime-free `Dispatcher`; positive u32 slots, independent max parked depth, strict FIFO (skip-ahead zero), oversized demand delegated to upstream before queueing, release-hook host `Notify`, explicit completion/poke, atomic closure and removal of parked descriptors.
- `src/uar/rag/mod.rs`: export admission module.
- `src/uar/rag/ingestion_worker.rs`: replace ignored queue argument and semaphore with lightweight durable descriptors admitted before claim or source load. Local admitted set deduplicates running/waiting document storage keys; discovery refills a bounded local capacity after actual completions. Database remains sole acceptance/backlog/retry authority. Active capacity remains CPU-count when zero supplied; each ingestion costs one CPU slot.
- Migrate `WorkerExecutor` to `Result<IngestionResult, TaskError>`, `SchedulerError`, `ResourceCost::new`, sequence zero. Failure results no longer masquerade as successful Failed-valued scheduler results. Execution panic, normal error, explicit deadline, source-loading failure and claim error attempt durable CAS settlement against retained exact generation before completion. CAS failure cannot overwrite a retry/deletion/publication. Failed store settlement is retained as lifecycle failure.
- Parent owns grant until blocking task joins and durable outcome handling completes, then sends completion containing permit. Dispatcher returns capacity through core complete and drains FIFO. Whole run_descriptor panic is caught in parent; actual generated attempted claim is retained, avoiding invented recovery generation. Exceptional grant drop notifies dispatcher to poke. Spawn refusal occurs before any durable claim; descriptor stays discoverable.
- Shutdown closes local admission, removes waiting descriptors without durable failure, signals cooperative interruption, and keeps dispatcher alive awaiting all active completions. No timeout detaches blocking work. Explicit deadlines checked between stages; default remains unbounded.
- Public operational observations `admission_snapshot()` and `subscribe_admission()` contain `active_jobs`, `waiting_jobs`, `active_mailboxes` by actual durable storage key. Legitimate mailbox retrieval wait cancellation cannot affect execution. Mailbox slots are removed by completion owner after outcome publication.

## Interfaces coordinated with acceptance writer

`IngestionAdmission::new(slots,max_queue_depth) -> Result<Self,SchedulerError>`; `submit(T,now_ms) -> Result<Admission<T>,SchedulerError>`; `complete(Permit,now_ms) -> Dispatched<T>`; `poke(now_ms)`; `close() -> Vec<T>`; count accessors and `install_release_wake(Arc<Notify>)`. Core ScheduledTask implements Parkable. Wrapper is used by production pool, not a test-only cost hook. Actual ingestion never accepts variable caller cost.

Preserved pool constructor argument order, submit document-ID return, retrieve signature (error migrated), shutdown and scope names `ingestion-dispatch`/`ingestion-prepare`.

## Source evidence and remaining uncertainty

Read exact SHA `4b747350c6883497813bed33623a9937ad7a5894` GitHub contents for core dispatch/dispatcher.rs, dispatch/parked.rs, semaphore/mod.rs, contracts/error.rs, contracts/executor.rs and facade core/mod.rs. Facade omits dispatch/semaphore exports; lead added direct core path dependency rather than editing upstream source. Source inspection of Postgres/Surreal next_due_at confirms they return only future deadlines, so pool reads them whenever local admission has room, including while another attempt runs.

No unresolved implementation decision within owned files. Source compatibility, supported-profile compilation and real runtime acceptance remain entirely unverified pending explicit operator-named commands. Acceptance writer owns six authored integration scenarios and has the final API shape. No unsupported claim of whole-phase completion is made.

### Source reasoning completion — settlement collaborator panic

Failure settlement itself is bounded by catch_unwind, so a persistence panic cannot escape the parent and skip Completion after execution has joined. The scope retains both failure and unknown durable-effect evidence; capacity still reflects actual execution lifetime and closure cannot claim clean release. CAS false is explicitly a settled stale attempt with no new claim. No retry or timeout was added. Guards are limited to observed/required scenarios: u32/usize constructor capacity representation, durable namespace boundary, closure/spawn refusal, exact generation CAS revocation, explicit deadline/cancellation between stages, executor/claim/settlement panic, and exceptional permit-drop wake. Queue bound is exactly caller-supplied waiting descriptors (including zero); running bound is caller worker count or CPU count for zero.
