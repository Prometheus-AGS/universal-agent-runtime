# iOS known failed HTTP settlement repair handoff

Phase: `uar-post-lifecycle-verification-readiness`, task2.9. Implementation authored;
verification gate **CLOSED**. Files frozen at this handoff. No source identity was
computed. Lead must capture the next authorized identity after collecting writers.

## Files changed

- `scripts/acceptance/ios/runner.py`: admit only known terminal completed/failed
  HTTP reservations with matching binding, epoch, input identity and model kind;
  require matching status/state, nonnegative response bytes, SHA256 response hash
  and finite positive ordered reservation/outbound/settlement timestamps. Completed
  reservations additionally require actual output tokens within the existing cap.
  Capture an invocation reservation baseline and mark its result FAIL when any new
  model reservation failed, even if XCTest returned zero. The terminal receipt and
  original initial-failure path are retained before optional authorized correction.
- `scripts/acceptance/ios/README.md`: document known failure settlement, unchanged
  full reservation retention, failed invocation outcome and closed unknown/pending
  accounting behavior.
- `scripts/acceptance/tests/test_ios_failure_policy.py`: **7 new test methods**, all
  authored and unrun. Cases cover500/401, malformed/status/binding/timestamp/hash
  failures, UNKNOWN, pending-to-failed settlement, original deadline expiry,
  completed status/usage evidence and preservation of an existing failed item.
- This handoff is the only additional documentation write.

## Mechanism and scope

Source inspection followed the reported blocker into `budget_proxy.reserve`,
`outbound`, `settle` and HTTP handling: non-2xx known terminal responses become
`failed`; only `completed` releases unused response bytes. The runner performs
read-only snapshots and never changes reservations, counters, limits or ledger
state. It does not refund, redispatch models, reset accounting, introduce a repair
controller or relax success usage. Existing optional operator-authorized correction
remains unchanged. A corrected invocation excludes already settled reservations in
its own baseline; the original failed terminal is retained.

The additional terminal guards address the exact risk introduced by admitting
failed terminal states: arbitrary failed labels cannot count as known HTTP
settlement. The per-invocation FAIL guard addresses zero-exit/native success
otherwise incorrectly earning PASS despite a known failed model request.

Pending settlement still uses `min(DEADLINE, monotonic()+90)` and existing0.25second
polling. Binding change, unknown accounting, cumulative limit enforcement and
ownership/cleanup predicates remain in place. No consumption or allowance changed:
repair18 is the assigned authored repair under unchanged28cap; fixture19 remains
future preparation only, leaving9future product slots; USD10 per-provider caps and
all prior consumption remain retained.

## Commands and verification limits

Only `cat`, `sed`, `rg`/`rg --files` source reads and `apply_patch` text writes were
issued. One `rg --files scripts -g AGENTS.md` search returned exit1 (no matching
paths); this was source discovery, not verification. No tests, imports, syntax
probes, builds, checks, formatting, source snapshot computation, external access,
services, providers, simulator actions, reviewers or delegates were run. The
systematic-debugging skill was read; execution/reproduction phases were deferred
under the explicit gate lock.

All runtime behavior, test syntax, platform acceptance, actual ledger integration,
timestamps under clock changes, cleanup and correction execution remain unverified.
The failed-item regression proves the intended policy when eventually run; it does
not exercise the full nested `execute.attempt` simulator path. No implementation
acceptance or whole-phase completion is claimed. No unresolved design choice was
introduced; any future test or review requires explicit operator authorization.
