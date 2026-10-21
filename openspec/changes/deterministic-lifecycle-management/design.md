# Unified lifecycle implementation contract

The operator approved the unified plan on 2026-10-20. Source architecture: `docs/perfect-lifecycle-management-architecture.md`; assessment: `/Users/gqadonis/.claude/plans/continue-with-plan-twinkling-teapot.md`.

## Ownership and evidence

Every task and resource has one trusted host owner. Atomic admission and task registration prevent work escaping shutdown. Retained state prevents lost lifecycle notifications. Cancellation never proves completion. Explicit finalizers survive caller cancellation. Owned engines close; borrowed clients release only UAR scope. Transaction commit uncertainty is distinct from local release.

## Completion order

Close admission; finish producers; settle admitted operations; drain accepted history writes; release durable execution ownership; close subscriptions and transports; drain resource leases; close owned stores; retain the aggregate report. Partially initialized resources use the same closure graph on failure.

## Event discipline

No polling, lock probes, retry sleeps, or fixed-duration waits establish readiness, cleanup or durable outcomes. Actual schedules/expiry and user budgets remain business events. Explicit authenticated reconnect commands and host events initiate connection attempts. Lag and disconnection require generation invalidation and authoritative snapshots.

## Evidence limits

Surviving handle ownership leads the historical lock hypothesis; existing cleanup logs do not establish all users are gone. Live-query residue is not established as the lock cause. Missing historical 503 bodies and sidecar failures remain unknown. The implementation must preserve sanitized future evidence rather than invent retrospective causes.

## Integration ownership

Lead: server.rs, manifests, realtime, release fixtures, canonical KBD and OpenSpec. Runtime: src/uar/runtime/**. State: src/uar/persistence/**, src/uar/memory/**, isolated pinned SDK patches. Trust: credential stores/API and sidecar/live fixture ownership. No overlapping writers. First-party memory export changes require an upstream source commit and normal pinned export; never patch exported bytes independently.

## Approved caller-budgeted shutdown observation

On2026-10-21 the operator approved an additive shutdown observation contract.
A caller supplies its own wait budget. The observer returns Pending with a
stable shutdown identity and outstanding resources when that budget expires.
The same supervisor continues owning cleanup, cancellation, tasks and dependency
ordering until actual acknowledgement. Repeated or canceled callers do not
create a second finalizer. Completion retains failures and unresolved effects;
completed observation does not imply a successful release report.

The budget may cancel only the caller's wait. It must not own the finalization
future, abort workers, silently detach resource obligations or close dependencies
early. Initiation must survive cancellation even before the async registry gate
is acquired. Pending observation must not block on that registry gate. Existing
unbounded shutdown remains compatible; no implicit duration is introduced.

Source ownership: runtime owns `src/uar/runtime/lifecycle.rs`; lead owns the
additive embedded-host wrapper if required, OpenSpec, documentation and canonical
coordination. Existing private sidecar framing and launcher behavior are not
expanded merely to expose this Rust host capability. No dependency upgrade or
new test is included.

The uncomfortable limit remains: a Pending return is a bounded observation,
not finite termination of uncooperative native or remote work. Dropping the
executor or killing the process still cannot provide graceful closure evidence.

## Verification boundary

Gate CLOSED. No tests, compiler probes, lint, format, builds, packaging, policy validators, git diff --check or artifact review until all implementation in the active phase is complete and the operator approves exact commands.
