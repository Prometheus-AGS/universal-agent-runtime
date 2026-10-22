# AG-UI settlement source review

Phase: Execute, `uar-post-lifecycle-verification-readiness`. Artifact-only scope: current authored settlement spans in `src/uar/runtime/manager.rs` and `src/uar/api/agui.rs`, plus `.agent-team/uar-core/agui-settlement-repair.md`. Reviewed actual producer ownership, graph-root join, actor-root finish, authenticated run lookup, shared handoff confirmation, and AG-UI reconciliation/stream source. No implementation conversation was used.

## Disposition

No blocking finding in the scoped repair from source inspection. Runtime concurrency and completion behavior remain unverified.

`await_settled_run_status` retains the exact producer handle under its slot lock while joining. Other waiters serialize on that same slot. After successful producer and graph joins, reading terminal status directly avoids revoking this established proof merely because another waiter acquired an idempotent join lock. Ordinary producer join errors set Error while retaining the producer-slot lock, before ordinary concurrent waiters can observe an empty handle. Actor finish retains its own failure state; graph joins retain worker failure and session cleanup. Shared caller handoff confirmation still requires Done and persists `source_settled` through CAS before admitting the public handoff.

The AG-UI terminal-frame branch calls `get_run_for_context` before its host-only await helper. That lookup checks exact verified subject/tenant ownership. It does not infer ownership from wire IDs. Before a terminal frame, reconciliation retains its existing nonblocking status probe. Once a retained terminal frame exists, actual join completion replaces the previous event wait, which could stall after the final frame. Projection still handles Error before reporting success and does not manufacture a completion event. Both public and background reconciliation use this same path.

No new lock cycle was found: the checkpoint registry lock is released after cloning the producer slot; the producer-slot guard ends before graph-root joining; AG-UI reconciliation holds no Entry or persistence transaction lock across the await helper; ordinary finalization does not await the AG-UI observer. The fixed diagnostic writes only run ID and static category, without raw error text.

## Evidence and limits

Only source reads (`cat`, `sed`, `rg`, scoped `git diff`) and this report write ran. No tests, builds, checks, imports, formatting, runtime, or network commands ran. No production file was edited. Verification remains CLOSED for this reviewer; the lead owns its scoped local functional test/fix loop. This review establishes no phase acceptance, build result, regression result, or publication authorization. CI and merge state were not assessed.

This pass covers normal join completion, ordinary join-error handling, and concurrent non-cancelled waiters. It does not establish behavior under interruption while publishing a join failure, forced task abortion, or panic outside the owned producer. These are not newly reproduced failures. Authored regressions and real AG-UI handoff/text-only completion must be exercised by the lead's authorized gate. Same-family review is not cross-model adversarial evidence.
