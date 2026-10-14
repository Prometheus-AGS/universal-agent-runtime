# Provider stream completion

Streaming success requires the terminal evidence of the **actual selected wire
protocol**, followed by clean transport EOF. A model name does not select the
terminal contract by itself. Liter resolves its provider first; an explicit
`base_url` normally pins OpenAI-compatible chat, with the existing Anthropic,
Azure exceptions. The DashScope exception selects a multimodal embedding adapter
and does not qualify an additional chat protocol. This change does not change provider routing.

| Selected route | Required terminal evidence | Additional completion condition |
| --- | --- | --- |
| Liter OpenAI-compatible chat, including OpenAI, Azure, Mistral and registered compatible providers | Fully dispatched SSE `data: [DONE]` event | A parsed choice finish reason; clean framed EOF |
| Liter Anthropic Messages | JSON event `type: message_stop` | Parsed `message_delta` finish reason; clean framed EOF |
| Native Anthropic Messages (the default Anthropic driver) | Typed JSON `message_stop` event | Clean framed EOF; any provider error remains an error |
| Liter Google AI Gemini and Vertex Gemini | First candidate's nonempty `finishReason` | Parsed normalized finish reason; clean framed EOF |
| Liter Cohere v2 | `message-end` with nonempty `delta.finish_reason` | Parsed normalized finish reason; clean framed EOF |
| Liter Bedrock ConverseStream | CRC-valid EventStream `messageStop` frame with nonempty `stopReason` | Parsed finish reason; complete frame EOF, including optional metadata |

For SSE, an event is complete after its empty dispatch line. Incremental UTF-8
bytes, CRLF separators and multiple `data:` lines are reconstructed before JSON
parsing. Comments and non-data SSE fields retain SSE framing semantics. Empty
transport EOF without terminal evidence, a partial event/codepoint, invalid UTF-8,
malformed JSON, inconsistent Anthropic/Cohere event names, or data after a terminal
sentinel fails the model turn. Neither a finish chunk nor `content_block_stop`
substitutes for an Anthropic `message_stop` or an OpenAI `[DONE]` event.

The existing byte-buffer limit is reused. No retry, timeout, cancellation or tool
permission is added. The existing stream idle timeout still applies when a peer
sends its terminal event but does not close the transport. Dropping/cancelling the
run closes its owned stream; cancellation remains an aborted outcome, never a
successful completion. Dropping a stream is not evidence that a tool's external
effect stopped.

## API outcomes and tools

Driver framing/terminal failures are typed provider stream failures. The legacy
orchestrator emits `Error` and does not emit provider `Done`. Graph model turns
retain their failure and the host records `RunStatus::Error`; interrupted assistant
history keeps its existing `TurnInterrupted` marker. Successful graph checkpoint
edges must not run after a failed model turn.

Graph API `RunDone` is an established stream-close event and can follow `Error`.
Consumers must inspect `Error`, `Cancelled` and run status; the existence of
`RunDone` alone is not a success receipt. Explicit cancellation remains
`Cancelled`. A non-streaming title/simple collector also requires driver `Done`
and propagates normalized `Error`.

Tool argument deltas remain incremental. Liter assembles completed tool calls only
after protocol terminal and trailing framing validation. Gemini's native `STOP`
can finish emitted function calls; those calls retain their assembled identifiers,
names and JSON arguments. Native Anthropic may announce a tool at
`content_block_stop`, but both orchestrator paths wait for validated model completion
before dispatch. Incomplete/malformed streams cannot execute those calls. Existing
MCP application failures and uncertain post-dispatch effects retain their dot-04
classification and do not gain retry authority.

## Qualification

The uncomfortable thing: controlled endpoints prove framing and host outcomes,
not genuine provider inference or every service's proprietary extensions. The
new integration target is authored; until its acceptance gate actually runs,
compilation, TLS routing and runtime assertions remain **unverified**.

The target covers actual selected generic OpenAI-compatible, Azure, Liter/native Anthropic,
Google AI, Vertex, Cohere and Bedrock drivers. Google/Vertex/Cohere/Bedrock retain
their actual provider URLs and parsers. A trusted host-owned client routes their
HTTPS DNS to a random loopback port and trusts a fixture-generated CA; certificate
hostname validation remains enabled. No external provider request, TLS bypass,
ambient proxy or real provider credential is used. Vertex requires the explicit
synthetic fixture project/location in the command below. Bedrock uses explicit
synthetic access key, secret, session token and us-east-1 region; the command clears
its constructor-owned endpoint and cross-region overrides. No in-process environment
mutation is used.

The `DefaultClient::with_http_client` constructor is a trusted embedding-host seam.
It retains ordinary provider/config/authentication selection, request transforms
and parsing. The caller owns the already built client's DNS, TLS, redirect/proxy
policy, timeout and default headers. `ClientConfig` cannot retrofit transport or
default-header settings into that client. The normal `DefaultClient::new` path
keeps its existing transport policy. Agent kernels gain no client or write authority.

The fixture has 83 protocol/scenario rows (81 full-matrix rows plus Azure valid/EOF wiring controls) and enters each through the real legacy
orchestrator and local/shared durable graph management APIs. It checks valid
completion, one-byte UTF-8/CRLF/JSON splits, empty EOF, finish without terminal,
trailing partial events/frames, malformed JSON/CRC, invalid UTF-8, malformed tails,
valid tool continuation, incomplete tool streams with no effect, and cancellation.
SSE-only cases are omitted for binary EventStream. Graph admission/terminal
checkpoints, persisted session history, run status and canonical tool receipts
are read through the real persistence collaborator. No unit/mock-only substitute
is the acceptance evidence. The remaining compatible provider identities and
registered/schema aliases share the source-wired default protocol contract; they
are not each independently exercised. The embedding-only DashScope adapter is not
a chat completion route.

Run the **whole integration target**, once source is frozen and task 2 authorizes
acceptance, with the existing private durable SurrealDB infrastructure manifest
and a fresh isolated namespace:

```bash
env -u BEDROCK_BASE_URL -u BEDROCK_CROSS_REGION \
VERTEXAI_PROJECT=dot05-fixture \
VERTEXAI_LOCATION=us-central1 \
UAR_DOT05_INFRA_FILE=/absolute/private/infrastructure.json \
UAR_DOT05_NAMESPACE=dot05_fresh_namespace \
cargo test --locked --no-default-features --features server-full \
  --test stream_terminal_contract -- --nocapture --test-threads=1
```

The fixture requires installed `openssl` and `/usr/bin/python3`, creates synthetic
certificates in a private 0700 artifact directory, retains wire bytes/request
bodies and logs there, and kills only its owned HTTPS child. It prints only
scenario receipts and the private evidence path. Acceptance must retain the exact
command/output, source hashes and failure stderr. These endpoints do not certify
shared cross-process scheduling, recovery, genuine inference or release packaging.

Direct `ExternalLlmDriver` callbacks have only a normalized `Done` contract; UAR
cannot certify the external host's unseen wire protocol. Mock drivers are test
support. Responses API, realtime, non-chat Liter consumers, external SDK runtimes
and unsupported proprietary framing are outside this change's chat guarantee.
Do not advertise protocol-qualified success for these surfaces from this evidence.

The patch changes vendored Liter source without changing the approved 2.1.1 /
`a6047386cc9fae4258b4a8577f6a016022095586` dependency pin. Final release provenance
must record the vendor patch and effective source hashes in addition to that pin.
