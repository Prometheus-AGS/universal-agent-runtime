# A2A streaming lifecycle handoff — 2026-10-20

Phase: `function-first-release-certification`; change:
`deterministic-lifecycle-management`; stage: Execute. Verification gate: **CLOSED**.
This record describes authored implementation and acceptance sources. No Cargo,
compiler, test, formatter, linter, build, script acceptance, or independent review
was executed by this writer. Compilation and runtime behavior remain unverified.

## Delivered contract

The operator selected “Require event streaming; reject unsupported peers before
dispatch.” `A2AClient::require_streaming_peer` retrieves the standard authenticated
`/.well-known/agent-card.json`, requires `capabilities.streaming == true`, and
requires an exact advertised `supportedInterfaces` URL with `protocolBinding:
JSONRPC` and `protocolVersion: 1.0`. URL parser canonicalization is permitted;
agent-path rewriting is not. `UnsupportedStreamingPeer` is a typed pre-dispatch
rejection. Host remote-child admission calls it before creating the child,
reserving capacity, or committing a durable child row. The task execution also
performs discovery before its first send, retaining discovery across waiter loss.
An existing exact-task continuation does not infer that its previous remote
context is closed when fresh capability discovery fails.

Outgoing sends use v1 `SendMessage`, a stable message ID within the retained send
future, camelCase v1 payloads, `ROLE_USER`/`ROLE_AGENT`, and
`configuration.returnImmediately: true`. Completion uses `SubscribeToTask` SSE
with `A2A-Version: 1.0`. The selected interface's optional tenant is echoed. The
250 ms `tasks/get` loop and the client-wide 30-second completion timeout are
removed. Root cancellation and explicit peer receipts govern settlement. A
pending send/cancel future is retained by the exact execution object if a waiter
is dropped. Subscription open/read futures are likewise retained across waiter
loss; parent cancellation may discard this read-only subscription before issuing
one retained `CancelTask`. No client reader is detached.

Every subscription starts with an exact Task snapshot; subsequent v1
`statusUpdate` and `artifactUpdate` payloads require matching JSON-RPC request,
task ID and context ID. Artifact replacement and append update the retained task.
A governed snapshot/delta must itself acknowledge the inherited contract.
Governed status events carry complete receipt metadata, allowing a later receipt
to clear earlier uncertainty. A terminal execution receipt does not imply that a
reusable peer context has been physically closed. Cleanup still requires an exact
cancel/reconciliation receipt and, for a governed peer, UAR's cleanup-closed marker.
Persisted cleanup now retains both task and context identity.

`StreamUnavailable` records premature EOF, stream error, missing initial snapshot,
unknown state, identity/contract mismatch, or lost stream continuity. It never
means successful completion. `reconnect_and_wait` is an explicit owner action:
one authoritative `GetTask` reconciliation of the same task, then subscription
if needed. It never replays a message. A failed cancel still permits one exact
status reconciliation; there is no autonomous read/retry/timer loop.

The JSON-RPC v1 and official gRPC v1 servers now implement both
`SendStreamingMessage` and `SubscribeToTask`. Both use
`A2AThreadService::subscribe_durable`: retained run and committed-work receivers
are installed before the authoritative task snapshot. A lifecycle scope admits
and registers the producer before execution, retains its result, and joins it on
shutdown. Receiver EOF and scope cancellation release the producer. An in-flight
durable projection is allowed to settle its CAS before producer exit. Event lag,
closed event sources, and unavailable active-run subscriptions close with an
unavailable/error result rather than fabricating a terminal state. Terminal or
interrupted snapshots close the stream; interrupted state is not completion.
The standard v1 AgentCard now advertises streaming for both advertised transports.
The distinct legacy AgentCard remains `streaming: false`.

The host's existing accepted-job registry is retained. A new oneshot start barrier
prevents a raw producer from running until its join handle has been published
under the same admission lock used by shutdown. The root host retains
`ThreadService::shutdown` and its actual join/remote cleanup obligations.

## Source ownership and interfaces

- `src/uar/api/a2a/client.rs`: discovery, typed rejection, negotiated v1 send,
  subscribe, cancellation and snapshot APIs; no fixed completion timeout.
- `src/uar/api/a2a/client_stream.rs`: new narrowly scoped v1 projection and owned
  bounded SSE decoder, included privately by the client; no dependency changes.
- `src/uar/api/a2a/task_execution.rs`: retained discovery/stream futures, explicit
  reconnect, exact receipt settlement, and authored cancellation/stream scenarios.
- `src/uar/api/a2a/thread_service.rs`: registered durable snapshot producer only;
  existing legacy task paths remain available.
- `src/uar/api/a2a/v1.rs`: JSON-RPC send/subscribe streams and streaming capability.
- `src/uar/api/a2a/grpc_v1.rs`: matching official gRPC streams over the same producer.
- `src/uar/runtime/thread/service.rs`: pre-reservation capability check, registered
  job start barrier, typed unsupported outcome, persisted cleanup context binding.
- `tests/test_a2a_client.rs`: existing client fixtures authored for v1 discovery,
  wrapped send receipts, v1 states and context fields.
- `tests/fixtures/dot18_a2a_real_path.py`: missing-task subscription now asserts the
  standard missing-task error and v1 streaming capability.
- `tests/fixtures/a2a_v1_approval_governance.py`: replaces obsolete unsupported
  streaming assertions with role-denied streaming/no-reservation assertions.
- `tests/fixtures/a2a_v1_streaming.py`: new installed-artifact JSON-RPC/gRPC streaming
  acceptance source for direct send and exact active-task subscription.
- This handoff records limits and the deferred production acceptance inventory.

## Configuration migration and uncomfortable limit

Existing `/a2a/compiler` and `/a2a/agents/<id>` trusted peer URLs are unsupported
outgoing legacy contracts. Merely changing a path string does not authorize
another agent. Configure the exact v1 URL actually advertised in discovery; this
UAR's current global card advertises `/a2a/v1/compiler` for `compiler-agent`.
The global card does not advertise arbitrary `/a2a/v1/agents/<id>` URLs. Those
named-agent outgoing configurations remain rejected until an authoritative
per-agent discovery contract is deliberately implemented. No hidden alias,
fallback to polling, or compiler-agent substitution was added.

An offline or silent peer can keep an accepted mutation/cleanup unresolved. The
client no longer turns a wall-clock deadline into lifecycle evidence. Unknown
creation is retained without replay; an authenticated remote receipt still
cannot attest to the internals of a third-party process. A terminal subscription
race may return unsupported before delivering a terminal snapshot; the owner must
explicitly reconcile that exact task. Streams are not durable event logs and
provide no replay cursor guarantee. Server producers react to committed snapshot
changes, rather than promising token-by-token output.

The only new resource guard is an 8 MiB unfinished SSE-event limit at the real
untrusted peer framing boundary. Capability, request/task/context identity and
contract guards enforce the operator's explicit streaming/admission requirement
and the existing authenticated delegation boundary. No unrelated business-budget
limits, dependency versions, retry policies or releases were changed.

## Deferred acceptance inventory

The authored task-execution scenarios cover: cancellation of multiple governed
children with exact task IDs, unsupported-peer rejection with zero dispatch,
terminal streaming with no status polling, early EOF retaining task/cleanup
uncertainty, and explicit snapshot/reconnect without duplicate send. These are
fixture-level scenarios, not phase acceptance evidence.

After the lead reconciles the entire phase ledger, publishes a new exact-source
implementation inventory, and obtains explicit operator authorization for exact
commands, acceptance must exercise the installed production path:

1. JSON-RPC and gRPC `SendStreamingMessage` produce an initial Task snapshot,
   matching updates/artifacts and an authentic terminal receipt. Capture complete
   wire events and installed artifact identity. No synthetic final status.
2. With a controlled provider held active, subscribe via each transport, observe
   the initial snapshot, then release the provider externally and observe terminal
   updates. Run `a2a_v1_streaming.py` in `subscribe` mode with the exact receipt.
3. Disable streaming/remove the exact interface/change protocol version in the
   peer card. Through the actual remote-child host admission, observe typed
   rejection before child rows, capacity reservation or peer message contact.
4. Drop a waiter during accepted send, stream opening, stream read and cancel;
   resume the retained object and prove one message mutation and exact cleanup.
5. Disconnect an active stream and inject event-channel lag/closure. Observe
   unavailable state with identity retained, then explicit exact-task reconciliation
   and subscription. Prove no automatic polling, resend or synthetic completion.
6. Verify wrong request ID/task ID/context ID/contract acknowledgement and unknown
   wire state cannot settle execution or release cleanup liability.
7. Race terminal commit with subscription installation. A terminal snapshot wins
   or the server rejects a terminal subscription; explicit reconciliation resolves
   the latter without sending another message.
8. Cancel parent/runtime with an active subscription. Record the lifecycle producer
   operation, receiver EOF and actual producer join, then exact remote cancel and
   UAR cleanup-closed receipt. An unresolved peer keeps shutdown unresolved.
9. Run the existing installed governance matrix with streaming role denial before
   reservation and both transports' unchanged approval projections.

No new verification command is authorized by this document. No fixture, build,
unit/module/integration test or conformance gate has been executed here.

## Protocol evidence consulted

The repository's existing `proto/a2a_v1.proto` is the implementation field contract;
no schema or dependency upgrade was introduced. Context7 resolved
`/websites/a2a-protocol` and returned the official specification's v1 method,
SSE-envelope, initial-snapshot, tenant, state and metadata definitions:
<https://a2a-protocol.org/latest/specification> and
<https://a2a-protocol.org/latest/definitions>.
The repository v1 adapter pins its normative source to
<https://github.com/a2aproject/A2A/tree/v1.0.0/specification>.
Firecrawl developer search also returned the official migration document at
<https://github.com/a2aproject/a2a/blob/1205f45951ca97c209634ce6046180e38c5dfb30/docs/whats-new-v1.md>,
which documents the renamed v1 streaming/subscription operations. These external
lookups establish the contract, not successful UAR interoperability.
