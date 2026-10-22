# ACP caller tools handoff

Execute phase. VERIFICATION CLOSED. All commands were source inspection or implementation/documentation writes. No tests, checks, compiler, formatter, builds, review gates, commit or push ran.

Owned files changed:
- src/uar/api/acp/types.rs: AcpClientTools wire declaration (tools, tagged tool_choice, parallel_tool_calls, history), optional session/run declaration, caller-tools event type.
- src/uar/api/acp/handler.rs: owner-scoped session declarations; run override/inheritance; shared prepare/claim/replay and completion; native runs/submit_tool_results operation; complete canonical result-batch validation through the shared service; tenant/agent/session binding; durable batch lookup in runs/get. Caller-tool execution collects terminal RunManager events and commits a continuation response before returning it. Subscribing before reading history avoids losing fast completion. Broadcast lag recovers retained history; failed recovery errors rather than dropping calls. Exact matching completed results replay without new execution. A durable authenticated batch can restore its session after process restart.
- src/uar/api/acp/routes.rs: runs/submit_tool_results streaming support; caller-tool events replay only after response persistence and end with acp_result containing continuation history. Other runs keep their existing live streaming behavior.
- src/uar/api/acp/mod.rs: native JSON-RPC extension contract and buffered streaming limitation documented.

Wire: sessions/create and runs/create accept client_tools:{tools:[OpenAI function schema],tool_choice:{type:auto|none|required|named,name?:string},parallel_tool_calls?:bool,history:canonicalMessages[]}. Session declarations cannot carry history. runs/create response for caller tools returns run_id/status/output/events and, when required, batch_id/client_tools with exact visible predecessor history. Append complete contiguous role=tool results (tool_call_id/content) and submit runs/submit_tool_results with session_id and client_tools; input is optional for this continuation. Results-only without predecessor history are rejected by shared preparation. runs/get includes pending_client_tools and client_tool_continuation with durable state and visible history.

Integration: common ClientToolInput lives runtime/turn/client_tools.rs. Persistence service prepare/complete/load_by_call_id and immutable row session_id/source_run_id were coordinated directly with persistence/runtime writers. Runtime guarantees ClientToolsRequested is persisted then followed by terminal RunDone/RunDoneWithUsage. Parent owns exports and any external documentation/spec deltas.

Uncomfortable limitations: caller-tool SSE is buffered until manager termination, which can increase time to first event. Fresh request disconnect does not undo an already claimed continuation; interruption preserves claimed state and requires operator recovery, matching shared fail-closed semantics. ACP sessions remain process-local outside durable caller-tool batches. Deleting the process-local session does not cancel an already durable pending batch; authenticated result submission can restore its exact session. No executable evidence establishes compilation, provider behavior, restart, retry or protocol acceptance.

Cohort 11.1: after collecting caller-tool run events, ACP now awaits manager.await_settled_run_status and accepts only Done before completion/public response. Error, Cancelled or join failure becomes JSON-RPC error. No verification ran.

Cohort 11.1 durable cleanup acknowledgment: ACP calls confirm_handoff_for_run for each nonempty pending batch before public pending state/events/response. Same owner/tenant/agent/run scope; only actual settled Done yields the durable source_settled acknowledgment consumed by cold continuation admission. No verification ran.

## Cohort 13.16 selected agent and cold session recovery

Observed defect: runs/create ignored the agent selected by sessions/create and
always built the default artifact. Cold result submission also searched only the
default agent's batch scope. Selected-agent policy and routing therefore did not
survive either the fresh path or restart.

`src/uar/api/acp/handler.rs` now resolves the session's selected registered artifact
on every run/result submission. Public alias default maps to default-agent; an
explicit selected ID is never replaced by that default. Cold restoration derives
the batch locator from a host-issued call ID and loads the exact authenticated
owner, tenant, session and batch binding. It accepts only ACP rows containing that
call ID, restores the stored agent ID, then re-resolves its current artifact.
The existing exact-agent continuation lookup and full-history prepare/CAS remain
in force. Each request still receives the current UserContext and credential
capture; the durable row supplies identity/history, not frozen execution policy.

Added load_client_tool_continuation_for_session to the PersistenceLayer contract,
scoped facade/read classification, both provider implementations and their
client_tools helper modules. PostgreSQL filters the existing JSONB binding fields;
Surreal filters the authenticated owner's existing serialized rows and compares
all session/tenant/batch fields before selecting. Both reject multiple matches
and validate the selected durable row. No schema migration or row rewrite is
required; neither lookup accepts caller-selected agent authority.

Owned files: src/uar/api/acp/handler.rs; src/uar/persistence/mod.rs;
src/uar/persistence/scoped.rs; src/uar/persistence/providers/postgres.rs;
src/uar/persistence/providers/surreal.rs;
src/uar/persistence/providers/client_tools/postgres.rs;
src/uar/persistence/providers/client_tools/surreal.rs; this handoff.

No tests, checks, builds, formatting, imports, probes or runtime requests ran.
Source inspection and authorized writing only. Source behavior and backend query
compatibility remain unverified. The driver owner was notified to exercise the
selected mutable registered agent across actual cold restart without a caller
agent override; lead owns that gate. The uncomfortable limit is that Surreal's
legacy serialized representation requires an owner-scoped row scan for recovery;
this bounded repair does not introduce an index migration or alter durable rows.

### Cohort 13.16 review repair: deferred session publication

Cold restoration now keeps recovered session identity local until the exact-agent
lookup and shared full-history/schema/result prepare succeed. The validated
session is published for both Continue and exact Replay outcomes; rejection
publishes nothing. Restored defaults retain declarations/controls with history
cleared, while explicit submitted continuation history remains request-local.
A later fresh runs/create inheriting declaration defaults translates its input
into a fresh user history, so it executes the new prompt instead of replaying an
old result batch or discarding input. No persisted continuation receipt or retry
comparison was weakened. Only handler.rs and this handoff changed in this repair.
No verification ran; invalid-history publication and fresh-after-cold execution
remain required real-path observations for the lead-owned gate.
