# Explicit cancellation and mixed crash recovery acceptance plan

Execute phase; source inspection/planning only. No executable files changed.
No execution, imports, tests, checks, builds, formatting or network requests ran.
The lead owns any implementation assignment and the authorized functional loop.

The uncomfortable thing: the existing closed-socket scenario is a disconnect
claim/retry test, not explicit cancellation. Existing ordinary cold continuation
uses graceful shutdown and single caller tools; it does not establish mixed host
effect preservation after a process crash. These claims must stay separate.

## Inspected actual cancellation contracts

`src/uar/api/routes.rs` mounts POST `/api/uar/runs/{run_id}/cancel` and POST
`/api/uar/sessions/{session_id}/cancel`. Neither consumes a request body. Both
require the verified UserContext; the run route returns404 with cancelled=false
for an inaccessible/absent run and otherwise returns cancelled boolean. Session
cancel resolves the current run through the complete owner/tenant session map
in manager.rs::cancel_session_run_for_context. Require cancelled=true for these
cases; successful HTTP status alone does not establish a cancellation occurred.

| Interface under test | Exact identity and cancellation path |
| --- | --- |
| OpenAI caller tools | openai/routes.rs emits first SSE chunk with id `chatcmpl-{run_id}` before consuming provider output. Strip only that source-defined prefix; POST the real run cancel route. The mounted managed-chat stream also exposes x-uar-run-id/session-id, but caller-tool routes must not be assumed to expose those headers. |
| Anthropic bound caller tools | server.rs::AnthropicMessagesRequest accepts session_id; api_bound_anthropic_messages assigns it to RunExecutionRequest. For the cancellation test, capture a fresh request and supply a unique explicit session_id before initial submission, retaining it through handoff/result submission. message_start has a msg UUID rather than a native run ID. Cancel through the real session route; do not decode a message ID into a run ID. |
| AG-UI | agui.rs emits RUN_STARTED with wire threadId/runId. Its native run is a separate UUID persisted in AguiRunBinding; wire runId cannot be sent to the native run cancel route. There is no inspected native AG-UI cancel route. Snapshot owner GET /api/uar/runs before the one isolated held continuation, then read again and require exactly one new active run for the selected agent. RunInspection exposes run_id, agent_id, conversation_id and status. Use that actual returned run_id and retain both snapshots and the wire IDs. Fail on ambiguity; never guess the binding hash or treat wire ID as native. |
| ACP | No runs/cancel RPC is present in handler.rs dispatch. sessions/create returns the actual session_id, and handle_runs_create assigns that ID to RunExecutionRequest. POST the real host session cancel route while runs/submit_tool_results is outstanding. routes.rs awaits caller-tool dispatch before serializing its buffered SSE, so waiting for its first SSE event before cancellation would deadlock against the held provider. Use the already captured session ID, then inspect the eventual buffered terminal. |
| A2A1.0 used by current driver | At the same `/a2a/v1/agents/{agent}` route, JSON-RPC CancelTask params `{ "id": actual_task_id }`, with A2A-Version:1.0 and existing workspace headers. v1.rs::TaskParams accepts id and optional historyLength/tenant. v1.rs invokes cancel_durable and projects a native task. GetTask uses the same id. Do not send legacy tasks/cancel to this v1 route; legacy handler.rs has a distinct schema and full-harness workspace/revision requirements. |

AG-UI is the only case needing owner-visible host inspection to bridge identities;
the inspected wire surface does not expose its native cancellation ID. This is a
bounded read through the production API, not a private store/manager callback.
The isolated gate must prohibit concurrent requests for that agent during it.

## Small explicit-cancel scenario for each interface

1. Use the real controlled peer and ordinary selected registered agent. Run the
   existing `mixed` fixture to a confirmed caller handoff, retaining its native
   identity/history and exactly one actual MCP host effect. This establishes a
   real prior effect whose repetition would be observable. Append exact caller
   results manually, as the existing driver does.
2. Snapshot provider/effect counts and (AG-UI only) owner-visible runs. Hold the
   provider with existing POST /_hold, then send the captured actual native result
   submission on a live HTTP connection. Keep reading SSE incrementally where
   available; never use the current whole-body reader as the only cancellation
   coordinator. Wait for the peer's held_requests receipt, proving admission and
   actual upstream contact before cancelling.
3. Resolve the exact identifier as above and issue cancellation concurrently with
   the open native request, retaining the complete authenticated cancel response.
   Do not close that native socket to manufacture cancellation. The host routes
   return a positive cancellation receipt; A2A CancelTask must yield or reconcile
   to TASK_STATE_CANCELED with no cleanup_unconfirmed flag. If cancellation waits
   on cleanup, keep its future owned and release the provider in finally under a
   bounded deadline; release cannot count as cancellation evidence.
4. Read the native result to a terminal and inspect the owner run where available.
   Require an explicit cancellation/failure terminal and no success terminal:
   OpenAI run_cancelled error and no stop/tool_calls finish reason (DONE alone is
   not success); Anthropic error and no end_turn/tool_use stop; AG-UI cancelled
   RUN_FINISHED or RUN_ERROR, never success; ACP error status with retained
   Cancelled/Error normalized event, never done/client_tools_required; A2A final
   TASK_STATE_CANCELED with cleanup closure, never TASK_STATE_COMPLETED. Retain the
   actual native transcript, not a synthesized success=false flag.
5. Release/join any held HTTP work in finally. Retry the exact claimed caller
   batch once and require failure or a durable saved cancellation/failure
   projection with no additional provider contact and no additional MCP effect.
   If the current adapter cannot produce that behavior, record the observed gap;
   do not reinterpret disconnection or a cleanup timeout as explicit cancellation.

Source bindings for terminals: openai/routes.rs handles NormalizedEvent::Cancelled
as run_cancelled error; server.rs bound Anthropic records cancellation as failure;
agui.rs::reconcile maps Cancelled to outcome.type=cancelled; ACP handler's collected
caller-tool events select status=error for Error/Cancelled; A2A v1 CancelTask uses
thread_service.rs::cancel_durable and confirms cleanup before cancellation success.

## Mixed process-crash seed/resume without repeating prior host effects

Reuse restart_case serialization and exact credential/store/session identity,
adding narrowly named mixed-crash seed/resume modes or an explicit mixed-crash
variant to the existing restart modes. Do not relabel graceful restart receipts.

Seed: configure the real peer to mixed once, without resetting it between five
interfaces. For each interface, reach the confirmed public caller handoff after
one real host effect; capture exact native history, declarations and result batch
using the existing Driver state serialization. Record per-interface pending IDs,
provider counts, MCP effect sequence/arguments and cumulative counts. Require five
actual host effects total and source cleanup-confirmed handoffs before announcing
seed ready. Preserve the same peer, port, JWTs, configuration, store and protected
restart-state file. No additional host effects should remain active at this point.

Crash supervisor action: intentionally SIGKILL only its owned candidate PID after
seed-ready, wait/join that exact child and retain the actual signal/returncode,
PID/generation and timestamp receipt. Name the action expected_crash_for_recovery.
Do not call the normal graceful stop helper and waive its clean-exit assertion;
do not call this cleanup success. Starting a successor requires the predecessor's
confirmed OS process join. The peer remains alive; final post-gate cleanup still
requires normal clean termination/join of the successor and peer.

Resume: restart the identical prebuilt candidate with same config/store/JWTs and
peer. Submit each exact saved result batch once; require native completion, one
resumed provider call per protocol, unchanged prior MCP effect counts and actual
resumed provider history containing the retained host result and caller result.
Then retry each completed submission and require replay with zero provider/MCP
effects. Preserve per-protocol histories and counts across the crash and restart;
do not reset _control or rewrite runtime continuation/ledger rows.

The existing mixed peer emits its host call only before the initial handoff and
returns final text when caller results arrive. Canonical runtime history must
retain that already completed host exchange; caller-visible history alone cannot
prove it. Compare the actual resumed provider request against saved pre-crash
host tool-call IDs/results plus the exact submitted caller results.

Scope limit: crashing after confirmed durable public handoffs proves recovery of
those mixed continuations without repeating previously settled effects. It does
not prove arbitrary crash-mid-effect exactly-once execution or resolve unknown
external side effects after an interrupted commit. Do not claim that stronger
property from this gate.

## Proposed artifacts and implementation ownership

Driver owner: incremental native HTTP cancellation coordination/receipts and
mixed-crash state variants only. Fixture owner: existing mixed/_hold/_release and
real MCP counts suffice; additional receipts only if needed, no internal manager
hooks. Supervisor owner: expected SIGKILL/join receipt, preserved peer and store,
successor launch/readiness and final graceful joins. Lead owns exact gate command,
whole-phase ledger reconciliation and approval; this proposal runs nothing.

## 13.18 source implementation handoff

The lead subsequently authorized driver authoring from this proposal. Changed
only `scripts/release/client_tools_acceptance.py` and this handoff; the existing
peer's mixed/_hold/_release/trace/effect surface already supplies these scenarios.
The fixture, supervisor and production runtime were not changed by this writer.

New `--mode cancellation` records five independent native-interface cases and
requires the existing actual `--host-tool-json`. Each settles one real mixed host
effect, captures its exact native result-submission tuple, snapshots owner runs,
holds the provider, and starts one owned HTTP request in a one-worker executor.
A2A uses actual CancelTask/GetTask on its retained task ID. Other interfaces read
the actual owner GET/runs delta and require exactly one new active selected-agent
run before POSTing that native run's cancel endpoint. No wire/native ID is guessed.

The provider hold is released in finally, including request-worker creation or
cancellation failures. The request future is joined under the executor lifetime;
the executor cannot escape the case as background work. No _control reset erases
the receipts. Cancellation requires explicit positive cancel receipt, native
cancelled status/task cleanup, absence of success terminals, one continuation
provider contact, unchanged settled host-effect count and an exact retry with no
new provider/MCP effects. It never uses socket disconnect as cancellation evidence.

New `--mode mixed-crash-seed` and `--mode mixed-crash-resume` each emit one aggregate
record and use the existing --restart-state/--instance-identity plus actual host
tool/model/auth inputs. Seed configures mixed exactly once, retains one settled
host effect per protocol and its actual ordered provider host call/result, then
saves exact pending Driver states. Aggregate observed fields are seed_ready=true,
pending_batches=5 and mixed_host_effects=5 for all five protocols, plus complete
host_effect_receipts. Private state adds mode=mixed-crash, host_effect_receipts,
and each driver's mixed_host_exchange (exact call/result and source indexes).

The supervisor must deliberately SIGKILL and OS-join its exact owned candidate,
retain peer/config/store/JWT/state, then start the successor. Resume does not
reset peer control or traces. It requires unchanged effect receipts, exactly one
new provider request per protocol, the selected conversation marker, the exact
settled host assistant-call/result before every submitted caller result, native
completion and exact completed retry without any provider/MCP reexecution.
Ordinary cold ACP's extra fresh-run regression is excluded from mixed-crash mode;
all original95 matrix,60 safety, graph and budget paths remain in place.

All new behavior remains unexecuted and unverified. No test, import, check, build,
network request or formatting command ran. SIGKILL lifecycle evidence belongs to
the supervisor; a changed instance-identity string alone cannot prove a crash.
The gate's crash boundary remains after confirmed durable public handoff and does
not claim arbitrary crash-mid-effect exactly-once execution.

## 13.20 A2A read diagnostics and failed native terminal distinction

Gate009 reported generic A2A stream-terminal read assertions in two upstream
negative scenarios. The runtime owner identified actual successful GetTask reads
returning TASK_STATE_FAILED. Driver.send previously applied failed() to the read
itself, confusing a legitimate failed native task with an HTTP/JSON-RPC failure.

The driver now requires a successful HTTP status, no JSON-RPC error and a task
result object for GetTask reads. It returns failed/canceled native terminal tasks
to the existing scenario assertions, preserving their explicit failure and
no-success requirements. HTTP/JSON-RPC failures still raise read assertions.

Added bounded diagnostics to initial-snapshot and task-read assertions: HTTP
status, integer JSON-RPC error code/message-presence boolean, and allowlisted task
state. Free-form error messages, payloads, metadata and credentials are omitted.
Only the acceptance driver and this handoff changed. No execution, imports,
tests, checks, builds, formatting or network requests ran; the repair remains
unverified until the lead's authorized gate.
