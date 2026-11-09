# Native chat turn and history recovery

Source implementation for production-readiness-476 P05. Runtime qualification is
pending the one final operator-approved batch for all issues462–475. The
verification gate is CLOSED; this document is not an acceptance receipt.

The uncomfortable case is a lost response after the host committed admission.
The browser cannot tell an unreceived request from an admitted request. An unknown
receipt never means that execution is safe to repeat. Recovery can remain
unconfirmed indefinitely; there is no global exactly-once external-effect claim.

## Admission and owner boundary

The shipped client persists `client_turn_id`, `user_message_id`, session identity,
and the full supported user content before POST. A local persistence failure
retains a visible recoverable draft and sends no request. The client never sends
an owner assertion. The ordinary native chat request accepts the two stable IDs
additively; the legacy request without them retains its existing contract and has
no durable turn recovery guarantee.

The host calls `chat_turns::reserve_native_chat_turn` only after it captured the
middleware-verified principal and resolved the session and workspace. It binds
the original client-requested semantic input/configuration to the reservation
digest, including the original user message ID and typed image parts. A secret value is
not stored in the reservation. Object property ordering is canonicalized; array
ordering remains meaningful. Changes to client-requested policy, model,
attachments, stream format, content or other caller settings under the same
intent ID conflict. Changing the original user message ID also conflicts.
Dynamic host settings, resolved catalogs and artifact state are not re-evaluated
as a new intent digest on retry. An identical client-intent retry returns the
original execution choices and receipt even after later host settings change;
it never dispatches a newly resolved run.

The helper requires the existing durable AG-UI capability plus an eligible durable
store. It uses the existing create/CAS store and owner partition, with reserved
protocol agent namespace `native-chat-turn/v1`. AG-UI rejects public agent IDs
starting `native-chat-turn/`; callers cannot enter this namespace. Each native
turn has its own context key derived from session and client turn ID, so it does
not participate in AG-UI thread predecessor chains. Existing native protocol and
AG-UI reservations do not share execution authority.

An identical existing intent is read before mutable catalog/settings resolution,
so removing or changing the original target cannot silently create a new run or
block its receipt behind fresh target admission. This read applies the same
owner/session/workspace/message/digest fences as reservation. Only a new intent
continues through ordinary validation and then creates its durable reservation.

Only `NativeChatReservation::New` permits the caller to dispatch the preallocated
`native_run_id` through the ordinary governed admission path. An existing record
returns the same safe receipt. A failed or uncertain create/read never permits a
fresh dispatch. Reservations remain held after an unconfirmed admission and do
not expire into automatic retry. The receipt does not store input or credentials.

## Read-only recovery

`GET /api/uar/chat-turns/{client_turn_id}?session_id=...` returns the original
`run_id`, `user_message_id`, `session_id`, `client_turn_id`, `state`,
`recovery: "available"`, and `retry_allowed: false` for an owned reservation.
States are `pending`, `running`, `paused`, `complete`, `error`, `cancelled`,
`interrupted`, or `admission_unconfirmed`. A durable nonterminal run with no
resident host lane projects `interrupted`; it does not claim live execution.
The original run must also belong to the same session.

No reservation returns `state: "unknown"` and `retry_allowed: false`. Unsupported
or unavailable persistence returns503 and `recovery: "unavailable"`. An absent
verified principal returns401. There is no anonymous-owner fallback, credential
minting, authorization relaxation, or dispatch operation in this route. Identical
POST recovery uses the same receipt shape. A deliberate new user intent uses a
new turn ID; an uncertain old intent remains visible.

## Canonical history and presentation

`SessionState.message_metadata` is an additive serde-default map keyed by canonical
message position. Both Surreal and PostgreSQL serialize the existing Session JSON;
old rows need no destructive migration. The canonical message vector, tool
messages, pending tool calls, and typed `MessageContent` remain intact.

Metadata stores the original display `message_id`, optional `run_id` and
`client_turn_id`, original persisted timestamp, and actual host status. A run-local
Session handle gives appends their run correlation without changing another run's
handle. Explicit original user metadata overrides generated IDs. Assistant IDs
are deterministic per run and assistant ordinal. Redaction can replace
same-length content while retaining IDs. Protected replacement history copies
metadata from its same-owner, same-session source.

A live run bubble can aggregate several tool-loop steps. On canonical reload it
is reconciled to the original assistant ordinals: an intermediate preamble or
tool-only segment and its later final answer remain separate canonical messages
in server order. A cached row with the same run ID does not stand in for every
message in that run. Warm and empty caches must preserve the same available final
answer exactly once. Local nontext evidence remains attached to its original
message; run-wide approval/usage cards are projected once. These source changes
and regression cases remain unverified until the whole-phase acceptance gate.

`GET /api/uar/sessions/{id}` preserves the existing response fields and adds:

- `history_version: 1` and `history`: user/assistant rows with typed `content`,
  `message_id`, `run_id`, `client_turn_id`, `created_at`, `status`, `provenance`.
- `history_runs`: exact owned run evidence with actual status, public citation
  and RAG citation data, reported usage, a safe scalar policy summary, and
  read-only approval decisions. No raw run context, host directory, credentials,
  tool arguments, or protected approval continuation is copied into this view.
- `history_runs_availability`: `complete` or `truncated`. Evidence is projected for
  the most recent64 distinct correlated runs, at most8192 persisted events/run.
  Partial text reconstruction is capped at1MiB. Truncation is disclosed; it is
  never treated as proof that no earlier evidence exists.

History statuses are `unknown`, `pending`, `running`, `complete`, `error`,
`cancelled`, or `interrupted`. Host run status overrides saved presentation status
only after exact owner and session qualification. If that evidence is absent,
status and provenance are unavailable rather than fabricated success.

A legacy row without metadata gets a deterministic ID from its owner partition,
session and position, `created_at: null`, `status: "unknown"`, and
`provenance: "unavailable"`. No random ID or reload-time timestamp is invented.
Available typed images remain in the original content. A run-event partial with
no canonical assistant row is recovered from the published `ChatDelta` sequence,
with deterministic first-assistant identity, no invented timestamp, and
`provenance: "run_events"`. Truncated event histories are not synthesized into a
supposedly complete partial answer. Canonical model history is never rewritten by
this read projection.

Sources, usage, scalar policy and approvals have explicit availability markers;
absence is not a billed zero or invented provenance. Approval history includes
only the actual durable challenge state and decision, and always
`historical: true`, `resolvable: false`. A historical card grants no permission.
Current pending authorization continues through the existing host approval path.

## Local interrupted-run reconciliation

The browser's persisted `run.status = interrupted` records a lost transport with
an unconfirmed outcome. Reattaching to the same run, replaying content, or a local
stream finalizer cannot promote that row to running or completed. The original
native run identity remains unchanged.

An official `RUN_FINISHED` or `RUN_ERROR` frame can settle the interruption once.
The stream persistence writer binds to the response/reattach native run ID (or
the first explicit official identity when no response ID is available) and
refuses a conflicting wire or normalized run ID before writing. The repository
requires the exact persisted terminal event ID, matching native run ID and
profile, and matching outcome: finished, error, or `CANCELLED`. This transition
replaces the provisional interruption timestamp with the terminal observation
time and retains the terminal's available usage/cost and phase timings. The time
is the browser's observation time; it is not an invented host completion time.
Absent error/cancellation usage remains unavailable. Native `RUN_ERROR` with
`code: "CANCELLED"` carries optional reported usage in `result.usage`; nullable
counters and separately available cost/cache fields are retained without filling
missing values with zero.

Once finished, error, or cancelled is persisted, later terminals, replay, and a
stale writer's transport interruption cannot overwrite its status or metadata.
Local finalization can only move a running row to interrupted; it carries no
native evidence and cannot repair an interrupted row. This applies to observer
AbortError (including Stop), retry preparation, legacy terminal fallbacks, and a
terminal whose local event persistence failed. A best-effort cancellation request
is intent, not confirmation of native cancellation. The visible partial message
on observer abort is interrupted, so later exact native success, error, or
cancellation can reconcile the persisted summary. The repository requires the
same exact persisted native proof even when the previous row is running; a weak
caller cannot freeze an authoritative outcome before reattachment. Persistence failures keep
their existing error reporting and never become proof of successful saving.
These source changes and real-PGlite regression sources still require the
whole-phase gate; no database, timing, reconnect, or browser result is claimed.

## Native cancellation and protocol limits

The shipped browser POST selects `stream_mode: "agui_spec"`; reconnect and action
continuation use `/api/uar/runs/{runId}/stream?stream_mode=agui_spec`. Its native
`uar.agui/1` adapter emits cancellation as `RUN_ERROR` / `CANCELLED`, distinct
from `RUN_FINISHED` success and ordinary error. Cancellation finalizes messages
as cancelled and rejects continuation completion, so action feedback cannot
advance to “Response complete” through that callback. The original payload,
event ID, sequence, phase timings, and available usage/cost stay in local run
persistence. A missing report remains unavailable.

The existing message and UsageChunk display contracts require complete numeric
input/output/total counters. Partial cancellation reports are therefore retained
in persisted run usage and the original event payload rather than displayed as
fabricated zeros. Fully reported counters, including actual zeros, can appear in
the usage display; partial reports remain available as raw cancellation evidence.

`src/uar/api/agui.rs` serves a separate standard AG-UI binding protocol. Its sealed
`RUN_FINISHED` / `outcome.type=cancelled` payload has no `uar.agui/1` profile,
profile event ID or numeric sequence, and its opaque SSE cursor differs from the
native run stream. The shipped native adapter rejects that payload. This source
repair does not add standard-protocol ingress or claim that its cancellation is
consumed by this browser path. No profiled cancellation terminal is invented to
stand in for that distinct wire contract.

## Final evidence still required

Author and retain actual production-path evidence for: lost pre-header response;
concurrent same-ID submission; same-ID changed content/message/config conflicts;
restart and reconnect to the original run; foreign owner/session refusal; no-owner
and unsupported-store recovery; local persistence failure/noPOST; supported image
restoration; partial/cancel/error/success restoration without a later turn;
explicit source truncation; answered approval reload without another effect; and
current pending approval behavior. Exact original requests/runs/effects must be
counted. Existing canonical-history and pending-tool assertions remain unchanged.

Functional scenarios use the externally provisioned identity endpoint and no
credentials. No command in this document authorizes a test, build, service action,
external mutation, or acceptance gate. Lead must complete all source packages,
record the whole-phase inventory, and obtain the operator's named final commands.

## Stop intent and original message identity

The actual `useChatRuntime` Stop callback requests server cancellation, aborts the
observer, and saves the partial as `interrupted`. A failed or delayed cancellation
request has the same provisional presentation. Request dispatch and AbortError
supply no native terminal proof; neither authorizes `cancelled`.

Reattachment binds existing interrupted assistant bubbles by exact original run
ID within the original thread. Terminal-only success, error, or cancellation can
settle those same message IDs without requiring another content delta or creating
an extra assistant/error bubble. Available partial text and rich content survive
terminal-only recovery, and each bubble retains its own run correlation. Actual
content replay rebuilds the active interrupted bubble under its original ID;
metadata alone does not erase its partial. Native error detail decorates the
failed bubble from that exact run. Other turns and chronological segments are
not selected through a generic “last assistant” match. A subsequent local Stop
or abort cannot relabel a settled bubble.

The authored actual-hook regression captures the callback delivered to
assistant-ui and calls it through `renderHook(useChatRuntime)`. It retains the
production HTTP service, stream loop, native adapter, message store, `UarDb`
message read/write path and real PGlite run-event repository. HTTP alone supplies
controlled failed/delayed cancellation and exact native terminal frames; the
unrelated runtime-console ingestion and assistant-ui renderer are substituted.
It asserts original message/run/turn identity, persisted same-row readback,
terminal-only success/error/cancellation, single original send, one terminal
record, and late Stop immutability. This is regression source, not executed
acceptance. No test, typecheck, build, database execution or browser outcome is
claimed while the phase verification gate is closed.

A native terminal may already be durably accepted while its persistence promise
is returning to the stream loop. Stop can clear observation ownership during
that interval. The native finalizer therefore carries the accepted event's exact
run ID and may finish only that run's provisional assistant rows; local Stop
supplies no such authority. The regression source holds the production
`finishRun` return after its real repository write, invokes the actual callback,
and releases the handler for each native outcome. A separate source control
covers success fully processed before Stop, and a rich multi-segment fixture
retains all original IDs/content and unrelated settled rows. These authored race
assertions have not been executed.

## Rejected native evidence before reduction

An observer attached to native run N may receive a schema-valid frame naming F.
That frame is rejected before adapter sequence/deduplication/state/phase bookkeeping,
run identity adoption, runtime entity ingestion, chat content, usage, native error
presentation or terminal settlement. Header identity and the original recovery or
continuation receipt must agree. Without a header, the first explicit official
identity is adopted only after its event persistence succeeds. The writer repeats
its normalized/wire identity check at persistence ingress.

Identity rejection and same-run storage failure are distinct errors. An append or
native finish failure for N stops reduction before the terminal's usage, phase
projection, message outcome or ChatTurn settlement can be applied. A successfully
appended terminal whose finish write fails remains raw evidence; append alone does
not certify settlement. The observer closes provisionally, the original partial
stays `interrupted`, and its original ChatTurn remains `uncertain` and eligible for
`pendingChatTurns` recovery. There is no automatic replay, replacement POST or
new assistant/error bubble. Weak legacy terminal notifications also supply no
native settlement authority to this stream consumer.

The existing interrupted message status is the visible presentation. An available
`onError` callback receives the local identity/storage category, and the console
records a bounded category without copying foreign provider details. The ordinary
chat hook may omit `onError`; this path deliberately shows interruption rather
than attaching inline provider/storage error details or falsely marking the
partial failed. A later deliberate read-only reattachment to N can settle the same
original message and turn from accepted exact native proof. Already confirmed
repository terminals retain the existing immutable update guard; Stop and local
finalization supply no replacement authority.

The R030 regression source extends the actual-hook/PGlite fixture across native
success, error and cancellation with distinct foreign F versus attached N, response
header and first-event identity adoption, and targeted synthetic terminal append
and native finish failures. It asserts absent foreign runtime/phase/usage/error
reduction, interrupted original content/IDs, pending ChatTurn eligibility, one
original POST, no automatic recovery request, and subsequent exact original
terminal-only recovery. Continuation fixture source covers receipt binding,
conflicting headers, foreign terminal/content/state/memory rejection and original
recovery without a chat POST. Prior Stop, delayed/failed cancellation and native
terminal race controls remain present. These are authored sources only: no test,
PGlite execution, typecheck, build, browser observation or acceptance is claimed.

Schema validity alone is insufficient terminal proof: the stream consumer also
requires a nonempty normalized native run ID, the same wire run ID, profile
`uar.agui/1`, and exact event ID/type agreement before invoking persistence or
reducing the terminal. Continuation regression source includes absent, empty and
nonstrings in terminal `runId`; each must leave the attached partial interrupted
with no terminal accounting or settlement. The persistence-only weak-terminal
interruption contract is retained. `finishRun` exposes no already-settled status
result; its existing repository immutability guard is unchanged by this repair.


## Usage availability after native success or cancellation

Native success and cancellation preserve usage availability independently for input, output and total tokens. Missing or null counters remain unavailable. A reported output count does not establish input usage, and reported input/output counts do not establish a missing total. Explicit numeric zero remains a known count. Cache-read, cache-creation and uncached-input evidence also remain independent; they are never added together to fill unavailable aggregate input or total counts.

The existing chat and Runtime Console numeric usage summary requires all three reported aggregate counters. For example, `{ inputTokens: null, outputTokens: 5, totalTokens: null }` omits the chat numeric usage summary and appears as a raw native terminal event in Runtime Console. It does not appear as `0 / 5 / 5`. A complete reported `{ inputTokens: 0, outputTokens: 0, totalTokens: 0 }` remains a numeric zero summary. Available model and estimated cost, cache evidence, and partial counts remain in the durable native event journal and run usage record. The live message can retain reported model metadata; partial values are not converted into a complete message usage chunk. On reload, local terminal recovery applies this same complete-count requirement.

The uncomfortable limit is that the current complete-only summary cannot display individual partial counters or partial cost as a numeric usage card. Inspect the retained raw event/run evidence for those values. Summary absence means unavailable aggregate evidence, not zero usage, no charge, failure or cancellation. This projection does not change billing settlement or replace the host ledger.

R031 evidence status: production and fixture source authored; no tests, build, browser, database execution or installed qualification was run for this change. Available raw evidence and omitted numeric display are the source contract, not release certification.


## Local journal recovery after message-finalization interruption

The browser may already hold an interrupted assistant message when later native events and their terminal become durable before the updated message is saved. Reopening the thread reads that local run journal even when the cached bubble exists. Available newer text and native success, error or cancellation can therefore repair the original bubble while the server is unavailable. Recovery performs local reads and writes only: it does not replace the admitted run, POST another turn, replay a tool, or require remote history backfill.

Recovery uses original native deltas once. Coalesced end rows summarize those deltas and never add another copy. Exact native run/event/type/sequence correlation, the original native message identity, ordered cached segment anchors and matching durable run outcome are required before a provisional interruption is settled. Native sequence gaps and ties are valid producer data. Original assistant and rich chunk IDs, text boundaries, tool/artifact/approval chronology and already-settled outcomes remain intact. Repeated local loads do not append the recovered text or rich cards again. Cached-only independent evidence remains present, and restored approvals remain historical.

The uncomfortable limit is ambiguous or incomplete history: a legacy or foreign bubble, wrong-message or malformed event, missing original run start/deltas, a coalesced summary without its original deltas, conflicting text, or non-unique multi-bubble anchors cannot establish a safe local join. Such evidence does not promote the cached interruption to success/cancellation/error. Remote read-only backfill or reattachment may still supply missing evidence later; local recovery never invents it. Numeric usage summaries remain complete-only, while independently unavailable counters remain in native/run evidence rather than becoming zero or a derived total. PGlite does not reconstruct top-level message model/usage fields; persisted rich usage/raw chunks and the journal remain the durable evidence.

R031 evidence status: production local-store recovery and existing real-PGlite fixture source are authored, not executed. This does not qualify actual crash/restart timing, browser storage or an installed release.


## Native artifact identity through chat projection

The native `uar.agui/1` artifact display and input-request custom events retain the producer's camelCase artifact identity and type in the durable journal. The adapter supplies those fields to the existing chat consumer without changing content, title, language or publication metadata. Native artifact type remains available in raw runtime projection; rich persisted display/input chunks retain the original artifact identity and original publication generation/source metadata. The input renderer passes the original backend artifact ID rather than its local card ID. An input without original identity remains read-only and reports “This input is unavailable. Request a fresh input form.” Existing legacy artifact events retain their working path. This field/identity bridge does not grant surface authority, reconstruct a missing publication generation, replay an action or submit an input response. The uncomfortable limit: authored local integration fixtures observe the policy-component boundary; actual form submission, A2UI surface validation, visual behavior and installed/runtime operation remain unverified until their separately authorized real-path gates.

R031 evidence remains source-only: the authored native/legacy/PGlite/renderer-policy-boundary cases have not run. This section does not certify actual form submission or installed behavior.


## Canonical A2UI recovery from the local journal

When a native display arrives after an interrupted assistant message was cached, local reload projects that new journal publication through the same declared A2UI interpretation used by live chat. The reserved durable display wrapper carries original artifact identity/type, content, language, nested metadata/profile, source run and host publication generation map; portable reconstruction retains the artifact ID and protocol version. Local recovery neither fetches server history nor submits an action. Mounting a valid recovered surface still performs the existing read-only authority lookup; an unavailable server leaves controls disabled. Missing historical generations remain missing and read-only, while absent/unsupported profiles keep the existing display refusal. Generic artifacts and previously cached rich evidence retain their existing paths and identities. Raw native journal evidence is not rewritten. These are authored source changes and regression scenarios; no runtime or renderer qualification has executed under the closed gate.


## Saved model policy in controlled browser history acceptance

The controlled shipped-UI tool-loop reload fixture uses normal Send without an explicit `model` override. The externally provisioned effect agent's default provider/model must equal the manifest's logical `model`; the fresh saved AgentSession selection must confirm that agent with `model: null` (inherit). Each original/later admitted run must expose the same logical route in its existing `effective_model` inspection, bound to its exact run, agent and conversation IDs. Actual peer journal arrivals must still carry the manifest's separate `peer.wireModel`. Both preamble and tool-only scenarios retain the original canonical-history, fresh/warm reload, unique final-answer, exact provider/effect count and no-new-POST/run/effect assertions. R034 corrects authored fixture expectations only; it supplies no executed browser or release acceptance evidence.


## Intentionally runless host context in local recovery

Local native recovery accepts the existing `uar.context.updated` ContextAction extension when its original run/thread identity is null or absent and its value matches the native producer shape. The durable outer row must still belong to the exact run, after its valid native start, with matching profile/event/type/sequence and ordered timestamped journal evidence. Context telemetry remains unchanged in storage and contributes no chat content or run/terminal/surface authority. Native partial/success/error/cancel recovery continues to depend on its original exact-run text/terminal evidence, preserving cached rich segments, portable content, metadata and settled outcomes. Missing-ID text/state/action/artifact/terminal and unknown runless extensions remain refused. MCP lifecycle is rebound before run-stream delivery; guardrail run-stream events are run-bound, so neither is included as a speculative runless exception. This is authored source behavior and fixture coverage, not an executed recovery claim.


## Anchored warm canonical history

Warm canonical text-only history now reuses the saved message's text anchors, keeping introductory text, retained tool/artifact/image/approval cards and later text in their cached rich order. Original message/card identities and historical approval evidence survive; canonical timestamps, status and history position remain authoritative, and canonical preamble/final-answer messages remain separate. When canonical text is a prefix of an earlier combined live bubble, obsolete tail text is removed from that bubble rather than duplicating the distinct final answer. Whole unchanged leading/trailing text anchors bound changed text; an ambiguous changed middle uses the first unmatched text anchor. Model history does not encode card offsets, so arbitrary rewritten text or already-lost boundaries cannot establish original intra-message timing. Portable coalesced text and rich rendered boundaries are reconciled independently. Repeated warm backfill and actual PGlite readback are covered by authored ordered fixture scenarios, but have not been executed under the closed verification gate. Recovery only reads session history and persists the projection; it does not submit a replacement run or replay an effect. Reserved canonical A2UI-wrapper duplication producer reachability remains unverified separate triage.


## Stable user identity and legacy media backfill

User-history backfill keeps exact stable message identity authoritative. When message IDs differ, only nonempty ordered text-only legacy aliases without client-turn or run evidence may match; equal captions, empty image prompts, identical or reordered media and unknown media representations do not establish user identity. A mismatch leaves the entire cached and durable history untouched, preserving original input IDs, images, rich chunks and historical decisions. This conservative fallback can decline recovery of a legitimate multimodal legacy snapshot whose stable identity is unavailable; it preserves evidence rather than guessing. No model input, provider routing, global migration or media retrieval is introduced.

The helper/store/real-PGlite mismatch cases are authored source only; no fixture or runtime qualification has run.


## Settled continuation observation

R037 source repair makes cursor-zero completed-continuation recovery observe already settled assistant evidence idempotently. Complete/failed/cancelled messages keep their original IDs, status, text, rich chronology, model and usage evidence; replay frames, missing RUN_STARTED agent metadata, observer abort, storage refusal and terminal-less errors cannot append/reset/downgrade them. Exact native identity/persistence still controls acceptance and the action receipt stays interrupted on rejected/failed/cancelled or unknown observation. Interrupted original bubbles and new admitted runs retain their production path. Authored real-domain/adapter/stream/PGlite scenarios cover lost final action-receipt persistence and repeated observation without new message writes or action/completion POSTs. Source/fixtures are authored only; executable verification and issue acceptance remain unclaimed under the closed gate.
