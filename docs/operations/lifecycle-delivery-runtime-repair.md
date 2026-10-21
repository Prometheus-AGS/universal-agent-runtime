# Delivery runtime repair handoff

## Scope

This source-only repair addresses the named diagnostics in the authorized ARM64
Docker delivery build receipt `run39523808909` at UAR source `ad557b0d`.
It does not claim a successful rebuild.

## Changed files

- `src/uar/runtime/manager.rs`
  - Uses the actual tuple returned by persisted resilience resolution.
  - Uses the actual tuple returned by presentation capture; its current API
    already returns catalog warnings with the snapshot.
- `src/llm/orchestrator.rs`
  - Clones the native skill `Arc` for execution so result formatting retains
    the original handle.
- `src/uar/api/a2a/thread_service.rs`
  - Retains a diagnostic run-id clone outside the owned projection task.
- `src/uar/api/full_harness.rs`
  - Retains a diagnostic task-id clone outside the owned receipt task.
- `src/uar/runtime/instance/pump.rs`
  - Makes `spawn_owned_task` visible to the sibling instance lifecycle module.
- `src/uar/runtime/schedules/controller.rs`
  - States the `Option<DateTime<Utc>>` accumulator type for next-due folding.
- `src/uar/rag/embeddings/failover.rs`
  - Performs generation/active-member/compatibility validation and publication
    in one mutex-held recovery CAS, then retires rejected candidates only after
    the guard is released.

## Lifecycle and cancellation effects

The A2A and full-harness repairs leave their retained `ResourceScope` tasks,
cancellation tokens, and unresolved-effect reporting unchanged. The embedding
recovery repair preserves the stale-generation rejection and closes an
unpublished candidate only after no `FailoverState` mutex guard remains live.

## Source-only status

No verification command ran. No Cargo, compiler, test, formatting, lint,
typecheck, build, or review command was run by this repair. The corrected
diagnostics and all surrounding behavior remain unverified until the lead
renews the delivery candidate and obtains authorization for the next delivery
build.

