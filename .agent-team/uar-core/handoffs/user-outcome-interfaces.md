# Task2.4 nonbrowser interface and SDK source handoff

2026-10-25. Phase uar-post-lifecycle-verification-readiness, Execute.
Verification CLOSED. No import, test, check, compiler, formatter, linter, build,
provider call, service, generated client, SDK, or acceptance runner was executed.
No subagents dispatched. Other writers' edits preserved. Root owns integration
and shared core after the task2.1 handoff.

## Owned changes

- scripts/acceptance/protocol_clients.py: real mounted protocol payloads and
  terminal parsing; native run/effective-agent/model/durable-accounting/genuine
  endpoint proof; concrete frozen-program execution with exclusive private
  receipt file; no shell, runtime creation or fake provider.
- scripts/acceptance/cases_interfaces.py:60 concrete U05 case IDs; each requires
  three repetitions, both backends, a fresh private fact, two real turns, and
  default/saved custom-agent selection. Custom prompt marker exists only in its
  saved artifact, not the model user message. Source clones the actual default
  schema, sets real provider policy, persists and selects the new ID.
- sdks/python/examples/user_outcome_acceptance.py: actual Client.chat and
  Client.stream_chat, owner bearer credentials, fixed UUID session and persisted
  agent_id, strict useful answer and successful terminal; exclusive JSON receipt.
- sdks/typescript/examples/user-outcome-acceptance.ts: actual UarClient.chat
  complete/stream, bounded real fetch, abort signal, agentId/sessionId, fixed
  result predicates; exclusive JSON receipt. No crafted replacement HTTP path.
- sdks/rust/examples/user_outcome_acceptance.rs: actual Client.chat complete/
  stream, real reqwest timeout, generated SDK request type with agent_id/session,
  strict output predicates and exclusive synced receipt.
- tests/user_outcome_grpc.rs: real installed-host legacy and v1 generated tonic
  clients, authenticated metadata x-uar-agent-id, unary/stream and actual task
  continuation. No in-process server, local DB, test driver or missing-env skip.
  Auth/endpoint/config failures fail. Companion owner-authenticated HTTP task
  lookup supplies native run/context metadata absent in legacy protobuf. The
  central Python case then verifies actual native run and provider receipts.
- sdks/typescript/src/types.ts and src/client.ts: root explicitly approved the
  exact additional ownership for optional ChatRequest.agentId and serialization
  as agent_id in chat.complete/chat.stream only. Existing optional API behavior
  and fields remain; no dependency/version edit. TypeScript review is deferred.
- this handoff. No unrelated production changes, canonical edits or guards.

## Case denominator

`cases_interfaces.cases()` expands these profile strings into
`U05.<profile with colons replaced by dots>.<default|custom>`:

| Profiles | Modes | Agents | Cases |
| --- | --- | --- | --- |
| chat (mounted /api/chat/completion) | json, openai, agui, dual, agui_spec | default/custom |10 |
| openai (/v1/chat/completions) | json, openai, agui, dual, agui_spec | default/custom |10 |
| native (/api/uar/runs + actual run stream) | agui, agui_spec | default/custom |4 |
| anthropic (/v1/messages) | json, stream | explicit default/custom |4 |
| a2a_legacy named agent JSON-RPC | json send + bounded exact task polling | default/custom |2 |
| a2a_v1 named agent JSON-RPC | json SendMessage, stream SendStreamingMessage | default/custom |4 |
| acp configured path | json runs/create + native completion stream, actual /stream normalized events | default/custom |4 |

Additional IDs U05.anthropic.json.implicit and
U05.anthropic.stream.implicit require root's new no-agent -> default-agent bound
behavior and still verify actual default-agent/native model/context receipts.

For each of python_sdk, typescript_sdk, rust_sdk, grpc_legacy and grpc_v1, IDs
`U05.<name>.<json|stream>.<default|custom>` add20 cases. Total60 cases,360 concrete
invocations after three repetitions on two backends, with two genuine task turns
per invocation plus the configure_provider genuine verification inference. Freeze
actual allowance/call consumption rather than reuse the example fixture caps.

AG-UI direct agent routes, browser/installed UI, rendered A2UI, management/MCP/
advanced controllers and full-harness paths remain separately owned by root/their
specific families. These60 rows do not claim those interfaces were exercised.
The compiler-only A2A route cannot prove default-agent parity; named agent routes
are used. No unsupported CLI chat path is invented.

Source-defined compatibility limits: legacy A2A JSON-RPC dispatch has only
message/send, tasks/get and tasks/cancel, no message/stream. Legacy tonic
MessageStream explicitly produces one status snapshot, not a live text stream.
The legacy gRPC stream case consumes that actual snapshot and uses exact TaskGet
to observe useful successful completion; it makes no token-streaming claim. If a
support claim requires incremental legacy streaming, that is a pending production
contract gap, not a synthetic successful client scenario. ACP has no dispatched
cancel method; ordinary sessions are process ephemeral, so these are warm
continuation rows, not cold durable ACP recovery claims.

## Program contract and root integration

Backend `clients` maps python_sdk/typescript_sdk/rust_sdk/grpc_legacy/grpc_v1 to
exact argv lists for the concrete programs above. Every executable, script,
compiled installed client binary/config and source module belongs in frozen_files.
The generic core fixture validator must also validate this map; run_program
already rejects nonfrozen executable/file args. `grpc_endpoint` must identify the
actual owned listener (root should admit loopback/exact advertised identity).
No compiler/build command should appear inside a client argv; build programs at
the approved build boundary, then execute the frozen bytes serially.

Root should register the new user_outcome_grpc Cargo test target with
required-features=[a2a-transport] consistent with existing test_a2a_grpc. Its exact
named test is installed_grpc_user_outcomes. The client argv should execute the
built test artifact with that exact test; omission/filter mismatch produces no
receipt and fails. Feature omission is never accepted as endpoint success.

run_program supplies UAR_ACCEPTANCE_CLIENT_INPUT JSON containing base_url,
grpc_endpoint, authorization_env, agent_id, model, session_id, mode, turns[2],
expected[2], timeout_seconds and max_bytes. It also supplies an exclusive
UAR_ACCEPTANCE_CLIENT_OUTPUT path. grpc names additionally receive
UAR_ACCEPTANCE_GRPC_VERSION=legacy|v1. Each program writes PASS only after useful
answer and protocol success, with turns containing content, native run_id and
session_id or context_id/task_id, protocol_terminal=success. SDKs use the provided
bearer reference, not a hardcoded credential. Root redacts and retains the actual
receipt and bounded failure message before temporary output cleanup. Failure
status/nonzero process/no receipt never yields PASS.

Root's runtime inference observer contract is reused exactly. Run identity,
owner-scoped inspection/accounting, actual effective model+saved artifact
revision, settled observed usage and genuine endpoint configuration are required.
Anthropic msg_UUID is not a native run ID; the adapter uses its unique owned
session and owner-scoped run listing to correlate exactly one new native run per
turn. Anthropic carries supported caller transcript plus session binding; other
session protocols use actual same session/task context. No caller transcript is
manufactured from expected answers.

## Deferred and uncomfortable

No compilation/syntax/import/runtime/API compatibility is claimed. Actual
provider verification cost, request/model limits, executable profile paths,
backend ownership, gRPC installed artifact, TypeScript SDK Node streaming
behavior and HTTP protocol terminals remain unverified. An actual SDK stream
may expose a shipped SDK/runtime defect; failure is retained, never replaced
with crafted HTTP to hide it. All first-failure and supporting gate behavior
remains under root's exact operator packet. The whole-phase gate stays CLOSED.

Named guards address the plan's observed false-positive risks: RUN_ERROR/error/
cancel cannot pass a positive stream, missing clients/credentials/listeners
fail closed, provider/model/agent labels require actual run evidence, and useful
answers require independently computed fresh facts plus private saved-prompt
markers. Exact argv/frozen files, referenced credentials, private temporary
output and awaited owned clients are real trust/cleanup boundaries. No
speculative runtime feature, dependency or broad refactor was added.
