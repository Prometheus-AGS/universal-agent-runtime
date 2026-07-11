---
sidebar_position: 1
title: Introduction
---

# Universal Agent Runtime

**Universal Agent Runtime (UAR)** is a production-grade, agentic streaming LLM
runtime written in Rust. It is a tool-first, streaming-native, HTML-centric
application server that works with **any of 142+ LLM providers** (OpenAI,
Anthropic, Google Gemini, Groq, Mistral, Cohere, Ollama, and more) through a
single unified configuration.

UAR is both a runnable server and a living reference template for building
agentic AI applications that:

- Support tool-first LLM interaction across **any provider** with automatic
  tool-call normalization (Anthropic `tool_use`, Google `functionCall`,
  Mistral blocks, and all others are converted into OpenAI-style `tool_calls`).
- Stream rich, typed model output through one internal event contract,
  regardless of the upstream provider's wire format.
- Remain HTML-first and inspectable (HTMX, Web Components, Alpine.js) with an
  optional React/TypeScript admin surface — no heavy SPA framework lock-in.
- Run identically as a web app, desktop app, or mobile app (Tauri-compatible).

## Why UAR

| Principle | What it means in practice |
|---|---|
| **Tools are non-optional** | The server is always an MCP client. Tools are discovered dynamically at startup from `mcp.json` and injected into every LLM call. Tool execution is deterministic, auditable, and server-side. |
| **Streaming is the default** | All LLM interactions stream through `LiterLlmDriver` → `Orchestrator` → SSE → the UI. |
| **One internal event contract** | Provider-specific events are normalized into a single typed stream (`message.delta`, `tool_call.delta`, `tool_call.complete`, `tool_result`, `error`, `done`, `usage`), also mirrored as AG-UI (`agui.*`) events. |
| **Compile-time model intelligence** | `build.rs` bakes the [models.dev](https://models.dev) catalog + liter-llm provider schemas into the binary. The `ModelRouter` selects the best model for a set of capability requirements with no network calls. |
| **Local-first persistence** | Runs against an embedded on-disk datastore (SurrealDB / SurrealKV) with no separate database process, or against a remote SurrealDB or PostgreSQL instance. |
| **Secure by default** | JWT auth required, rate limiting enabled, prompt-injection/PII guardrails active, and secrets redacted from logs. |

## The LLM layer: liter-llm

UAR's LLM layer is powered by
[liter-llm](https://github.com/GQAdonis/liter-llm), a Rust-native universal LLM
client. A single `LiterLlmDriver` replaces per-protocol drivers and provider
enums, and provides:

- **142+ providers** behind one API shape.
- **`provider/model` addressing** — `openai/gpt-4o`, `anthropic/claude-sonnet-4`,
  `groq/llama-3.3-70b-versatile`.
- **Unified tool calling** across all providers.
- **A compile-time model catalog** with capabilities, pricing, context limits,
  and modalities.
- **Capability-based model routing.**

## High-level architecture

```
Configuration (CLI > UAR_*__* env > legacy env > config.yaml > defaults)
        │
        ▼
   AppConfig ── LlmConfig ──► LiterLlmDriver ──► Orchestrator (tool loop)
        │                          │                    │
   Persistence               Tool-call            NormalizedEvent stream
   (Surreal / Postgres)      normalization        (message.delta, tool_call.*, done)
        │                                                │
   MCP Registry ──────────────► tools injected           ▼
   (mcp.json, stdio + HTTP)                         Axum server
                                                    REST + SSE
                                                         │
                                              ┌──────────┴──────────┐
                                        HTMX / Web Components    Admin UI (React)
```

## Where to go next

- **[Installation](./installation)** — run UAR via Docker/compose or a prebuilt
  binary, and the minimal boot configuration.
- **[Configuration reference](./configuration)** — every environment variable,
  the `UAR_*__*` nesting convention, and the precedence order.
- **[API reference](./api-reference)** — the REST + SSE surface: chat
  completions, the `/api/uar/*` runtime endpoints, and knowledge endpoints.
- **[Backup and restore](./backup-and-restore)** — runbook for the embedded
  datastore and notes for remote providers.
- **[Upgrade guide](./upgrade-guide)** — version pinning, upgrading a
  self-hosted deploy, and rollback.
- **[Troubleshooting](./troubleshooting)** — fixes for the most common boot and
  runtime problems.

## Licensing

UAR is dual-licensed: open source under `AGPL-3.0-only`, and available under
separate commercial terms for AGPL-incompatible usage. See `LICENSE` and
`LICENSE-COMMERCIAL.md` in the repository.
