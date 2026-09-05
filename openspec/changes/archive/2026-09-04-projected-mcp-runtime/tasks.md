# Tasks — projected-mcp-runtime

scope: src/mcp/**, src/uar/runtime/manager.rs (MCP block), src/uar/api/mcp_admin.rs, src/uar/telemetry/metrics.rs, src/uar/runtime/native_skills/search_tools.rs, src/sandbox/os_native/** (conditional), tests/mcp_projection.rs

## 0. Decision

- [x] 0.1 Evaluate porting Codex `sandboxing` (Seatbelt, Landlock, bwrap) for stdio children; record the decision and, if not ported, implement rejection of `sandboxed: true` at config load

## 1. Failing tests first

- [x] 1.1 `tests/mcp_projection.rs`: two consecutive runs using the same skill-declared server reuse one connection; a config-hash change forces a new one
- [x] 1.2 A skill-contributed server with a complete cached catalog is not started until its first tool call; the call waits for readiness and succeeds
- [x] 1.3 A required server that fails to start aborts preflight with an error naming the server; an optional one warns and its tools are absent
- [x] 1.4 With 200 deferred tools, the eager set is bounded; `search_tools("calendar")` activates matching descriptors for the next step
- [x] 1.5 A skill cannot replace a global server definition or loosen its sandbox policy; delegation transfers frozen bindings locally and no connection recipes to remote UAR peers
- [x] 1.6 Concurrent refreshes of one server produce one connection attempt; a cancelled refresh leaves the entry dirty and the next call refreshes
- [x] 1.7 State transitions emit normalized events in order and `set_mcp_server_status` receives each
- [x] 1.8 `sandboxed: true` either launches under the sandbox or fails config load; it is never a no-op

## 2. Catalog and projection

- [x] 2.1 Add `src/mcp/catalog.rs` (`McpCatalog`, `ServerDefinition` with authority and config hash)
- [x] 2.2 Add `src/mcp/projection.rs`: resolve a step to exact servers and tools; authority ordering enforced
- [x] 2.3 Add `src/mcp/binding_cache.rs`: key by owner, hash, auth identity, environment; generation invalidation; single-flight

## 3. Lifecycle

- [x] 3.1 Lazy start for skill-contributed servers gated on cached catalog completeness; `wait_until_ready` on the call path; keep delegated MCP authority frozen or peer-local
- [x] 3.2 Required versus optional failure handling in preflight
- [x] 3.3 State machine and normalized state events; wire the existing MCP-status recorder
- [x] 3.4 `search_tools` model-only tool and `Deferred`/`Hidden` exposure in the tool list

## 4. Manager integration

- [x] 4.1 Replace per-run `from_config` with catalog resolution and binding lookup
- [x] 4.2 Preserve reconnect generation guard, timeouts, and shutdown guarantees

## 5. Verification

- [x] 5.1 Tier 1: extend the existing reconnect tests, then run `cargo test --locked --no-default-features --features server-full --test mcp_projection` and the focused registry reconnect target
- [x] 5.2 Tier 2: fmt check and full test run
- [x] 5.3 Local integration with a real stdio server: lazy start, reconnect, cancel, shutdown (record commands and output)
- [x] 5.4 `openspec validate projected-mcp-runtime --strict`

## Execution notes — 2026-09-02, task 0.1

Decision: use the specified config-load rejection alternative; do not port
Codex sandboxing in this change. Codex reference checkout
`986ff1cc7ced0081ec5014b700a376333d87f869` couples its launcher to permission
profiles, network proxy plumbing, Seatbelt profiles, and a separate Linux helper
with bubblewrap/seccomp and legacy Landlock utilities. UAR's SandboxRunner has
no persistent stdio-process launch interface. The phase decision log records
the inspected source paths and the tradeoff.

McpConfig deserialization now validates every named entry. Registry startup
also validates programmatically constructed configurations before connecting
anything. Both initial and reconnect launch paths reject unsupported sandbox
requests before command resolution/provisioning/spawn. HTTP and embedded saves
reject before settings or connection changes, including disabled entries;
hydration validates the effective stored configuration before seeding or
removing existing connections. Ordinary stdio and remote HTTP are unchanged.

Compile-only Tier 0: `cargo check --locked --no-default-features --features
server-full` passed without warnings after each source edit (30.97s, 8.23s,
8.01s, 10.18s). `git diff --check` passed for the edited source files. Tests,
formatting, strict OpenSpec validation, and acceptance review remain deferred
to phase completion under the operator's instruction. Task 1.8 remains open.
No dependency or workflow changes. This rejects sandbox requests; it does not
deliver OS sandbox support or child-thread filesystem/network enforcement.

## Execution notes — 2026-09-02, task 2.1

Added and exported `src/mcp/catalog.rs`. McpCatalog holds immutable
ServerDefinition candidates indexed deterministically by server name and source;
no services, mutable registry, process I/O, environment reads, or tool grants.
Each definition captures config, source-derived authority (global > skill >
child), required/optional status, host authentication metadata and a derived
sandbox policy. Construction shares task 0.1's sandbox rejection. Invalid empty
server/skill/credential identities and conflicting same-source duplicates are
rejected at the catalog boundary; identical duplicates coalesce.

Configuration hashing uses the existing SHA-256 dependency with a versioned
domain, transport discriminator, fixed-width length prefixes, ordered arguments,
and sorted environment keys. All configured values participate, including secret
values, so hashes have redacted Debug and no serialization/display surface.
Definitions likewise omit launch config from Debug and redact authentication
binding IDs. Hashes cover declared inputs only: task 2.3 must separately key
resolved environment and authentication identities before reusing connections.

Tier 0: `cargo check --locked --no-default-features --features server-full`
passed, zero warnings, `Finished dev profile ... in 28.92s`. Source diff checks
passed. No tests or formatter ran. Call-site inspection confirms the exported
catalog is not yet consumed by RunManager; tasks 2.2 and 4.1 own projection and
runtime integration. Do not interpret this task as live connection reuse.

## Execution notes — 2026-09-03, task 2.2

Added/exported `src/mcp/projection.rs`. McpServerProjection::resolve applies the
resolved eligible IDs in every selection mode, validates active skills against
policy, excludes inactive skills and foreign child declarations, and chooses
Global > Skill > current Child. Conflicting highest-authority declarations fail;
identical settings at that rank choose one stable source. Lower-rank conflicts
cannot displace or poison a global winner. Required/auth/sandbox configuration
is preserved, not merged or downgraded.

ServerToolCatalog captures compiled descriptors with explicit source/server
identity and completeness. with_tools matches catalogs to the chosen origin,
configuration hash, required flag and authentication state, rejects stale,
missing, incomplete or conflicting snapshots, and does not fall back to lower
origins. Tools must pass both server and tool eligibility; Hidden tools are
omitted and Deferred tools remain eligible but outside model_tools(). The exact
server Arc accompanies each unmodified descriptor. Provider-name collisions
fail; two differing complete snapshots are never unioned.

Tier 0 `cargo check --locked --no-default-features --features server-full`
passed with zero warnings in 45.12s. Tracked and untracked source whitespace
checks passed. No tests, test authoring, formatter or acceptance review ran.
The module is exported but has no RunManager/ResolvedStep call site yet: task
4.1 owns that integration. This pure projection is not an executable grant;
task 2.3 must validate owner/auth/environment binding identity, and the host
must enforce the actual sandbox and Cedar policy. No dependency changes.

## Execution notes — 2026-09-03, task 2.3

Added/exported binding_cache.rs. Keys include verified ActorOwner (including
tenant), server name/source, declared config hash, required flag, authentication
revision, and an exact OS-string environment/cwd snapshot. Private key fields
and redacted Debug prevent accidental omission or secret emission. Relative cwd
is rejected; Required authentication cannot enter connection startup.

Ready lookups share an owned registry lease. Concurrent misses share one
caller-owned attempt and its result through watch; no background task, automatic
retry or fallback. A drop guard clears failed/cancelled refreshes, leaving the
entry dirty. Invalidation rotates an opaque generation and cancels the current
attempt; stale publication is rejected. Owner-wide invalidation revokes all
revisions. Returned registries must contain the exact single configured server,
not a frozen borrowed view or foreign/native tool descriptors.

Registry begin_shutdown starts cancellation synchronously for drop/invalidation.
Retired registry leases remain owned until reap_retired or shutdown awaits their
existing transport closure checks. Shutdown closes admission before awaiting
refreshes, then drains retired transports. Cancelling cleanup retains the queue.
No reconnect implementation or generation guard was replaced.

T0 cargo check --locked --no-default-features --features server-full initially
finished in 41.87s with two warnings (redundant Future import and missing cache
Debug). Both were fixed; the final check passed with zero warnings in 31.44s.
Tracked whitespace check exited 0; the new-file no-index check emitted no
whitespace errors (exit 1 denotes the added-file diff). No tests, formatter,
acceptance critic, dependency or workflow changes.

Uncomfortable limit: there is no manager consumer or snapshot-aware connector
yet. The callback contract requires exact environment/credential inputs and
partial-transport cleanup; the existing ambient-reading from_config is not a
compliant connector without further wiring. Task 3.1 owns lifecycle/lazy start
and complete discovery, 4.1 owns integration, and 4.2/phase-end tests must prove
real reconnect/cancel/shutdown behavior. Cache compilation is not that evidence.

## In-progress execution — 2026-09-03, task 3.1 (not complete)

config.rs adds pure placeholder expansion against a supplied OS-string map;
binding environments can now resolve all declared overrides against one captured
parent map without sibling order dependencies or ambient reads. Non-interpolated
OS values remain byte-exact; missing/non-UTF-8 interpolated values fail without
including secret inputs in errors.

The cache now pins generation tickets before catalog reads and readiness calls.
ConnectedMcpServer carries owned transport plus explicit complete discovery;
publication checks exact definition and every registry descriptor. Complete
catalogs can survive host-requested transport retirement, but ordinary identity
invalidation clears them. Old prepared tickets fail after either operation.
Cancelled connector outputs initiate registry shutdown rather than leaking a
returned connection.

New runtime.rs (exported by mod.rs) provides McpRuntimeManager::prepare and
PreparedMcpServer::wait_until_ready/call_tool. Globals remain eager; skill/child
definitions may defer only with a complete exact-key cached catalog. Calls
validate projection identity, wait for matching discovery and use one total
timeout for readiness plus execution. A changed catalog requires a fresh step;
no failed call is replayed. Policy/approval remain upstream host responsibilities.

T0 cargo check --locked --no-default-features --features server-full passed
with zero warnings after both edit groups, in 46.55s and 34.57s. Whitespace
checks had no diagnostics; new-file no-index exit 1 denotes added content.
No tests, test authoring, fmt, acceptance critic or dependency changes ran.

NOT DONE: McpConnector has no production implementation and the runtime has no
RunManager consumer. Do not end-task 3.1 yet. The concrete connector needs
snapshot-based stdio/HTTP startup, complete paginated tools/list and preserved
snapshot reconnect/cleanup. rmcp 3.1.2 depends on reqwest 0.13.4 in Cargo.lock;
UAR directly uses 0.12.28, so its client cannot simply be passed to the SDK's
with_client. Resolve this explicitly; do not re-read ambient proxy/credential
settings and label them a captured environment. This is remaining implementation,
not a repeated external blocker. Begin-task succeeded at revision 1843.

### Continuation — concrete stdio connector

registry.rs now provides connect_stdio_binding; runtime.rs's StdioMcpConnector
is its concrete McpConnector caller. Command lookup uses captured cwd,
MCP_SERVER_DIR and PATH (PATHEXT on Windows), with no ambient fallback or implicit
provisioning. Launch uses env_clear, the full captured environment, explicit cwd
and kill_on_drop. Handshake is bounded by the existing 20-second limit; complete
paginated tools/list and descriptor compilation share the 10-second discovery
limit. Invalid discovery is not published; normal failure awaits service cancel.

Shared reconnect slots now retain request plus complete catalog for snapshot
bindings. Reconnect uses those same inputs and rejects any catalog change before
generation-checked publication. Administrative replacement clears the old
snapshot alongside replacing config/generation. ReconnectAttempt drop decrements
the in-flight counter even on timeout/cancellation; the prior explicit-only
decrements could strand shutdown when the new total call deadline expired.
Legacy reconnect handshake is now bounded too; no failed tool call is replayed.

T0 cargo check --locked --no-default-features --features server-full passed with
zero warnings in 32.21s. Tracked/new-source whitespace checks emitted no errors.
No tests, fmt, critic, manifest changes or task-completion hook. Task 3.1 stays
open: HTTP adapter, production manager wiring and complete partial-launch
shutdown joining remain unfinished/unverified.

The dependency-pin-discipline skill requires a new direct alias's pin in the
operator-owned versions.toml. Requested reqwest_mcp = "0.13.4" under [pins].
This reuses rmcp's existing locked version, not a version bump. The cached
official crates.io index reports 0.13.4, yanked=false, checksum
219c5811de6525e5416c7d5d53bb656d3afdbc6c5af816e0802bcfa42dbdc1c3,
matching Cargo.lock. Fresh crates.io HTTP access returned 403; exact docs URLs
could not be opened, so no fresh-online-verification claim. No pin/manifest was
edited on the operator's behalf. The HTTP prerequisite is pending, not grounds
to count stdio-only delivery as the whole task.

### Continuation — supervised partial stdio startup

New private stdio_process.rs keeps direct-child ownership outside the MCP
handshake future. The transport retains a cancellation token; dropping it
starts the child's bounded graceful-exit period, followed by kill and reap.
TaskTracker retains a join barrier even when startup never returns a service.
Admission closes under the same lock used for process registration. Completed
tasks leave the tracker, and cleanup failures survive until shutdown reports
them. This is host lifecycle ownership, not an OS sandbox or descendant-process
tree guarantee.

registry.rs uses this transport for captured initial launch and reconnect;
SnapshotBinding retains the same supervisor. runtime.rs's StdioMcpConnector is
now stateful, McpConnector requires an awaited shutdown method, and runtime
shutdown drains the cache then the connector. mod.rs registers the private
module. No dependency, pin, workflow or legacy-global-launch changes.

Tier 0 cargo check --locked --no-default-features --features server-full:
Finished dev profile [unoptimized + debuginfo] target(s) in 26.57s; exit 0,
zero warnings. Whitespace checks produced no diagnostics. Source search finds
the supervisor in registry/runtime, but McpRuntimeManager still has no external
RunManager caller. No tests, test authoring, fmt or acceptance review ran.
Task 3.1 remains open at 4/22; HTTP requires the requested operator pin before
its direct client alias can be added. Real process behavior and end-to-end
shutdown remain unverified until phase-end integration tests.

## Execution notes — 2026-09-03, task 3.2

Implemented preflight.rs and McpRuntimeManager::preflight, independent of the
still-open HTTP adapter in task 3.1. The input is an already authority-selected
McpServerProjection, verified ActorOwner, and one captured parent environment.
Each selected declaration resolves its own overrides against that same snapshot.
Required environment/auth/connection/discovery/timeout failures return a named,
actionable error. Optional availability failures emit a bounded warning and
remove the exact selected server from the tool projection; they never activate
a lower-authority declaration. No raw SDK error, URL, env value or token enters
these diagnostics.

McpPreflight returns the resulting exact tool projection, matching prepared
bindings, and serializable warnings. Invalid binding, generation revocation,
cancelled refresh, shutdown and projection violations abort even for optional
servers. projection.rs's private narrowing helper rejects omission of required
or unselected servers and retains tool eligibility unchanged. Complete cached
skill/child catalogs still defer startup as specified; preflight does not force
their first call just to measure availability. Ready cache entries acquired
before a later required failure remain reusable, not globally shut down.

Tier 0 cargo check --locked --no-default-features --features server-full passed
exit 0, zero warnings, Finished dev profile in 31.79s. Whitespace checks produced
no diagnostics. Source call-site search finds runtime.rs calling prepare_servers
and preflight.rs calling the projection narrowing helper; neither is merely an
unexported implementation island. RunManager wiring remains explicitly task 4.1.
No tests, test authoring, formatter, acceptance review or dependency edits ran.
Uncomfortable limit: preflight behavior is compiled, not yet proved by a live
run. Task 1.3 and phase-end integration verification remain open. Task 3.1 is
not being marked complete; its operator-owned HTTP client pin remains absent.

## Execution notes — 2026-09-03, task 3.3

Added lifecycle.rs and domain McpServerLifecycle/McpServerState/McpStateReason
types plus NormalizedEvent::McpServerStateChanged. Exact cache entries publish
disabled, connecting, ready, auth_required, failed and shutting_down. Records
carry random binding/generation/event IDs, monotonic sequence, timestamp and
secret-free reason codes, never config hashes, environment or credential data.
The existing set_mcp_server_status recorder runs inside the publication lock;
only Ready maps to true. Its legacy server-name label remains a compatibility
last-observation metric, not a new per-owner health aggregation.

binding_cache.rs admits owner/config/auth/environment-scoped observations and
publishes refresh, cancellation, invalidation, retirement and shutdown states.
runtime.rs exposes observe and pins authentication-required entries before
preflight rejection. Subscriptions atomically capture current state plus future
events, expose bounded-channel lag, and resynchronize without claiming replay.
Weak publisher references allow receiver closure when the publisher is dropped.
mod.rs exports the module. adapters.rs and sse.rs map the new normalized variant
to uar.agui/1 CUSTOM uar.mcp.server_state and legacy agui.mcp.state respectively.

registry.rs attaches the exact publisher only after cache identity validation.
Shared reconnect slots publish connecting/ready/typed failure and shutdown;
administrative replacement detaches the old identity. One reconnect per slot
prevents competing same-generation attempts from overwriting state. The drop
guard publishes Cancelled only for that generation's still-Connecting state.
ShuttingDown is terminal within a generation; late reconnect completion cannot
resurrect it. Projected bindings no longer issue unordered compatibility metric
writes beside the publisher. Snapshot handshake authentication errors remain
typed instead of collapsing into ConnectionFailed; raw SDK diagnostics are
not included in lifecycle events. No failed tool call is replayed.

Tier 0 cargo check --locked --no-default-features --features server-full passed
zero warnings after the first event/cache group (32.23s), reconnect wiring
(23.66s), and shutdown publication ordering (11.95s). Tracked source diff check
exited 0 without diagnostics. Source search confirms cache-to-registry publisher
attachment, registry transition callbacks, runtime-to-cache observation and both
production AG-UI adapters. RunManager does not subscribe/forward these events
yet; task 4.1 owns that integration. No tests, test authoring, fmt, acceptance
critic, dependency, workflow or pin edits. Task 1.7 and phase-end process tests
remain open; compile success is not behavioral proof. No unrequested feature.

## Execution notes — 2026-09-03, task 3.4

New exposure.rs owns per-stream MCP discovery state. Each projection admits at
most32 MCP tools (runtime/native controls are separate), with recently searched
tools first and stable provider-name fill for other eager tools. Remaining
eligible MCP descriptors are effectively Deferred; Hidden is absent from
visibility and search. Search accepts1..512 nonblank Unicode characters,
matches all terms against name/id/server/description, ranks exact/name matches
first and selects at most8. Prior selections are rechecked against the current
authorized descriptor snapshot; removed, hidden or changed descriptors lose
selection. Selections are bounded to32, not an ever-growing tool window.

Visibility is separate from immutable ToolDescriptor metadata. No schema,
approval, effect, source, server identity or declared exposure is rewritten.
McpExposureSnapshot exposes the effective classification and frozen visible map.
projection.rs bounds model_tools and supports applying discovery to an exact
McpStepProjection without replacing its descriptor/server associations.

New native_skills/search_tools.rs is BuiltIn/ModelOnly, ReadOnly with a discovery
concurrency key. Its only effect is selecting descriptors for a later step.
It returns full matching tool definitions, performs no I/O/startup and grants
no execution authority. Registration happens only when deferred MCP tools
exist, in an orchestrator-owned copy of the native registry per chat stream;
ordinary host clients and sibling/child streams do not inherit this handler or
its selections. Conflicting reserved names fail assembly rather than being
silently overwritten. A registered handler is omitted from steps without
deferred tools.

orchestrator.rs reprojects after skill activation on every iteration, in both
legacy and typed paths. The advertised and executable descriptor maps are the
same frozen visible map; search cannot make a later call in its own batch valid.
resolved.rs now consumes that host-projected map without dropping discovered
tools merely because their original declaration was Deferred. manager.rs uses
the bounded initial projection plus search-control name for its turn manifest.
Both module declarations are exported. No new dependencies or config keys.

Tier0 cargo check --locked --no-default-features --features server-full:
initial exposure/handler group passed zero warnings in28.81s. Integration check
passed in17.14s with one unused Exposure import, removed immediately. Final
check passed exit0, zero warnings, in11.30s. Tracked source git diff --check
exited0 without diagnostics. Source search confirms real orchestrator project/
register calls, manager manifest consumption, and both parallel and serial
execution lookups against the frozen map. No tests, test authoring, fmt,
acceptance critic, dependency, workflow, pin, commit or deletion operations.

No unrequested feature. Guards address model-input bounds, Hidden/policy-omitted
tool exclusion, stale selections, reserved-name collision and same-batch scope.
Uncomfortable limit: search and200-tool behavior are compile-checked, not live
verified; task1.4 and phase-end integration remain open. This live exposure work
does not complete task3.1's HTTP adapter or task4.1's binding-cache integration.

## In-progress execution — 2026-09-03, task 4.1 (not complete)

Prepared execution now has a host call path: preflight.rs resolves a governed
tool name to its exact projected descriptor and prepared server, with no legacy
fallback. Orchestrator retains that preflight, uses it in serial and parallel
MCP dispatch, and replaces legacy MCP descriptors during assembly. ResolvedStep
validates every advertised MCP descriptor against the same prepared snapshot.
In-process native tools retain their existing execution path.

activation.rs adds ProjectedActivationHost and new_projected. The host captures
runtime, catalog, policy, owner and environment once. Activation prepares the
combined active-skill set, validates dependency presence and native collisions,
then publishes body and bindings together. Failure leaves activation unchanged.
RunManager reads MCP registry, preflight, descriptors and active skills under
one context lock, passes the preflight to Orchestrator, and derives skill-use
server attribution from this paired descriptor snapshot instead of the legacy
tool index. No model-supplied server-name parsing is used.

Preflight's freeze_bindings now prepares a concrete child handoff: it keeps only
in-process companion tools, waits for exact prepared bindings, narrows to the
projected tool set, retains leases through capture and rechecks revocation.
ActivationContext exposes this operation without changing its active state.
The registry freeze path now lists all discovery pages, matching the captured
connector's complete catalog instead of rejecting tools beyond the first page.
No transport recipe or reconnect authority is handed to the child. This explicit
delegation operation can start a lazy binding; ordinary preflight stays lazy.

Tier 0 cargo check --locked --no-default-features --features server-full:
initial execution and activation groups passed zero warnings in20.13s and31.94s.
The prior manager-check session could not be recovered; a fresh check passed
zero warnings in19.55s. Outcome integration passed in9.55s; delegation/pagination
passed in7.35s, both exit0 and zero warnings. Tracked source git diff --check
exited0 without diagnostics. No tests, test authoring, formatter, acceptance
critic, dependency, workflow, pin, commit or deletion operations in this work.

Uncomfortable limit: source search still finds no root caller of new_projected
or freeze_mcp_bindings. Root ActivationContext::new and its from_config fallback
are live, and graph nodes still use the legacy registry. Shared runtime/bootstrap,
captured global definitions/auth metadata, policy-universe discovery, lifecycle
forwarding, root delegation wiring and shutdown joining remain unfinished.
The current policy universe contains only global MCP tools/servers; do not widen
resolved tool IDs ad hoc to admit newly discovered skill tools. The HTTP adapter
still requires the operator-owned reqwest_mcp="0.13.4" pin before a direct alias.
Task4.1 remains unchecked at7/22; no end-task hook. Revision1857 is its existing
begin-task boundary. Tests and acceptance remain at the phase end.

### Continuation — verified ingress identity reaches MCP step assembly

RunExecutionRequest now carries optional ActorOwner, preserving verified tenant
identity rather than reconstructing it from user_id. Host context capture uses
the existing verified-context constructor; anonymous middleware context remains
unverified. Manager rejects a stamp/user_id mismatch before session mutation,
then retains the stamp in both actual and shadow ResolvedTurn snapshots.
McpPreflight retains its cache owner; ResolvedStep rejects a missing or unequal
owner (including tenant) before accepting prepared bindings. Descriptor equality
alone cannot justify attaching another principal's connection.

Production callers: HTTP create/resume/checkpoint-resume and chat attach their
middleware context; actor sessions retain ActorOwner instead of discarding its
tenant. Checkpoint absence of new input remains None, with the same exact history
and graph state. No JSON credential fields, token parsing or credential lookup.

T0 cargo check --locked --no-default-features --features server-full passed with
zero warnings after core identity18.47s, HTTP9.87s, actor8.14s and prepared-owner
checks12.90s. Tracked diff check exit0. No tests, fmt, acceptance critic,
dependencies, pins or workflows changed. Guard scenarios are cross-tenant
binding substitution and independently altered user IDs at the host boundary.

Task4.1 remains incomplete,7/22: root runtime/catalog, eligible tool universe,
event/shutdown and graph/delegation wiring are still absent. Legacy embedded,
interaction-continuation and positional APIs intentionally remain unstamped;
their host identity/isolation must be supplied before enabling shared caching.
Anonymous MCP behavior also needs an explicit host isolation solution, not a
fabricated authenticated ActorOwner. HTTP alias still awaits operator pin.

### Continuation — manager consumes captured root MCP resources

runtime.rs adds McpRunResources: immutable owner/catalog/environment plus a
clone of the shared runtime/cache/supervisor. RunExecutionRequest can carry this
host-only, non-JSON bundle. Manager rejects foreign owner, child reuse, divergent
cwd or absent resolved policy before session mutation, defaults world-state cwd
to the captured cwd, and calls ProjectedActivationHost::new/new_projected when
the bundle is present. A failed required preflight terminates the run; it never
falls back to from_config. Existing None requests remain legacy for now.

ProjectedActivationHost now retains the run CancellationToken and races every
preflight against it, covering later model/attachment activation as well as
initial globals. Typed McpPreflightError::Cancelled distinguishes that outcome.
Existing cache guards and the process supervisor own cancelled startup cleanup;
the application host still owns shared-runtime shutdown, not individual runs.

T0 cargo check --locked --no-default-features --features server-full passed
zero warnings in34.12s (manager integration) and13.40s (activation cancellation).
Tracked source diff check passed. No tests, fmt, critic, dependency, workflow,
pin, commit or delete. Identity/cwd/child checks enforce the real captured-grant
boundary. Graph requests with a supplied bundle currently fail explicitly
instead of executing legacy bindings; this is an unfinished adapter, not accepted
graph support. Existing default graph behavior was not changed.

Task4.1 stays open: new_projected now has a manager caller, but no bootstrap/API
caller constructs McpRunResources yet. Need standard root capture, catalog-aware
policy-universe discovery, HTTP transport, graph and child handoff, lifecycle
forwarding and joined shutdown. Remove the temporary graph-unavailable branch
when the governed graph adapter is wired. Do not count optional supplied-resource
support as the specified default migration. Revision1857, MCP7/22 unchanged.

### Continuation — explicit graph tool dispatch enters the trusted host

ToolNode no longer calls the legacy registry directly or supplies an empty object
for missing arguments. GraphToolHost is constructed by manager and retained by
ActorRootBinding. Orchestrator's graph operation captures the activation registry
and preflight together, checks run/owner, descriptor/schema/exposure/sandbox, then
uses the existing approval and root tool-budget gate before exact MCP dispatch.
Result bounds and ToolStart/ToolEnd apply. Pending work remains host-owned if the
node waiter is dropped; shutdown drains the exact future without replay.

Source review caught panic re-polling; the retained future now catches unwinds
inside its slot. Shutdown closes admission/cancels before drain, and a cancellation
check after ToolStart prevents starting an operation during cleanup. Final source
review found no remaining concrete defect in the revised explicit-tool path.

Tier0 cargo check --locked --no-default-features --features server-full first
failed on missing Exposure/redundant Future imports; both fixed. Subsequent
passes40.88s/33.49s, zero warnings. Tracked whitespace checks exit0; new-file
no-index check exit1/no diagnostics. No tests/test authoring/fmt/strict validation.

Task4.1 remains open. LlmNode still uses legacy advertisement and ignores tool
calls; manager's captured-MCP graph rejection remains pending a complete host turn
adapter. Root bootstrap/policy-universe/lifecycle and HTTP transport remain.
Settlement is local future completion, not remote effect rollback. Waypoint1870;
MCP7/22, actual91/182, implementation2/10. No repeated task hook.

### Continuation — graph model turns use the governed host

LlmNode now requests a tool-aware turn from GraphToolHost. It no longer advertises
the raw registry or discards tool calls. The host retains the exact pending stream,
records assistant/tool dialogue in the run and conversation, persists settlement,
and returns graph history even on interruption. Node-local system overlays stay
outside that returned history. GraphContext no longer exposes the raw MCP registry.
The manager's blanket captured-graph rejection is removed; remote AgentNode still
rejects captured MCP and inherited child authority before legacy A2A dispatch.

Review found and fixed cancellation waiting for an activation lock before draining
the operation that held it; swallowed persistence failures; panic protection that
excluded settlement; synthesized success at provider EOF; and accumulating node
prompts. Host settlement failures are sticky across repeated shutdown. The outer
producer finalizer retains the host independently of ActorRootBinding, so child
graphs without their own root binding are drained too. Cleanup attempts all other
resource families even after graph settlement failure. No mutation replay added.

Tier0 cargo check --locked --no-default-features --features server-full: initial
opaque-lifetime error fixed with the compiler-suggested precise capture; passes
29.27s,37.38s,19.03s,10.56s, zero warnings. Independent source reviews cleared the
revised receipt path, terminal/prompt handling and inherited-remote boundary.
This is source acceptance only; no tests/test authoring/fmt/strict validation.

Task4.1 remains open for standard root catalog capture, catalog-aware policy
discovery, default binding-cache migration, lifecycle forwarding and joined shared
runtime shutdown. HTTP transport remains3.1 and needs its manual reqwest_mcp pin.
GraphContext test fixture migration and runtime event/history/cancellation coverage
remain phase-end work; full graph protocol event parity still needs integration
review. Remote execution enforcement is not implemented by rejecting that path.
Waypoint1870 and all task/change counters unchanged; no task-end hook.

### Continuation — exact-binding lifecycle events reach projected runs

ProjectedActivationHost now supplies a verified owner/run event bridge to initial
and later skill preflights. Each prepared binding subscribes before readiness;
startup, lazy readiness, reconnect failure and governed tool-call operations
forward their existing lifecycle records with the run id attached. Cancellation
drops the exact startup refresh before draining its terminal transition and remains
the typed McpPreflightError::Cancelled. Calls are never replayed. No observer task
or second transition source was introduced.

Repeated/concurrent operations sequence-deduplicate by lifecycle binding id. A
lagged receiver publishes the current snapshot with an explicit warning rather
than claiming replay. The compatibility server-health gauge now aggregates all
exact binding identities: observing one owner's dormant binding cannot overwrite
another owner's Ready state. Final LifecycleState drop unregisters its binding;
aggregate mutation and gauge publication share one lock to prevent stale writes.

Independent review accepted lifecycle ordering, identity, cancellation, no-replay
and no-detached-task mechanics. It found the server-name gauge conflation and a
publication race; both were fixed and the final review accepted the source.
Tier0 cargo check --locked --no-default-features --features server-full passed
39.82s,9.96s,36.03s,10.25s, zero warnings. No tests, test authoring, fmt or strict
validation. Task4.1 remains open for production root catalog capture/default
binding lookup, policy universe, HTTP adapter and shared runtime shutdown.
Counters/waypoint unchanged; no task-end hook.

## Execution notes — 2026-09-04, task 4.1

Verified root requests now capture one immutable global/skill MCP definition
catalog from RunManager and use the application host's shared McpRuntimeManager,
binding cache, environment snapshot, and stdio supervisor. OpenAI, ACP, A2UI
continuations, A2A actor sessions, the main chat path, and `/mcp/uar` retain
middleware-verified identity into RunExecutionRequest. Legacy `user_id` entry
points cannot manufacture an ActorOwner or acquire projected bindings.

Catalog definitions receive a secret-free per-boot opaque binding revision;
Unknown and Required authentication fail closed. Cache keys retain the complete
verified subject/tenant owner, configuration hash, authentication revision, and
resolved environment. `/mcp/uar` no longer accepts payload identity and its
status lookup compares the complete host-only ActorOwner. Administrative server
replacement, disablement, and removal invalidate every projected owner/revision,
begin revocation immediately, and join retired transports.

The configured connector supports captured stdio and streamable HTTP snapshots.
HTTP uses the rmcp-compatible pinned reqwest 0.13.4 client with ambient proxies
and redirects disabled; connection errors never include expanded URLs. Stdio
children receive only explicit server environment keys plus minimal process
launch variables, not the host's unrelated credentials. Snapshot reconnect
retains the exact request/catalog and rejects descriptor changes. Application
shutdown joins the legacy registry, projected cache/connector, supervised partial
stdio launches, and live-query cleanup.

Effective All/Auto resource selections remain open only while no parent
Selected/None/deny restriction has closed them. This admits tools discovered
from the frozen selected server catalog without allowing a lower scope to reopen
access. Production source search finds `McpRegistry::from_config` only in test
fixtures; skill activation no longer constructs a registry per run.

Observed Tier 0 checks: the offline lock refresh passed after adding the pinned
reqwest_mcp alias. Repeated `cargo check --locked --no-default-features --features
server-full` runs passed with exit 0 and zero warnings; the final run completed
in 37.23s. Targeted `git diff --check` emitted no diagnostics. Successive
history-free artifact reviews found and drove corrections for identity loss,
authentication state, open-world discovery, environment leakage, admin
revocation, URL leakage, and tenant-only status lookup; the final review reported
no actionable P0-P2 findings.

No tests or test authoring, formatter, full build, integration run, strict
OpenSpec validation, workflow change, commit, or destructive operation ran.
Those checks remain phase-end tasks 1.1-1.8 and 5.1-5.4. Compilation and source
review do not prove process reuse, lazy readiness, cancellation, or shutdown
behavior; task 4.2 and the phase-end suite retain that risk.

## Execution notes — 2026-09-04, task 3.1

The shared runtime now starts host-global MCP definitions eagerly and defers a
skill-contributed definition only when the exact owner/config/auth/environment
generation already has a complete catalog. `PreparedMcpServer` pins that
generation; `wait_until_ready` performs the lazy connection, validates that
fresh discovery still matches the prepared catalog, and the governed tool-call
path applies one total deadline to readiness plus execution without replay.

The initially planned child-definition source conflicted with the completed
delegation architecture and had no production caller. It was removed rather
than left as type-only authority. Local children receive only narrowed frozen
bindings and cannot reconstruct or reconnect a server. Authenticated remote UAR
children execute as roots on the peer and resolve that peer's host-global and
skill catalog. The proposal and spec now state this boundary explicitly; no
connection recipe or credential crosses the UAR-to-UAR delegation contract.

Configured stdio and streamable-HTTP connectors use immutable binding snapshots,
complete paginated discovery, the shared binding cache, and host-owned shutdown.
Connection reuse remains logical MCP binding reuse; live bindings are retained
instead of being killed merely to manufacture dormancy. Unexpected disconnect
recovery remains task 4.2, not a reason to replay a possibly mutating call.

Observed Tier 0 command: `cargo check --locked --no-default-features --features
server-full` passed in 58.52s, exit 0, with zero warnings after the authority
cleanup. A targeted source search found no remaining `ServerSource::Child`,
`ServerAuthority::Child`, or MCP `child_thread_id` references. Targeted diff
checking emitted no whitespace diagnostics. A fresh history-free artifact critic
reported no actionable P0-P2 findings.

No tests or test authoring, formatter, full build, integration run, strict
OpenSpec validation, workflow change, commit, deletion, retry, or fallback ran.
Phase-end tasks retain behavioral proof. The guard is the existing delegation
trust boundary, not a speculative restriction: local children cannot receive
connection recipes, and remote UAR peers own their credentials and catalogs.

## Execution notes — 2026-09-04, task 4.2

Reconnect retains the exact captured owner/config/auth/environment request and
complete catalog. Publication remains generation-checked, catalog drift is
rejected, concurrent reconnect is single-flight per service slot, and a failed
tool call is never replayed. The projected call deadline continues to cover
readiness plus execution; connector handshake and discovery retain their own
bounded deadlines beneath it. Private HTTP clients may reuse idle connections
only inside one exact binding; ambient proxies and redirects remain disabled.

Shutdown ownership was strengthened after three history-free review passes.
Every replaced or rejected `RunningService` is synchronously cancelled and
retained in its shared slot until an awaited, cancellation-safe reap. Registry
upserts register before connection I/O; shutdown atomically closes admission
with the service-map snapshot and waits for every admitted producer. A second,
slot-local producer count covers merged and filtered registries that share live
slots but not registry-level admission state. The final insertion guards the
actual slot found under the map lock, including concurrent double-upsert races.

Synchronous removal now enters registry admission and transfers the removed
slot into a shutdown-owned queue before releasing the service-index lock. Slot
shutdown cancels current and retired transports; application shutdown waits for
reconnect, update, removed-slot, rejected-result, cache, connector, and supervised
stdio cleanup before reporting completion. Cancelled reap futures leave their
records in shared queues for the next reap or application shutdown.

Observed Tier 0 checks after successive source edits: locked server-full checks
passed in 39.12s, 12.89s, 1m23s, 38.76s, 52.30s, and 9.98s, all exit 0 with zero
warnings. Targeted `git diff --check` emitted no diagnostics. The final fresh
artifact critic reported no actionable P0-P2 findings.

No tests or test authoring, formatter, full build, integration run, strict
OpenSpec validation, workflow change, commit, deletion, call replay, retry
policy, or fallback was added. Under the operator's phase-end-only test rule,
the task's test-extension clause moved to Tier 1 task 5.1. Source review and
compilation do not prove runtime races; tasks 1.1-1.8 and 5.1-5.4 retain that
behavioral evidence obligation.
