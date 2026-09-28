# Spec Delta

## Purpose

Define durable, owner-scoped ordinary-agent instances whose bounded turns use the existing UAR execution kernel while activation and recovery remain observable and controlled.

## ADDED Requirements

### Requirement: Logical instance identity is separate from service and run identity

UAR MUST persist each logical agent instance under an authenticated owner and workspace with its own stable identity, exact definition and effective binding, revision, activation profile, lifecycle state, and bounded-resource policy. A logical instance MUST NOT be identified by a port, process, actor handle, service-instance ID, or individual root run ID. Instance reads and mutations MUST enforce the current owner and workspace.

#### Scenario: Same definition in two workspaces

- **WHEN** two authorized workspaces instantiate the same immutable agent definition
- **THEN** each receives a distinct logical instance and neither workspace can read, address, or mutate the other's state.

#### Scenario: A process restarts

- **WHEN** UAR restarts after committing an instance record
- **THEN** the instance retains its identity and revision while its old process-local activation is no longer considered active.

### Requirement: Activation profiles produce bounded fresh turns

UAR MUST distinguish request execution, on-demand activation, and explicitly enabled resident activation. Every admitted mutating turn MUST create a fresh root run through the existing thread kernel, resolve current policy and effective binding, and retain a durable association between logical instance, submitted command, attempt, and root run. No profile MAY require an immortal model loop or reuse a completed root as a new turn.

#### Scenario: Warm worker receives another request

- **WHEN** an on-demand instance finishes one turn and receives another
- **THEN** the second turn receives a new root run and current authority while the logical instance identity remains stable.

#### Scenario: Unsupported required binding

- **WHEN** the instance's required definition, service placement, policy, model, or skill binding is no longer admissible
- **THEN** UAR refuses the new turn with an inspectable diagnostic and does not execute a substitute model or fallback agent.

### Requirement: Instance inbox and turn scheduling are durable and bounded

UAR MUST commit admitted commands to a bounded, revisioned inbox with idempotent command receipts and a declared retention limit. It MUST serialize mutating turns for one logical instance, enforce finite queue, active-turn, restart, and retention limits, and report overload rather than silently drop admitted work. Authorized status and cancellation MUST remain available while a turn is blocked.

#### Scenario: Duplicate submission and full inbox

- **WHEN** a client repeats an accepted command ID with identical content and then submits new work after the inbox reaches its limit
- **THEN** the repeat returns the existing receipt without another turn and the new work is refused with a structured capacity reason.

#### Scenario: Tool call blocks

- **WHEN** one instance's turn is blocked in a tool call and another mutating turn is queued
- **THEN** no second mutating turn starts for that instance, while authorized status and cancellation remain responsive.

### Requirement: Single-host activation is fenced and recoverable

UAR MUST assign a monotonically increasing ownership epoch to each activation, condition state transitions and protected commits on the current epoch, and refuse stale owners. After restart, it MUST reconcile admitted work and any uncertain external effect before replaying or advancing it; uncertainty MUST NOT be represented as successful completion or safe retry.

#### Scenario: Old activation completes after replacement

- **WHEN** an earlier activation returns after a newer activation has claimed the instance
- **THEN** the earlier epoch cannot commit a turn result, state transition, or protected effect.

#### Scenario: Crash after admission

- **WHEN** UAR restarts with an admitted but nonterminal command
- **THEN** the command remains visible with its exact attempt and recovery posture; UAR resumes only work proven safe to resume and exposes unresolved effects for reconciliation.

### Requirement: Lifecycle controls are repeat-safe and observable

Authorized operators MUST be able to activate, passivate, drain, disable, and restart an instance. Each command MUST have a revisioned receipt and a defined effect on queued and active work; a repeated command MUST not trigger duplicate lifecycle hooks or duplicate execution. Repeated activation failures MUST stop at a finite budget in an inspectable failed state. Disabling MUST prevent new executable admission without erasing history or uncertain work.

#### Scenario: Drain and disable during active work

- **WHEN** an operator drains an instance with one active turn and then disables it
- **THEN** UAR stops admitting new turns, reports the active and queued dispositions, and preserves their committed receipts and unresolved effects.

#### Scenario: Repeated failed activation

- **WHEN** activation repeatedly fails until its configured restart budget is exhausted
- **THEN** UAR stops automatic retries and exposes the failure and recovery action.

### Requirement: Administration and events tell the truth about runtime state

Authenticated administration MUST expose instance identity, profile, lifecycle state, active root/attempt association, inbox depth, epoch, effective binding, limits, and recovery posture without revealing credentials or private reasoning. Realtime updates MUST be based on committed transitions, carry stable instance and attempt attribution, and distinguish a live stream, a durable replay, and a current snapshot. Capability discovery MUST advertise this behavior only when implemented; ordinary run and protocol clients MUST retain their supported behavior without opting into instances.

#### Scenario: Reconnect after an event gap

- **WHEN** a client reconnects after the retained event cursor has expired
- **THEN** UAR reports the replay gap and returns a current authorized snapshot instead of fabricating missing events.

#### Scenario: Legacy ordinary run client

- **WHEN** a client submits an ordinary run without an instance identity
- **THEN** its existing admission and result contract remains supported and no durable instance is implicitly created.
