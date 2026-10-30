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
- `usage.input_tokens` and `usage.output_tokens` report the run's totals from
  the model calls.

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

## Not covered

- Workspace-bound agent targets (`uar:agent:*`): use `/v1/chat/completions`.
- Mid-run agent or model switching.
- Anthropic fields UAR doesn't map, such as `metadata`, `stop_sequences` and
  `thinking`, which are refused rather than ignored.
