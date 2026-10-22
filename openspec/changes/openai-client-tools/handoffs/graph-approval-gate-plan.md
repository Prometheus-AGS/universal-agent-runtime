# Real graph spawn approval servicing plan

Execute phase. Source inspection and documentation only; no executable changes,
execution, imports, tests, checks, builds, network requests or formatting ran.
Gate010's95 core,60 safety,5 cancellation and ordinary-cold results are lead
receipts; this document neither reruns nor independently certifies them.

The uncomfortable thing: the first warm graph OpenAI request timed out while a
real required spawn_agent approval waited unanswered. Automatically removing the
approval, preapproving the tool, changing artifact execution policy or replacing
the graph would change the behavior under test. Servicing that exact challenge
through the existing authenticated API is required. It does not repair the
separate graph child caller-tool continuation gap owned by the runtime writer.

## Inspected wire and source contracts

- `src/uar/api/routes.rs`: owner-authenticated GET
  `/api/uar/runs/{run_id}/tool-approval/pending` returns version/runId/pending.
  For an active run, pending is the flattened PendingApprovalView. The live
  snapshot in runtime/thread/approvals.rs is camelCase: rootRunId, approvalId,
  issuerId, challengeId, admissionId, admissionOwner, decisionOwner, callIndex,
  toolCallId, name, argumentsJson and riskReason. The view additionally exposes
  eventId and optional cursor. A read never creates an approval waiter.
- POST `/api/uar/runs/{run_id}/tool-approval` accepts exactly the needed
  `{ "approval_id": "<live approvalId>", "approved": true }`. These request
  keys are snake_case, unlike the snapshot. Use the same owner JWT and existing
  workspace header. Omit resume: this plan services a current live waiter, not
  recovered checkpoint execution. The live response must be HTTP200 with
  resolved=true and decision=allow; its record retains the decision receipt.
  HTTP200/resolved=false is not an approval success.
- Pending reads for a nonlive run can instead return a durable recovery wrapper
  with approvalId, record, resolvable/recoveryAvailable. That wrapper lacks the
  live argumentsJson proof needed here. Fail rather than infer destination or
  submit a recovered approval/resume using this helper.
- `runtime/graph/delegation.rs::execute` serializes the actual AgentSpawnRequest
  into the host admission gate before controls.spawn. AgentNode supplies
  artifact_id=general-purpose, delegated_prompt=current delegated input,
  task_name=None and default history_fork. `runtime/thread/spawn.rs` uses
  snake_case fields; default history_fork serializes as `{ "mode": "none" }`.
  Graph delegation separately checks spawn_agent and wait_agents policy.
- Owner GET `/api/uar/runs` returns RunInspection with actual run_id, agent_id,
  conversation_id and status. AG-UI's wire run ID is not a native root ID. The
  same API was used for the authored cancellation identity bridge; never derive
  a native approval root ID from a wire thread/run hash or message ID.

## Bounded helper around each actual graph request

1. Snapshot owner GET/runs immediately before the isolated graph request and
   retain its run IDs. Start the real native HTTP request in an owned executor;
   do not block the only coordinator in the whole-body HTTP reader while the
   approval waits. This changes orchestration of the request, not its payload,
   declared tools, authentication or selected orchestrator-agent route.
2. While that request is pending, poll the same owner's run list with a deadline
   shorter than the native request timeout. Select exactly one newly admitted
   root with agent_id=orchestrator-agent and pending/running/paused status. Record
   the baseline/current snapshots and actual run_id/conversation_id. No unrelated
   request may run for this graph agent in this isolated interval. Fail on zero
   at deadline or multiple candidates; never approve the first arbitrary run.
3. Poll that exact root's pending endpoint. Null means keep waiting within the
   deadline. Require body.runId and pending.rootRunId equal the selected native
   root; require live pending.name == spawn_agent; approvalId == challengeId;
   require a nonempty toolCallId and decode argumentsJson as a JSON object.
4. Require arguments exactly match the expected graph child intent:
   artifact_id=general-purpose; task_name=null; history_fork.mode=none; and
   delegated_prompt equals the expected current input supplied by this Driver
   invocation (including its unique acceptance marker). Reject unknown fields,
   another child, remote endpoint, changed prompt or unrelated tool. Do not
   approve a worker's host effect, wait_agents, another principal's challenge or
   a durable recovery wrapper under this helper.
5. POST the exact observed approval_id/approved=true to that root endpoint.
   Retain actual pending response, decoded expected synthetic arguments,
   request identity and POST response in the graph receipt. Require resolved=true
   and decision=allow; retain challenge/tool-call/issuer/admission identifiers.
   Approve each challenge at most once. A second unexpected spawn challenge
   fails the scenario rather than extending authority silently.
6. Await the original native request and stop the poller when it completes.
   Use an owned bounded helper/executor lifetime with a shared stop event. In
   finally set stop, cancel this exact admitted root through the real owner
   cancel route only if its request/helper failed while still active, and join
   every request/helper future before leaving the case. No daemon/background
   poller may remain. A polling timeout is a failure, not implicit permission.
   Preserve the original failure alongside cancellation/join receipts.

No helper changes require granting credentials, bypassing Cedar, changing tool
approval policy, editing the pending record or directly resolving a broker. The
only approval mutation is the operator-authorized exact native POST above.

## Graph acceptance retained after servicing approval

Keep fixture routing exact general-purpose and identify nodes from system
prompts, never user text. Require actual provider sequence router/worker/worker
and one real router request total. Retain both worker request traces, caller
handoff/result IDs, selected worker marker, exact continuation history and saved
graph identity. The initial handoff must follow the one expected spawn approval;
resumed work must not replay routing. Per the subsequent lead decision below,
its fresh governed root requires its own exact spawn approval. Completed retries
must not admit a root, spawn work or create approval decisions. Preserve the
initial challenge receipt with each protocol's graph seed.

Cold graph mode must keep the same provider/MCP peer, config/store/JWTs and actual
supervisor stop/join/restart evidence. Preserve initial approval receipt across
seed/resume. Cold continuation's fresh governed root receives exactly one fresh
expected spawn approval, never the initial grant. Exact completed retry still
requires zero new roots, provider contacts, real host effects and approval decisions.

If runtime's separately planned child caller-tool repair changes the legitimate
graph continuation contract, reconcile that contract with this plan before
authoring the helper. Passing the spawn approval alone proves neither child
caller-tool handoff propagation nor warm/cold cursor preservation.

## Authoring boundary

Proposed driver scope: a graph-only owned request/approval coordinator, strict
live-snapshot selection, receipt storage through restart state, and graph
assertions above. Supervisor scope: unchanged actual selected-agent provisioning,
isolated ownership and lifecycle receipts; no preapproval shortcut. The lead
owns implementation assignment, whole-phase inventory and the exact gate. This
document requests no execution and makes no unverified success claim.

## 13.22 lead decision and source implementation

The lead selected a fresh governed root for each warm/cold continuation. Returning
to the saved AgentNode cursor skips router replay but requires fresh spawn_agent
approval for that new root. The old approval is evidence, never its authority.
This supersedes earlier plan statements forbidding any approval on continuation.
Router/worker/worker remains the required provider sequence. Completed exact
retries require zero new roots/approvals/provider calls/real MCP effects.

Changed only `scripts/release/client_tools_acceptance.py` and this handoff. Added
graph_send, used for initial and resumed graph requests and for completed retry.
The coordinator snapshots owner runs, owns one request worker, and polls on its
calling thread for exactly one new pending/running/paused orchestrator-agent root.
Its90-second coordinator deadline uses10-second owner HTTP polling/decision
timeouts; existing native requests retain their180-second timeout. All polling
stops when the request completes; no daemon or detached poller exists.

Live challenges require exact root, spawn_agent name, matching approval/challenge
IDs, nonempty tool-call ID, and exact serialized AgentSpawnRequest: general-purpose,
latest supplied user text, null task name and history_fork mode none. The helper
POSTs only the observed approval_id/approved=true and requires resolved=true and
decision=allow. Prior roots/approval IDs cannot grant a new continuation. At most
one challenge is serviced per request. Unexpected/durable-wrapper requests fail.

Failure cleanup cancels only the identified owned root, records cleanup responses,
and joins the request future/executor, preserving the original failure. EVIDENCE
retains baseline/root/pending/decision/cleanup observations even when a case fails.
No state/config preapproval or broker bypass was introduced.

Driver state retains graph_approval_receipts/observations across cold restart.
Warm and cold completion require two distinct actual approval decisions in total
(initial plus continuation); replay is wrapped with allow_spawn=false and checks
no new root or real host effect. Original provider contact/replay checks and
router/worker/worker assertions remain. Graph receipts are included in warm/cold
reports. No fixture, supervisor or runtime production file changed in this task.

All new Python orchestration, approval schema matching, cleanup and warm/cold
graph behavior remain unverified. No execution, imports, tests, checks, builds,
network requests or formatting ran. Runtime's child continuation implementation
is owned separately and must complete before the lead's concrete gate.

## 13.22 source critic repair: native interrupt attachment

The isolated source critic found a blocker: completion of an HTTP future is not
completion of a graph operation. AG-UI seals an approval interrupt and closes its
SSE; A2A can return INPUT_REQUIRED for approval before caller tools are available.
The original helper could stop before servicing the challenge or return that
sealed interrupt after approving it. This paragraph supersedes the earlier
generic-POST-only orchestration for AG-UI.

graph_send now continues after a closed native approval response. It distinguishes
AG-UI RUN_FINISHED interrupt and A2A active/INPUT_REQUIRED without the actual caller
extension pending_calls from final caller handoff/completion. The same deadline,
root/challenge/argument binding and at-most-one decision still apply.

For AG-UI, it waits for and retains the original sealed interrupt, requiring the
same thread/wire IDs and exactly one tool_approval interrupt matching the live
challenge. Instead of generic POST first, it submits the real native resume to
the same agent with same threadId, fresh wire runId, parentRunId=sealed wire,
messages=[], tools=[], forwardedProps.uar={}, and exactly one resume entry:
interruptId=observed approvalId, status=resolved, payload.approved=true. agui.rs
validates that live challenge, chooses warm=true and the predecessor's same
native_run_id, and invokes resolve_host_approval. No original input/tools or new
authority resources are sent. The helper owns and joins both HTTP futures.

The actual resumed native response and owner approval-record GET are retained;
the exact challenge must be state=approved with decision.approved=true. Final
owner run inspection requires the one original physical orchestrator root, not
a new root created by approval resume. No resolved=true receipt is invented for
this protocol.

Driver.run advances to the approved projection wire ID for the next caller-result
parent. Original agui_requests submission identity/digest remains unchanged;
persisted agui_graph_projections maps history to its approved projection. An
actual completed retry first POSTs the identical original submission, then GETs
the exact `/api/uar/ag-ui/agents/{agent}/threads/{thread}/runs/{projection}/stream`
if that original receipt is sealed at approval. This retrieves the final native
projection without resending original input or inventing a fresh run. The helper
graph_completed_retry explicitly wraps this path and retains duplicate/attach
responses; no new root/approval/provider/MCP effects remain required.

For A2A, exact generic approval POST remains. The helper then polls actual GetTask
on the same task ID and context, preserving both, until the actual caller-tools
extension appears or a genuine completed/failed/canceled native terminal arrives.
It never sends SendMessage again to retrieve post-approval output. Task reads
and final native response are retained; failed tasks reach existing failure
assertions rather than becoming successful handoffs.

These repairs changed the acceptance driver and this handoff only. Existing
router/worker/worker assertions, fresh-root initial/continuation approvals and
zero-work completed retries remain. No execution, imports, tests, checks, builds,
network requests or formatting ran; all new native handling remains unverified.

## Gate011 redacted public intent: source-only amendment proposal

Gate011 reached each graph approval but the driver compared public argumentsJson
against AgentSpawnRequest. The actual public value was exactly
`{"operation":"spawn_agent","detailsAvailable":false}`. This is intentional
presentation, not evidence of a malformed host invocation.

Source chain: runtime/tool_admission/standalone.rs::prepare builds action_display
from operation=invocation.provider_tool_name/detailsAvailable=false, exposing a
target only for file_read. Graph delegation passes host.action_display into its
ToolApprovalRequest. runtime/thread/approvals/recovery.rs::request_invocation
passes request.action_display.to_string() into the public approval snapshot's
arguments_json. Actual validated spawn arguments remain in PreparedToolInvocation
and are bound by the host's invocation/admission seam, not exposed in this string.

No inspected public API supplies a pre-decision full AgentSpawnRequest. Durable
approval records and tool-admission evidence are deliberately content-free. Do
not change production redaction, treat the public display as execution authority,
invent a full-intent API, or claim the helper verified hidden delegated_prompt.

Proposed driver adaptation is restricted to this isolated installed graph:

1. Keep unique new owner-visible orchestrator-agent root, same live rootRunId,
   exact spawn_agent name, approvalId/challengeId/toolCallId checks and no reused
   root/grant. Decode argumentsJson and accept only the exact source-defined
   two-key redacted object above (boolean false, no extra fields). Retain the
   exact redacted receipt and label details unavailable. Do not generalize this
   exception to other tools, arbitrary agent graphs or opaque display shapes.
2. Before deciding, require the peer's configured graph scenario and the one
   actual router request for this unique conversation marker, with the exact
   configured router system identity and no tools. The controlled peer's graph
   produce path returns exactly general-purpose for that request; the installed
   defaults.rs graph condition chooses that named specialist. Retain router trace
   and fixture settings as the source-bound controlled selection evidence. A
   continuation root has no new router request: bind to the saved initial router
   receipt/unique marker and require provider sequence still router/worker/worker.
3. Service the same exact live challenge through the existing native/generic
   approval path already authored. Keep fresh approval per fresh governed root
   and zero root/decision/provider/MCP work on completed retry. No preapproval or
   policy edit is introduced.
4. After approval and before crediting caller handoff/completion, GET the actual
   owner `/api/uar/runs/{root}/events` snapshot. Its inspected public run_events.rs
   contract returns version, runId, retention/cursor/gapReason and events with
   eventName/data. sse.rs projects AgentThread* as agui.subagent.* with
   data.request_id=root and data.lifecycle containing artifact_id,
   root_thread_id,parent_thread_id,child_thread_id,parent_run_id,child_run_id,
   canonical_path,status,revision/lifecycle_id. Require matching root identity,
   no retention gap, actual lifecycle artifact_id=general-purpose and
   parent_run_id=root, nonempty child IDs, and no unexpected specialist child.
   Retain actual child/root IDs and lifecycle receipt. This is a public production
   read, not a private thread/manager callback.
5. Require the actual worker provider request to carry the configured unique
   general-purpose system marker and selected conversation marker. Bind it to
   the approved root's child lifecycle receipt and preserve ordered warm/cold
   router/worker/worker assertions and exact canonical result history. A wrong
   child/worker or missing lifecycle receipt fails the gate after the decision;
   it cannot be described as successful target validation.

The uncomfortable limit is explicit: the public challenge does not reveal the
full target before approval. This proposal relies on the operator-authorized
isolated deterministic router/graph fixture before the decision and actual
public child identity/worker trace afterward. It is not a general approval bot
for private intent. If that bounded evidence is unacceptable, the gate must stop
instead of bypassing the intentional redaction.

This amendment is a plan only. No executable source changed for Gate011 at this
point. No execution, imports, tests, checks, builds, network or formatting ran.
Await the lead's agreement before implementing the narrowed public-contract
matcher and post-approval child/worker evidence.

## 13.23 approved source implementation

Implemented the lead-approved Gate011 amendment in the acceptance driver only;
production redaction and binary sources remain unchanged. The public matcher now
accepts only a JSON object whose exact keys are operation/detailsAvailable, with
operation=spawn_agent and the actual boolean false. Raw AgentSpawnRequest DTOs,
extra fields and alternate opaque displays are refused. Receipt fields explicitly
record public_display and hidden_arguments_verified=false.

Before each decision, graph_selection_receipt requires the actual peer to remain
in the configured graph scenario, matching configured router/worker markers. It
requires exactly one tool-free router request with the current unique conversation
marker and distinct router system identity. The saved controlled router request,
fixture settings and source-defined general-purpose selection are persisted in
Driver.graph_selection and must remain identical across warm/cold continuation.
This is controlled fixture provenance, not a claim that hidden arguments became
public. Exact owner root/name/challenge binding and fresh approvals remain.

After each decision, graph_child_receipt reads the actual public owner run events.
It requires matching root and no retention gap, rejects unexpected subagent
artifact/parent identity, and requires one general-purpose child thread with one
actual child run ID. Initial pending lifecycle events may legitimately have no
run ID; a later real run identity is required before accepting the receipt. The
same root must have exactly one post-decision provider contact with the selected
worker system marker and conversation marker. Actual lifecycle events, child IDs
and worker request are retained in the approval receipt. Existing total
router/worker/worker and zero-work completed replay assertions remain intact.

Added graph_owned_request to scope an absolute deadline to each owned native HTTP
worker. In that scope, the existing HTTP helper reads actual response chunks with
HTTPResponse.read1, checks the absolute EOF deadline between chunks and caps each
socket I/O wait at10 seconds. Thus SSE keepalives cannot indefinitely extend the
owned request; a missed EOF deadline fails, closes the response context, cancels
the identified root and joins all request/executor work. Actual source of the
standard-library HTTPResponse.read1 was inspected without importing/executing it.
AG-UI completed projection attach also carries the absolute deadline. Non-graph
HTTP behavior is unchanged when no scoped deadline exists.

The actual public render, controlled router selection and post-decision identity
form the bounded evidence for this local deterministic graph. No claim of hidden
prompt preverification, arbitrary-graph approval safety or general private-intent
authorization is made. Only the acceptance driver and this handoff changed.

No execution, imports, tests, checks, builds, network requests or formatting ran.
All authored public-contract matching, lifecycle/worker evidence, deadline and
native warm/cold behavior remain unverified until the lead's concrete gate012
after fresh isolated source review.

## 13.23 corrective source span: HTTP error EOF deadline

The source critic identified that HTTPError still used an unbounded read even
when the successful response path carried an absolute graph EOF deadline. A
trickling error body could therefore outlive the coordinator and delay its owned
executor join.

Added read_http_body and shared its deadline-bound read1 loop across successful
and HTTPError graph responses. The error response is closed through its context
manager even when the EOF deadline or socket read fails. Without a scoped graph
deadline, the original response.read().decode() and error.read().decode() paths
remain unchanged. No other helper, scenario or production source changed.

Source inspection confirmed HTTPError inherits urllib.response.addinfourl,
whose addbase inherits tempfile._TemporaryFileWrapper; that wrapper delegates
read1 to the underlying HTTPResponse. Its context manager closes the response.
The inspected standard-library files were read without importing/executing them.

Source inspection also confirms root registration produces no subagent event:
runtime/thread/service.rs::publish calls lifecycle_event only for Some(parent)
and uses None for a root. The existing public child matcher therefore needs no
root-artifact exclusion change; no matcher change was made in this correction.

Only the acceptance driver and this handoff changed. No execution, imports,
tests, checks, builds, network requests or formatting ran. The lead owns gate012
and its inventory; the current production binary is unaffected by this source
repair. Runtime deadline behavior remains unverified here.
