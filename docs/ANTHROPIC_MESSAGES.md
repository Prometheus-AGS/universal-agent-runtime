# Anthropic-compatible Messages endpoint: agent mode

`POST /v1/messages` accepts the Anthropic Messages request shape and runs it
through a UAR agent. It uses the same run manager as `POST /v1/chat/completions`.
Agent tools, policy, budgets, run history and usage accounting apply to both.

This page describes the behavior as of 1.4.0-rc.2. The design history of the
native Anthropic driver is in [ANTHROPIC.md](ANTHROPIC.md).

## Every request runs an agent

There is no separate "direct provider" mode. Every request runs a registered
agent, selected in this order:

1. `agent_id` in the request body.
2. The `agent_id` in the conversation policy stored for `session_id`.
3. `default-agent`.

An unknown agent is refused with the status its resolver returns. The agent and
model are fixed when the run is admitted. Switching agent or model in the middle
of a run is not supported. To change either, send a new request with a different
`agent_id` or `model`.

Bound agent targets (`uar:agent:*`) are refused with 400 on this route. They
need a workspace binding (`x-uar-workspace-id`), which only
`/v1/chat/completions` resolves.

## Model

`model` is either `provider/model`, naming a configured provider, or a bare
model id on the default provider. An unknown provider, an empty model, or a
missing default provider is refused with 422. The caller must be authorized for
that model, as on the chat completions route.

## Identity

The route needs an authenticated owner, normally supplied by the identity proxy
in front of UAR. Without one, it returns 401. Runs and caller-tool state are
scoped to that owner.

## Request fields

UAR accepts the Anthropic fields it maps: `messages`, `system`, `tools`,
`tool_choice`, `max_tokens`, `stream`, and the UAR extensions `agent_id`,
`session_id`, `parallel_tool_calls` and `prompt_caching_enabled`. Requests are refused with 400 when:

- they contain any other top-level field;
- a tool definition has fields other than `name`, `description`,
  `input_schema` and `strict`;
- `tool_choice.disable_parallel_tool_use` contradicts `parallel_tool_calls`.

`max_tokens` caps the output of each model call. It also sizes the run's output
budget reservation, so a capped request on a large-context model is not refused
for the model's full output window.

## Response and streaming

Without `stream`, the response is a single Anthropic `message` object. With
`stream: true`, UAR sends Anthropic server-sent events in this order:
`message_start`, then `content_block_start`, `content_block_delta` and
`content_block_stop` for each text or `tool_use` block, then `message_delta`,
then `message_stop`.

- `stop_reason` is `tool_use` when the agent asks the caller to run a tool,
  otherwise `end_turn`.
- `usage.input_tokens` reports uncached input; `usage.output_tokens` reports
  output. Known `cache_read_input_tokens` and `cache_creation_input_tokens` are
  separate input portions. Shared runtime/OpenAI input counts include those
  portions, so adding Messages uncached input plus its known cache portions
  matches the aggregate input. When the run reports usage, the successful Chat
  stop chunk carries the nonstreaming Chat `usage` object before `[DONE]`.
  The projection never charges a call again.
- Unavailable input/output retain numeric zero for compatibility, accompanied
  by sibling `uar_usage.input_tokens` and `uar_usage.output_tokens` objects with
  `available:false, source:"unavailable"`. Provider-reported zero has
  `available:true, source:"provider_reported"`. Unknown cache counters are
  omitted. Initial `message_start.message` carries unavailable placeholders;
  final `message_delta` carries `usage` and sibling `uar_usage`, as does the
  non-streaming response. No token estimate is relabelled provider usage.
- Cache breakdowns use checked arithmetic. An invalid breakdown is unavailable,
  never wrapped or clamped into apparent uncached usage. The native provider
  decoder emits no usage for an entirely absent report; a partial native report
  without input or final output refuses with `provider_usage_incomplete`, since
  the existing normalized numeric counter contract cannot represent those
  missing aggregate values. Counter overflow likewise refuses explicitly.

Run aggregates cover every actual provider submission in the run, including
tool-loop calls and submitted retry/fallback attempts. Starting an iteration or
refusing a request before dispatch does not establish a provider call. If one
submitted call supplies no usage, the run's input, output, total and cache
aggregates are unavailable even when another call reports counts. Known reports
remain retained by their original calls; an unavailable aggregate does not erase
them or settle a missing call's liability. Repeated cumulative reports within a
call are one report sequence, not extra calls.

Cache availability is independent per counter. A missing cache-read count makes
the aggregate cache-read count unavailable without erasing known cache-creation
or input/output counts. Native Anthropic uncached input is retained independently
of normalized input, so mixed cache availability cannot relabel a known cached
portion as uncached. A known zero remains available zero. These are source-authored
contracts; the current candidate has not been qualified by the closed gate.

OpenAI responses retain aggregate `usage.prompt_tokens` with optional
`prompt_tokens_details.cached_tokens` and UAR's `cache_creation_tokens` detail.
Unknown details are omitted; measured zero remains present. The same shared
per-call accounting ledger owns settlement for both protocols. Missing usage
leaves liability pending and never proves a call was free.

## Image content boundary

User `image` blocks require exactly `source:{type:"base64",media_type,data}`
with inline PNG/JPEG bytes. The host validates base64, MIME/signature, full decode,
the existing 10 MiB encoded/envelope bound, at most eight images and bounded
decoder allocation before compatibility conversion or provider dispatch. Valid
mixed text/image order is preserved. This extends no remote URL retrieval.
Missing/malformed sources, URLs, unsupported MIME/blocks, assistant/system images,
image-bearing tool results and unsupported block fields refuse explicitly instead
of disappearing behind a successful text answer. Existing text/caller-tool
conversion remains. This conversion boundary is source-authored and unqualified
on the current artifact; it is not a vision-understanding promotion claim.

## Caller tools

When the agent calls a tool that the caller declared in `tools`, the response
ends with `stop_reason: tool_use` and a `tool_use` block. Send the
`tool_result` in the next request with the same `session_id`. A result that
doesn't match the pending call's session is refused with 409.

## Cancellation and errors

A run that is cancelled inside UAR, or fails after it starts, ends a streamed
response with an `error` event of type `api_error`. Without streaming, the same
failure returns 502. Request refusals use the Anthropic error body
`{"type":"error","error":{"type":"invalid_request_error","message":…}}` with
the status given above.

## Parity with chat completions

For the same agent, model, prompt and tools, `/v1/messages` and
`/v1/chat/completions` run the same agent turn. Only the wire format differs.
`scripts/behaviors/features/anthropic-messages-parity.feature` checks the answer
text and a caller tool call against the keyless stub model. The stub reports no
token usage, so that feature does not compare non-zero usage counts.

Its tagged `@messages-usage` scenarios now author a separate controlled-provider
qualification using `scripts/behaviors/messages-usage.cucumber.mjs`. They cover
known nonzero/cache, actual zero, absent reports and unavailable cache breakdowns
in both wire forms; the real per-run ledger must show one settlement per provider
call that reports priced terminal usage. Genuine two-call tool-loop cases submit
one public request per protocol, with no caller-declared tools or caller-result
continuation. The provider requests the existing runtime-owned `native_echo`,
then accepts the second call only when it contains the original tool-use id,
exact arguments and successful real echo result. Both streaming variants require
exactly two provider arrivals in that single run, one successful governed native
tool completion, a gap-free owned terminal run snapshot, two original accounting
call identities and stable ledger rereads. Missing→known and known→missing leave
aggregate usage unavailable while retaining the known call's settled evidence
and the missing call's pending liability. The same loop supplies all-known,
known-zero and independently missing cache-read/cache-creation controls.
Cancellation cases hold the actual provider before output, mid-output and
before a caller-tool handoff becomes public, cancel the exact owned run through
the mounted API, and require producer closure without later success. A positive
caller-result continuation is replayed verbatim and attempted under an externally
provisioned foreign owner; neither may create another provider call or settlement.
Unsupported image cases require a working text control, then an explicit
content-boundary refusal with no new provider arrival. All cases are authored,
not executed, while the full-phase verification gate is closed.

The external host starts the bounded keyless
`scripts/acceptance/fixtures/messages-usage-peer.mjs` collaborator separately and
configures a dedicated native Anthropic provider/agent on the exact candidate.
Importing that module starts nothing. The functional runner uses only
`UAR_BASE_URL`, `UAR_WORKSPACE_ID`, and a local `UAR_MESSAGES_USAGE_FIXTURE` JSON
manifest; it neither starts UAR nor installs/sends credentials. Every endpoint is
literal loopback with no URL credentials. The manifest supplies `dedicated:true`,
`namespace`, `identity:{platform:"linux/amd64",sourceCommit,imageDigest,configDigest}`,
the scoped `agentId`, actual configured `model`, `adapter:"native_anthropic"`,
peer `controlUrl`, and `foreignOwnerBaseUrl`. The host independently establishes
serving source/image/config and distinct owner identities; manifest syntax alone
is not that evidence.

For the genuine loop cases, the manifest additionally selects
`runtimeToolProfile:"native_echo"`. The externally owned candidate must actually
register/expose that existing BuiltIn/ReadOnly tool, permit it for the dedicated
agent through the agent tool policy and Cedar, and provide the existing durable
checkpoint/canonical-receipt path required by an owned tool run. The runner does
not install policies, modify the agent, bypass approval, start services or supply
authentication. The name is grounded in
`src/uar/runtime/native_skills/echo.rs` and its built-in registration; live profile
availability, serving artifact selection and provisioning remain unverified.
Absent prerequisites cause failure, never a caller-tool substitute or a skipped
assertion.

Namespace-bound peer controls are `GET /__messages/state`,
`POST /__messages/control` with `{namespace,mode}`, and
`POST /__messages/release` with `{namespace}`. Modes are `nonzero`, `zero`,
`absent`, `uncached-only`, `tool`, `before-output`, `mid-output`, `tool-handoff`.
Two-call modes are `loop-missing-known`, `loop-known-missing`, `loop-known`,
`loop-zero`, `loop-missing-read` and `loop-missing-creation`. Configuring a mode
resets its sequence. Call one emits `native_echo` with
`{message:"<namespace>-governed-echo"}`; call two requires the same id and
`{echoed:{message:"<namespace>-governed-echo"}}`, then emits the final marker.
An additional call or missing/mismatched real result refuses explicitly. Request
summaries retain only sequence/id/declaration/result-match evidence and counters;
they do not retain prompt, tool-result contents or headers.
Known native oracle: uncached7/read3/create2/output5, so aggregate input12 and
total17. The `uncached-only` oracle reports input7/output5 with cache unavailable.
The loop's second known call reports uncached11/read17/create19/output13, so
all-known totals are normalized input59/output18/total77, uncached18/read20/create21.
Known-zero reports all counters as zero on both calls. Missing-read on call one
keeps normalized input56/output18/total74, uncached18/create21 and unavailable
aggregate read; missing-creation keeps input57/output18/total75,
uncached18/read20 and unavailable aggregate creation. Missing→known retains
the second call's token60 ledger evidence; known→missing retains the first
call's token17 evidence. In both cases whole-run counters remain unavailable.
The peer retains at most128 request summaries, never prompt/tool-result contents
or headers, and owns each held producer until closure or its fixed25-second
fixture deadline. Deadline expiry is not cleanup acceptance. The runner requires
actual zero active/held producers, exact run status/events and safe accounting
observations. Receipts omit raw protocol bodies, owner credentials and memory
facts. Existing stub receipts keep their original source/profile provenance.

The external fixture must supply the existing pricing/ledger prerequisites for
known settlement, without any paid provider path. Absent usage and interrupted
calls retain pending accounting; cleanup does not reconcile them as zero or
erase their audit rows. Only owned sessions/runs are cleaned up. An uncertain
producer/cleanup outcome poisons further cases until external reconciliation.

## Not covered

- Workspace-bound agent targets (`uar:agent:*`): use `/v1/chat/completions`.
- Mid-run agent or model switching.
- Anthropic fields UAR doesn't map, such as `metadata`, `stop_sequences` and
  `thinking`, which are refused rather than ignored.
- The generic `client_capture_failed` branch has no public fixture switch. It
  already projects a generic reference and writes a scrubbed cause at the trusted
  private factory boundary. This packet does not widen production admission to
  inject failures, and that narrow branch remains explicitly unverified. No
  current-candidate log-privacy or capture-failure pass is claimed.
