# Design

## Context

See [proposal.md](proposal.md) and [the capability delta](specs/durable-agent-instances/spec.md). At source revision `fcfce6d226b50eee502c5f272e9215112d933e29`, `src/uar/runtime/thread/service.rs` attaches `ThreadService` to exactly one fresh root and rejects an existing child tree. `src/uar/runtime/thread/actor_host.rs` already creates a fresh root per actor turn, with conversation history in a session ID; `src/uar/runtime/actor/system.rs` keeps actor handles in a process-local registry. C04's `src/uar/service_instance.rs` identifies the UAR service, not a logical agent. C03 stores private deployment bindings and still advertises team activation as unsupported.

The default durable provider is SurrealDB 3.3.0 as pinned in `versions.toml`. The in-memory provider is process-local; it cannot truthfully advertise restart durability. Postgres is optional only if its focused instance-store implementation meets the same transactional contract. No Surreal Memory API change is assumed.

## Goals / Non-Goals

**Goals:** One UAR process can reactivate ordinary-agent identities after a crash or passivation, admit bounded work, and project trustworthy state without changing the existing run kernel or delegating scheduling to The Boss.

**Non-goals:** Rehydrate a completed root as a new turn; cross-host automatic takeover or migration; activate TeamDefinition or WorkflowDefinition; add a second model loop; guarantee replay of an external effect with uncertain outcome; make an in-memory process restart durable.

## Decisions

### 1. One focused durable-instance authority above root runs

Create a focused logical-instance state/store under planned `src/uar/runtime/instance/` and `src/uar/persistence/agent_instances.rs`. It holds the owner/workspace key, instance ID, immutable definition resolution, private effective-binding reference, profile, revision, lifecycle state, current epoch, bounded inbox, command receipts, and turn/root associations. It does **not** copy the whole `CollaborationCatalogState` or make the actor registry durable. The catalog remains definition and binding authority; the logical-instance store owns only runtime state. Alternative: extend catalog state with every queue and activation write. Rejected because catalog revisioning is for installation changes, while turns have a much higher write rate and different recovery semantics.

The existing `ActorCollaboration` and `ActorThreadSession` may remain the live turn adapter: source inspection confirms they create a distinct root for each turn and keep history in the session. The instance authority must claim the record before creating an actor/session and retain the root association after the actor disappears. The active actor is a cache of execution capability, not instance identity or the sole admission gate. Alternative: make `ThreadService` itself survive roots. Rejected because its captured root approval, budget, cancellation and child limits are intentionally root scoped.

### 2. Bound and fence every admitted command

The durable store exposes atomic owner/workspace-scoped compare-and-swap operations for instance revision, command ID, inbox admission and activation epoch. Duplicate command ID plus identical content returns the original receipt; changed content conflicts. A configured finite inbox and retention policy refuses new work before acceptance when full. One instance has at most one mutating turn; independent instances may execute concurrently subject to host capacity. Status and cancellation use a separate control path so a blocked tool cannot monopolize them. Alternative: rely on the actor mailbox as the only queue. Rejected because it is lost on process exit and does not provide a durable admission receipt.

Only the current single-host epoch may commit state, terminal result or protected-effect transition. Epoch checks must surround the trusted host's effect/approval boundary, not merely the UI projection. Restart invalidates process-local activation and reclaims after reconciling exact command/root/effect receipts. Work with uncertain external effects remains blocked for explicit reconciliation. This is not cross-host fencing: C18 must prove shared lease semantics before takeover is advertised.

### 3. Lifecycle is explicit and repeat-safe

Request profile has no retained worker after its turn. On-demand wakes on accepted work and passivates after the configured idle policy. Resident is opt-in, carries a measured resource policy and has no always-running model call. Activate, passivate, drain, disable and restart are revisioned commands. Hooks record their attempt and result so replaying a command cannot run a hook twice without an explicit retry receipt. Drain stops new admission and settles or reports queued work before releasing the activation; disable retains history while refusing executable work. Restart has a finite attempt budget and an inspectable failed state. Alternative: automatic indefinite restart. Rejected because it can consume provider budget and hide a persistent configuration failure.

### 4. Additive administration, capability and event projection

Extend authenticated UAR administration and `src/uar/api/capabilities.rs` with a negotiated durable ordinary-agent profile only after the backing implementation is operational. Planned route integration belongs near `src/uar/api/routes.rs` or a focused routed module; exact request/response names are chosen during implementation after reconciling route conventions, rather than invented here. `service_instance_placement_v1` remains a distinct C04 service identity. `full_harness_delegation_v1` remains the C05 run profile and does not imply durable logical instances. Legacy ordinary run/actor/A2A clients continue their current flow without implicit instance creation.

Events are projections of committed instance transitions, attributed by owner-scoped instance, command, attempt and root run. A reconnect receives retained cursor replay or an explicit gap plus current snapshot; process-local AG-UI and A2A streams cannot be rebranded as durable history. Provider/model resolution remains at each fresh turn's private binding and policy admission. No credential values enter instance records or events.

## File claims and ownership

The C06 UAR runtime owner claims planned `src/uar/runtime/instance/**` and the integration points `src/uar/runtime/mod.rs`, `src/uar/runtime/actor/system.rs`, `src/uar/runtime/thread/actor_host.rs`, and `src/uar/runtime/manager.rs` only where the existing fresh-root adapter requires it. The persistence owner claims planned `src/uar/persistence/agent_instances.rs`, `src/uar/persistence/mod.rs`, and `src/uar/persistence/providers/{surreal,memory,postgres}.rs`; in-memory support is explicitly non-durable and Postgres may remain unsupported with a clear capability refusal. The API owner claims `src/uar/api/{routes,capabilities,administration_capabilities}.rs` and a focused new instance-route module if needed. Owners coordinate a single runtime contract before overlapping edits; these are claims, not permission to refactor adjacent code.

## Risks / Trade-offs

- **A crash follows a protected external effect but precedes its terminal receipt** → retain uncertainty, block automatic replay, and require observed reconciliation; an epoch alone cannot undo an effect.
- **SurrealDB is unavailable or does not provide the required conditional commit** → refuse the durable profile; do not silently fall back to an in-memory claim of durability.
- **An old actor remains alive after an epoch handoff** → reject its commits at storage and effect boundaries, request cancellation, and keep the new owner from starting an overlapping effectful turn until cleanup or uncertainty is recorded.
- **Long queues or resident workers exhaust resources** → finite limits and visible overload; separate runtime admission from model execution and use the existing root budget policy per turn.
- **Current actor session persists conversation history but not a reusable root** → rehydrate history under explicit owner/workspace policy and create a fresh root; never overwrite old lineage.

## Migration Plan

Introduce new instance records and capability discovery additively. Existing runs and actors remain outside the instance store. Create instances only through explicit authenticated commands; require an exact definition and effective binding. Add a versioned store schema and backup/restore receipt before promoting durable mode. On rollback to a version without C06, disable new instance admissions and retain records for forward recovery; do not reinterpret queued work as ordinary runs or retry uncertain effects. Re-enable only after schema and current authority reconciliation.

## Acceptance boundary

After C06.1-C06.3 production code is complete, run one local, real-provider integration gate against SurrealDB 3.3.0 and the actual UAR host: two owners/workspaces using one definition, distinct root IDs across turns, process crash and reactivation of accepted work, stale epoch refusal, blocked-tool cancellation/status, bounded inbox and restart exhaustion, exact duplicate command receipt, event reconnect gap/snapshot, old ordinary-run compatibility, and protected-effect uncertainty. Pin source revision, database version, policy/binding revisions and observed outcomes in one receipt. No unit, mock-only, per-edit or partial gates are completion evidence.
