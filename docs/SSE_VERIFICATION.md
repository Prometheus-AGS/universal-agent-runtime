# SSE Verification

This document verifies streaming behavior for the OpenAI-compatible chat endpoint.

## Endpoint

- `POST /api/chat/completion`
- Request includes `"stream": true`
- Response is `text/event-stream`

Compatibility alias:

- `POST /v1/chat/completions`

## Expected SSE Shape

Stream emits OpenAI-style chunk payloads as `data:` lines, followed by `[DONE]`.

```text
data: {"id":"chatcmpl-...","object":"chat.completion.chunk","choices":[{"index":0,"delta":{"role":"assistant"}}]}

data: {"id":"chatcmpl-...","object":"chat.completion.chunk","choices":[{"index":0,"delta":{"content":"Hello"}}]}

data: {"id":"chatcmpl-...","object":"chat.completion.chunk","choices":[{"index":0,"delta":{},"finish_reason":"stop"}]}

data: [DONE]
```

## Session Header Requirement

Streaming responses include:

- `X-UAR-Session-ID: <session-id>`

Clients should persist and resend this header for multi-turn context.

## Manual Verification Commands

```bash
# Streaming verification
curl -i -N -sS --max-time 20 \
  -H 'content-type: application/json' \
  -d '{"model":"gpt-5.2","message":"Say exactly: stream-ok","stream":true}' \
  http://localhost:6565/api/chat/completion
```

Success criteria:

- `HTTP/1.1 200 OK`
- `content-type: text/event-stream`
- `X-UAR-Session-ID` header present
- one or more `chat.completion.chunk` payloads
- terminal `data: [DONE]`

```bash
# Non-streaming verification
curl -i -sS \
  -H 'content-type: application/json' \
  -d '{"model":"gpt-5.2","message":"Say exactly: completion-ok","stream":false}' \
  http://localhost:6565/api/chat/completion
```

Success criteria:

- `HTTP/1.1 200 OK`
- JSON body object `chat.completion`
- `X-UAR-Session-ID` header present
- `session_id` in response body
