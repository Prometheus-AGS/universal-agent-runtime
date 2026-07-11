# Chat Completion Protocol

This document defines the canonical client protocol for chat in UAR.

## Primary Endpoint

- `POST /api/chat/completion`

Compatibility alias:

- `POST /v1/chat/completions`

Both endpoints use the same request/response behavior.

## Request Shape

OpenAI-compatible request body (UAR also accepts `message` for convenience):

```json
{
  "model": "gpt-5.2",
  "messages": [
    { "role": "user", "content": "Tell me a joke." }
  ],
  "message": "Tell me a joke.",
  "temperature": 0.2,
  "tools": [],
  "stream": false
}
```

Supported fields:

- `model`
- `messages`
- `message` (UAR convenience alias)
- `temperature`
- `tools`
- `stream`
- `stream_mode` (`openai` default, `agui_spec`, deprecated `agui`, or `dual`)

## Session Semantics

The request does not require a session ID.

If no session is provided, UAR generates an anonymous session and returns it.

If the client provides a session ID, it must be a valid UUID. Non-UUID values return:

- `400 Bad Request`
- message: `session_id must be a valid UUID`

### Send Session Back (for context continuity)

Preferred:

- Request header: `X-UAR-Session-ID: <session-id>`

Also accepted:

- Request body: `session_id`
- Cookie: `uar_session_id`

If session is not sent back, each request is treated as a new conversation.

### Session Returned by Server

Every successful response includes:

- Response header: `X-UAR-Session-ID`
- Cookie: `uar_session_id=...`
- Non-streaming JSON body: `session_id`

## Model Resolution Rules

### 1) Model only (`"gpt-5.2"`)

- Resolves against the default provider only.
- If default provider does not support the model:
  - `404 Not Found`
  - message: `Unknown model`

### 2) Provider/model (`"openai/gpt-5.2"`)

- Resolves provider and model explicitly.
- If provider or model not found:
  - `404 Not Found`
  - message: `Unknown model`

Error body:

```json
{
  "error": {
    "message": "Unknown model",
    "type": "invalid_request_error",
    "param": "model",
    "code": "model_not_found"
  }
}
```

## Non-Streaming (`stream: false`)

Returns OpenAI-style completion JSON plus `session_id`:

```json
{
  "id": "chatcmpl-...",
  "object": "chat.completion",
  "created": 1771515606,
  "model": "openai/gpt-5.2",
  "choices": [
    {
      "index": 0,
      "message": { "role": "assistant", "content": "..." },
      "finish_reason": "stop"
    }
  ],
  "usage": null,
  "session_id": "..."
}
```

## Streaming (`stream: true`)

Returns SSE with OpenAI-style chunk payloads and `[DONE]` terminator.

Streaming modes:

- `stream_mode: "openai"` (default): OpenAI chunk payloads only.
- `stream_mode: "agui_spec"`: conformant [`uar.agui/1`](protocols/ag-ui-profile.md) events using official AG-UI vocabulary.
- `stream_mode: "agui"`: deprecated legacy UAR events (`agui.*`); not AG-UI conformant.
- `stream_mode: "dual"`: emits both AG-UI events and OpenAI chunks.

Tool chunk behavior:

- In `openai`/`dual` mode, tool calls are emitted as OpenAI `choices[].delta.tool_calls` chunks.
- In `openai`/`dual` mode, tool execution replies are emitted as UAR extension chunks at `choices[].delta.tool_results`.
- In `openai`/`dual` mode, activated skills are emitted as UAR extension chunks at `choices[].delta.skills` with `selection_method`.
- In `openai`/`dual` mode, context actions are emitted as UAR extension chunks at `choices[].delta.context_updates` with `strategy`.
- In `agui`/`dual` mode, tool lifecycle is emitted as `agui.tool_call.delta`, `agui.tool_call.complete`, and `agui.tool_result`.
- In `agui`/`dual` mode, skills/context lifecycle is emitted as `agui.skill.activated` and `agui.context.update`.

### OpenAI Stream Delta Extensions

When `stream_mode` is `openai` or `dual`, UAR may include the following delta extensions:

#### `choices[].delta.skills`

```json
{
  "skills": [
    {
      "id": "skills.weather",
      "title": "Weather Skill",
      "selection_method": "skill_service.keyword"
    }
  ]
}
```

Field definitions:

- `id`: Skill ID (`skill.skill_id`).
- `title`: Human-readable skill title.
- `selection_method`: How selection was made. Current values include:
  - `skill_service.keyword`
  - `skill_service.embedding`
  - `skill_service.local_embedding`
  - `skill_service.llm`
  - `skill_service.hybrid`
  - `legacy_classifier.rules`
  - `legacy_classifier.tfidf`
  - `legacy_classifier.wasm`
  - `legacy_classifier.local_embedding`
  - `legacy_classifier.llm`
  - `legacy_classifier.hybrid`
  - `legacy_fallback.tag_vector_hybrid`

#### `choices[].delta.context_updates`

```json
{
  "context_updates": [
    {
      "strategy": "sliding_window",
      "messages_removed": 4,
      "tokens_saved": 1800,
      "was_applied": true,
      "summary_generated": false
    }
  ]
}
```

Field definitions:

- `strategy`: Context strategy. Current values:
  - `sliding_window`
  - `progressive_summarization`
  - `hierarchical_memory`
  - `keep_first_last`
  - `none`
- `messages_removed`: Number of messages removed/truncated.
- `tokens_saved`: Estimated tokens saved by context management.
- `was_applied`: Whether a strategy was actually applied.
- `summary_generated`: Whether summarization was generated as part of the action.

### AG-UI Skill/Context Events

When `stream_mode` is `agui` or `dual`, equivalent events are emitted as named SSE events:

- `event: agui.skill.activated`

```json
{
  "kind": "skill",
  "phase": "activated",
  "request_id": "run-id",
  "skill": {
    "id": "skills.weather",
    "title": "Weather Skill"
  },
  "selection_method": "skill_service.keyword"
}
```

- `event: agui.context.update`

```json
{
  "kind": "context",
  "phase": "update",
  "strategy": "sliding_window",
  "messages_removed": 4,
  "tokens_saved": 1800,
  "was_applied": true,
  "summary_generated": false
}
```

### Client Parsing Guidance

- Treat `choices[].delta.skills` and `choices[].delta.context_updates` as optional extension arrays.
- Parse extension deltas incrementally like `content` and `tool_calls`.
- Do not assume one skill/context chunk per response; multiple can appear.
- Treat `selection_method` and `strategy` as enums that may grow in future versions.

Event delivery guarantee:

- `/api/chat/completion` replays buffered run history before live events, so early skill/context events are not dropped.

Example stream:

```text
data: {"id":"chatcmpl-...","object":"chat.completion.chunk","created":...,"model":"openai/gpt-5.2","choices":[{"index":0,"delta":{"role":"assistant"}}]}

data: {"id":"chatcmpl-...","object":"chat.completion.chunk","created":...,"model":"openai/gpt-5.2","choices":[{"index":0,"delta":{"content":"Hello"}}]}

data: {"id":"chatcmpl-...","object":"chat.completion.chunk","created":...,"model":"openai/gpt-5.2","choices":[{"index":0,"delta":{},"finish_reason":"stop"}]}

data: [DONE]
```

## Multi-turn Example

### Turn 1

```bash
RESP=$(curl -sS -i -H 'content-type: application/json' \
  -d '{"model":"gpt-5.2","message":"Tell me a joke.","stream":false}' \
  http://localhost:1906/api/chat/completion)
```

Extract `X-UAR-Session-ID` from response headers.

### Turn 2 (same session)

```bash
curl -sS -H 'content-type: application/json' \
  -H 'X-UAR-Session-ID: <session-id>' \
  -d '{"model":"gpt-5.2","message":"Another one.","stream":false}' \
  http://localhost:1906/api/chat/completion
```

## Route Contract

The supported chat-completion routes are:

- `POST /api/chat/completion` (primary)
- `POST /v1/chat/completions` (alias)

Legacy routes are intentionally disabled (`404 Not Found`):

- `/api/chat`
- `/api/chat/*` (except `/api/chat/completion`)
- `/api/sessions/*`
