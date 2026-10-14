# Dot19 task 2 — MCP client elicitation source handoff

Phase: Dot Execute. Production source authored on 2026-10-12. Full-phase verification
boundary CLOSED. No tests, checks, builds, lint, formatting, probes, OpenSpec
validation, policy validators or product reviews were executed. This is a source
handoff, not acceptance evidence. Canonical KBD/projection files and task checkboxes
were not changed by this worker.

## Delivery against plan

Delivered the run-scoped client handler, durable interaction CAS, authenticated
read/respond API, AG-UI integration seam, root-owned connection cleanup, official
SDK pin documentation and unexecuted wire/peer fixtures. The necessary extension
to the original packet is an interaction-aware execution timer plus root cleanup:
source inspection found both ordinary tool timers fixed at30seconds and the old
cache shared connections across runs. Human waiters now pause only that ordinary
budget, bounded by ten additional minutes; run channel IDs partition the cache
and finalizers reap those new run-owned connections.

The other necessary SDK change selects `service.peer().call_tool`. In pinned
rmcp3.4.1, the higher-level `RunningService::call_tool` invokes input handlers and
resends a tools/call for multi-round results. Installing a real accepting handler
without changing that method would silently enable another call. The selected
MCP2025-11-25 reverse-request profile does not drive those SDK rounds.

## File inventory and responsibility

- `src/mcp/interaction.rs`: channel, live-waiter availability, pinned client
  capabilities and request handler; durable create/wait/settle; cancellation,
  transport/credential recheck; bounded input-wait timer.
- `src/mcp/mod.rs`: exports interaction module.
- `src/mcp/binding_cache.rs`: optional channel in request and cache key; retains
  partition in registry/catalog comparison; targeted invalidation/reaping.
- `src/mcp/runtime.rs`: resources/runtime channel propagation; root lifetime
  guard; projected execution budget observes actual wait activity.
- `src/mcp/preflight.rs`: attaches captured channel before cache/observation.
- `src/mcp/registry.rs`: all five client serve sites use explicit handler;
  snapshot reconnect retains run+credential lease; single-call peer dispatch;
  exact interactive connection closes when a call is abandoned or uncertain.
- `src/uar/persistence/mcp_elicitations.rs`: distinct typed durable interaction
  record/view and terminal transition validation; response schema validation.
- `src/uar/persistence/mod.rs`: durable capability and create/list/CAS seams.
- `src/uar/persistence/providers/mcp_elicitations/{postgres,surreal}.rs`: durable
  create/history/CAS and database-time response expiry arbitration.
- `src/uar/persistence/providers/{postgres,surreal}.rs`: module/delegate additions
  in agreed writer window, preserving other teammates' edits.
- `migrations/20261012040000_mcp_elicitations.sql`: PostgreSQL interaction table
  and principal/run history index. Surreal follows existing schemaless provider
  record pattern and durability capability.
- `src/uar/api/mcp_interactions.rs`: authenticated read/respond router and AG-UI
  list/resolve seams; exact full principal, workspace and live native-run checks.
- `src/uar/api/mod.rs`: exports authenticated interaction router.
- `src/uar/runtime/manager.rs`: teammate first installed channel before projected
  activation; this worker then used an explicitly granted narrow window to add
  root guard capture and producer-finalizer close/join/error propagation.
- `src/server.rs`: lead mounted interaction router in the authenticated shared
  RunApiState route tree. This worker did not edit server.rs.
- `src/uar/api/agui.rs`: A2A/AG-UI teammate owns union interruption/resume projection
  using the exact list/resolve seams. This worker did not edit agui.rs.
- `docs/protocols/mcp-elicitation.md`: source contract, ownership and operations.
- `openspec/changes/dot-19-interactive-protocols/design.md`: MCP source-contract
  append, preserving teammate's AG-UI content.
- `openspec/changes/dot-19-interactive-protocols/specs/mcp-runtime-projection/spec.md`:
  run partition, lifetime, cold history and no automatic replay scenarios.
- `tests/fixtures/dot19_mcp_wire.json`: exact pinned protocol examples.
- `tests/fixtures/dot19_mcp_peer.py`: real stdio/StreamableHTTP controlled peer,
  independent fsynced capabilities/answers/effects. AUTHORED, NEVER EXECUTED.

A separate source-only installed journey in `scripts/dot/mcp_journey.py` belongs
to Dot17 and is being authored after this production handoff; it is not production
functionality or current verification evidence.

## Registration and interface packet

`McpInteractionChannel::new(store, &ActorOwner, native_run_id, session_id,
actual_host_workspace, run_cancellation)` requires durable storage and a full
credential principal. `McpRunResources::with_interactions(channel)` embeds it in
the cloned runtime used by ProjectedActivationHost. The root stores
`resources.interaction_guard()` outside the fallible execution future, moves it
into the producer finalizer, and awaits `finish()` with other resource cleanup.
A child inherits frozen bindings and does not own a second root guard.

`mcp_interactions::build_router(SecurityConfig)` returns
`Router<Arc<RunApiState>>` for the authenticated `/api/uar` tree:

- GET `/runs/{native_run_id}/mcp-interactions` returns `interactions`, each
  `record` plus `resolvable`.
- POST `/runs/{native_run_id}/mcp-interactions/{id}` consumes exact SDK
  `ElicitResult` (`action` accept/decline/cancel, form-only accepted content).
- Success commits response and explicitly reports transport delivery unconfirmed.
  Duplicate/stale/cold is409, invalidcontent400, uncertain persistence503.

AG-UI calls `list_for_context(&RunManager,&UserContext,run,workspace)` and
`resolve_for_context(same arguments,id,ElicitResult)`. Missing durable capability
returns an empty history so approval-only AG-UI remains usable. A cold pending
record remains inspectable; no new connection can consume it. Expiry is committed
on read. `owner_key` is always CredentialPrincipal::storage_key, never the legacy
presentation subject/tenant key.

## Authority and truthful outcomes

The interaction record contains no tool approval, execution grant, checkpoint,
lease renewal or duplicate executor. All existing descriptor/policy/approval
admission and canonical tool receipts remain authoritative. Request-level,
root-level and credential lifetimes are checked before answering; pauses never
extend credential or delegation deadlines. A committed human response can race
disconnect or revocation; it remains history but is not a transport receipt.

Named guard scenarios: malformed peer schema/content and URL schemes at the real
MCP trust boundary; cross-owner/run/cache callback mixups; stale or duplicate
responses; root/request cancellation; transport disconnect and dropped callbacks;
HTTP lease expiry/revocation; abandoned tools/call; cleanup failure and process
restart. No unrelated production features, dependency changes, or policy bypass
were added.

The uncomfortable thing: remote cancellation cannot establish absence of an
effect. Unknown transport outcomes stay unknown. Closing an abandoned interactive
connection can also interrupt concurrent calls on it; those calls are not replayed.
Pending cold records can remain until deadline, and resuming that old protocol
request is deliberately unavailable. The API reports this rather than accepting
input that no original waiter can receive.

## Source references and verification disposition

Context7 resolved `/websites/rs_rmcp_rmcp`, then exact local rmcp3.4.1 source was
read. Cargo pins remain unchanged. Official source commit is
9427a929959e665e0d12e9395f674026baf4bd48. Exact official handler docs were fetched
at https://docs.rs/rmcp/3.4.1/rmcp/handler/client/trait.ClientHandler.html;
request/result/capability details were read from that exact crate source. The
versioned docs fetches for the three individual model pages returned internal
errors, so they are not cited as fetched evidence. All successful source links
and the jsonschema format choice are recorded in the protocol document.

Only source reads and file writes ran. No verification command/output exists for
this slice. Rust type/borrow correctness, SQL execution/CAS races, actual wire
interoperability, provider parity, cleanup timing, restart, the Python fixture and
the installed customer artifact all remain unverified until the lead opens the
full Dot implementation boundary. The later gate must cover both transports,
every action, invalid/duplicate/wrong-owner responses, >30second input waiting,
credential revocation, child attribution, restart, root cleanup, no automatic
multi-round resend, and independently Required effects in local/shared profiles.

## Dot17 fixture addendum — source only

The requested bounded `scripts/dot/mcp_journey.py` is now authored and the Dot17
owner has wired `prepare_peers` before installed-host startup, `run_mcp` after live
journeys, and peer cleanup in `finally`. It uses the real Dot19 stdio/HTTP peer,
all form/URL actions, invalid content, principal isolation, duplicate responses,
a31second human wait, a separately approved/denied Required effect, explicit
single-call rejection of a multi-round result, controlled peer disconnect,
concurrent independent root channels, then a single killed-host cold-history stage.
No logical run is started after that abrupt host restart: same-boot supervisor
exclusion cannot be assumed. The peer gained explicit fixed HTTP port and
fixture-owned disconnect-file controls. None of these files have been executed,
parsed as a check, compiled or linted.

The primary memory server successfully created entity
`uar-dot19-mcp-elicitation-2026-10-12` with the source decisions, automatic-round
constraint and unverified disposition. No append-only history was rewritten.
