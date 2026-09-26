# A2A team endpoint contract

Status: **Official Draft Specification 0.1.0-draft.1**. This proposes a new adapter for the released [A2A v1.0.1 specification](https://github.com/a2aproject/A2A/releases/tag/v1.0.1), using **wire protocol version `1.0`**. It does not certify existing RC-labelled UAR routes as compatible.

## Binding and discovery

The authoritative release sources are [v1.0.1 protocol definitions](https://github.com/a2aproject/A2A/blob/v1.0.1/specification/a2a.proto) and [versioned binding text](https://github.com/a2aproject/A2A/blob/v1.0.1/docs/specification.md). Their ProtoJSON mapping uses camelCase fields and named enums such as `ROLE_USER` and `TASK_STATE_WORKING`; older examples using `role: "user"`, lowercase task states or `message/send` methods are not this adapter's wire contract.

A team is an opaque standard agent with an AgentCard. Its `supportedInterfaces` advertises the exact absolute URL, `protocolBinding` (initially `JSONRPC`), and `protocolVersion: "1.0"`. If an interface declares `tenant`, the client sends that routing value on requests; it is not authentication or a claim of team membership. Cards describe public capabilities, skills and authentication requirements without exposing private members, prompts, tools or grants.

The endpoint registry returns each team's card URL. A service's `/.well-known/agent-card.json` may identify its default agent/registry; the server must not pretend that every team owns that one host-root discovery path. Public cards may require no authentication, but task access must use the configured authenticated boundary. Production interface URLs use HTTPS. Local-only loopback setup is a separately documented deployment condition, not a public network exception.

Clients send `A2A-Version: 1.0`. Missing version has the standard older-version interpretation; this new interface rejects unsupported versions rather than silently treating them as 1.0. Existing UAR legacy endpoints remain separately versioned and documented. They may not share a decoder that guesses the version from message contents.

## One task service, two kinds of clients

`SendMessage`, `SendStreamingMessage`, `GetTask`, `CancelTask` and `SubscribeToTask` operate on the same authoritative TeamTask service used by The Boss. A2A task IDs are stable external projections bound to owner, team and task. A2A context IDs group permitted interactions; they do not replace workspace identity, bind credentials or select a different tenant. UAR attempts can change while the external task ID remains stable.

An ordinary A2A client sees one agent doing one task, with status messages and artifacts. It needs no team extension to use the development or feedback skill. A remote client does not imply remote team-member execution; first-release members still run in one UAR instance.

Enhanced observation MAY negotiate extension URI `urn:prometheus:uar:team:1` through standard extension mechanisms. The AgentCard declares it optional (`required: false`). Namespaced metadata can contain authorized public task/team correlation and event cursors; it never grants member access or changes execution ownership. A request requiring unsupported extension semantics fails explicitly. Optional unsupported metadata cannot be used to smuggle commands. The extension does not add new core A2A task states.

## State mapping

| UAR condition | A2A `TaskStatus.state` | Meaning |
|---|---|---|
| queued, ready | `TASK_STATE_SUBMITTED` | Accepted; may wait for bounded admission |
| running | `TASK_STATE_WORKING` | An attempt is active |
| reconciling | `TASK_STATE_WORKING` | Safe status message says outcome needs reconciliation; no success claim |
| awaiting_input | `TASK_STATE_INPUT_REQUIRED` | Client input needed |
| awaiting_approval | `TASK_STATE_INPUT_REQUIRED` | Trusted approval needed; an A2A message cannot itself confer that grant |
| succeeded | `TASK_STATE_COMPLETED` | Authoritative successful terminal outcome |
| failed | `TASK_STATE_FAILED` | Authoritative failed terminal outcome |
| cancelled | `TASK_STATE_CANCELED` | Cancellation settled, not merely requested |
| admission refused before execution | `TASK_STATE_REJECTED` | Request cannot be performed under current contract |
| genuine authentication challenge | `TASK_STATE_AUTH_REQUIRED` | Authentication required; not an ordinary tool approval |

`TASK_STATE_UNSPECIFIED` is not used to disguise an uncertain effect. Cancellation request preserves the current state until execution/effects settle. A terminal task cannot be resurrected by a duplicate message or stream subscription; further work receives a new task according to the standard lifecycle. A changed attempt is not a changed authorization principal.

## Streaming, artifacts and reconnect

JSON-RPC streaming responses use SSE with each data body a JSON-RPC response whose `result` is one `StreamResponse` variant: `task`, `message`, `statusUpdate` or `artifactUpdate`. A status update contains `taskId`, `contextId` and `status`; this release does not use the old `final` discriminator. Artifacts carry stable `artifactId`, typed parts and explicit `append`/`lastChunk` at the artifact-update level. Artifact IDs are unique within the external task. No client should concatenate unrelated member results merely because they arrive consecutively.

Ordinary clients obtain current state with `GetTask`; `SubscribeToTask` starts with the current nonterminal task and subsequent updates. Subscription is not an arbitrary historical replay guarantee. A completed task is obtained with `GetTask`, not by pretending it is live to reopen the stream. Enhanced durable replay may use the UAR cursor extension, with current disclosure checks and explicit snapshot/reset semantics; absent negotiation, no cursor fields are injected into closed core objects.

The adapter preserves task IDs across reconnects and does not create a task merely because an SSE connection was lost. A repeated message ID within the authenticated task/owner scope returns its recorded mapping; changed content with the same ID is rejected as an idempotency conflict. This is UAR's declared adapter guarantee, not a universal exactly-once effect guarantee supplied by A2A. Outbound artifact access is authorized independently.

## Cancellation and approval

`CancelTask` attempts to cancel and returns the updated Task. It may return a still-working state while cancellation settles, or the standard not-cancelable/not-found error as appropriate. It must not return canceled while an external effect remains unknown. Canceling the transport alone does nothing to task state. An A2A client cannot cancel another owner's task by knowing its ID.

Feedback-to-issue may return input-required with a safe explanation and a trusted review destination. The specific repository, issue payload, actor, request revision and effect identity must be approved via the existing boundary. Sending text such as "approved" through A2A is ordinary input unless the authenticated trusted approval service separately records the exact authorization. If the external issue response is lost, UAR keeps the original effect identity and reconciles; it does not create a second issue because the client reconnected.

The uncomfortable case is a long-running operation whose effect is uncertain but whose client expects a quick cancel acknowledgment. This adapter prioritizes a truthful working/reconciliation status over a false terminal success or cancellation. The UI must make that state and available recovery actions visible.

Illustration: [ordinary client task, artifact and cancel](../traces/a2a-ordinary-client.jsonl). The trace labels HTTP/header setup as fixture context; JSON-RPC objects are the proposed actual wire payloads. No private team internals appear in that ordinary-client projection.
