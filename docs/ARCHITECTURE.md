# Architecture Overview

## Chat Serving Path

UAR exposes an OpenAI-compatible chat interface with session continuity extensions.

### Endpoints

- `POST /api/chat/completion` (primary)
- `POST /v1/chat/completions` (compatibility alias)

Legacy routes are intentionally disabled (`404`):

- `/api/chat`
- `/api/chat/*` (except `/api/chat/completion`)
- `/api/sessions/*`

### Request/response mode

- `stream: false` → immediate `chat.completion` JSON response
- `stream: true` → SSE chunk stream (`chat.completion.chunk` + `[DONE]`)

### Session continuity

- If no session is provided, UAR creates an anonymous session.
- If client provides a session ID, it must be UUID format (`400` on non-UUID).
- Session ID is returned in `X-UAR-Session-ID` and persisted as `uar_session_id` cookie.
- Clients retain context by sending `X-UAR-Session-ID` on future turns.

## Runtime Flow

```text
Client
  POST /api/chat/completion
    - model
    - messages/message
    - stream
      |
      v
Axum Handler (src/server.rs)
  - resolves provider/model
  - resolves/creates session id
  - starts run via RunManager
      |
      v
RunManager
  - appends user turn to session
  - executes Orchestrator + tools
  - emits normalized run events
      |
      +--> stream=false: aggregate final assistant text -> chat.completion JSON
      |
      +--> stream=true: map run events -> chat.completion.chunk SSE
```

## Model Resolution Rules

1. `model` is plain (example `gpt-5.2`)
- validate only against default provider model catalog
- if unknown, return `404 Unknown model`

2. `model` is scoped (example `openai/gpt-5.2`)
- validate provider + model tuple
- if unknown, return `404 Unknown model`

## Route Contract

Supported client-facing chat routes:

- `POST /api/chat/completion` (primary)
- `POST /v1/chat/completions` (compatibility alias)
