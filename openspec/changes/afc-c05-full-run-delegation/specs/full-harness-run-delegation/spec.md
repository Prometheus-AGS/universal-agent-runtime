## Purpose

Define a retry-safe, owner-scoped control contract for delegating one complete run to UAR's existing native executor while truthfully reporting process-ephemeral retention.

## ADDED Requirements

### Requirement: One native executor per delegated run

The full-harness surface MUST admit a complete native UAR run through the same run assembly and execution loop used by `/api/uar/runs`. It MUST return distinct native task and run identifiers, the effective service binding, and admission diagnostics without replaying tools in the adapter.

#### Scenario: Full run is delegated
- **WHEN** an authenticated owner admits a valid full-harness task
- **THEN** UAR reserves task and run identities before kernel entry and executes the request once through the native run manager

### Requirement: Idempotent admission and reconciliation

Admission MUST be scoped by authenticated owner, normalized workspace identity, `admission_id`, and a canonical digest of the complete request including workspace identity. An exact retry MUST return the original receipt; reuse with a different digest MUST return a typed conflict. The owner MUST be able to reconcile by admission ID within the same workspace without creating another run.

#### Scenario: Admission response is lost
- **WHEN** an owner repeats the same admission or queries its `admission_id`
- **THEN** UAR returns the previously reserved task and run identifiers without executing another run

#### Scenario: Admission key is reused for different input
- **WHEN** the same owner submits the same `admission_id` with a different canonical digest
- **THEN** UAR returns `admission_digest_conflict` and does not create another task or run

#### Scenario: Task is addressed from another workspace
- **WHEN** the same authenticated owner attempts to reconcile, observe, or control a task using a different workspace identity
- **THEN** UAR returns `task_not_found` and reveals no cross-workspace task state

### Requirement: Explicit process-ephemeral authority

Task receipts MUST expose the current runtime epoch, terminal TTL, terminal record cap, and `process_ephemeral` retention mode. A task from another runtime epoch MUST produce `recovery_unsupported`; `retention_expired` is reserved for a known current-epoch task removed by terminal TTL or record-cap pruning. A current-epoch task whose effect cannot be established MUST produce `task_unresolved`.

The versioned profile capabilities response MUST expose the current runtime epoch and `unsupported_after_restart` recovery posture so admission-only reconciliation can distinguish process restart from durable recovery.

#### Scenario: Runtime restarts before lookup
- **WHEN** a caller presents a task identifier issued by a prior runtime epoch
- **THEN** UAR returns a typed `recovery_unsupported` result and does not claim durable recovery

### Requirement: Lifecycle controls preserve their meanings

Observation and replay MUST NOT cancel a run when a client disconnects. Detach MUST require and record an `observer_id` without requesting cancellation. Approval, cancellation, and detach MUST require `expected_revision`; a stale revision MUST return `revision_conflict` without applying the requested mutation. Cancellation receipts MUST distinguish requested, acknowledged, terminal, and cleanup-uncertain states: terminal is true only after native `RunStatus::Cancelled`, and cleanup uncertainty is true only when native history contains explicit cleanup-unconfirmed evidence. Tool approval MUST require an exact non-empty `approval_id` and forward that identity to the existing run approval authority; an empty identity MUST return `approval_invalid` before mutation. Steer MAY be unsupported only through a typed `capability_unsupported` refusal and MUST NOT create a replacement run.

#### Scenario: Observer disconnects
- **WHEN** a full-harness event-stream subscriber disconnects
- **THEN** the native run continues and remains available to another authorized observer

#### Scenario: Caller detaches
- **WHEN** an owner detaches from a task
- **THEN** UAR records the detach and leaves cancellation unrequested

#### Scenario: Caller submits a stale mutation
- **WHEN** an approval, cancellation, or detach request presents an `expected_revision` different from the current receipt revision
- **THEN** UAR returns `revision_conflict` and does not apply that requested mutation

#### Scenario: Caller cancels
- **WHEN** an owner requests cancellation
- **THEN** UAR reports request and executor acknowledgement separately from terminal completion and cleanup certainty

#### Scenario: Caller steers an unsupported run
- **WHEN** an owner sends a steer request in the process-ephemeral profile
- **THEN** UAR returns `capability_unsupported` and does not create a new run

### Requirement: One externally visible lookup authority

Full-harness REST reconciliation and A2A `tasks/get` and `tasks/cancel` MUST resolve the same owner-and-workspace-scoped full-harness task record. A2A transports MUST require authenticated workspace context for those task IDs and an expected revision for cancellation or refuse the projection. Existing non-full-harness A2A tasks remain backward compatible.

#### Scenario: Task is read through A2A
- **WHEN** the authenticated owner looks up a full-harness task ID through A2A
- **THEN** UAR projects the authoritative full-harness record rather than creating or consulting a second task record
