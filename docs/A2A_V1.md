# A2A 1.0 interface

Discover the compiler with `GET /.well-known/agent-card.json`. The public card
advertises the JSONRPC interface `/a2a/v1/compiler`. Named registered agents use
`/a2a/v1/agents/{agent_id}`. Task operations require current verified credentials;
only the exact card GET paths are anonymous.

Send `A2A-Version: 1.0` and `Content-Type: application/json` with bearer JWT or
X-API-Key authentication. The schema is pinned to the official
[A2A v1.0.0 protocol](https://github.com/a2aproject/A2A/blob/v1.0.0/specification/a2a.proto)
and [JSON-RPC binding](https://a2a-protocol.org/v1.0.0/specification/#9-json-rpc-protocol-binding).

```json
{
  "jsonrpc": "2.0",
  "id": "request-1",
  "method": "SendMessage",
  "params": {
    "message": {
      "messageId": "caller-generated-unique-message-id",
      "role": "ROLE_USER",
      "parts": [{"text": "Help me write an agent specification."}]
    },
    "configuration": {"returnImmediately": true}
  }
}
```

The result contains `task`, with `id`, `contextId` and a `TASK_STATE_*` status.
Use `GetTask` with `{"id":"..."}` to poll. `historyLength: 0` omits history.
Without `returnImmediately`, SendMessage waits for a terminal or interrupted
state. This agent accepts text/plain input. Output uses text/plain and
application/json. Streaming, push notifications and extended cards are not
advertised and return the protocol's unsupported errors. `ListTasks` lists only
the current complete principal's tasks, with context/state/time filtering and
pagination. `CancelTask` returns cancellation only after host settlement.

Missing/invalid credentials return HTTP401, insufficient current scope HTTP403.
A private task outside the caller's owner partition returns TaskNotFound.
Changing a tenant in JSON never changes the authenticated tenant.

The old `/.well-known/agent.json` card and `/a2a/compiler`, named-agent legacy
JSON-RPC routes, and existing gRPC binding remain compatibility interfaces.
Their legacy fields/methods are not the advertised A2A1.0 wire contract. An
omitted version is legacy0.3 and is rejected by the version1.0 endpoint.

Task correlation is persisted before an exact actor run is submitted. An
identical message retry reuses its binding; altered reuse of an initial
messageId is rejected. Complete issuer/tenant/subject identity partitions task
records. Cold reads project existing durable roots and never grant execution
from an ID. Restart continuation uses the existing protected checkpoint host
admission; stale process identity alone never authorizes replay. Durable cancel
intent survives caller disconnect and is consumed by the current host.

Pending tool decisions appear as `TASK_STATE_INPUT_REQUIRED`, with the existing
host records under task metadata `uar.approvals`. Answer using `SendMessage` for
that exact `taskId` and context, a fresh user `messageId`, a text part describing
the response, and request metadata `{"uar.approval":{"id":"<challengeId>",
"approved":true}}` (or `false`). Free text alone does not approve a tool.
Governed peer requests must include their freshly authenticated delegation
contract. Repeating the same committed decision does not invoke the tool again;
a contradictory decision conflicts. Denial projects `TASK_STATE_REJECTED` only
after the host confirms terminal cleanup.

The uncomfortable limit: a dead producer with uncertain effects cannot safely
be called canceled or restarted from a task ID. It needs supervisor exclusion;
pending tool approval needs the explicit approval-specific continuation path.
Cold governed delegation requires the authenticated source peer to resubmit the
exact contract. The host compares protected checkpoint provenance, resolves
current policy, and reuses durable accounting balances, attempt counts and the
original absolute grant deadline. Saved contract JSON never grants execution.
Supervisor exclusion and interactive approval remain required Dot integration
boundaries, not a promise of automatic timeout takeover.
Separate tasks sharing a context ID use an owner-and-agent-bound host session.
An atomic context slot admits the next task only after the prior task has a
confirmed terminal result. The host persists conversation history and collected
artifacts before committing that result; cold reads reconcile the transport
projection from the exact durable root. Final release acceptance must exercise
these paths across actual process replacement.

The card version is compiled from Cargo package provenance. Equality to an
installed published tag/checksum is a separate release gate; source version
alone is not proof of the installed artifact.

New task reservations retain the exact trusted runtime epoch before dispatch.
After trusted supervisor exclusion (or confirmed orderly host shutdown), a cold
cancel read uses the existing host effect barrier: unresolved tool outcomes,
canonical receipts, descendant accounting liabilities and unconfirmed remote
children keep the task nonterminal. A native Surreal transaction compares the
exact binding and closed epoch, refuses an existing continuation claim, cancels
only nonterminal native thread rows, disables source checkpoint continuation,
and commits the canceled task projection together. Existing terminal kernel
results remain immutable. No approval, spend or effect is fabricated or released.
Legacy bindings without protected epoch provenance and stores without qualified
supervisor recovery retain an unresolved cancellation. CancelTask may report a
conflict while the durable intent remains pending; later GetTask exposes confirmed
completion. Process absence alone is never a successful cancellation.
