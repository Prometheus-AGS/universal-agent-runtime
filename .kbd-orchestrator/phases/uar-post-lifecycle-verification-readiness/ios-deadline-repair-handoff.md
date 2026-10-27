# iOS deadline repair handoff — source authored only

Phase: `uar-post-lifecycle-verification-readiness`, task `2.9`.
Verification gate: **CLOSED**. Source files are frozen at this handoff; source
identity was not computed. No review, test, check, import, syntax probe, build,
service, network, provider, simulator, or other verification was run or delegated.

Changed files:

- `scripts/acceptance/ios/runner.py`: ledger_snapshot accepts an optional remaining
  monotonic allowance, clips it to the original DEADLINE, clips connection and
  per-query SQLite busy waits to remaining time and the existing five-second
  default ceiling, interrupts reads through SQLite's progress handler, checks
  expiry after constructing the snapshot, and closes via contextlib.closing.
  settled_ledger passes remaining time, refuses an expired snapshot immediately
  after acquisition and again before successful return (including no-pending and
  empty snapshots), and clips polling sleeps to remaining time.
- `scripts/acceptance/tests/test_ios_failure_policy.py`: five new test methods
  authored for late completed/empty snapshots at settlement or overall expiry,
  validation crossing expiry, clipped pending polling/timeout, SQLite wait/read
  clipping plus deterministic close on success/error and default overall bound,
  and expired default snapshot refusing connection. One existing monotonic mock
  sequence is extended for the added pre-snapshot check. Existing snapshot mocks
  already accept the new keyword through unittest.mock.
- `scripts/acceptance/ios/README.md`: documents the deadline/read/close mechanism
  and explicitly unverified regressions.
- This handoff records source work and remaining limits.

All production behavior, authored regressions, SQLite progress callback timing,
lock interruption and connection closure are **UNVERIFIED**. SQLite progress
callbacks run at instruction intervals; post-read and pre-return checks refuse
late success rather than claiming exact wall-clock scheduling. No absolute
real-time preemption guarantee is claimed. No unresolved design choice was
introduced. Epoch/binding identity, counters, reservation retention, terminal
validator, known failed states and item outcomes retain their existing semantics.

The observed failure addressed is the supplied preparation-006 code-source-review
finding: a late SQLite snapshot could return settled success beyond the90second
settlement or original overall bound. Every added deadline guard traces to it.
No additional production scope was added. Standard-library contextlib only;
no package/dependency change. No source identity or acceptance result is supplied.

Accounting supplied by lead, not independently verified: repair20 authored once
inside unchanged28cap; fixture21 is only a future preparation entry, leaving7
later product slots. All prior consumption, original host-call4000 allowance,
and USD10 each caps remain unchanged. Last supplied account0293830/4000 was
not recomputed here. No new fixture execution or accounting reset was authorized.

Root owns packet refresh and any proposal to reuse retained isolated artifact-only
critic context. No reviewer was messaged or executed by this writer.
