# Runtime failure cohort handoff — 2026-10-16

## Scope

This handoff covers task 6.2's binding, stream, and native-MCP failures from
`evidence/acceptance/20261016-local-02`. The KBD phase remains Execute. The
full-phase verification boundary remains closed; no verification command was
started for this work.

## Changed paths

- `src/uar/runtime/turn/host/event_projection.rs`
  - Adds `AdmissionProjection::begin_reduced_failure`.
  - It discards all staged pre-admission events and enables projection of only
    scrubbed `Error`, `RunDone`, `RunDoneWithUsage`, and `Cancelled` frames.
  - It does not mark normal admission complete and cannot publish ordinary
    model, skill, or MCP content.
- `src/uar/runtime/manager.rs`
  - Uses the reduced failure projection only in the model-binding error branch,
    before emitting the existing public stage/category diagnostic reference.
  - Heap-indirects the admitted producer future and the spawned producer
    future before handing the full tool loop to Tokio.
- `tests/support/uar_1_1_1_runtime_fixture.rs`
  - Keeps the binding refusal diagnostic-reference assertion and adds an
    assertion that a binding refusal emits terminal/error frame types only.
- `scripts/release/uar_1_1_1_runtime.py`
  - Passes the support-layer whole-body deadline, `total_timeout=240`, for
    both runtime streams.
  - Requests AG-UI replay with `last_event_id=0`, so a settled run returns the
    retained terminal event instead of opening a live-only stream at the last
    retained cursor.

## Diagnosed failure causes

### Binding diagnostic reference

The observed failure at `runtime_fixture.rs:121` was caused by
`AdmissionProjection::project_or_stage`: before ordinary admission completed,
it replaced every error message with generic pre-admission text. The manager
did generate the binding stage and UUID reference, but the projection erased
it before history publication.

The new narrow failure seam preserves only the already-reduced host diagnostic
and terminal lifecycle. It clears staged private material, retains the captured
scrubber, and drops later nonterminal frames. This avoids the unsafe alternate
of calling `complete` after a refused or over-limit admission projection.

### AG-UI stream did not finish

The local receipt reports `IncompleteRead(2568 bytes read)` after 7,705
seconds. `routes.rs` defaults an AG-UI request with no cursor to the last
retained event. A settled run then has no replay and waits for future broadcast
traffic. The runtime integration had omitted that cursor on its second stream.
The script now requests the zero cursor and uses the support layer's monotonic
whole-body deadline.

The later Docker/OrbStack disappearance was separately explained by the host
VM's `StorageFull` / read-only BTRFS event. It does not explain the earlier
open AG-UI stream.

### Native MCP incidental `isError`

The native control test's expected product semantics are supported by source:
the native skill's top-level `isError` is opaque successful payload data;
`ToolOutcome::Succeeded` is retained, a tool-result message is added for the
second model call, and `ToolEnd.ok` stays true. The failure record only proves
a Tokio test-worker stack overflow while executing that real native tool turn.
It contains no stack trace or diagnostic report locating a recursive frame.

The full producer captures the tool loop, checkpoint host, and finalizers on
the path that the early binding refusal never reaches. `manager.rs` now boxes
that producer construction and spawn future at the Tokio boundary. The change
keeps the existing cancellation and join-handle ownership; it is a bounded
mitigation for the observed large future construction shape. It is not a claim
that recursion was proven as the root cause.

## Interface effects

- The new projection method is a host-only failure-publication seam. Callers
  can publish terminal/error frames after an early admission refusal without
  enabling normal content publication.
- The release integration now requires the already-added support API
  `Http.stream(..., total_timeout=...)` and sends an explicit AG-UI replay
  cursor.
- Native tool result and receipt semantics are unchanged.

## Verification state

No test, Cargo command, compiler probe, lint, formatter, build, or review
verification was run. This is required by the phase-wide full implementation
boundary. The changes remain source-inspected only.

## Remaining limits and coordination

- The actual stack-overflow frame is unverified because the failed run produced
  no backtrace and the phase boundary prohibits a new probe. The boxed-future
  mitigation must be evaluated by the already-planned real native-MCP case
  after the phase implementation inventory closes.
- Real supervisor qualification remains an external prerequisite. Recovery
  code can validate evidence from a genuinely qualified isolated controller;
  this checkout cannot fabricate the guest/controller identity, controller
  journal, image/setup identity, or physical-containment proof. The lead is
  coordinating the disposable guest authorization.
- The stream whole-body timeout implementation belongs to the shared support
  owner. This script call site is aligned to the agreed backward-compatible
  interface.
