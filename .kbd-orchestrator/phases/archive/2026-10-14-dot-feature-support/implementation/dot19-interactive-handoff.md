# Dot19 interactive source handoff

Phase: dot-feature-support / Execute. Production authoring only; no tests,
checks, builds, formatters, validators, probes or review gates have run.

## Authoritative contract

AG-UI official generated schema/types and package provenance are under
`implementation/protocol-sources`, pinned by the lead to upstream commit
`704b190e44aedda71a5388d2676370711c6b6b49` and published `@ag-ui/core` 1.0.2.
Context7 `/ag-ui-protocol/ag-ui` confirms standard `resume[]`; generated pinned
schemas govern exact fields. A resume uses a new wire runId, answers every open
interrupt by interruptId, and does not replay earlier tool starts. Interrupt
outcome is `RUN_FINISHED` with `outcome:{type:"interrupt",interrupts:[...]}`;
entry is `{interruptId,status:"resolved"|"cancelled",payload?}`.

## Authored A2A and shared host seam

- `api/a2a/thread_service.rs`: exact broker views under task metadata
  `uar.approvals`; pending recoverable/resolvable records project input-required.
  `SendMessage` for the existing task accepts request metadata
  `uar.approval:{id,approved}`. It renews peer identity/contract, resolves the
  existing broker CAS, or reserves the exact cold checkpoint continuation.
  Previous approval source run IDs remain correlation metadata for idempotent
  duplicate answers. Denial records cancellation intent; it is not fabricated
  cancellation success.
- `api/a2a/v1.rs`: committed denial projects REJECTED only after a confirmed
  terminal task without unresolved cleanup. Arbitrary client metadata never
  changes authoritative task metadata.
- `api/routes.rs`: `resolve_host_approval(state,user,capture,run_id,approval_id,
  approved,workspace,resume,reserved_run_id,fresh_a2a_contract)` returns
  `HostApprovalResolution {record,run_id,delivered,resumed}` or HTTP response.
  It authenticates exact broker records; supports idempotent same decisions,
  warm delivery, cold denial and protected cold approval. `ResumeRequest::for_approval`
  creates an empty input; approval never substitutes prose for checkpoint state.
- `runtime/manager.rs`: owner-capability approval views and explicit descriptor
  Required exclusion from eligible governance-Off bypass. Other Off risk
  semantics remain unchanged.
- `runtime/thread/approvals/recovery.rs`: protected A2A checkpoint kinds may
  carry approval evidence. Claimability does not grant execution; fresh current
  host admission is still mandatory.
- `docs/A2A_V1.md`: exact decision metadata and denial behavior documented.

## Authored AG-UI production

- `api/agui.rs` exports `build_router(SecurityConfig) -> Router<Arc<RunApiState>>`.
  Root mounts `/api/uar/ag-ui`; POST `/agents/{agent_id}` consumes the pinned
  RunAgentInput; GET `/agents/{agent_id}/threads/{thread_id}/runs/{run_id}/stream`
  reconnects. Exact full principal and current workspace authorization precede
  storage reads. Native target admission is reused; catalog target IDs use the
  same `uar:agent:<binding>` vocabulary. Builtin/catalog workspace differences
  follow the actual native protected context.
- `persistence/agui_runs.rs`, `providers/agui_runs/{postgres,surreal}.rs`,
  `20261012030000_agui_runs.sql`, `surrealdb/agui_runs.surql`: immutable transport
  identity, atomic owner/agent/thread slot reservation, terminal projection CAS.
  No execution, approval, lease or credential authority is added to these rows.
  Trait, native provider and Surreal migration registrations are authored.
- AG-UI requires messages, validates protocol/identity/context fields, explicitly
  rejects frontend tools, handles text/native history, and includes state/context
  as untrusted client prompt data. Forwarded native host resources never replace
  target/session/history identity or enter persisted correlation records.
- All open approval and MCP elicitation IDs must be answered before a resume is
  reserved. Warm resumes retain exact native host identity and use a new wire ID;
  cold approval reserves one exact protected checkpoint continuation. No client
  disconnect cancels an execution. Persisted terminal/interrupt projection survives
  reconnect; unavailable bounded token history produces a truthful stream error.
- Pure stream projection emits valid RUN_STARTED, text, tool proposal/argument/end,
  tool result, RUN_FINISHED outcome or RUN_ERROR events. An approval proposal is
  not execution. The next wire segment suppresses old tool starts; native source
  boundaries and stable projected event IDs are preserved. Existing legacy SSE
  adapters are unchanged.
- `api/mcp_interactions.rs` is MCP teammate-owned. AG-UI consumes its exact-owner
  pending and resolution seams. MCP ElicitResult payloads never enter approval
  CAS. Disconnected/restarted elicitation cannot be replayed to a new connection.
- `runtime/manager.rs` now attaches a root-only McpInteractionChannel before
  activation when verified principal and durable store capabilities exist.
  Exact run/session/catalog workspace/cancellation bind it. Construction failure
  emits native Error and fails admission. The MCP teammate confirms
  `with_interactions` carries the channel through runtime clones into projected
  activation and later skill connection preparation; children inherit the root.
- `routes.rs` ResumeRequest is Clone for same-request multi-interaction delivery.
- `docs/AGUI_V1.md` documents profile/authority/reconnect/recovery; deferred fixture
  `tests/fixtures/dot19_agui_real_path.py` calls real POST/SSE endpoints and prints
  outcomes for submit, resume approve/deny, and reconnect. Dot17 runner owner has
  its exact interface. It has never run.
- Root-requested `supports_durable_tool_receipts()` capability was added default
  false, Postgres true, Surreal `durable_instances` (surrealkv or remote server
  with UAR_REMOTE_SURREAL_DURABILITY_ATTESTED=1). Root consumes it in capabilities.

## Remaining phase integration and uncertainty

MCP19/2 remains teammate-owned. Its API seams and runtime carry are integrated
at source; its handler/provider lifetime work and fixtures are separate. Root
owns server composition and corrected governance warning. Dot11 predecessor
exclusion remains the authority for cold approval recovery; nonlocal A2A
cancellation now calls its resolved-effects barrier and atomic excluded-epoch
native reconciliation (see the updated Dot18 handoff). A2A completion observer now reloads and retries a projection CAS
conflict instead of abandoning live cancellation solely for a concurrent read.

The uncomfortable case is a committed decision followed by a lost continuation
response, or a reserved wire/native ID with no host admission receipt. Adapters
read the existing durable mapping and do not replay an uncertain command. A
missing native producer alone never authorizes takeover. Multiple independent
cold continuations cannot be partially admitted in one AG-UI batch. A source
error or bounded replay gap is reported as such, not as success. These boundaries
require real restart/race qualification after implementation is complete.

Everything above remains uncompiled and unexecuted under the full-phase boundary.
No test, build, formatter, lint, validator, probe or review gate ran. No canonical
task, waypoint or acceptance state was mutated by this teammate.

Final host lifecycle integration: the MCP teammate added the resource interaction
guard to the outer producer finalizer, including graph, panic and early-return
paths. The A2A/AGUI teammate added manager
`settled_run_status_for_context` and graph supervisor `is_settled`: AG-UI terminal
projection waits for producer finalization and graph/root persistence instead of
sealing the earlier native RunDone frame. Final Error status takes precedence
over an earlier success frame; pending interrupts still end the wire segment
without waiting for the paused producer to finish. This remains source-only.

Latest source status: root accepted Dot19/1 for canonical source-complete
recording. The MCP teammate owns Dot19/2's remaining delivery record; its channel
attachment and resource lifetime edits in manager are integrated. Server wiring
and the governance warning are root-owned. No task acceptance or protocol
qualification follows from this source handoff. Dot17 consumes the real-path
AG-UI fixture and its exact submit/resume/reconnect interface at the final gate.
