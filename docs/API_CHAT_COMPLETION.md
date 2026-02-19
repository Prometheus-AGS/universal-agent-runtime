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
- `stream_mode` (`openai` default, `agui`, or `dual`)

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
- `stream_mode: "agui"`: AG-UI named SSE events (`agui.*`) only.
- `stream_mode: "dual"`: emits both AG-UI events and OpenAI chunks.

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
  http://localhost:6565/api/chat/completion)
```

Extract `X-UAR-Session-ID` from response headers.

### Turn 2 (same session)

```bash
curl -sS -H 'content-type: application/json' \
  -H 'X-UAR-Session-ID: <session-id>' \
  -d '{"model":"gpt-5.2","message":"Another one.","stream":false}' \
  http://localhost:6565/api/chat/completion
```

## Route Contract

The supported chat-completion routes are:

- `POST /api/chat/completion` (primary)
- `POST /v1/chat/completions` (alias)

Legacy routes are intentionally disabled (`404 Not Found`):

- `/api/chat`
- `/api/chat/*` (except `/api/chat/completion`)
- `/api/sessions/*`
