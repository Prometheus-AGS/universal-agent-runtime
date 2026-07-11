# UAR AG-UI profile (`uar.agui/1`)

Status: stable for UAR 1.0  
Upstream vocabulary pin: AG-UI core events as documented 2026-07-11  
Wire selection: `stream_mode: "agui_spec"`

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
| `uar.memory.recall` | `items`, `count` | Memories injected into context. |
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
