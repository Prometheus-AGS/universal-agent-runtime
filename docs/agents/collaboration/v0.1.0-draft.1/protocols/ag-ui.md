# AG-UI team interaction contract

Status: **Official Draft Specification 0.1.0-draft.1**. MUST, MUST NOT, SHOULD and MAY describe the proposed collaboration profile, not implemented conformance. The [runtime](../runtime.md) remains the authority for task execution. The protocol is a projection of that runtime.

## Profiles and endpoint

The proposed team route is `POST /api/v1/team-instances/{teamInstanceId}/ag-ui`. Authentication resolves the owner and workspace before looking up the team. An AG-UI thread ID is bound server-side to that conversation/team; supplying another ID MUST NOT select another owner's team. Explicit member targeting requires current membership and authorization; the coordinator remains the default target.

The base profile preserves AG-UI run, text, tool and state events. The enhanced UAR profile is `uar.team.agui/1`, with optional capability `agui.subagents/1` and the custom-event schema in this package. These identifiers are UAR capability identifiers, not claims that upstream registered them. The trusted host adapter negotiates them during connection setup; untrusted prompts and `forwardedProps` MUST NOT grant capabilities or change identity. A client without explicit enhancement support gets the base projection. A client requiring an unsupported profile receives an actionable pre-run error rather than a silent semantic downgrade.

Before implementing an enhanced client, I3 MUST pin an exact upstream AG-UI SDK/schema revision containing the negotiated standard subagent events. This draft follows the inspected [upstream event definitions](https://github.com/ag-ui-protocol/ag-ui/blob/b8ebd02c84a3a2757990da47aebf7c55b708b1ef/spec/1.0/schema.json) and [subagent contract](https://docs.ag-ui.com/concepts/subagents); the schema snapshot is pinned in source-pins.json, while the explanatory documentation URL is mutable. This does not select or certify an upstream SDK release. Base clients MUST NOT receive event types their declared schema rejects.

## Runs, durable tasks and completion

A submitted work request creates or addresses one durable TeamTask. Each admitted interaction gets a distinct AG-UI run segment. Every segment starts once with `RUN_STARTED` and ends once with `RUN_FINISHED` or `RUN_ERROR`. Resume/follow-up creates a new segment and preserves the durable task ID. A durable team can outlive many segments while no model loop is resident.

`RUN_FINISHED` means that segment's output is settled, not that every durable task or team has completed. If work awaits input or approval, the base projection emits a clear assistant status message and finishes the segment; the enhanced projection also supplies `uar.team.v1.task.updated`. A segment cannot finish while its attached child streams are still emitting. The service first settles or explicitly suspends/detaches those activations; any later activation belongs to a new segment or the independently subscribed administrative feed. Nothing may append live events after a terminal run event.

Closing a transport does not cancel a task. Cancellation uses an authenticated task command and exposes its accepted request separately from a settled terminal state. A failure of one member must not turn the coordinator's stream into a false success; task policy determines whether to retry, request intervention or fail.

## Standard events and attribution

Standard `TEXT_MESSAGE_START`, `TEXT_MESSAGE_CONTENT` and `TEXT_MESSAGE_END` retain one `messageId` per message. Text deltas concatenate only within that identity and in emission order. Parallel messages may interleave; neither a member label inside text nor arrival order is an identity mechanism. Standard tool events retain their tool-call IDs and parent-message associations. Tool completion is not evidence that an unconfirmed external effect succeeded.

With negotiated subagent support, `SUBAGENT_STARTED` uses an actual invocation's `subagentRunId` and `name`. Nested invocation attribution uses `parentSubagentRunId`. Text/tool events attributable to a child carry its `subagentRunId`; lifecycle events use the upstream fields, not invented `teamId` properties. Root run lifecycle events describe the segment as a whole and are not attributed to a child. `SUBAGENT_FINISHED` or `SUBAGENT_ERROR` closes that invocation; a pre-admission queued member has no invented child run ID. A later activation of the same member has a new invocation ID.

For base clients, the server projects coordinator text, supported tool activity, state and root run lifecycle. It does not relabel interleaved private member text as coordinator text. The coordinator may summarize authorized public results. The underlying task and scheduler are identical for both client profiles.

## Missing team semantics: CUSTOM events

The only new AG-UI discriminator in this draft is the existing standard `CUSTOM`. Its `name` is namespaced and versioned; `value` follows [custom-event.schema.json](../schemas/custom-event.schema.json). Values contain `schemaVersion: "1"`, `eventId`, monotonically increasing team `sequence`, `teamInstanceId`, timestamp and typed `payload`. Optional task/member/attempt/causation IDs are server-derived and disclosure-filtered. Envelope schema version 1 is independent of draft publication version 0.1.0-draft.1.

| Name after `uar.team.v1.` | Payload and meaning |
|---|---|
| `task.updated` | Task revision, state and optional cancellation-request flag; authoritative committed change |
| `membership.updated` | Member revision and lifecycle state; not a new turn by itself |
| `routing.selected` | Selected member, binding revision and safe reason; no private prompts or hidden reasoning |
| `message.receipt` | Message ID plus accepted/delivered/processed/rejected; none means task success |
| `budget.updated` | Reserved and settled consumption, limit and unit from the root budget ledger |
| `recovery.required` | Safe reason and pending-approval/unknown-effect/stale-ownership/retention-gap/budget-exhausted classification |
| `surface.message` | A2UI profile/catalog, owned surface ID/revision and validated declarative message |

Acceptance means durable enqueue; delivery means available to the recipient; processed means the consuming turn acknowledged application. A task's terminal event is the distinct completion signal. A receipt must never claim processing from a socket write alone.

Text, structured progress, surface data and artifacts remain separate payload kinds. Progress uses task/budget events; A2UI uses `surface.message`; permitted larger results use authenticated artifact references in supported standard result/state payloads. No new generic binary chunk format is introduced. A UTF-8 text delta is a transport piece, not a task boundary. Structured JSON is an atomic validated message, never concatenated text masquerading as an action. Artifact transfers carry their own media type, size and integrity metadata; downloading them rechecks authorization.

## Ordering, replay and snapshots

The durable event log orders committed team events by `sequence`; event IDs are immutable. Event emission after the commit uses the outbox. Per-message chunk order is preserved. There is no global total order across unrelated teams. Nested-team event sequences remain distinct; parent projections preserve source team identity and causation rather than copying a child's sequence into the parent sequence space.

Administrative observation uses the separately cursor-addressable team events resource. A UAR SSE cursor is an opaque subscription token bound to the owner, team and projection, not an approval credential or a raw model token. The adapter records a frame-to-event mapping so replay preserves emitted event/message IDs. Clients de-duplicate by event identity, not text contents. They may see sequence gaps when disclosure filtering excludes events; such gaps do not alone mean data loss.

On reconnect, current authorization is evaluated before replay. A retained cursor replays subsequent authorized frames; a revoked member receives neither old nor new private content. If retention no longer covers the cursor, the server reports a reset requirement and supplies an authorized snapshot with a consistent log watermark. Client replacement of local state and resumption after that watermark is atomic. A `recovery.required` event may describe this reset, but its payload is not the snapshot itself. Administrative snapshots follow [runtime-snapshot.schema.json](../schemas/runtime-snapshot.schema.json). `STATE_SNAPSHOT` inside an AG-UI segment remains run-scoped state, not a claim that AG-UI defines UAR's durable journal protocol.

Slow clients use bounded buffering. On overflow they are disconnected with a resumable position; the scheduler does not wait for a UI socket or discard durable task state. Artifact bodies and secrets do not enter the replay log. Retention, pagination limits and export policy are visible administration settings with authorized bounds.

## Compatibility and failure cases

Unknown optional namespaced events may be ignored; unknown required profile semantics prevent activation. Standard event schemas must not be modified to smuggle UAR identities into closed upstream objects. Public projections cannot expose hidden member prompts, credentials, scoped memory, approval handles or connector payloads solely because a caller can view the team.

The uncomfortable case is a client that displays only a finished run while the durable task awaits human action. Base projection must explain the wait in ordinary text, and The Boss must retain the task board/status beyond the completed segment. Protocol completion and business completion are different records.

Illustrations: [interleaving](../traces/ag-ui-interleaving.jsonl), [base client](../traces/ag-ui-base-client.jsonl), [reconnect](../traces/reconnect-retention-gap.jsonl). These are expected traces, not captured execution evidence.
