# A2A mailbox handoff source review

Phase: Execute. Reviewed current `src/uar/api/a2a/thread_service.rs` changes and `openspec/changes/openai-client-tools/handoffs/a2a-mailbox-handoff-repair.md`, with the exact actor reply/stop and manager notification source. Own report only; no production edits.

## Disposition

No blocking source finding in the scoped mailbox/registration repair. Real continuation, streaming and shutdown behavior remain unverified.

- A pending caller batch no longer authorizes local public handoff while `Entry.turn` is live. `persist_entry_projection` forces Working and removes the caller extension until mailbox completion is consumed. `get_durable` preserves the local get's decision instead of independently projecting InputRequired from producer settlement. Approval InputRequired remains a separate existing mechanism.
- Actor UserRun sends its reply after session execution returns, the session lock leaves scope, and artifact collection closes. Entry consumption of that exact reply therefore supplies the local mailbox completion boundary. The projection still performs actual manager settlement and durable handoff confirmation before exposing caller calls.
- Streaming and settled waiters install retained registration and committed-work receivers before initial run lookup and authoritative reads. A registration racing missing-channel lookup remains a watch change; a completion racing a snapshot remains a committed-work change. The loop reattempts exact run subscription when needed rather than treating initial absence as source loss.
- Existing subscribed-source closure and streaming lag remain errors. The nonstream snapshot waiter retains its existing lag reconciliation behavior. Notifications are wakeups for authoritative reads, not proof of success. The owning observer notifies only after settled projection and immediately exits; it does not create a self-sustaining notification loop.
- Extraction remains inside admitted observer execution; rejected spawn leaves the original completion receiver in Entry. Normal observer cancellation/error restores any unconsumed original receiver to the exact unchanged run before dropping the proxy sender. Forwarded completion is handled synchronously. The prior receiver-loss correction remains intact.
- Removing caller projection joins while a local mailbox turn remains live removes the reviewed premature join-under-Entry-lock path. After the actor reply is consumed, its session execution lock is already released. No new lock cycle was found in these spans. This is not proof that all other shutdown paths are free of stalls.

## Evidence and limits

Source reads only (`cat`, `sed`, `rg`, scoped `git diff`) and this report write. No tests, builds, checks, imports, formatting, runtime, network or compiler probes ran. Global reviewer verification stays CLOSED; the lead owns its scoped local functional test/fix loop. No acceptance, publication or graceful teardown claim is made.

The cause of the recorded three shutdown-observer stalls remains UNKNOWN; removing a plausible source dependency does not establish that attribution. The required-choice Internal error and all ten gate007 A2A failures are not certified fixed by this review. Forced abort/panic restoration is outside the normal-exit ownership claim. This same-family isolated source review is not cross-model adversarial evidence.

## Gate008 corrective source review

Reviewed the new stale-revision rebase, local approval projection, completed replay and observer loop-top cancellation spans together with the latest handoff section. **No blocking source finding in these scoped corrections.** This does not review or accept the separate current-actor-policy cohort 13.15.

- New-turn admission reloads the authoritative owner/agent/task binding under Entry lock, requires unchanged run/process and no cancellation, then uses that current record as the CAS predecessor. Context, contract, current history and metadata are preserved in that rebased transition; only the new run/digest/process/epoch/submitted state and appended request change. Actual CAS still refuses competing transitions. Local publication deliberately resets per-turn metadata under the existing rules; this is not a blanket promise to preserve all prior-run metadata indefinitely.
- Local approval metadata now updates the shared Entry task and commits through its existing projection while holding Entry's lock. The run is rechecked after reacquisition; a newer local turn is not overwritten by the earlier out-of-lock snapshot. This removes the separate local outer-CAS writer that advanced revision without synchronizing Entry. Current storage CAS arbitration and same-run/process checks remain.
- Completed replay locates the batch by exact authenticated owner/tenant/agent and call locator rather than assuming the binding still points to the predecessor run. The required session derivation matches `ActorCollaboration::spawn_a2a_session`. Shared preparation still validates protocol, complete histories, declarations/controls and exact results. The returned saved Task must match the requested task and context. The read-only completed path starts no new actor turn and does not revive cancellation or a closed current turn.
- Noncompleted batches still require no cancellation, exact source-run match, InputRequired and no live/closed local Entry turn before shared preparation can claim. Completed read-only replay is the only state exempted from these new-execution guards. Delegation contract and exact context checks remain before either path.
- The top-of-loop cancellation check runs even when a CAS conflict sets `first_snapshot=true` and bypasses select. Returning from that inner loop still reaches the outer receiver restoration; the original receipt is restored under the exact run binding before the proxy sender drops. No accepted operation is abandoned merely by this new top check. This addresses starvation between operations, not an indefinitely blocked operation itself.

Only source inspection and this report append ran. No tests, builds, checks, imports, formatting, runtime or network commands ran. These are source-level corrections, not proof of gate008 failure attribution, successful continuation/replay or graceful shutdown. The recorded observer-stall cause remains unproven. Global reviewer gate remains CLOSED and the lead owns its scoped local functional loop.

## Current registered-agent preclaim follow-up

Inspected the one-line `resolve_registered_agent(agent)` addition in `resume_client_tools`. It is inside the noncompleted batch branch, after existing task/mailbox admission checks and before shared preparation can claim. A missing current registered artifact propagates an error before that claim. Completed read-only replay skips this execution prerequisite and still passes shared exact schema/history/result validation and saved task/context checks. This preclaim existence check does not itself snapshot or apply current policy; the separately reviewed actor-root refresh performs that work at execution. No blocking source finding in this addition. Only a source read and this report append ran; no executable verification or runtime command ran.

## Build009 error conversion follow-up

Inspected the sole corrective expression `map_err(|error| TaskError::Host(error.into()))` and the handoff's reported E0631. `resolve_registered_agent` returns typed `AgentStoreError`, which derives `thiserror::Error`; `TaskError::Host` accepts `anyhow::Error`. Explicit conversion preserves the original error as the host error source rather than discarding it or manufacturing a fallback. Admission order and Completed replay behavior are unchanged. No blocking source finding. Compilation remains unverified by this reviewer; only source reads and this report append ran, without checks, builds, tests, imports, runtime or network commands.
