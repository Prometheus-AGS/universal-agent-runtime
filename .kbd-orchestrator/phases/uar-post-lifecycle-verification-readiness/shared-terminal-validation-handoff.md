# Shared terminal validation handoff

Phase: `uar-post-lifecycle-verification-readiness`, task `2.9`, Execute.
Status: source authored, frozen; all behavior and regression execution UNVERIFIED.
Operator verification gate: CLOSED. No checks, tests, imports, syntax/schema
probes, source-identity commands, builds, services, provider/network operations,
simulator operations, review gates, or extra delegation were performed.

This bounded authoring is repair19 within unchanged cap28; fixture20 is future
work, with8 later product slots retained. No counters, limits, prior consumption,
provider USD10 ceilings, approvals, waypoint or other phase metadata changed.

## Exact changed files

- `scripts/acceptance/budget_proxy.py`: pure `terminal_receipt_error` returns
  accounting/usage diagnostic category or None, with explicit caller-supplied
  epoch/input/binding/kind and response/output limits. No producer rewrite.
- `scripts/acceptance/packet_processes.py`: terminal admission calls that shared
  function, preserving existing `require` messages and outer Blocked wrapping,
  source/config/owner checks, read-only consistent transaction, per-reserved
  bounds, cumulative arithmetic, failed full reservation, completed response-byte
  reconciliation and at-most90second settlement wait.
- `scripts/acceptance/ios/runner.py`: snapshot minimally projects reservation
  counters and approved response ceiling; terminal admission calls the same pure
  function with the stricter actual reservation/binding bounds. Keeps existing
  RuntimeError diagnostics, pending wait, UNKNOWN stop, failed-invocation guard,
  native genuine-success evidence and unchanged collection behavior.
- `scripts/acceptance/tests/test_packet_settlement.py`: fixture now includes real
  reservation timestamp and response digest; known401/500 preserve full reservation.
  Four new test methods author malformed-terminal rejection, shared-validator
  wiring/limits, actual output-cap rejection and embedding usage behavior.
- `scripts/acceptance/tests/test_ios_failure_policy.py`: fixture now includes
  projected actual reservation and response ceiling. Two new test methods author
  shared-validator wiring and reservation-vs-binding limits. Existing malformed
  terminal regression expanded for both failed statuses and successful receipts.
- `scripts/acceptance/README.md`: shared contract and server retained guards.
- `scripts/acceptance/ios/README.md`: shared validator and projected per-call bounds.
- This handoff file.

New test methods:6 total (server4, iOS2). None executed. Expanded existing cases
cover null/NaN/infinity/zero/boolean timestamps, reversed chronology, missing or
malformed digest, completed vs failed HTTP consistency, actual successful model
usage, known401/500 reservation retention, pending/UNKNOWN behavior and FAIL
aggregation. Both callers have authored sentinel-validator rejection tests; this
is wiring coverage, not an executed parity claim. Shared embedding coverage is
pure-validator coverage because the iOS consumer intentionally admits model only.

## Shared admission contract and limits

Only completed/failed terminal states with actual integer HTTP100–599 and matching
success classification may pass. All four timestamps must be positive finite
numbers in nondecreasing order; booleans are rejected. Digest must be exactly
`sha256:` followed by64 lowercase hexadecimal characters. Actual response bytes
must be nonnegative integers within caller's bound. Completed model requires
nonnegative integer actual output usage within actual reserved output cap;
completed embedding requires nonnegative integer input/total usage. Failed HTTP
requires no successful usage and receives no refund or item promotion.

All added guards trace to the named malformed-evidence continuation failure in
`evidence/four-platform-preparation-005/code-source-review.md`, and the explicit
existing reservation contract. No dispatch, reset, ledger write or new generalized
framework was added. Documentation and requested tests are the only accompanying
work. No unrelated changes were added.

## Unknowns and next owner action

No runtime, syntax, import, platform integration, schema, source identity or test
claim is verified. The new iOS snapshot projection is internal; actual SQLite
read and integration are pending authorized verification. No independent review
was requested or run. Exact source identity must be recorded by the lead only
when separately authorized; none was generated here. Lead must reconcile this
handoff with the complete phase ledger and present the renewed whole-phase
inventory and exact proposed commands before any operator-approved gate.

Authoring frozen on delivery of this handoff. Subsequent changes require a new
lead instruction and renewed phase-boundary accounting.
