# Governed MCP client elicitation

Status: source implementation, October 12, 2026. No compilation, protocol fixture,
restart exercise, or production gate has run for this change. The full Dot phase
implementation boundary remains closed.

UAR's MCP client uses the pinned `rmcp = 3.4.1` and explicitly selects MCP
`2025-11-25`. The callable SDK contract was located with Context7
`/websites/rs_rmcp_rmcp`, then read from the installed exact-version crate source,
whose `.cargo_vcs_info.json` identifies commit
`9427a929959e665e0d12e9395f674026baf4bd48`, path `crates/rmcp`.

- [Exact SDK client handler](https://docs.rs/rmcp/3.4.1/rmcp/handler/client/trait.ClientHandler.html)
- [Pinned request, result and action source](https://github.com/modelcontextprotocol/rust-sdk/blob/9427a929959e665e0d12e9395f674026baf4bd48/crates/rmcp/src/model.rs)
- [Pinned capability source](https://github.com/modelcontextprotocol/rust-sdk/blob/9427a929959e665e0d12e9395f674026baf4bd48/crates/rmcp/src/model/capabilities.rs)
- [Pinned single-call and automatic-round implementations](https://github.com/modelcontextprotocol/rust-sdk/blob/9427a929959e665e0d12e9395f674026baf4bd48/crates/rmcp/src/service/client.rs)
- [MCP multi-round proposal](https://github.com/modelcontextprotocol/modelcontextprotocol/pull/2322)

`ClientHandler::create_elicitation` receives `ElicitRequestParams` and returns
`ElicitResult`. Form mode includes `message` and `requestedSchema`; URL mode
includes `message`, `url` and `elicitationId`. Results carry lowercase `accept`,
`decline` or `cancel`; content is present only for an accepted form. UAR validates
form content against the SDK's typed schema with format checking enabled using
the already pinned jsonschema 0.49.4. URL requests are shown as untrusted HTTP(S)
links; UAR does not navigate to them or fetch their contents.

## Capability and connection ownership

The host attaches an `McpInteractionChannel` to an authenticated root's
`McpRunResources` before projected activation and preflight. Storage must support
durable elicitation; a full credential principal and actual host run, session
and workspace are captured. A subject-only identity or memory-only store does
not enable the capability. This channel owns input delivery, never execution.

All five client construction sites in `src/mcp/registry.rs` use `RunMcpClient`:

| Construction path | Handler capability |
| --- | --- |
| `connect_server` stdio reconnect/administration | Unavailable: no current governed run |
| `connect_configured_http` ambient HTTP | Unavailable: no current governed run |
| `connect_configured_server` initial stdio discovery | Unavailable: no current governed run |
| `connect_stdio_snapshot` projected root/skill and reconnect | Captured run channel when installed |
| `connect_http_snapshot` configured/request-owned HTTP and reconnect | Captured run channel and original credential lease |

The capability is `{"elicitation":{"form":{"schemaValidation":true},"url":{}}}`.
The handler checks the negotiated version before accepting a mode. The run's
channel UUID participates in the binding cache key, so a new run cannot inherit
another run's callback. A reconnect receives a new connection UUID and retains
the original run identity; it never receives old responses. Catalog-key comparison
retains the same channel partition.

Local children receive the already frozen root connections and narrowed tool
projection. They do not own the root's interaction guard, cannot install a new
human authority, and cannot renew credentials. Their requests are presented under
the root interaction run; child tool policy/admission still owns execution.
Projected skill activation clones the runtime containing the same channel.

A root-owned lifetime guard lives outside the fallible execution body. The
producer finalizer closes/reaps that channel's connections and settles pending
records even when graph execution returns early or panics. Admission failure or
task drop revokes synchronously and retains cleanup in the shared cache. Pending
settlement errors become `mcp_interaction_cleanup_unconfirmed`, not successful
terminal cleanup.

## Authenticated host interface

The standard authenticated API mounts:

- `GET /api/uar/runs/{native_run_id}/mcp-interactions`
- `POST /api/uar/runs/{native_run_id}/mcp-interactions/{interaction_id}`

Both recheck workspace authority and derive the full principal storage key from
verified middleware context. Reads return `interactions`, each containing its
`record` and `resolvable` flag. Scoped records require their actual host workspace;
builtin records have no catalog workspace, matching the existing approval broker.

Each record binds owner, native run, session, optional workspace, channel,
connection, server, transport request ID and exact typed request. States are
`pending`, `answered`, `cancelled`, `disconnected` and `expired`. An answered record
retains the precise accept/decline/cancel response. Other terminal states describe
the host disposition; none purport to be a human answer.

POST consumes an SDK result, for example:

```json
{"action":"accept","content":{"value":"accepted"}}
```

Decline/cancel and URL acceptance omit content. A committed response returns 200
with `response_committed: true` and `transport_delivery: "unconfirmed"`; invalid
content returns 400, stale/duplicate/cold decisions 409, and unknown persistence
outcomes 503. Callers read the record after an uncertain response instead of
replaying execution. The CAS allows exactly one terminal transition from pending;
PostgreSQL evaluates expiry after acquiring its row lock, and SurrealDB evaluates
expiry inside the conditional update.

The AG-UI adapter projects elicitation and approval interruptions together and
requires exact interrupt coverage in `resume[]`. Elicitation payloads are SDK
results; AG-UI cancellation becomes `{"action":"cancel"}`. Approval booleans stay
on the existing approval broker. A transport run correlation is not a new ledger.

## Lifetime and authority

Each elicitation expires after ten minutes. The ordinary MCP tool execution
budget pauses only while a live input waiter exists; a call can spend at most ten
additional minutes waiting, including repeated requests. Absolute caller,
delegation and credential deadlines do not change. A disconnected request,
revoked/expired HTTP credential, cancelled root, or abandoned callback is no
longer resolvable. Before returning a response to the SDK, the handler rechecks
those same captured lifetimes.

The registry uses the low-level `Peer::call_tool`, which sends one call, rather
than the SDK's automatic multi-round helper. An unsupported `input_required`
response therefore cannot cause an accepted elicitation to resend a tool call.
The existing unknown-outcome path remains authoritative after an uncertain
transport. Abandoning an interaction-enabled call closes that exact connection,
so a later answer cannot restart its already abandoned operation. Concurrent calls
on that connection may also become uncertain; they are not replayed.

The uncomfortable thing: the client cannot prove that a remote server received a
response or obeyed cancellation. A tool already dispatched may have produced an
effect. The durable interaction records only the user's committed decision and
host disposition; the ordinary canonical tool receipt still determines whether
execution is completed, failed, or unknown. Acceptance never satisfies a separate
Required approval or changes descriptors, policy, grants, fences or epochs.

After process death a record can remain pending, with `resolvable: false` because
its exact live waiter no longer exists. Reading after its deadline commits expiry.
No new connection or root consumes that response. A new explicit operation goes
through ordinary host admission; historical acceptance is not replay authority.

## Deferred fixtures

`tests/fixtures/dot19_mcp_wire.json` records the selected wire shapes.
`tests/fixtures/dot19_mcp_peer.py` is a real stdio/Streamable HTTP peer with fsynced
capability, request, answer and independent effect traces. It does not replace
the runtime, approval broker or persistence. The Dot17 source runner owns runtime
startup/termination and final assertions.

After the full phase inventory opens verification, exercise both transports and
local/shared durable providers for every action, invalid content, duplicates,
wrong owner/workspace, concurrent response/disconnect, a wait beyond the ordinary
30-second budget, credential expiry, root cancellation, child request attribution,
clean root cleanup, killed runtime history, new-run isolation and rejected SDK
automatic retry. Inspect the separately invoked Required effect and its ordinary
approval record; the peer's elicitation answer is not evidence of tool authority.
