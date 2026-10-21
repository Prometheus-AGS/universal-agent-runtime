# Runtime lifecycle handoff

## Scope and verification state

This handoff covers the runtime lifecycle lane in KBD phase
`function-first-release-certification`. The operator verification gate remains
closed. No test, formatter, Cargo command, lint, build, compiler probe, or
review check ran for this work.

## Implemented ownership contracts

`src/uar/runtime/lifecycle.rs` adds the trusted-host lifecycle primitives.
`RuntimeSupervisor` registers resources in acquisition order, closes registry
admission atomically, and closes resources in reverse registration order.
`register_with_finalizer` installs a finalizer before a resource becomes
visible. A `ResourceScope` atomically admits operations and spawned tasks,
retains state snapshots, records task panics/errors, and retains one
`CloseReport` for every closer. Scope finalizers run only after admitted leases
and tracked tasks settle. `RuntimeHandle` shares the same retained shutdown
result, so abandoning a waiter cannot abandon finalization.

Snapshots now include an explicit `Owned`/`Borrowed` ownership mode, declared
dependency resource IDs, and stable outstanding operation IDs. Use
`set_ownership`, `declare_dependencies`, and `admit_named` before admission
closes. The supervisor retains each terminal child report and propagates its
actual failure and unknown-effect entries into the aggregate report; it does
not replace them with a generic “did not release” string.

Supervisor shutdown starts independent dependency leaves concurrently, with
reverse registration order only as the tie-breaker. A declared `resource -> dependency` keeps the
dependency available until its dependent report completes. A missing edge holds
its owner and every reachable known prerequisite open, while unrelated acyclic
branches still close. Cycle detection identifies only cycle members; resources
retained because those members still require them carry unresolved obligations,
not a false cycle label. Missing and cyclic edges are retained as
`LifecycleDependencyFailure` entries. `RuntimeSupervisor::{resource_scope,
declare_dependencies}` support late server composition metadata without a
second ownership registry. `RuntimeHandle` also exposes a host-authorized
generation-bound `reconnect`; it runs in the resource scope, so dropping the
control waiter cannot abandon the attempt.

Lifecycle transitions are emitted only when resource state, ownership, or
declared dependencies change. They carry the resource identity, generation,
owner, prior and next state, ownership/dependency graph, outstanding operation
IDs, and a sanitized error category. Snapshots also retain failure categories,
unresolved effects, and dependency failures. `ResourceScope::close_report` and
`RuntimeHandle::{resource_report, close_reports}` provide retained terminal
reports without starting another shutdown.

`RunManager` now has a lifecycle scope in every construction path. A host
should replace its standalone scope with `attach_lifecycle` before admitting
background work. `shutdown_lifecycle` cancels the root run token before closing
the scope. Skill evolution work, retention ownership, instance pumps,
passivation, and the team notifier use lifecycle accounting. Team worker joins
remain owned by `TeamExecutionRuntime::shutdown`; each worker also acquires an
`OperationLease` before it can claim durable work.

A one-shot due time is never converted into a zero-delay retry. The retention
owner performs one authoritative sweep, then schedules only future terminal,
local-session, or persisted-session expiry boundaries. An unchanged overdue row
after a failed cleanup waits for a durable/activity signal. Observer delivery
records a due boundary and its committed-work generation. If the boundary passes
while its timer is being armed, it receives one post-boundary authoritative
delivery attempt; only an unchanged overdue boundary then waits for committed
work instead of rescanning immediately.

`attach_lifecycle` now derives the manager root cancellation capability from
the host scope. Closing the supervisor scope therefore requests cancellation
of producer roots before it waits for their joins. The former admission
backstop and shared admission deadline have been removed. The retained
`with_admission_timeout` builder is compatibility-only; persistence results or
explicit scope cancellation determine admission outcomes.

Run history has a dedicated scope. `shutdown_run_history` closes queue
admission, drains every accepted write, joins the writer through its retained
scope report, and returns write failures while persistence remains available.

`ObserverController` has `start_in_scope`, `observe_catalog_generations`, and
`close`. Its delivery task observes both controller shutdown and the owning
scope cancellation, so scope closure cannot wait forever for an unobserved
cancellation token.

## Event and schedule wiring

`CollaborationCatalogService::subscribe_catalog_generation` is a retained
`watch` receiver. The service advances it only after its central CAS reports a
committed catalog generation. Acquire the receiver before the initial durable
catalog read, register `ObserverController::observe_catalog_generations` in the
observer scope, then start delivery and call `notify_work` for the initial
authoritative read. Remote catalog writers still require the internal realtime
topic supplied by the server owner.

The observer has no periodic delivery fallback. It wakes on committed work or
on the earliest actual schedule/follow-up due time. `next_due_after` is only a
business-time wakeup calculation; durable scan/CAS remains execution authority.
The instance idle-passivation sleep is also a business due time and listens for
scope cancellation.

Run replay and local session retention have no interval. Terminal emission,
stream attachment/detachment, and retained session activity wake the retention
owner, which waits for the earliest terminal-plus-detach or local idle expiry.
The old 250ms disconnect decision was removed: a broadcast receiver count
cannot identify a disappearing client or prove that replay is not attaching.
The guard now reports only a retention transition; trusted host code must make
an explicit owner-scoped cancellation decision from attachment-generation
state.

`RunManager::subscribe_committed_work` returns a retained monotonic wakeup
generation and `notify_committed_work` advances it after a confirmed durable
or internal realtime change. A generation never describes an outcome; each
consumer resnapshots its authoritative record. AG-UI terminal projection and
SSE streaming, A2A durable projection and continuation observers, and
full-harness receipt projection all subscribe before their first snapshot and
wait on registered scope tasks instead of an interval. The internal A2A,
AG-UI, and MCP-elicitation topics must advance the generation on committed
events, lag, and reset.

MCP elicitation settlement is a manager-scope task. `McpInteractionChannel`
uses a per-channel durable-change generation and one actual persisted expiry
due time; an answer is observed by an authoritative row reread after a wakeup.
`McpInteractionChannel::notify_durable_change` must be called for each
committed MCP-elicitation topic event, including lag/reset. No stdio process
implementation was changed.

An instance lifecycle command that arrives before a run installs its
cancellation token now waits for the retained run-admission generation. It no
longer uses a timed retry loop. Scope cancellation retains an unresolved
attempt rather than fabricating completion.

## Required server and embedded ordering

The server owner should register resources in this acquisition order: provider
finalizer, bus/subscription finalizer, run-history finalizer, manager/task
scope, observer scope, then external request admission. Reverse closure then
stops external work, cancels/joins manager work, drains history while the
provider remains usable, closes observer subscriptions, and finally releases
the provider.

For embedding, call `manager.shutdown_lifecycle().await`,
`manager.shutdown_run_history().await`, `observer.close().await`, and
`TeamExecutionRuntime::shutdown().await` from the embedding's retained
shutdown owner before closing UAR-owned stores. A borrowed store is finalized
only by its host.

## Unresolved source contracts

`SessionStore` now provides retained activity and earliest local idle expiry.
Session-owned mutations advance the same retained generation, so a message
added after lookup also resets the one-shot schedule. The state lane has now
provided `subscribe_persisted_session_changes` and
`next_persisted_session_expiry`; manager retention subscribes before it reads
the next authoritative persisted deadline. Provider errors leave no guessed
timer and are recorded as a failed schedule read.

The server must bind authenticated attachment generations to explicit run
abandonment. Runtime intentionally does not infer abandonment from a broadcast
receiver count. External writer wakeups remain required for remote catalog,
observer occurrence, and ingestion changes.

Tool-admission cancellation receipts are now manager-scope tasks. Each prepared
invocation has a settled cancellation token that is removed on rejection,
terminal finish, or confirmed cancellation; receipt failure becomes a tracked
scope failure rather than a detached best-effort task. `RunDelegationLifetime`
now makes only a synchronous cancellation request in `Drop`; the retained run
producer calls `release().await` and records a run-scoped MCP shutdown failure
in its terminal cleanup result. `SkillFileWatcher::new_in_scope` owns retained
filesystem event handling without a time debounce, while the compatibility
constructor creates a standalone scope that callers must close explicitly.

Agent-instance SSE subscribes to committed work before its first durable
snapshot and emits a gap/error when that event source closes; it no longer
re-reads every 500ms. A2A v1 JSON-RPC and gRPC synchronous send responses use
`A2AThreadService::wait_durable_until_settled`, which subscribes to run and
committed-work notifications before each durable reread, replacing their 100ms
status loops. The generic outgoing `A2ATaskExecution` still has a 250ms poll:
its remote-peer contract exposes no completion subscription or authenticated
push callback. It must gain that remote event contract before polling can be
removed without changing a working remote task into an unobservable pending
operation.

`SandboxSessionManager` now uses its own in-memory `last_accessed + ttl`
authority. `start_gc_task` is a supplied-scope task that waits for the earliest
real expiry or its retained mutation/completion generation; there is no
60-second sweep. Failed expiry destruction is blocked until a real later touch,
so it cannot form an overdue self-wakeup loop. **Incomplete:** runner creation
and explicit destruction are still awaited by their caller rather than by a
retained operation owner, and `shutdown` currently closes its scope before
destroying handles. This does not meet the required dropped-waiter and
finalizer-owned shutdown contract; do not integrate the sandbox lifecycle
changes as complete until that state-machine redesign is implemented.

The MCP registry now retains one consuming `RunningService` owner per slot and
gives callers only a cloneable peer/client capability tagged with its service
generation. It closes admission, waits accepted operations, takes each owner,
and awaits `cancel()` for both current and retired transports. This removes the
former 10ms transport-closed polling loops. Slot update and reconnect guards
publish an event-driven idle acknowledgement before a slot owner is consumed.
`shutdown_report` retains cancellation/join errors for the host lifecycle
finalizer; the compatibility `shutdown` method only logs that report. Connect,
catalog discovery, and projected readiness no longer convert fixed elapsed
durations into failure; a silent peer remains an explicit outstanding operation
until cancellation or connector evidence resolves it. The existing tool-call
budget remains an execution policy and does not certify a remote effect.
`McpBindingCache::shutdown_report` carries those errors through projected MCP
runtime shutdown; its compatibility reaper preserves existing callers while
logging an unpropagated report.

The SurrealDB lifecycle patch now contains `conn/lifecycle.rs`, an engine
facade that registers every accepted SDK operation before starting it in a
tracked task and returns its result through a receiver. Engine shutdown closes
admission, joins those tasks, and then awaits the underlying engine close. A
dropped SDK operation future never cancels an accepted operation. The parent
SDK integration must include the module, wrap each `Router` engine, and enable
the required task runtime feature; that wiring is intentionally owned by the
lead and engine owners.

## Authored acceptance scenarios (not executed)

- Cancel a prepared tool admission and assert its scope joins the receipt task
  with confirmed cancellation evidence or an explicit unknown outcome.
- Unwind a run with run-scoped MCP and assert the retained producer awaits
  `RunDelegationLifetime::release`, preserving any close failure.
- Deliver a skill filesystem event, then close its supplied scope and assert
  refresh and join occur without a debounce duration deciding correctness.
- Hold one independent lifecycle finalizer and assert another leaf starts and
  reports independently while its prerequisite remains open until dependent
  acknowledgement.
- Race an instance or A2A task mutation with an initial stream snapshot and
  assert the retained wakeup causes the next durable read without polling.
- Race sandbox creation with shutdown and assert a late-created handle is
  destroyed before shutdown is reported to the creator.

## Changed paths

- `src/uar/runtime/lifecycle.rs`
- `src/uar/runtime/mod.rs`
- `src/uar/runtime/manager.rs`
- `src/uar/runtime/actor/system.rs`
- `src/uar/runtime/native_skills/schedules.rs`
- `src/uar/runtime/native_skills/followups.rs`
- `src/uar/runtime/observer/controller.rs`
- `src/uar/runtime/observer/delivery.rs`
- `src/uar/runtime/schedules/calendar.rs`
- `src/uar/runtime/schedules/controller.rs`
- `src/uar/runtime/followups/controller.rs`
- `src/uar/runtime/instance/controller.rs`
- `src/uar/runtime/instance/pump.rs`
- `src/uar/runtime/instance/lifecycle.rs`
- `src/uar/runtime/team_execution/controller.rs`
- `src/uar/compiler/collaboration/service.rs`
- `src/session/thread.rs`
- `src/uar/api/a2a/thread_service.rs`
- `src/uar/api/a2a/grpc_v1.rs`
- `src/uar/api/a2a/v1.rs`
- `src/uar/api/agent_instances.rs`
- `src/uar/api/agui.rs`
- `src/uar/api/full_harness.rs`
- `src/uar/api/full_harness/handlers.rs`
- `src/mcp/interaction.rs`
- `src/mcp/binding_cache.rs`
- `src/mcp/registry.rs`
- `src/mcp/runtime.rs`
- `src/sandbox/session_manager.rs`
- `src/sandbox/mcp_tools.rs`
- `src/uar/runtime/tool_admission/lifecycle.rs`
- `src/uar/runtime/tool_admission/mod.rs`
- `src/uar/runtime/turn/bindings.rs`
- `src/uar/runtime/skills/watcher.rs`
- `vendor/patches/surrealdb-lifecycle/surrealdb-3.3.0/src/conn/lifecycle.rs`
- `docs/operations/lifecycle-runtime-handoff.md`

## Integration superseding earlier incomplete notes — 2026-10-20

The source states described as incomplete above are preserved as handoff history.
They are superseded by `lifecycle-a2a-handoff.md` and
`lifecycle-sandbox-handoff.md`: outbound A2A now requires the exact advertised v1
streaming contract, and sandbox create/use/destroy work plus physical cleanup
have retained scope ownership. Neither source handoff certifies execution.

The lead added `RunManager::subscribe_run_registrations` and
`TeamExecutionRuntime::await_run_registration`, exposed through the owner-scoped
`GET /api/v1/collaboration/team-instances/{id}/attempts/{attemptId}/run-registration`
route. The wait subscribes before reading the durable attempt and exact owner's
local run. Local registration, catalog commit and internal committed-work
generations wake reconciliation. A settled attempt without a local run or scope
cancellation returns an explicit conflict. A queued attempt may remain pending;
there is no registration probe loop or guessed deadline.

The channel authority client no longer supplies a fixed ten-second request
deadline. An unknown release response remains uncertain; an explicit execution
budget elsewhere still cannot prove that a Gate effect stopped. Configuration
and settings descriptions identify shutdown/admission timeout fields as legacy
compatibility values rather than lifecycle controls. These edits implement the
approved contract, with no tests, builds, checks or review gate executed.
