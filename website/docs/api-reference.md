---
sidebar_position: 4
title: API Reference
---

# API Reference

UAR exposes a REST + Server-Sent Events (SSE) surface over Axum. This page is a
high-level map of the endpoints. For byte-level request/response detail, follow
the links to the in-repo protocol docs under `docs/`.

Unless noted, JSON bodies use `Content-Type: application/json`, and streaming
endpoints return `text/event-stream`. When `security.jwt_required` is `true`
(the default), protected endpoints require an
`Authorization: Bearer <jwt>` header — see [Troubleshooting](./troubleshooting)
for 401s.

## Health

| Method | Path | Description |
|---|---|---|
| `GET` | `/health` | Liveness probe. |
| `GET` | `/healthz` | Liveness probe (used by the Docker healthcheck). |

## Chat completions

The primary chat endpoint plus an OpenAI-compatible alias.

| Method | Path | Description |
|---|---|---|
| `POST` | `/api/chat/completion` | Primary chat endpoint. Streams a normalized event stream (SSE) by default. |
| `POST` | `/v1/chat/completions` | OpenAI-compatible alias. |
| `GET` | `/v1/models` · `/v1/models/{model_id}` | OpenAI-compatible model listing. |

Key request behavior:

- **Model addressing** — `"model": "gpt-4o"` resolves against the default
  provider; `"model": "openai/gpt-4o"` is explicit `provider/model`; an unknown
  model returns `404`.
- **Streaming modes** — `stream_mode`: `"openai"` (default, standard SSE
  chunks), `"agui"` (AG-UI named events), or `"dual"` (both).
- **Sessions** — optional; pass a UUID via the `X-UAR-Session-ID` request header
  to retain context. The session id is returned in the `X-UAR-Session-ID`
  response header. If omitted, an anonymous session is generated.

Full protocol: [`docs/API_CHAT_COMPLETION.md`](https://github.com/Prometheus-AGS/universal-agent-runtime/blob/main/docs/API_CHAT_COMPLETION.md).

## Model catalog and discovery

Backed by the compile-time model catalog (models.dev + liter-llm schemas).

| Method | Path | Description |
|---|---|---|
| `GET` | `/api/models` | Full catalog: all 142+ providers with model capabilities, pricing, and limits. |
| `GET` | `/api/catalog` | Summary: provider count, model count, auth env vars. |
| `GET` | `/provider_catalog.json` | Raw embedded catalog JSON. |
| `POST` | `/api/uar/route` | Dynamic model selection by capability requirements (tools, vision, min context, max cost, preferred provider). |
| `GET` | `/api/uar/resolve-model` | Resolve a model reference to its concrete `provider/model` and catalog entry. |

Example capability-routing request:

```bash
curl -X POST http://localhost:3000/api/uar/route \
  -H 'Content-Type: application/json' \
  -d '{"needs_tools": true, "needs_vision": false, "min_context": 32000,
       "max_cost_per_1m_tokens": 5.0, "preferred_provider": "openai"}'
```

## Runtime — agent runs (`/api/uar/runs`)

Create and drive agent runs, including streaming output, human-in-the-loop tool
approval, cancellation, and checkpoint/resume.

| Method | Path | Description |
|---|---|---|
| `POST` | `/api/uar/runs` | Start a run from an agent artifact + input. Returns `run_id` and a `stream_url`. |
| `GET` | `/api/uar/runs/{id}/stream` | SSE stream of the run's normalized events (`message.delta`, `tool_call.delta`, `tool_call.complete`, `tool_result`, `done`, …). Supports `last_event_id` for resumption. |
| `POST` | `/api/uar/runs/{run_id}/tool-approval` | Approve or reject a pending tool call (human-in-the-loop gate). |
| `POST` | `/api/uar/runs/{run_id}/cancel` | Cancel an in-flight run. |
| `GET` | `/api/uar/runs/{run_id}/checkpoints` | List saved checkpoints for a run. |
| `POST` | `/api/uar/runs/{run_id}/resume` | Resume a run from its latest checkpoint. |
| `POST` | `/api/uar/runs/{run_id}/resume/{checkpoint_id}` | Resume from a specific checkpoint. |

Typical flow: `POST /api/uar/runs` → open the returned `stream_url`
(`GET …/stream`) → when a tool call needs approval, `POST …/tool-approval` →
consume events until `done`.

## Runtime — providers (`/api/uar/providers`, `/api/providers`)

Runtime-managed provider configuration and health. The same router is mounted at
both prefixes.

| Method | Path | Description |
|---|---|---|
| `GET` / `POST` | `/api/uar/providers` | List configured providers / create a provider config. |
| `GET` | `/api/uar/providers/health` | Health/availability of configured providers. |
| `GET` / `PUT` / `DELETE` | `/api/uar/providers/{id}` | Get, update, or delete a provider config. |
| `GET` | `/api/uar/providers/{id}/models` | Models available for a provider. |
| `POST` | `/api/uar/providers/{id}/default` | Set a provider's default model. |

Provider API keys and other secrets are never returned in these responses. See
[`docs/API_KEYS.md`](https://github.com/Prometheus-AGS/universal-agent-runtime/blob/main/docs/API_KEYS.md)
and
[`docs/PROVIDER_CONFIGURATION.md`](https://github.com/Prometheus-AGS/universal-agent-runtime/blob/main/docs/PROVIDER_CONFIGURATION.md).

## Runtime — A2UI schemas (`/api/uar/a2ui`)

Serve and inspect A2UI artifact schemas, plus a test-trigger for run artifacts.

| Method | Path | Description |
|---|---|---|
| `GET` | `/api/uar/a2ui/schemas` | List available A2UI artifact schemas. |
| `GET` | `/api/uar/a2ui/schemas/{schema_id}` | Fetch one A2UI schema by id. |
| `POST` | `/api/uar/a2ui/{run_id}/a2ui/test-trigger` | Emit a test A2UI artifact for a run (development aid). |

## Knowledge bases (`/api/uar/knowledge-bases`, `/api/knowledge`)

Manage knowledge bases, upload documents, and run retrieval.

| Method | Path | Description |
|---|---|---|
| `GET` / `POST` | `/api/uar/knowledge-bases` | List knowledge bases / create one. |
| `GET` / `PUT` / `DELETE` | `/api/uar/knowledge-bases/{id}` | Get, update, or delete a knowledge base. |
| `GET` / `POST` | `/api/uar/knowledge-bases/{id}/documents` | List documents / upload a document for processing + embedding. |
| `POST` | `/api/uar/knowledge-bases/{id}/search` | Retrieval query against a knowledge base. |

## Realtime, MCP, and other endpoints

| Method | Path | Description |
|---|---|---|
| `GET` | `/api/live/{topic}` | SSE stream of entity mutations for a topic (`providers`, `agents`, `models`, `skills`, `settings`, `sessions`, `knowledge_bases`, …). Powers the admin UI's realtime graph. |
| `GET` | `/api/uar/mcp/health` | Health of connected MCP servers. |
| — | `/mcp/uar` | UAR's own MCP endpoint. |
| — | `/mcp/memory` | Memory MCP endpoint (when `memory.mcp_http_enabled = true`). |
| — | `/acp` | ACP JSON-RPC endpoint (when `acp.enabled = true`; path from `acp.path`). |

## Related protocol documentation

The following live in the repository's `docs/` directory:

- [`docs/API_CHAT_COMPLETION.md`](https://github.com/Prometheus-AGS/universal-agent-runtime/blob/main/docs/API_CHAT_COMPLETION.md) — full chat protocol, streaming modes, session handling.
- [`docs/API_KEYS.md`](https://github.com/Prometheus-AGS/universal-agent-runtime/blob/main/docs/API_KEYS.md) — provider key handling and multi-tenant credentials.
- [`docs/API_METRICS_REFERENCE.md`](https://github.com/Prometheus-AGS/universal-agent-runtime/blob/main/docs/API_METRICS_REFERENCE.md) — metrics endpoints and fields.
- [`docs/A2A_PROTOCOL.md`](https://github.com/Prometheus-AGS/universal-agent-runtime/blob/main/docs/A2A_PROTOCOL.md) — the A2A (agent-to-agent) v0.3 transport, including the gRPC port (`server.grpc_port`, default `50051`).
