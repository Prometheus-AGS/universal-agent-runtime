---
sidebar_position: 2
title: HTTP Compatibility
description: UAR, OpenAI-compatible, and Anthropic-compatible HTTP boundaries.
source_records:
  - docs/API_CHAT_COMPLETION.md
current_authority: /docs/protocols/http-compatibility
---

# HTTP compatibility

## Boundary statement

**Compatibility is an adapter contract, not provider ownership or total
upstream parity.** All three HTTP entrances resolve a configured model, call the
shared UAR provider/runtime path, and return the vocabulary selected by the
client-facing adapter.

## Routes

| Adapter | Route | Principal behavior |
|---|---|---|
| UAR chat | `POST /api/chat/completion` | OpenAI-shaped request with UAR session and stream extensions |
| OpenAI compatibility | `POST /v1/chat/completions` | same chat handler under the OpenAI route |
| OpenAI model discovery | `GET /v1/models`, `GET /v1/models/{model_id}` | configured-provider model views |
| Anthropic compatibility | `POST /v1/messages` | Anthropic message/tool input translated into UAR execution and translated back |

The legacy `/api/chat` and `/api/sessions` families are disabled routes. New
clients use the routes above or the governed run API.

## Model addressing

A bare model name resolves against the configured default provider. A
`provider/model` value selects the provider explicitly. Unknown provider or
model references fail rather than silently selecting a different provider.
OpenAI-shaped input therefore does not mean the request uses OpenAI.

Provider configuration and credentials are separate. See
[Configure providers](../providers/configuration.md) and
[Credentials](../security/credentials.md).

## Authentication and sessions

When JWT authentication is required, protected calls carry
`Authorization: Bearer <token>`. UAR creates trusted user and tenant context
only after verification. Health and discovery exceptions are documented in
[Authentication](../security/authentication.md).

For chat, `X-UAR-Session-ID` is the preferred continuity header. The body and
cookie forms are also accepted by the current handler. An omitted id creates a
new session; an invalid non-UUID session id fails with a client error. Returning
an id to the next request is what preserves conversation continuity.

## Streaming selection

`stream: false` returns one adapter-shaped response. `stream: true` returns
SSE. The UAR/OpenAI chat request also accepts:

- `openai` — OpenAI chunks, the default;
- `agui_spec` — the `uar.agui/1` profile using official AG-UI event names;
- `agui` — deprecated legacy `agui.*` events;
- `dual` — OpenAI chunks plus the current AG-UI mapping.

Tool results, activated skills, and context actions can appear as documented
UAR extensions in OpenAI chunks. Clients that require only upstream fields must
ignore unknown extension members.

Anthropic input supports the message and tool forms implemented by the current
adapter, including its translated streaming output and prompt-cache markers.
It still uses UAR routing and does not bypass runtime governance.

## Errors

Bad session identifiers and malformed requests return client errors. Unknown
models return `404`. Missing or invalid required credentials return `401`.
Provider startup, timeout, and stream failures remain distinguishable from UAR
validation or governance outcomes; consult the returned adapter error and
runtime logs together.

## Compatibility limit

These routes implement the documented subset in current source. They do not
promise every OpenAI or Anthropic beta header, object field, transport, error
code, or future extension. Generated OpenAPI is a summary, not a conformance
certificate. Verify the exact client workflow against the deployed version.

## Profile limits

`minimal` and `server-full` include the HTTP adapters. `server-full` adds the
broader governed release composition, but the wire shape alone does not prove
those features ran. `embedded-mobile` exposes no HTTP routes; its host calls
shared services directly.

Next: [Events, AG-UI, and A2UI](./events-and-ui.md).
