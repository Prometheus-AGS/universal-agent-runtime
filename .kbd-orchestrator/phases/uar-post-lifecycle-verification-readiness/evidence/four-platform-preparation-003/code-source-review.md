# A18 code source review

Outcome: BLOCKED_SOURCE_ONLY — first required unresolved blocker; remaining scope unreviewed.

Phase: uar-post-lifecycle-verification-readiness.
Supplied current source identity: sha256:c32a2b5dc153a28511c2c4e76ac88b4a8acbdaf2dfaa9d935a934982a8eeae0c.
Scope was derived exclusively from changed_regular_inputs_since_A15 in four-platform-whole-phase-inventory-002.json, plus the current preparation configuration artifacts subsequently assigned. No producer handoffs, generation history, previous review reports, execution history, amendment narratives, or .prometheus history were consulted.

## Finding

[HIGH] Server collection dispatches later items before shared upstream usage settles

File: scripts/acceptance/failure_sweep.py:239

Exact relevant source (lines 239–240):

    ctx.cleanup()
    result["cleanup_confirmed"] = True

The attempt then calls remaining() and returns (lines 258–259). remaining() (lines 188–194) checks local counters/time and packet.alive(), and packet_processes.py:82–85 checks only process liveness and its Context wall/output allowance. The row loop can subsequently call attempt(row, 0) for an independent item (failure_sweep.py:289–310) without reading shared reservation settlement.

Concrete trigger: an item dispatches a genuine provider request, then its downstream runtime/waiter is cancelled or fails while the proxy is still receiving the upstream response. Per budget_proxy.py:539–570, the proxy deliberately continues draining that already-dispatched request after downstream disconnection so authoritative usage can settle. The reservation remains reserved until budget_proxy.py:580–584. Meanwhile the runtime and exact-owned store can be successfully cleaned up, satisfying the controller's only continuation gate. The next independent item can start while the prior upstream request is still running and its usage remains unsettled. This violates the explicit serial continuation requirement and can dispatch additional work before a prior unknown-accounting condition becomes visible.

Existing guards do not close this gap:
- core.py:503–521 joins owned item processes and requires an empty resource cleanup receipt. The shared proxy is packet-owned, outside that item cleanup.
- observers_runtime.py:868–936 checks exact runtime/store/private-state teardown; it does not inspect shared forwarding reservations.
- budget_proxy.py:353–378 reserves under cumulative ceilings and rejects an already blocked epoch, but does not reject an existing reserved request. Conservative ceilings prevent an allowance reset; they do not establish serial completion or settled usage.
- packet_processes.py:104–108 checks the final ledger only at packet stop, after intervening dispatches.
- user_outcomes.py:192–214 admits runtime/store/source identity, not forwarding settlement.
- The iOS path already has an explicit settled_ledger gate (ios/runner.py:226–239); the server collection has no corresponding gate.
- Authored tests in tests/test_failure_sweep.py use a fake packet whose alive() is empty and a fake cleanup with no shared reservation state; these tests do not address this trigger.

Required correction: establish bounded, source-bound shared ledger settlement after exact-owned item cleanup and before correction/rerun or independent dispatch. Pending reservations must settle within the same epoch/allowance; unknown accounting or unconfirmed settlement must stop continuation. Preserve initial failure receipts and all shared counters. This is a requested real-boundary guard, not speculative hardening.

Confidence: high from the explicit draining and continuation paths. This is a source proof of a reachable unsafe ordering, not an observed runtime reproduction.

## Inspection record and stopping boundary

Read in full: scripts/acceptance/failure_sweep.py, scripts/acceptance/core.py, scripts/acceptance/packet_processes.py, scripts/acceptance/user_outcomes.py, scripts/acceptance/ios/runner.py, scripts/acceptance/observers_runtime.py, scripts/acceptance/budget_proxy.py.
Read partially: scripts/acceptance/platform_suite.py (through its platform execution loop), scripts/acceptance/prepare_platform_inputs.py (first 260 lines), scripts/acceptance/tests/test_failure_sweep.py (first 140 lines).
Read input: four-platform-source-005.json; extracted only the permitted changed_regular_inputs_since_A15 field from the inventory.

These reads were source inspection; no source was executed or imported. The report is the only file written. No shell, test, compiler, syntax/type/format probe, build, network/service/provider call, source change, or delegated review was performed. commands_allowed=[] was retained.

Stopped at this first blocker as instructed. The rest of the changed source set, remaining platform-suite/preparation code, and all three newly assigned current draft preparation configurations remain unreviewed. This report does not certify their consistency. Rust and Swift specialist reviews were not consumed.

No runtime certification, compilation claim, source-identity recomputation, or whole-scope PASS is made. Fresh-context native same-family isolation is weaker than an independent cross-model review.

## Review Summary

| Severity | Count | Status |
|----------|-------|--------|
| CRITICAL | 0 | pass |
| HIGH | 1 | warn |
| MEDIUM | 0 | info |
| LOW | 0 | note |

Verdict: WARNING — resolve the HIGH continuation blocker before resuming the remaining source-only review. Verification gate remains CLOSED.
