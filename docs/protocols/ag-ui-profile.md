# UAR AG-UI profile (`uar.agui/1`)

Status: stable for UAR 1.0  
Upstream vocabulary pin: AG-UI core events as documented 2026-07-11  
Wire selection: `stream_mode: "agui_spec"`

The stability label names this profile's historical compatibility contract,
not current Linux-container conformance. Use the [canonical recovery table](../product-support-matrix.md#execution-and-recovery-guarantees)
for interface-specific durable admission, bounded cursors/gaps and uncertain
effects; [chat](../operations/chat-turn-recovery.md) and
[A2UI interaction](../operations/a2ui-interaction-recovery.md) intent receipts
remain separate from event replay. Current pinned-peer acceptance is unverified.

UAR implements the official AG-UI event model through a versioned compatibility profile. AG-UI itself remains pre-1.0, so the profile pins a dated vocabulary rather than claiming conformance to an upstream semantic version that does not exist. The authoritative upstream references are the [event vocabulary](https://docs.ag-ui.com/concepts/events), [architecture](https://docs.ag-ui.com/concepts/architecture), and [serialization/replay rules](https://docs.ag-ui.com/concepts/serialization).

## Transport and envelope

Events are delivered as UTF-8 Server-Sent Events. The SSE `event:` value and JSON `type` discriminator are the same official upper-case event type. Every UAR profile event also carries:

- `profile: "uar.agui/1"`
- `eventId`: stable logical event identifier
- `sequence`: zero-based monotonic run sequence
- `threadId` and `runId` when the event belongs to a run
- `timestamp`: Unix milliseconds when available

The SSE `id:` is the replay cursor. Clients reconnect with `Last-Event-ID`; replayed logical events retain their original `eventId` and `sequence`.

## Supported standard events

| Category | Events | UAR contract |
| --- | --- | --- |
| Run | `RUN_STARTED`, `RUN_FINISHED`, `RUN_ERROR` | Exactly one start and one terminal event per run. Cancellation is `RUN_ERROR` with `code: "CANCELLED"`. |
| Step | `STEP_STARTED`, `STEP_FINISHED` | Tool-loop/runtime step boundaries. |
| Text | `TEXT_MESSAGE_START`, `TEXT_MESSAGE_CONTENT`, `TEXT_MESSAGE_END` | Content is non-empty and concatenated by `messageId`. |
| Reasoning | `REASONING_START`, `REASONING_MESSAGE_START`, `REASONING_MESSAGE_CONTENT`, `REASONING_MESSAGE_END`, `REASONING_END` | Visible provider reasoning. Deprecated `THINKING_*` events are never emitted. |
| Tools | `TOOL_CALL_START`, `TOOL_CALL_ARGS`, `TOOL_CALL_END`, `TOOL_CALL_RESULT` | One lifecycle per `toolCallId`; args are streamed JSON text. |
| State | `STATE_SNAPSHOT`, `STATE_DELTA`, `MESSAGES_SNAPSHOT` | Snapshots replace; deltas are ordered RFC 6902 patches. |
| Escape hatches | `RAW`, `CUSTOM` | Provider payload passthrough and registered UAR extensions. |

Draft upstream activity/meta and interrupt extensions are not part of `uar.agui/1`. Approval interruption is represented by a stable UAR custom event until the upstream interrupt shape is finalized.

## UAR custom-event registry

Custom events use the official shape:

```json
{
  "type": "CUSTOM",
  "profile": "uar.agui/1",
  "name": "uar.tool.approval_required",
  "value": {},
  "eventId": "run-1:approval:call-1",
  "sequence": 7,
  "threadId": "thread-1",
  "runId": "run-1"
}
```

| Name | Required value fields | Meaning |
| --- | --- | --- |
| `uar.citation.added` | `citation` | Source citation became available. |
| `uar.memory.recall` | `items`, `count` | Recalled/selected records; this event alone does not prove final provider-input submission. See [injection provenance](../operations/memory-injection-provenance.md). |
| `uar.memory.mutation` | `operation`, `memoryId`, `scope`, `memoryType` | Durable memory mutation. |
| `uar.artifact.available` | `artifactId`, `artifactType`, `title`, `content` | Renderable artifact. |
| `uar.artifact.input_required` | `artifactId`, `artifactType`, `title` | Artifact awaits human input. |
| `uar.skill.activated` | `skill`, `selectionMethod` | Skill selected for the run. |
| `uar.context.updated` | `strategy`, `messagesRemoved`, `tokensSaved` | Context compaction/action summary. |
| `uar.tool.approval_required` | `toolCallId`, `name`, `arguments`, `riskReason` | Tool execution is interrupted for a decision. |
| `uar.guardrail.flagged` | `category`, `reason` | Safety guardrail signal. |
| `uar.quality.sycophancy_flagged` | `score`, `classifications` | Quality review signal. |
| `uar.quality.sycophancy_corrected` | `correctedText` | Corrected output signal. |
| `uar.budget.alert` | `scope`, `scopeId`, `spentUsd`, `limitUsd`, `exceeded` | Budget warning or denial. |
| `uar.usage.reported` | token counts, optional model/cost | Usage telemetry when not carried on `RUN_FINISHED`. |
| `uar.sandbox.output` | `stream`, `data` | Sandboxed execution output. |

Custom values are typed, JSON-serializable, and must not contain secrets. Consumers must persist unknown `uar.*` events for inspection and otherwise ignore them safely.

## Compatibility

- `agui_spec` is the only conformant UAR mode.
- `agui` is the deprecated legacy lower-case dialect and is not `uar.agui/1`.
- `dual` combines legacy `agui.*` and OpenAI chunks; it is not conformant.
- `openai` remains the default OpenAI-compatible streaming mode.

## Unsealed durable native terminals

A native run can persist its terminal record and event before the independent
AG-UI observer seals its wire binding. Recovery authenticates the current
verified owner (including tenant), wire binding, native run, agent and session.
A resident producer still has to finish its outer cleanup: observing a terminal
frame alone never settles it. Recovery also joins any locally retained producer
and graph root when the resident run entry has disappeared. Dropping a join
waiter retains the owned handle for the next waiter.

When the run is genuinely historical, a durable terminal record and a matching
native terminal event after the binding's start cursor permit closure through
the existing binding CAS. They do not prove cleanup in the previous process.
The wire terminal is therefore `RUN_ERROR` with
`code: "host_cleanup_unconfirmed"`. Its `metadata.uar` contains:

- `cleanup_unconfirmed: true`;
- `nativeRunId` and the original serialized `nativeStatus`;
- the original `nativeTerminal` and `nativeTerminalCursor`;
- `nativeTerminalEvidence`, preserving terminal events and host diagnostics with their native cursors.

A native success remains recorded as native success evidence, while its wire
closure reports unconfirmed cleanup. Native errors and cancellation retain their
original status and event; they never become successful wire outcomes. Ordinary
error paths can append a final native Done marker: it cannot replace the error.
Cancellation has an explicit ordered evidence contract: under a durable Cancelled
record, zero or more same-run host diagnostics or legacy Error diagnostics may
precede exactly one matching Cancelled event. A cancelled conversation terminal
save failure emits the existing nonterminal `PresentationDiagnostic`, retaining
`session_persist_failed` and `Conversation terminal could not be saved`, before
Cancelled. Native `agui_spec` transports it as `CUSTOM` named
`uar.presentation.diagnostic`; it must not close the stream before the final
`RUN_ERROR` with code `CANCELLED`. Ordinary noncancelled save failure remains an
Error. Historical Error→Cancelled evidence from earlier producers remains
supported under the same correlation and ordering checks. The matching
Cancelled event supplies `nativeTerminal`, `nativeTerminalCursor` and the sealed
source cursor; all preceding diagnostics retain their original codes, messages
and cursors in `nativeTerminalEvidence`. Cancellation usage metadata is retained
with the original event. Diagnostics alone never establish cancellation. A Done
event before or after cancellation, a diagnostic after cancellation, or a second
Cancelled event contradicts this terminal proof and leaves the binding unsealed.
These checks enforce the durable correlation and terminal-proof boundary; they
do not certify cleanup, suppress errors, or authorize another dispatch.
Missing, foreign-owner, mismatched-agent/session/run, malformed or nonterminal
evidence leaves the binding unsealed and execution uncertain. An absent resident
entry is never evidence of completed cleanup. Local producer or graph join
failure remains a projection failure and cannot seal a successful outcome.
Existing approval and caller-tool handoff reconciliation keeps its own contract.

The sealed terminal and source cursor remain immutable on reconnect and identical
request retry, including after competing observers attempt the CAS. Closure can
unblock the existing next-wire-run admission path; its identity, parent,
authority, approval and budget gates still apply. Recovery dispatches no model
or tool work for the original run and grants no permission to replay effects.
It does not implement full-harness restart or certify external-effect settlement.

Regression source covers an owned SurrealKV close/reopen with an unsealed wire
binding and native success/error/cancellation, owner and correlation negatives,
competing/repeated CAS, cursor replay, identical POST retry and zero dispatch for
the original request. A mounted successor POST uses a persisted fixture agent
and an owned Keyword skill service, retains parent-gate rejection for an incorrect
predecessor, and checks a distinct native identity plus exactly one controlled
driver-input arrival for the new request. This is not independent provider HTTP
or billing evidence. Private manager regression source covers held local producer joining
and dropped-waiter retention with and without residency. These cases are authored
but unexecuted under the current closed verification gate; the public-route cases
reuse the existing external identity host profiles and carry no credentials.
The same reopened public-route matrix includes `session_persist_failed`→Cancelled
and multiple diagnostics before Cancelled, with exact evidence and primary cursor
assertions plus the unchanged successor path. The additional reopened cancellation
negative matrix covers missing/late/conflicting terminal evidence, wrong-run
diagnostics/cancellation, cursor exclusion, malformed evidence, owner/agent/session
conflicts and incompatible durable status, with repeated unsealed readback and
zero dispatch. These seed persisted evidence; they do not inject a real session
save failure or reproduce its crash window. Runtime/profile availability and
actual failure timing remain unverified until an explicitly authorized gate runs.


## Wire selectors and admitted native identity

The AG-UI run endpoint stores the original URL selector as `agent_id`. A deployment
selector such as `uar:agent:r026-binding` names a binding, while the native run's
`agent_id` names its admitted artifact. These namespaces are separate. New native
admissions capture `native_agent_id` from the single resolved and prepared host
request and durably create the wire binding before dispatch. The existing
owner/thread/wire storage keys retain the original selector. The admitted artifact
identity is immutable across terminal CAS; a retry uses the original reservation
and cannot submit another execution after an uncertain admission outcome.

Both resident and historical projection require the authenticated owner, exact
native run ID, captured native artifact ID and exact session. Warm and checkpoint
approval continuations inherit the predecessor's original native artifact identity;
a deliberate new successor captures its own actual admission. The current mutable
catalog never supplies identity proof for an already-admitted run. Older direct
artifact selectors retain exact equality to the original `agent_id`. An older
deployment-selector binding without `native_agent_id` cannot establish that proof:
projection refuses with `projection_unavailable` and leaves the binding unsealed.
It does not guess, rewrite the binding, consult today's target, or repeat work.

Resident Cancelled projection joins the retained producer and refreshes native
history after settlement. It applies the same ordered cancellation evidence
contract as historical recovery: host diagnostics or legacy Errors may precede exactly one matching
Cancelled; missing, foreign, duplicate, contradictory or trailing terminal evidence
cannot seal the binding. The final Cancelled supplies the sealed cursor and primary
`RUN_FINISHED` cancelled outcome. Its original usage and all preceding diagnostics
remain in `metadata.uar.nativeTerminal`, `nativeTerminalCursor` and
`nativeTerminalEvidence`. A diagnostic-only prefix remains unsealed. Historical
closure continues to report unconfirmed cleanup, with its original native outcome
preserved as evidence.

Standard AG-UI also projects host diagnostics as nonterminal `CUSTOM` events
named `uar.presentation.diagnostic`. The `value` retains the original code and
message. Wire `threadId` and `runId` identify the binding; `metadata.uar` retains
`nativeRunId`, `nativeCursor` and the complete original `nativeEvent`. The SSE
frame cursor retains that native cursor. Only diagnostics from the exact bound
native run may be projected or included as cancellation proof; a foreign or
post-cancellation diagnostic cannot seal the binding.

R026 regression source extends the existing public-route SurrealKV fixture. An
installed collaboration package and binding admit a selector distinct from its
artifact through public POST, followed by a separate successor and an empty-catalog
reopen. The model configuration proof is seeded fixture data; this is not live
provider verification. A reconstructed unsealed window uses that actual admitted
identity to exercise historical closure; no crash is executed. Further cases cover
legacy target refusal, exact owner/run/agent/session mismatches and legacy direct
selectors. Resident cases use actual retained manager cancellation and provider
usage with public diagnostic-event injection, including diagnostic-only and
contradictory terminal sequences. They do not inject an actual session-storage
failure. R027 adds separate local armed `PersistenceLayer` forwarding in the same
fixture: after actual manager/provider admission, only the named conversation's
terminal `save_session` fails. This synthetic fault drives the actual manager,
public native stream projector, standard resident projection and store-reopened
historical evidence. It also retains ordinary noncancelled Error assertions.
Seeded diagnostic matrices remain explicitly distinct from this fault scenario;
the reopened observer window is reconstructed and is not a crash experiment.
All regression source remains unexecuted; compilation, actual storage failure
timing, transport behavior and runtime acceptance remain unverified under the
closed gate.


## R028 graph cancellation and deployment action receipts

The graph manager applies the same cancellation-save rule as the ordinary tool
loop. After successful host, terminal, actor, sandbox and approval cleanup, a
failed terminal conversation save emits nonterminal
`uar.presentation.diagnostic` / `session_persist_failed`, followed by the original
Cancelled terminal. In-memory message and run status remain cancelled. A genuine
cleanup failure remains Error; a later session-save diagnostic cannot turn it
into cancellation. Graph cancellation without provider usage leaves usage absent,
not zero. The failed save does not establish durable conversation persistence.

A logical action receipt retains its original wire selector and storage key. Its
wire-binding snapshot may precede the host's native artifact capture. Recovery
joins that snapshot to the exact original durable wire row: owner, wire selector,
wire thread/run, native run/session, workspace, interaction, input digest,
predecessor and cursor identities must remain equal. Only initial native identity
capture and the valid terminal/revision transition may differ. A snapshot that
already captured a native identity cannot change it. The actual continuation and
prior source native runs must match their own original admitted artifacts and
sessions under the verified owner; the prior source must belong to the same wire
conversation and precede the action admission. Source declaration folding uses
those owner/native/session fences before granting action authority.

Receipt reads and identical logical retries use this original proof, including
when the deployment catalog is empty after restart. They never resolve today's
catalog or dispatch again. Exact Done/Error/Cancelled continuation records report
completed/failed/cancelled; nonterminal durable history without a resident producer
stays uncertain. Missing or conflicting proof remains uncertain on receipt reads
and refuses retry attachment. This contract proves admission and native outcome,
not external effect settlement or global exactly-once execution.

The existing historical-terminal fixture now authors a held graph node through
`RunManager::with_agent_graph` and the actual orchestrator-agent branch. Its armed
local forwarding store faults only the terminal session save, with a separate
approval-record read fault as the true cleanup-failure control. It asserts native
and message status, diagnostic ordering, absent usage, public native terminal and
standard cancellation evidence. This is synthetic collaborator fault source,
not an executed store outage, graph restart, or qualification result. All new
regressions remain unexecuted while the gate is closed.
