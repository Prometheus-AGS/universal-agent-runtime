# AG-UI run and approval transport

The authenticated endpoint is `POST /api/uar/ag-ui/agents/{agent_id}`. It accepts
AG-UI protocol `1.0` `RunAgentInput` and returns SSE `data:` frames. The contract
is pinned to published `@ag-ui/core` 1.0.2 and upstream commit
`704b190e44aedda71a5388d2676370711c6b6b49`; source copies and provenance are in the
Dot implementation `protocol-sources` directory. Existing run/OpenAI endpoints
retain their compatibility behavior.

```json
{
  "threadId": "conversation-1",
  "runId": "client-run-1",
  "protocolVersion": "1.0",
  "messages": [{"id":"message-1","role":"user","content":"Perform the configured task"}],
  "tools": [],
  "context": []
}
```

`agent_id` selects an existing native agent or the escaped `uar:agent:<binding>`
target returned by target discovery. Catalog targets require the same
`x-uar-workspace-id` header as native admission. Authentication, current workspace
policy, configured model availability, catalog grants, credential capture, and
host tool admission use the existing run path. A wire run ID never becomes an
execution capability or native run ID.

The current content profile accepts text user input and user/assistant/tool
history. Assistant history may contain standard function `toolCalls`; tool
history requires matching `toolCallId`. The native host validates the complete
history before use. System/developer, multimodal, activity and reasoning input
messages are not promoted into host instructions and are rejected as unsupported
history. Nonempty frontend `tools` is rejected: clients cannot install a tool by
sending its description. `state` and standard `{description,value}` context
entries are appended as explicitly untrusted prompt data. The server does not
claim to synchronize a client state store.

`forwardedProps.uar` accepts the current native host resource input fields such
as model selection, renewed run credentials, MCP grants, service placement and
presentation negotiation. The endpoint fixes the target, session and history;
these identities cannot be overridden through forwarded properties. Credentials
are neither echoed in `RUN_STARTED` nor persisted in the wire correlation row.
The row stores a request digest for exact retry equality. Authentication headers
may be renewed independently.

## Interrupt and resume

A pending host approval closes the current SSE segment with:

```json
{"type":"RUN_FINISHED","threadId":"conversation-1","runId":"client-run-1","outcome":{"type":"interrupt","interrupts":[{"id":"host-issued-approval-id","reason":"tool_approval","toolCallId":"host-tool-call-id","responseSchema":{"type":"object","properties":{"approved":{"type":"boolean"}},"required":["approved"],"additionalProperties":false}}]}}
```

The next request uses the same thread, a **new** wire `runId`, and answers every
open interrupt exactly once:

```json
{
  "threadId":"conversation-1",
  "runId":"client-run-2",
  "parentRunId":"client-run-1",
  "messages":[],
  "resume":[{"interruptId":"host-issued-approval-id","status":"resolved","payload":{"approved":true}}]
}
```

`status:"cancelled"` or `approved:false` denies the approval. A resume cannot
replace protected prompt/history/session/artifact state. Pending IDs, current
owner/workspace, expiration and exact protected continuation are validated before
resolution. Warm approval resolves the existing host waiter. After restart,
current admission and predecessor exclusion must permit the existing protected
checkpoint; fresh resources may be supplied in `forwardedProps.uar`. Multiple
separate cold continuations cannot be partially resumed by this endpoint.
Descriptor Required remains mandatory even with governance Off.

Tool start/args/end events describe the proposed invocation. A result is emitted
only from the existing host's actual tool-result event. Resumed segments do not
repeat the interrupted segment's tool start. A denial is never a successful
result. The broker retains the only decision record and CAS authority.

## Retry and reconnect

`GET /api/uar/ag-ui/agents/{agent_id}/threads/{thread_id}/runs/{run_id}/stream`
reconnects to the owner-bound segment. SSE IDs bind the exact wire run, native
source cursor and projection offset. `Last-Event-ID` for another wire run is
rejected. The same POST body/runId returns the existing correlation without
submitting another run; a different body with that ID conflicts. A context slot
prevents simultaneous different wire runs in the same owner/agent/thread.

A transport observer persists an interrupt/terminal projection even if the
client disconnects. Cold reconnect can read the persisted terminal projection or
reconstruct an exact pending interaction from the durable broker. Native token
history is bounded and is not a durable transcript: a missing prefix returns
`RUN_ERROR` with `stream_history_unavailable`, and a reserved execution without
confirmed status returns `execution_status_unconfirmed`. It is never replayed
because a process or socket disappeared.

The uncomfortable case is a crash after reserving a wire ID but before the host
acknowledges admission, or after a decision commits but before its continuation
is observed. The reservation stays held; transport retries cannot manufacture
completion or repeat a possibly executed command. Supervisor exclusion and
protected host recovery remain the authority for resolving that uncertainty.

This document describes source behavior. Protocol fixtures and installed-runtime
restart/race qualification remain deferred until the full Dot production phase
implementation boundary.

MCP elicitation uses the same interrupt outcome with `reason:"mcp_elicitation"`.
Its response payload is an MCP `ElicitResult`, for example
`{"action":"accept","content":{"name":"Ada"}}` for a form or
`{"action":"decline"}`. `status:"cancelled"` maps to `{"action":"cancel"}`.
The host validates the server's requested schema, exact original connection and
live waiter; an accepted form never approves a tool. Approval and elicitation
IDs together form the all-open interrupt set. Cold disconnected elicitation is
history and cannot be replayed or delivered to a replacement server connection.

## v1.1.0 complete interrupt profile

The release-pinned [Kubo integration contract](releases/v1.1.0-kubo.md#complete-ag-ui-interrupt-contract) specifies every approval and MCP elicitation field, response boolean/action, and exact all-pending resume coverage.
