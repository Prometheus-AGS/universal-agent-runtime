# Repair004 CAS source-only handoff

Phase: `uar-post-lifecycle-verification-readiness`, Execute. Operator verification gate: **CLOSED**. This is preparation only, not acceptance or permission to execute a gate. Other writers share this checkout; ownership was limited to the two CAS files below. COUNT remains lead-owned.

## Observed defect and scope

Packet004 `main-boundary-v1.3.0-004/cas-store-collaborator.json` records an actual unrelated adapter contention result of `Ok(false)` while `exactTargetUnchanged=true`, classified `FAIL_UNRELATED_CONFLICT_REPORTED_AS_CAS_MISS`. Source in `transition` previously converted every typed `QueryError::TransactionConflict` to `Ok(false)`, without inspecting the target predicate. This is a production classification defect, not grounds to weaken the contention oracle.

The same packet records valid exact success/miss pairs in both concurrent attempts, with `b` winning the `a-first` launch and `a` winning the `b-first` launch. Both exact readbacks matched the winner. The fixture nevertheless required the first-launched future to win. The shared barrier proves attempts are concurrent; neither barrier nor `tokio::join!` establishes a database commit ordering. This is a fixture acceptance defect.

## Files changed

- `src/uar/persistence/providers/agui_runs/surreal.rs`: serialize `before` once, retain the existing exact-key conditional UPDATE, and on a typed transaction conflict read raw `data` from that same key. Return a CAS miss only if that read observes absence or one value different from the exact serialized UPDATE predicate. If the target is unchanged, the read fails, decoding fails, or multiple values appear, preserve the original typed transaction conflict. Other errors keep their existing propagation. No retry, dependency, schema, credential, or persistence-contract change.
- `tests/support/post_lifecycle_store.rs`: retain `expectedOrderingObserved` as diagnostic evidence, record `observedWinner`, and require two exact matching success/miss readbacks with both actual winner directions represented. A repeated winner still leaves coverage BLOCKED. All foreign-owner, raw schema error, unrelated adapter contention, schema event cleanup, and scope-release assertions remain in place. No additional schedule, attempts, or fixture effects were introduced.
- This handoff records the evidence and limits. No other source file is owned or changed by this teammate.

## Predicate semantics and remaining uncertainty

The follow-up read is a separate observation. It can race with additional target writers. A mismatch makes the CAS predicate false at that later observation; it does not prove that the target caused the original transaction conflict, and does not establish a forced internal interleaving. An unrelated conflict with an unchanged target retains its original typed error. If independent target mutation occurs between the original error and the read, the mismatch disposition is justified only by the later predicate observation, not by causal attribution. If the intended contract requires preserving an unrelated conflict even in that latter case, a stronger datastore-supported causal discriminator is required; none is claimed here.

No upstream version-behavior claim was added. The repair uses existing local query, bind, response-check, and result extraction APIs already present in this module. Pins remain unchanged, including SurrealDB 3.3.0.

## Commands and verification status

Only source inspection (`cat`, `sed`, `rg`) and authorized source/documentation writing (`apply_patch`) were performed. No test, lint, formatter, typecheck, compiler probe, Cargo command, build, packaging command, review-driven verification, `git diff --check`, service access, signing, adoption, or external mutation ran. The first earlier combined source read reported a missing project-local Rust workspace skill; the installed skill at `/Users/gqadonis/.codex/skills/prometheus-rust-workspace/SKILL.md` was then read. Two speculative source paths were absent; the actual CAS module was located through the parent module's `#[path]` and `rg --files`. These were inspection failures, not verification results.

Rust workspace, Rust best practices, Rust async patterns, version pins, waypoint, relevant gotchas and Rust rules were consulted. The operator lock overrides skill/rule verification suggestions. Source changes are uncompiled and unexecuted. Packet004 evidence remains failed; this handoff does not replace or revise it.

The lead must reconcile the whole-phase ledger, collect all handoffs, bind the final exact source identity in a renewed inventory, present remaining tasks and exact gate commands, and wait for operator instruction before any verification. No fresh source-identity check was executed by this teammate. The lead owns COUNT repair, global inventory, integration ordering and the authoritative waypoint.
