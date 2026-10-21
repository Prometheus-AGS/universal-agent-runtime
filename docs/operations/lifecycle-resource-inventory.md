# UAR lifecycle resource inventory

Status: implementation source inventory, 2026-10-20. Execute phase:
`uar-outstanding-issues-v1-1-1::function-first-release-certification`.
Verification is **CLOSED**. This document is not an implementation-complete
receipt or acceptance result. The authoritative waypoint and complete phase
ledger must be reconciled after all writers finish.

## Authority and state

`src/uar/runtime/lifecycle.rs` supplies the trusted host supervisor, resource
scopes, admission leases, retained snapshots and terminal reports. A scope's
admission lock registers accepted work before it can execute. Closing a task
tracker alone does not close admission. Task errors and panics remain failures
even after the task count reaches zero.

The host declares edges in `src/server/lifecycle_graph.rs`; an edge from A to B
means B remains usable until A's operations and finalizer finish. Missing edges
and cycles produce unresolved obligations. They do not authorize closing a
prerequisite underneath its still-running dependent. Independent branches have
their own completion obligations.

Snapshots identify the resource, owner, ownership mode, generation, dependencies,
outstanding operation identities, task count and failure. Host control observes
them on demand. No periodic unchanged-state logging or lock probing is needed.
Transitions are registered, ready, admission closed, draining, finalizing and
released or failed. A cancellation request is never a release transition.

The approved caller-budgeted host API adds observation of the same retained
supervisor shutdown. Pending carries a per-supervisor UUID and either explicit
pending inventory acquisition or current unreleased resource snapshots. Expiry
does not stop joining or change this dependency graph. The embedded wrapper
requests its run-root cancellation first. See `lifecycle-budgeted-shutdown.md`;
this source contract has no compilation or runtime evidence yet.

## Server ownership graph

| Resource / source | Authority and admission | Cancellation and completion | Finalization prerequisites |
| --- | --- | --- | --- |
| Process telemetry, `src/main.rs`, sidecar and Windows service entrypoints | The entrypoint owns the initialized SDK provider across every command/server outcome | Shutdown return/error or panic retained after server/configuration cleanup; errors cannot produce a successful exit | Server resources and configuration watcher finish first |
| OS shutdown signals, `src/server/process_signals.rs` | Host scope owns subscriptions acquired before database initialization; external Windows hosts retain SCM/input cancellation authority | Actual signal requests cancellation; scope cancellation and task join release local subscriptions | Signal observation remains usable through configuration/server cleanup |
| Configuration watcher, `src/config_manager.rs` | Configuration manager owns filesystem watcher scope; explicit and filesystem reloads require admitted leases | Close reload admission, request stop, join native watcher and admitted reloads | First close stage; every other registered scope remains available until acknowledgment |
| Physical persistence, `providers/surreal.rs`, `providers/postgres.rs` | Host owns the selected engine or pool; endpoint model identifies ownership | Surreal `close_scope()` / SDK `shutdown()`; SQLx pool `close()` | All persistence users and subscriptions drain first |
| Persistence facade, `persistence/scoped.rs` | Every domain operation admits a named lease; retained facade rejects closed admission | Scope close waits for leases; canceled call records unresolved effects | Physical persistence and required committed-change source |
| Primary realtime, `realtime/{surreal_bus,postgres_bus}.rs` | Bus owns only its subscriptions/listener and workers | Stop token, awaited stream `close()` or listener pool close, tracked worker completion | Physical persistence |
| Credential migration, `security/credentials/migration.rs` | Service owns atomically registered single-flight tasks; each caller separately authorizes | `shutdown_migrations()` joins accepted migrations and retains unresolved identities | Persistence access |
| Memory, `memory/service.rs`, `memory/scoped_storage.rs` | Separate engine and scoped storage capability; owned versus borrowed is explicit | `close_scope()` retains scope result, owned-engine acknowledgment and failures | Memory users, expiry and subscriptions finish first |
| Memory realtime, `realtime/surreal_bus.rs` | Table-scoped memory connection capability; separate bus owner | Stream command acknowledgment, local registration removal and worker joins | Memory service |
| Memory expiry, `memory/background.rs` | Host task scope owns expiry and committed-change forwarding | Cancellation stops new work; current expiry operation completes; tasks join | Memory realtime and memory service |
| MCP, `mcp/{registry,stdio_process,interaction}.rs` | Registry owns running services; host tool admission and run bindings admit calls | Service cancellation completion; actual child wait/reaping; `shutdown_report()` preserves errors | Memory, credential migration and persistence |
| Projected MCP, `mcp/{runtime,binding_cache}.rs` | Host owns runtime and retired binding services | Awaited shutdown/reaping reports | MCP registry, memory and persistence |
| Execution owner | Host retains durable execution ownership while producers finalize | Drain ownership, then release ownership; unresolved durable state stays explicit | Realtime and persistence access |
| Run history, `runtime/manager.rs` | Explicit writer admission and owned receiver | Finish terminal-record producers, close writer admission/receiver, drain writes, join writer | Persistence and execution ownership |
| Run manager, `runtime/manager.rs` | Lifecycle scope admits runs, streams and post-run work | Root cancellation plus tracked task and producer joins | History, execution owner, ingestion, projected MCP, MCP, memory and persistence |
| Actors and A2A thread work | Host owns graph roots, actor work and retained task handles | Explicit shutdown and joins; accepted durable writes finish | Run manager, history, projected MCP, memory and persistence |
| Sandboxes and terminals | Host owns scoped runner facade, exact handle bindings, accepted create/use/destroy work and actual subprocess handles | Busy leases prevent expiry; retained finalizer joins work before physical destroy/reaping; uncertain remote effects quarantine the binding | Actors, run manager, MCP and persistence |
| Team execution | Host owns team task scopes and leases | Cancel admitted work; await retained task outcomes | Actors, sandboxes, run manager and execution owner |
| Observer, schedules and follow-ups | Host strongly owns controllers; tools use weak command facades | Scope cancellation and delivery-task completion | Team execution, sandboxes, run manager and ingestion |
| Ingestion, `rag/ingestion_worker.rs` | Scope admits dispatcher and blocking work before execution | Stop dispatch; join accepted blocking jobs and retain their results | Persistence access |
| Filesystem ingestion, `rag/ingest.rs` | Watcher subscribes before initial snapshot | Stop accepting files; finish accepted ingestion; release watcher | Second close stage after configuration; other registered scopes remain available until acknowledgment |
| Committed-work forwarding | Host owns notification-to-worker bridge | Scope cancellation and joined forwarding tasks | Observer, run manager, ingestion and both realtime buses |
| HTTP / SSE / AG-UI / full harness | HTTP scope admits request and retains lease through response stream | Close external admission; cancel owned streams; actual body completion releases lease | All services used by admitted handlers |
| A2A gRPC | Host owns serving future, listener and request work | Cancellation requests graceful stop; serving task joins | Run manager, team execution, credentials, MCP and persistence |
| Model streaming, response quality, auto-capture and skill evolution | Run manager owns run/producer scopes rather than detached request tasks | Run cancellation requests completion; actual stream and task futures finish | Respective model/tool, memory, history and persistence capabilities |

The composition root registers each acquired resource immediately and unwinds the
same graph after startup failure or panic. Normal cleanup has one supervisor
authority. The old “Async resources shut down” message is a cleanup-block result,
not a substitute for engine or process completion evidence.

External HTTP/A2A admission closes atomically before the first watcher is
joined. Admitted requests drain later under their scopes. Immutable configuration
snapshots remain readable; live reload production ends first. The optional skill
watcher closes its bounded receiver before dropping the native watcher, so a
blocked callback cannot prevent its native thread from joining. The current
server composition does not instantiate that optional watcher.

Process entrypoints retain their outer configuration/telemetry finalization
outcomes through exit. The server snapshot describes supervised server resources;
it does not attest to an upstream telemetry exporter's internal release semantics.
`lifecycle-entrypoint-handoff.md` records this process boundary, the removal of
hard command exits and the Windows service's actual readiness acknowledgment.

Sandbox scope state retains actual handles after failed finalization; its observer
holds a weak scope reference to avoid an ownership cycle. A canceled oneshot
waiter does not abandon backend creation, use or destruction. Remote outcomes
without an authoritative acknowledgment remain `OutcomeUnknown`, reserve their
capacity and prevent automatic reuse or destruction of potentially busy work.
The current remote protocol supplies no reconciliation capability for these
unknown effects. See `lifecycle-sandbox-handoff.md` for the named failure scenarios
and authored acceptance inventory.

Outbound A2A admits a remote child only after authenticated discovery advertises
the exact v1 JSON-RPC interface and streaming capability. Status progression uses
owned SSE subscriptions; premature EOF or lost continuity is unavailable, never
terminal completion. An explicit reconnect command reconciles the same task
before subscribing again, without resending its message. The v1 JSON-RPC and
gRPC producers subscribe before their authoritative snapshot and register in the
run manager's lifecycle scope. Legacy and unadvertised named-agent interfaces
are rejected before dispatch. `example.config.yaml` uses the actually advertised
`compiler-agent` identity and `/a2a/v1/compiler` endpoint together; changing just
the path of an existing named-agent configuration would change its authority.
See `lifecycle-a2a-handoff.md` for the wire contract and migration limits.

## Embedded hosts and retained capabilities

`src/embedded.rs` owns initialization independently of its waiting caller. Startup
failure and panic close acquired resources before returning their error. If a
completed runtime cannot be delivered because its caller disappeared, the owner
awaits its shutdown. Explicit `EmbeddedRuntime::shutdown()` provides the retained
completion report; dropping an object only requests cleanup and cannot supply a
graceful completion receipt.

The embedded graph contains `embedded-store` when physical close authority is
transferred, `embedded-persistence-access`, optional `embedded-memory`, optional
owned `embedded-mcp`, `embedded-history`, `embedded-runs` and
`embedded-execution`. Ordinary persistence and MCP injection borrow the host's
underlying resource. `owned_persistence` and `owned_mcp` explicitly transfer
close authority. `borrowed_memory_service` creates an independent UAR scope over
host-owned memory. Retaining a UAR facade does not retain permission to use a
closed UAR-owned engine.

The endpoint model preserves path casing and records the configured RocksDB
alias separately from a host-injected real RocksDB engine. Before independently
opening memory, startup compares filesystem identity with persistence after
persisted memory configuration is resolved. The engine's acquisition result is
still authoritative.

## SDK and dependency boundaries

The SurrealDB overlay owns accepted engine operations independently of their
waiting request. It retains local session runners, notification pumps, delivery
work, maintenance tasks, index builders and remote transport drivers. Engine
shutdown closes shared admission across existing clones and awaits those
obligations. Existing clones remain inert capabilities.

Surreal live-query closure uses the original session, awaits `KILL` command
completion and removes local registration. It does not wait for an undelivered
terminal notification. It does not sweep other owners' live queries.

WebSocket reconnection is an explicit generation-checked operation. Readiness
waits for authentication/session replay acknowledgment. Transport loss or HTTP
dispatch/body failure makes the client unavailable; a lost mutation response is
outcome unknown. Local transport release never asserts a remote mutation rolled
back. SQLx pool close follows operation/listener completion.

The first-party memory export uses opt-in externally managed connection mode.
It has no autonomous replacement, timed retry or deadline-based outcome claim in
that mode. The pinned source/export contract is recorded in
`vendor/git/README.md`; the SDK patch provenance and replacement conditions are
in `vendor/patches/surrealdb-lifecycle/README.md`.

## Sidecar, launcher and fixture obligations

The private inherited input carries the launch token first, then versioned
lifecycle requests. Windows requires an inherited named pipe opened for
overlapped I/O. The Boss launcher supplies Node's `overlapped` stdin and the
explicit launch declaration; the reader joins cancellation of outstanding I/O.
Unix uses an owned nonblocking descriptor. The reader joins before sidecar exit.

`host_lifecycle_control_v1` identifies the private control protocol. `READY:port`
compatibility remains. Main-process launcher methods expose lifecycle snapshots
and generation-checked reconnect commands without consulting database-backed
authorization. A process close rejects pending control requests. Ending stdin
requests shutdown; the launcher awaits actual child `close` and retains errors.

Fixtures own child handles, readers, service acknowledgments and configuration
generations. PostgreSQL bootstrap completion precedes final service readiness.
Configuration names identify immutable content. Skill roots are explicit and
exclude ambient home-directory configuration. Body panics do not skip joined
cleanup. Controlled probe handshakes replace process scans and linger durations.
Every future historical-failure receipt must retain its individual sanitized
error category, response body when available, body result and cleanup result.

Ingestion status uses an owner-authorized SSE snapshot stream rather than
exposing the private ingestion topic. Team run registration has an authenticated
attempt-specific acknowledgement route that waits on retained registration and
catalog generations. These ports separate an acknowledged installed run or job
snapshot from a claim that its producer or engine has closed. The release fixture
owns its stream response and treats EOF or a gap as unavailable continuity.

## Acceptance and uncomfortable limits

No build, test, formatter, lint, typecheck, validator, compiler probe or review
gate ran for these changes. Source inspection is not evidence of compilability,
physical lock release, Windows installer compatibility or remote commit outcome.
The exact historical migration 503 and individual c9 sidecar failures still lack
sufficient recorded evidence for a definitive cause.

A permanently silent remote peer or uninterruptible engine operation can keep
its completion pending forever. The host must expose the outstanding operation
and allow authorized recovery; it cannot convert elapsed time into success.
Business due dates, expiry, quotas, explicit request budgets and protocol
heartbeats remain distinct from lifecycle completion. Their expiration may
request cancellation but does not prove rollback, child reaping or engine close.

Before acceptance, collect current writer handoffs, reconcile the complete KBD
phase ledger and publish exact source identities and proposed commands. Obtain
operator authorization for those exact commands. Any implementation edit closes
that gate again.
