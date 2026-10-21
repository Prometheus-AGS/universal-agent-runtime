# Sandbox lifecycle ownership handoff

Date: 2026-10-20. Active waypoint: `function-first-release-certification`,
change `deterministic-lifecycle-management`, stage `execute-lifecycle`.
Operator verification gate: **CLOSED**. This is an implementation handoff,
not an acceptance receipt. No test, Cargo, compiler, formatter, lint, build,
validator, diff check, or review gate was executed.

## Source-grounded failure scenarios

The incoming `SessionManager::get_or_create` reserved `creating` and then
awaited `runner.create` in the caller future. Dropping that caller after the
backend had accepted creation could abandon both the reservation and the
eventual handle. `destroy_session` removed an entry before awaiting destruction
in the caller future, so dropping that waiter lost the manager's owned handle.
`shutdown` first awaited `ResourceScope::close` and only afterward destroyed
handles, making a retained `Released` report possible before physical release.

The incoming MCP tools called the raw runner and touched the session only
after success. A TTL deadline during execute/read/write could therefore
destroy a busy session. `RemoteRunner` mapped missing or invalid effect
acknowledgments to ordinary failures: a remote create could succeed while its
response was lost, leaving an unbound remote child eligible for an implicit
retry. These are source reproducers, not executed runtime reproductions.

## Changed paths and interfaces

- `src/sandbox/session_manager.rs`: resource state separated from the service
  facade; construction registers both retained state and its only finalizer
  before any acquisition. Create/destroy/use reservations and tracked task
  registration happen synchronously under the resource lock. Backend work
  runs in owned tasks with oneshot result delivery. A dropped result receiver
  never cancels that work. Handles stay in the ownership map until a successful
  destruction acknowledgment. Destruction waits for creations and admitted
  uses through subscribe-before-inspect retained events.
- `src/sandbox/mcp_tools.rs`: removes four success-only timestamp touches;
  execute/read/write now obtain their use lifetime through the managed runner,
  including failure and canceled-result paths.
- `src/sandbox/runner.rs`: documents the backend's known-failure versus
  unknown-effect outcome contract.
- `src/sandbox/types.rs`: adds `SandboxError::OutcomeUnknown(String)`.
  External exhaustive matches on this public enum must account for it.
- `src/sandbox/remote_runner.rs`: missing/invalid create/execute acknowledgments,
  missing write/destroy acknowledgments, and non-success effect HTTP responses
  preserve outcome uncertainty. Missing responses and received HTTP failures
  have distinct diagnostics. The inspected protocol has no authoritative
  negative-acknowledgment/rollback contract, so an HTTP error alone is not
  treated as proof that no effect happened. Mutation diagnostics omit remote
  response bodies and request URLs. An empty creation ID is an unbound outcome.
- `src/sandbox/execution.rs`: only the lead-assigned exhaustive telemetry arm,
  `OutcomeUnknown(_) => "outcome_unknown"`.
- `docs/operations/lifecycle-sandbox-handoff.md`: this ownership and acceptance
  handoff.

Public `SessionManager` method signatures and `runner() -> &dyn SandboxRunner`
are preserved. `runner()` now returns a scoped facade: arbitrary external or
stale handles are rejected, and `create`/`destroy` use manager ownership.
`new_in_scope` remains infallible but closes admission and records failure if
state/finalizer registration fails. Supplying a scope already owned by another
resource therefore fails closed before acquisition. Concurrent duplicate
destroy requests report destruction in progress rather than falsely returning
a completed release. `shutdown()` returns the retained `CloseReport`; callers
must inspect `released`, failures, and unresolved effects, not merely `Ok`.

The lead owns the supporting `ResourceScope::downgrade`,
`WeakResourceScope::upgrade`, `retain_resource_state`, and `resource_state`
APIs in `src/uar/runtime/lifecycle.rs`. Those are integration dependencies,
not this writer's changed paths. The scope retains `Arc<SessionResources>`
through failed finalization; that state owns no scope or manager. Finalizer
reporting uses only a weak scope observer. The GC task has a weak manager,
drops every upgraded facade before waiting, and stops on closed admission as
well as cancellation. This avoids spinning on an already-due TTL while task
admission is closed.

## Completion and uncertainty

Scope closure joins registered creation, destruction, use, and GC tasks before
physical cleanup. The single retained finalizer polls independent handle
destructions concurrently and aggregates failures, so one failed handle does
not prevent attempting another. Successful acknowledgments remove only their
own handles. Failed handles remain in scope-retained state. Late or canceled
shutdown waiters observe the same terminal report; shutdown does not launch
another cleanup pass.

Unknown creations reserve capacity and block recreation of that session.
Unknown destruction blocks reuse/retry of that handle. Unknown execute/write
effects quarantine the handle because the remote operation might still be
running; neither expiry nor shutdown destroys it on the assumption that the
local future ending proves remote idleness. An already-admitted destroy
rechecks that uncertainty after waiting for active use. Terminal reports retain
escaped opaque resource IDs (or the session key when no handle was bound) and
explicit unresolved outcomes. No timeout is treated as proof of completion.

TTL remains a one-shot business deadline. Active uses have no expiry deadline.
A known failed expiry is blocked per session until real session activity;
its own failure notification cannot restart an automatic retry loop. Work
generation uses `send_modify`, preserving the prior no-read-guard overwrite
fix and subscribe-before-inspect ordering.

The uncomfortable case is an unreachable remote service after accepting a
mutation: UAR cannot honestly recover its unknown child or effect using this
protocol. This implementation retains/quarantines that obligation and reports
failure; it does not invent a reconciliation API or silently retry. A backend
that never acknowledges admitted work can keep shutdown draining indefinitely.
Standalone embeddings must retain their owner and explicitly call shutdown;
dropping an unclosed standalone scope is not asynchronous cleanup.

## Authored acceptance scenarios — not executed

Use the actual manager and managed runner path with explicit peer/barrier
acknowledgments; wire remote scenarios through the real `RemoteRunner` HTTP
protocol. These scenarios are handoff specifications, not executable fixtures
or passing tests. The lead must integrate them into the whole-phase production
gate after all implementation is complete and the operator names the commands.

1. Hold a create acknowledgment after acceptance, cancel its waiter, then
   complete creation. Observe one backend create, one owned handle, and one
   destruction during close. A second same-session caller observes the first
   creation rather than creating another child.
2. Hold a destroy acknowledgment, cancel its waiter, and start two shutdown
   waiters. Neither may report release while destruction is held. Cancel one
   shutdown waiter, release the backend acknowledgment, and require the late
   waiter to receive the identical retained report without a second destroy.
3. Hold execute, read, and write separately; drop each result waiter while the
   backend operation is accepted. Advance the business TTL or explicitly
   request destruction. No destruction reaches the backend until the operation
   acknowledges completion. New use after a destruction reservation is denied.
4. Close admission while a TTL is already due. GC exits from retained lifecycle
   state rather than repeatedly scheduling rejected cleanup. No elapsed-time
   polling or retry interval is used as an assertion of completion.
5. Hold creation and request destruction for the same key. Creation remains
   owned; destruction waits for its bound handle. Race shutdown at reservation
   registration and require either an admitted owned task or a rolled-back
   reservation with no backend effect.
6. Fail one known expiry destruction. Its own completion event must not cause
   another attempt. Renew that session through actual use, then reach a new TTL
   to permit another attempt. Another session remains independently eligible.
7. Fail one finalizer destruction, panic another, and acknowledge a third.
   Require the third to be released, both unresolved handles retained, and
   aggregate failure/unknown diagnostics. A held fourth acknowledgment keeps
   closure pending without preventing the other attempts.
8. Let the remote peer accept creation and close the response without a usable
   ID. Require `OutcomeUnknown`, retained unbound-session diagnostics, occupied
   capacity, and no automatic second create. Distinguish received non-success
   HTTP status from a missing response without claiming rollback for either.
9. Let remote execute/write lose its acknowledgment while an explicit destroy
   is waiting. Require quarantine and an unknown terminal report; the queued
   destroy and finalizer must not destroy that possibly busy remote handle.
10. Drop the manager facade while retaining its supervisor scope. Close that
    scope and observe physical cleanup plus retained failed resource state.
    Repeat close observation and require no new finalizer authority or work.
11. Use `runner()` directly with an owned handle, an unrelated handle, and a
    handle removed/replaced before destruction reservation. Only the exact
    available owned binding reaches the backend. After close, all effectful
    facade entrypoints reject admission.
12. Supply an already closed scope or a scope with another state/finalizer.
    Construction must acquire no sandbox, and subsequent operations must fail
    admission instead of running with no cleanup owner.

## Execution record and remaining limits

Only source reads/searches and authorized source/document writes ran. Commands
included `cat`, `sed -n`, and `rg` against the waypoint, project instructions,
skills, task ledger, lifecycle source, sandbox source, and existing sandbox
tests. Existing tests were read, not executed. Python text edits and direct
`apply_patch` performed implementation writes. One remote-runner heredoc stalled
before Python execution; process inspection identified its own shell PIDs
78306/78766, which were terminated, producing exit 143. The edit was then
applied directly. That incomplete write attempt was not a verification gate.

No verification command ran and no tier earned acceptance. Compilation,
runtime ordering, panic behavior, remote acknowledgment classification, and
integration with the lead's concurrently authored lifecycle APIs remain
unverified. Production acceptance fixture execution is pending under the
operator lock. No pins, manifests, vendored code, or unrelated subsystems were
changed. No unsolicited feature was added. Guards implement the named canceled
waiter, busy-expiry, exact-handle, and lost-acknowledgment boundaries above.
