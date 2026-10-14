# Dot15 follow-up source handoff — 2026-10-12

Active phase: `dot-feature-support`, Execute. Production source, API/native
registration modules, operator docs and fixtures are authored. This is not
acceptance. No tests, compiler, formatting, validation, build or product review
gate ran. Root owns the canonical task status and full-phase implementation
boundary; shared changes were preserved.

## Source ownership inventory

- `src/uar/domain/followups.rs`: request, quiet policy, protected rule,
  authority references, decision/outbox, inbox item and safe inspection views.
  `domain/mod.rs` registers it. `domain/collaboration.rs` adds default-empty
  rule and inbox maps to existing durable catalog snapshots.
- `src/uar/security/claims.rs`: private-field `FollowupOwnerGrant`, a protected
  scoped wrapper over the already implemented complete schedule principal
  grant. `runtime/actor/messages.rs` restores it through the private host path.
- `src/uar/compiler/collaboration/followups.rs`: owner-scoped CRUD and catalog
  CAS transactions. Cursor/decision and inbox/delivered state commit together.
  Current binding revision, policy, binding representation grants and attached
  representation grants are rechecked in the committing snapshot. Its `mod.rs`
  registers the module.
- `src/uar/runtime/followups/mod.rs`: concrete host and distinct safe errors.
  `controller.rs`: default-disabled creation, revision edits, enable/disable/
  revoke, current authority, durable source cursor/gap/suppression handling,
  outbox dispatch, inbox read and idempotent acknowledgement.
  `quiet_hours.rs`: IANA instant membership, fold-aware ends and transition
  candidates for earliest real release after a missing end. `runtime/mod.rs`
  registers the capability.
- `src/uar/runtime/observer/controller.rs`: concrete optional follow-up host
  attachment. `delivery.rs`: invokes it from the existing two-second wakeup.
  No second timer, queue, agent executor or remote transport was introduced.
- `src/uar/api/followups.rs`: authenticated REST surface; `api/mod.rs` registers
  it. `runtime/native_skills/followups.rs`: nine host tools with normal mutation
  approval and verified execution context; its `mod.rs` registers the module.
- `docs/proactive-followups.md`: operator/API/native contracts, backend limits,
  authority boundary, quiet dates and deferred acceptance obligations.
- `tests/dot_followup_quiet_hours.rs`: exact dated quiet membership/release
  fixtures. `tests/fixtures/dot15_followups_real_path.py`: installed HTTP driver
  with seed/read/delivered/deferred/suppressed/enable/disable/revoke/edit/
  acknowledge/hidden modes and optional second-runtime reads. Both unexecuted.

Related narrow Dot14 updates made during integration: schedule `supported()`
calls its exact `require_store()` predicate; that predicate now requires durable
observer support because root starts the shared wakeup only with that capability.
Schedule get/control also use it. Destination binding storage failures stay
storage errors rather than becoming permission denial. Schedule API module now
has the same server feature gate as its neighbors. No dependency/pin was changed
by this worker.

## Root wiring packet

Root reports the following startup/API/native wiring source-authored:

```rust
let followups = FollowupController::new(catalog.clone(), agent_instance_controller.clone());
observer.attach_followups(followups.clone()).await;
native_skills::followups::register(&native_registry, followups.clone()).await?;
```

Attach before observer start; register before listening. Mount authenticated
`api::followups::build_router().with_state(followups.clone())` at
`/api/uar/followups`. Router state is concrete `Arc<FollowupController>`; no new
AppState field or unresolved abstract authority seam is needed.

Routes: GET/POST `/`, GET/PUT `/{id}`, POST `/{id}/{operation}` (enable, disable,
revoke), GET `/inbox/{conversation}`, and POST
`/inbox/{conversation}/{item}/acknowledge`. Create takes raw `FollowupRequest`;
edit takes `{expectedRevision, rule}`; controls take `{expectedRevision}`.
All require verified owner plus `x-uar-workspace-id`.

Native names: `followup_create`, `followup_list`, `followup_get`,
`followup_update`, `followup_enable`, `followup_disable`, `followup_revoke`,
`followup_inbox`, `followup_acknowledge`. Create/update use the `rule` property.
All take `workspaceId`; inbox/ack take `conversationId`, ack takes `itemId`.
Three read tools are ReadOnly/NotRequired; all mutations are
ExternalMutation/Required, Deferred, BuiltIn, UarRuntime admission.

Both controller `supported()` functions are public and call exactly
`self.require_store().is_ok()`. Followups requires a nonmemory catalog plus
durable agent instances and durable observer occurrences. Schedules additionally
requires durable accounting. Discovery must use live configured controller
instances, not broad provider labels. A predicate is availability information,
not evidence that cold-restart and multiruntime qualification have passed.

## Authority and atomicity

Authenticated CRUD captures full issuer/tenant/subject scope and exact durable
rule/workspace/destination authority. Restoration accepts the protected grant
type, not caller JSON, roles or a fresh arbitrary UserContext. Existing legacy
presentation partition is retained only to locate the owned instance/catalog
records; the rule and inbox use the complete principal key. Current instance,
conversation, binding and representation references are rechecked before
admission/dispatch/read. Failed instances can still report failure metadata;
disabled instances are denied. Storage outages do not consume events as revoked.

Each source cursor step and decision is one existing catalog CAS. Queued/deferred
decisions are the durable outbox. Delivery atomically inserts one inbox item and
marks its decision delivered in the same snapshot, with stable rule/occurrence
identity. Revision/consent and catalog grant checks are repeated on CAS retries.
Inbox reads recheck current authority, so revocation hides prior metadata too.
The logical inbox contains event identity/kind/timestamps, never source payload.

Competing follow-up scanners need no execution ownership transfer: this operation
only mutates metadata under catalog CAS. It never executes a model, tool, or
external connector. Dot11 exclusion remains a schedule/agent admission concern.
There is no external send status to reinterpret or retry. Adding a connector
later would require an explicit capability, approval and durable reconciliation
contract; this source does not claim those exist.

## Unresolved verification and runner obligations

No unresolved implementation design decision is handed to root. Source remains
uncompiled and unexecuted, and no acceptance claim is made. Concrete risk:
collaboration state, actor owner, logical instance and server wiring have been
changed concurrently and have not been compiled together. Windows IANA data
availability stays experimental, matching Dot14's qualification note.

The HTTP driver does not itself produce source events, start/stop installed
runtimes, revoke real grants, control a VM clock, or prove the complete
source-outbox-to-follow-up-inbox journey. It must be composed into Dot17's real
runner. Required caller work: create real authenticated logical instances and
conversation, generate committed production events, prove durable source
occurrences, crash at evaluation/decision commit/acknowledgement, restart cold,
race two shared runtimes, change source/destination grants and quiet policy,
then exercise the driver's assertions and independently count logical delivery.
The seed mode deliberately creates disabled consent; enable before the selected
source event for positive delivery, or retain disabled for suppression evidence.

The quiet-hours Rust fixture only proves calendar behavior when eventually run;
it does not prove durable delivery. Its dated Chicago cases are 2027-03-14
07:30Z -> 08:00Z for missing 02:30 end and 2027-11-07 06:30Z -> 06:45Z plus
07:30Z -> 07:45Z for both repeated 01:30 candidates. Retain installed timezone
data provenance in final evidence. All final gates remain deferred until root's
complete expanded-phase implementation inventory.

No unrequested framework, dependency, connector or UI was added. New guards
trace to explicit owner/grant/consent/revision/quiet-hour boundaries, real source
retention gaps, and the named storage-outage-versus-revocation failure scenario.
The uncomfortable thing: an authored fixture and plausible CAS code do not prove
cross-process revocation ordering or crash correctness; only final production
evidence can support those claims.
