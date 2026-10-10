# Exact execution approval cutover

Both root decision endpoints require the exact approval_id from the originating
approval event: POST /api/uar/runs/{run_id}/approval and /tool-approval. Send an
explicit boolean approved. Missing, null or empty identity returns 400 with
approval_id_required; malformed typed JSON may be rejected by the extractor.
A stale, foreign, consumed or unavailable identity cannot select the current
waiter. Authorized-owner lookup remains required before decision delivery.

The SSE event agui.tool_call.approval_required carries request_id (root run),
approval_id (opaque decision identity), admission_id, tool-call identity and
prepared arguments. Clients must retain that event identity through display,
decision, reconnect and submission. Never substitute the tool name, run ID,
stream cursor or a newly fetched latest waiter. An owner-scoped pending snapshot
can restore the same identity after reconnect; it does not create a decision.
Repeated frames with the same identity do not permit repeated effects.

The public run-only RunManager::resolve_approval helper is removed. Internal
trusted callers use resolve_approval_request with Some(originating_id); None
fails closed. All standalone roots and actor descendants share ApprovalBroker.
Actors inherit the request-only RootApprovalChannel, not a human-resolution
message or capability. Exact ID comparison occurs under the pending mutex before
the sender is removed. Foreign/stale decisions leave legitimate pending state
intact. Root and caller cancellation refuse resolution, including a cancelled
child whose future has not yet polled; waiter drop clears only its own ID.

A decision is separate from host authorization and effect claim. Existing C05
full-harness expected_revision checks, immutable prepared invocation, host
receipt revalidation and single claim-before-effect lifecycle remain unchanged.
Cancellation cannot undo an already started external effect. An uncertain result
remains unknown until authoritative reconciliation; retries and reconnects must
not restore permission to claim the invocation again.

This is a coordinated strict cutover with owned Boss clients and UAR helpers.
Old run-only clients are incompatible; there is no local or remote fallback.
Rollback disables admission rather than silently approving the latest waiter.
UAR helper clients parse complete originating SSE frames and deduplicate identity;
negative ownership fixtures submit valid IDs so payload errors cannot masquerade
as owner isolation. Existing cancelled-epoch live scenarios already retain their
originating IDs and need no migration.

Source scenarios cover missing/stale/foreign/replayed decisions, exact root and
child decisions, cancellation before resolver polling, waiter drop, concurrent
decisions and legitimate pending preservation. HTTP fixture source pairs denied
owner/foreign decisions with one successful owner effect and duplicate rejection.
These scenarios and compiler/build gates have not been run. This source handoff
provides no runtime acceptance receipt. The independent blocked session diagnosis
and any conditional session-manager work remain outside this change.
