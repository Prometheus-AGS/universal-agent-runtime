<div align="center">

# Universal Agent Runtime

**Agentic streaming LLM runtime — 142+ providers, MCP-first, Tauri-ready**

[![CI](https://github.com/Prometheus-AGS/universal-agent-runtime/actions/workflows/ci.yml/badge.svg)](https://github.com/Prometheus-AGS/universal-agent-runtime/actions/workflows/ci.yml)
[![License: AGPL v3](https://img.shields.io/badge/License-AGPLv3-blue.svg)](LICENSE)
[![Providers](https://img.shields.io/badge/providers-142%2B-8A2BE2.svg)](https://models.dev)
[![Rust](https://img.shields.io/badge/Rust-edition%202024-CE422B.svg?logo=rust)](Cargo.toml)

</div>

## Agentic Streaming LLM Application — 142+ Providers, MCP-First, Tauri-Ready

UAR is a production-grade reference implementation and living template for building agentic AI applications that:

- support tool-first LLM interaction across **any provider** (OpenAI, Anthropic, Google, Groq, Mistral, Ollama, and 137 more)
- stream rich, typed model output with unified tool-call normalization
- remain HTML-first and inspectable (no heavy SPA frameworks)
- run identically as a web app, desktop app (Tauri), or mobile app (Tauri)

This is not a demo toy. Everything is wired against real protocols, real streaming, and real tools from day one.

---

## What's New: liter-llm Integration

UAR's LLM layer is now powered by **[liter-llm](https://github.com/GQAdonis/liter-llm)** — a Rust-native universal LLM client that provides:

| Capability | Details |
|---|---|
| **142+ providers** | OpenAI, Anthropic, Google Gemini, Groq, Mistral, Cohere, Together.ai, Perplexity, Ollama, and more |
| **Unified tool calling** | Converts Anthropic `tool_use`, Google `functionCall`, Mistral blocks, and all others into OpenAI-style `tool_calls` |
| **`provider/model` addressing** | `openai/gpt-4o`, `anthropic/claude-sonnet-4`, `groq/llama-3.3-70b-versatile` |
| **Compile-time model catalog** | Model capabilities, pricing, context limits, and modalities embedded from [models.dev](https://models.dev) at build time |
| **Model routing** | Automatic best-model selection based on capability requirements (tools, vision, context size, cost) |
| **Single API shape** | One `LiterLlmDriver` replaces the old `ChatCompletionsDriver` + `ResponsesDriver` + provider enum |

---

## 🚀 Using This Template

This project is a GitHub template. Create your own project from it:

### Option 1: GitHub UI
Click **"Use this template"** → the cleanup workflow runs automatically on first push.

### Option 2: cargo-generate
```bash
cargo generate --git https://github.com/Prometheus-AGS/universal-agent-runtime
```

### Option 3: Bootstrap Script
```bash
git clone https://github.com/Prometheus-AGS/universal-agent-runtime my-project
cd my-project && ./bootstrap.sh
```

See [TEMPLATE_USAGE.md](./TEMPLATE_USAGE.md) for detailed configuration options.

---

## High-Level Goals

1. **Always-on tool use with LLMs** — tools are discovered dynamically from `mcp.json`; every model, every provider, every request
2. **First-class streaming** — token streaming, tool call streaming, tool result streaming, structured chunk types
3. **142+ provider support** — one configuration, any provider, with automatic tool-call normalization
4. **Compile-time model intelligence** — provider capabilities, pricing, and limits are baked in at build time from models.dev
5. **Local-first persistence** — PGlite (Postgres in WASM) for complete client-side history and offline capability
6. **HTML-centric UI** — HTMX, Web Components, Alpine.js; no React, Next.js, or SPA routers
7. **Tauri-compatible** — identical codebase for web, desktop, and mobile

---

## Architecture Overview

The authoritative feature, provider, persistence, routing, tool, platform, and
integration maturity contract is the [Product Support Matrix](docs/product-support-matrix.md).
Catalog breadth is not a certification claim; use the matrix before selecting
a production bundle or provider.

```mermaid
graph TB
    subgraph Config [Configuration Layers]
        CLI["CLI args<br/>--llm-model, --llm-api-key"]
        EnvNew["UAR_LLM__* env vars<br/>(structured)"]
        EnvLegacy["LLM_* legacy env vars<br/>(backward compat)"]
        ProviderKeys["Provider shortcuts<br/>OPENAI_API_KEY, GROQ_API_KEY…"]
        YAMLFile["config.yaml<br/>llm: section"]
    end

    subgraph Catalog [Compile-Time Model Catalog]
        BuildRS["build.rs<br/>fetches models.dev + liter-llm schemas"]
        CatalogJSON["provider_catalog.json<br/>baked into binary"]
        ModelCatalog["ModelCatalog<br/>142+ providers, capabilities,<br/>pricing, limits"]
        ModelRouter["ModelRouter<br/>selects best model<br/>for capability requirements"]
    end

    subgraph LLMLayer [LLM Layer — liter-llm]
        LlmConfig["LlmConfig<br/>Unified config struct"]
        LiterDriver["LiterLlmDriver<br/>wraps liter-llm DefaultClient"]
        ToolNorm["Tool Call Normalization<br/>Anthropic · Google · Mistral → OpenAI"]
        ProviderReg["ProviderRegistry<br/>catalog-enriched,<br/>runtime-managed"]
    end

    subgraph Orchestrator [Orchestrator]
        Orch["Orchestrator<br/>tool loop + streaming"]
        NormEvent["NormalizedEvent stream<br/>message.delta · tool_call.delta<br/>tool_call.complete · done"]
    end

    subgraph MCPLayer [MCP Tool Ecosystem]
        MCPReg["MCP Registry<br/>tool discovery"]
        MCPClient["rmcp client<br/>stdio + HTTP"]
    end

    subgraph Server [Axum Server]
        API["API Endpoints<br/>REST + SSE"]
        APIModels["GET /api/models<br/>catalog-backed"]
        APICatalog["GET /api/catalog<br/>provider summary"]
        APIRoute["POST /api/uar/route<br/>capability routing"]
    end

    subgraph UI [Admin UI — React/TypeScript]
        ProvidersPage["Providers Page<br/>catalog + configured status"]
        ModelsPage["Models Page<br/>capability filters + pricing"]
        SettingsPage["Settings Page<br/>llm config namespace"]
    end

    subgraph Persistence [Settings Persistence]
        SettingsMgr["SettingsManager<br/>llm namespace seeded"]
        DB[(PostgreSQL / SurrealDB)]
    end

    subgraph ClientUI [Client UI]
        HTMX["HTMX 2.0.8"]
        WC["Web Components"]
        PGLite["PGlite<br/>Client DB"]
    end

    Config --> LlmConfig
    BuildRS --> CatalogJSON --> ModelCatalog
    ModelCatalog --> ModelRouter
    ModelCatalog --> ProviderReg

    LlmConfig --> LiterDriver
    LlmConfig --> ProviderReg
    LiterDriver --> ToolNorm
    LiterDriver --> Orch
    MCPReg --> Orch
    Orch --> NormEvent --> API

    API --> APIModels
    API --> APICatalog
    API --> APIRoute
    ModelRouter --> APIRoute

    Persistence --> SettingsMgr --> DB

    UI --> ProvidersPage
    UI --> ModelsPage
    UI --> SettingsPage

    API -.->|SSE| ClientUI

    classDef config fill:#e1f5fe
    classDef catalog fill:#f3e5f5
    classDef llm fill:#e8f5e9
    classDef server fill:#fff3e0
    classDef ui fill:#fce4ec
    classDef persist fill:#e0f7fa

    class Config,CLI,EnvNew,EnvLegacy,ProviderKeys,YAMLFile config
    class Catalog,BuildRS,CatalogJSON,ModelCatalog,ModelRouter catalog
    class LLMLayer,LlmConfig,LiterDriver,ToolNorm,ProviderReg llm
    class Server,API,APIModels,APICatalog,APIRoute server
    class UI,ProvidersPage,ModelsPage,SettingsPage ui
    class Persistence,SettingsMgr,DB persist
```

---

## LLM Configuration

### Quick Start

The simplest way to configure UAR is via provider-specific environment variables. UAR automatically maps them to the correct provider:

```bash
# OpenAI
OPENAI_API_KEY=sk-...
UAR_LLM__MODEL=openai/gpt-4o

# Anthropic
ANTHROPIC_API_KEY=sk-ant-...
UAR_LLM__MODEL=anthropic/claude-sonnet-4

# Groq (fast, free tier available)
GROQ_API_KEY=gsk_...
UAR_LLM__MODEL=groq/llama-3.3-70b-versatile

# Local Ollama (no key required)
UAR_LLM__MODEL=ollama/llama3.2
UAR_LLM__BASE_URL=http://localhost:11434
```

### Configuration Precedence

Settings are merged from multiple sources (highest priority first):

| Priority | Source | Example |
|---|---|---|
| 1 | CLI arguments | `--llm-model openai/gpt-4o` |
| 2 | `UAR_LLM__*` env vars | `UAR_LLM__API_KEY=sk-...` |
| 3 | Legacy `LLM_*` env vars | `LLM_MODEL=gpt-4o` (backward compat) |
| 4 | Provider shortcuts | `OPENAI_API_KEY=sk-...` |
| 5 | `config.yaml` `llm:` section | see below |
| 6 | Compiled defaults | `openai/gpt-4o`, 60s timeout |

### `config.yaml` LLM Section

```yaml
llm:
  model: "openai/gpt-4o"         # provider/model format
  # api_key: "sk-..."            # override; normally use env var
  # base_url: "http://localhost:11434"  # for local/proxy endpoints
  timeout_secs: 60
  max_retries: 3
  cost_tracking: false
  tracing: true
  # parallel_tool_calls: true
  # cooldown_secs: 5
  # health_check_secs: 30
```

### Supported Providers (selected)

| Provider | Model format | Key env var |
|---|---|---|
| OpenAI | `openai/gpt-4o` | `OPENAI_API_KEY` |
| Anthropic | `anthropic/claude-sonnet-4` | `ANTHROPIC_API_KEY` |
| Google Gemini | `google/gemini-2.0-flash` | `GEMINI_API_KEY` |
| Groq | `groq/llama-3.3-70b-versatile` | `GROQ_API_KEY` |
| Mistral | `mistral/mistral-large-latest` | `MISTRAL_API_KEY` |
| Cohere | `cohere/command-r-plus` | `COHERE_API_KEY` |
| Together.ai | `together/meta-llama/Llama-3-70b` | `TOGETHER_API_KEY` |
| Perplexity | `perplexity/llama-3.1-sonar-large` | `PERPLEXITY_API_KEY` |
| Ollama (local) | `ollama/llama3.2` | *(none)* |
| LM Studio | `lmstudio/model-name` | *(none)* |
| OpenRouter | `openrouter/openai/gpt-4o` | `OPENROUTER_API_KEY` |
| Azure OpenAI | `azure/gpt-4o` | `AZURE_API_KEY` |
| AWS Bedrock | `bedrock/anthropic.claude-3` | AWS credentials |

142+ providers total — see `GET /api/catalog` for the full runtime list.

### Model Routing API

UAR can automatically select the best available model based on capability requirements:

```bash
curl -X POST http://localhost:3001/api/uar/route \
  -H 'Content-Type: application/json' \
  -d '{
    "needs_tools": true,
    "needs_vision": false,
    "min_context": 32000,
    "max_cost_per_1m_tokens": 5.0,
    "preferred_provider": "openai"
  }'
```

Response:
```json
{
  "provider_id": "openai",
  "model_id": "gpt-4o",
  "full_model": "openai/gpt-4o",
  "reasoning": "Matched preferred provider with tool support"
}
```

---

## Core Design Principles

### 1. Tools Are Non-Optional

- The server is always an MCP client
- Tools are discovered dynamically at startup from `mcp.json`
- Every model, every provider, every request has tools available
- Tool execution is deterministic, auditable, and server-side

### 2. Streaming Is the Default

All LLM interactions stream through `LiterLlmDriver` → `Orchestrator` → SSE → Web Components. The server normalizes all upstream events into one typed contract regardless of provider-specific wire format.

### 3. One Internal Event Contract

```
message.delta          → incremental assistant text
thinking.delta         → reasoning/chain-of-thought (when exposed)
tool_call.delta        → streaming tool invocation accumulation
tool_call.complete     → fully assembled tool call
tool_result            → tool execution result
error                  → error event with code
done                   → stream completion signal
usage                  → token counts
```

These are also mirrored as AG-UI events (`agui.*`) for future compatibility.

### 4. Compile-Time Model Intelligence

`build.rs` fetches `models.dev/api.json` and merges it with `liter-llm`'s provider schemas at compile time. The result is embedded in the binary as a `ModelCatalog` that provides:

- Tool calling capability flags per model
- Vision/multimodal support detection
- Context window and output limits
- Input/output pricing (per 1M tokens)
- Open-weights status

The `ModelRouter` uses this catalog to select the optimal model at runtime without any network calls.

---

## Chat API Protocol

Primary chat endpoint: `POST /api/chat/completion`
Compatibility alias: `POST /v1/chat/completions`

Model addressing:
- `"model": "gpt-4o"` → resolves against default provider
- `"model": "openai/gpt-4o"` → explicit provider/model
- Unknown → `404 Unknown model`

Streaming modes:
- `stream_mode: "openai"` (default) — standard SSE chunks
- `stream_mode: "agui"` — AG-UI named events
- `stream_mode: "dual"` — both formats simultaneously

Session continuity:
- Session ID optional; server generates anonymous session if omitted
- Provide UUID via `X-UAR-Session-ID` header to retain context
- Session ID returned in `X-UAR-Session-ID` response header

Full protocol: [docs/API_CHAT_COMPLETION.md](docs/API_CHAT_COMPLETION.md)

### Discovery / Catalog Endpoints

| Method | Path | Description |
|---|---|---|
| `GET` | `/api/models` | Full catalog: all 142+ providers with model capabilities, pricing, limits |
| `GET` | `/api/catalog` | Summary: provider count, model count, auth env vars |
| `POST` | `/api/uar/route` | Dynamic model selection by capability requirements |
| `GET` | `/api/uar/providers` | Runtime-configured provider overrides |
| `GET` | `/api/uar/discovery/agents` | Registered agents |
| `GET` | `/api/uar/discovery/tools` | Available MCP + native tools |
| `GET` | `/api/uar/discovery/skills` | Registered skills |

---

## MCP (Model Context Protocol)

UAR is always an MCP client. Configure servers in `mcp.json`:

```json
{
  "mcpServers": {
    "time": {
      "command": "npx",
      "args": ["-y", "@mcpcentral/mcp-time"]
    },
    "tavily": {
      "url": "https://mcp.tavily.com/mcp/?tavilyApiKey=${TAVILY_API_KEY}"
    }
  }
}
```

Tools are namespaced automatically (`time::now`, `tavily::search`) and injected into every LLM call.

---

## UI Stack

### HTMX 2.0.8
Navigation and server interaction. Not used for high-frequency streaming updates.

### Web Components (TypeScript)
- `<chat-stream>` — SSE connection, stream management
- `<transcript-view>` — DOM updates, markdown rendering
- `<conversation-sidebar>` — history, PGlite interactions
- `<token-counter>` — real-time usage metrics

### Admin UI (React/TypeScript)
Built-in admin panel at `/admin` with:
- **Providers page** — all 142+ catalog providers with configured status, model counts, auth env var hints
- **Models page** — searchable model catalog with capability filters (tools, reasoning, vision), context limits, and pricing
- **Settings page** — runtime-editable `llm` config namespace backed by the persistence layer

### PGlite (Postgres WASM)
Full Postgres in the browser for conversation history, full-text search, and offline capability.

---

## Frontend Architecture — Realtime Entity Graph

UAR's admin surface and chat sidebar share a single realtime spine: every
mutation in SurrealDB fans out through a tokio broadcast bus, an SSE
endpoint, and the `useGraphStore` graph in the SPA. Admin pages and chat
state read from the graph; SSE keeps it fresh. **No admin page polls. No
page goes stale.**

### Data flow

```mermaid
graph LR
  subgraph DB [SurrealDB]
    Tables[("providers · agents · models · skills<br/>settings · sessions · memory<br/>knowledge_bases · knowledge_documents · compiler_sessions")]
  end

  subgraph Backend [Backend bus + SSE]
    Live[".select(table).live()<br/>per topic"]
    Bus["LiveQueryBus<br/>(tokio broadcast)"]
    SSE["GET /api/live/{topic}<br/>(Axum SSE)"]
  end

  subgraph Frontend [Frontend graph]
    Adapter["uar-sse-adapter.ts<br/>create | update | delete"]
    Graph["useGraphStore<br/>Zustand 5 + Immer 11"]
  end

  subgraph Consumers [Consumers]
    AdminPages["Admin pages<br/>(direct: useEntityList / useEntityView)"]
    ChatSidebar["Chat sidebar<br/>(SSE-reconciler: use-thread-graph-sync)"]
    SettingsPage["Settings page<br/>(form-cache + graph reads)"]
  end

  Tables -->|Notification| Live
  Live --> Bus
  Bus --> SSE
  SSE -->|EventSource| Adapter
  Adapter -->|EntityChange| Graph
  Graph --> AdminPages
  Graph --> ChatSidebar
  Graph --> SettingsPage

  classDef db fill:#e0f7fa
  classDef be fill:#fff3e0
  classDef fe fill:#fce4ec
  classDef cn fill:#e8f5e9
  class DB,Tables db
  class Backend,Live,Bus,SSE be
  class Frontend,Adapter,Graph fe
  class Consumers,AdminPages,ChatSidebar,SettingsPage cn
```

### Entity inventory

| Entity | Topic | Pattern | Notes |
|---|---|---|---|
| Provider | `providers` | direct | catalog + configured rows + `ProviderMeta` singleton for default |
| Agent | `agents` | direct | nested `metadata` / `policy` / `memory` shape preserved |
| Model | `models` | direct | flattened from `CatalogModelsResponse` on hydration |
| Skill | `skills` | direct | optimistic toggle / edit / delete |
| Memory | `memory` | direct | per-query view; `MemoryMeta` singleton holds stats |
| CompilerSession | `compiler_sessions` | direct | tiny page; established shared admin components |
| KnowledgeBase | `knowledge_bases` | direct | via `useKnowledgePage` compat hook |
| Document | `knowledge_documents` | direct | optimistic upload status progression |
| Setting | `settings` | direct (form-cache) | dirty/conflicts/saving via `settings-form-cache.ts` |
| Tool | (no SSE) | direct (fetch-on-mount) | registry is static after server startup |
| McpStatus | (no SSE) | direct (poll-fed graph) | 30 s poll hydrates graph rows |
| Thread | `threads` (alias `sessions`) | SSE-reconciler | client-first creation; graph events reconcile into PGlite registry |
| ApiKey | (none) | non-realtime | secrets never broadcast |

### Patterns

1. **Direct migration playbook.** Graph is the source of truth. Pages read
   via `useEntityList` / `useEntityView` / `useEntity` hooks; mutations
   call services directly, wrapped in
   [`optimisticUpsert` / `optimisticRemove`](./frontend/src/lib/realtime/optimistic.ts).
   SSE keeps the graph fresh.

2. **SSE-reconciler pattern.** For client-first entities (Threads). The
   local store is authoritative; a small hook
   ([`use-thread-graph-sync.ts`](./frontend/src/stores/use-thread-graph-sync.ts))
   subscribes to the graph and reconciles server events into the local
   store. No REST refetch needed — live-only sync is acceptable when
   client actions dominate the input.

3. **Form-cache pattern.** For pages with dirty/save semantics (Settings).
   A module-level `Map<namespace, DirtyState>` consumed via
   `useSyncExternalStore` holds transient form state; commits POST in
   bulk with optimistic graph upsert + rollback. Replaces the retired
   `settings-store.ts` without touching the 3334 LOC settings page.

Full migration history, playbooks, and contract tests:
[`docs/migration-stale-data-audit.md`](./docs/migration-stale-data-audit.md).

### CI architectural gates

Every PR runs [`scripts/ci-grep-gates.sh`](./scripts/ci-grep-gates.sh) plus
the standard frontend pipeline (vitest, typecheck, build). The gates
block regressions on the architectural invariants this spine depends on:

- `useGraphBridge` retired (interim pattern, permanently retired 2026-05-27)
- `useSettingsStore` retired
- No banned fonts (`Inter` / `Roboto` / `Arial` / `Space Grotesk`) in admin CSS — see [`docs/admin-aesthetic-spec.md`](./docs/admin-aesthetic-spec.md)
- No `outline: none` on admin interactive elements (a11y contract)

Local equivalent: `pnpm run ci-gates`.

### Terminal admin aesthetic

Admin pages render under a scoped `data-admin-theme="terminal"` attribute
on `<html>` (set by `pages/admin-page.tsx` on mount). CSS tokens
(`--terminal-bg`, `--phosphor`, `--amber`, `--signal-red`) live under
that selector in `frontend/src/index.css`; the chat surface retains its
existing Ember/UAR Dark theme.

Shared aesthetic components — `<LoadingCursor>` (blinking `▍` phosphor),
`<EmptyFrame>` (ASCII frame + slot), `<ErrorBar>` (mono error-code
prefix) — live in `frontend/src/components/admin/`.

### Historical: bridge pattern

An interim `useGraphBridge` hook briefly carried per-entity bridges
during the migration arc. Permanently retired 2026-05-27 once every
consumer adopted the Direct migration or SSE-reconciler pattern. The
file is gone from the tree; the CI grep gate blocks its return. See
[the audit's Historical appendix](./docs/migration-stale-data-audit.md#historical-bridge-pattern-permanently-retired-2026-05-27)
for the full retirement story.

---

## Memory System

UAR ships a durable, multi-scope memory system that gives agents persistent recall across sessions. It captures facts automatically, injects relevant context before each LLM call, builds a knowledge graph, and exposes all operations as MCP tools.

**Key capabilities**: session, user, agent, global, and task memory scopes · hybrid vector + BM25 retrieval · auto-capture from conversations · knowledge graph (entity/relation/observation model) · full MCP server at `/mcp/memory`

Enable with `UAR_MEMORY__ENABLED=true`. See [docs/MEMORY_SYSTEM.md](docs/MEMORY_SYSTEM.md).

---

## Getting Started

### Prerequisites

- **Rust** (latest stable)
- **Bun** — frontend asset building
- **PostgreSQL** (with pgvector) or **SurrealDB** — persistence
- **Redis** — caching and sessions (optional)

### Quick Start

```bash
# 1. Clone
git clone https://github.com/Prometheus-AGS/universal-agent-runtime.git
cd universal-agent-runtime

# 2. Configure
cp .env.example .env
# Edit .env: set UAR_LLM__MODEL and your provider API key

# 3. Build frontend
bun install && bun run build

# 4. Run
cargo run
# → http://localhost:3001
```

### Minimal `.env`

```bash
# Pick any provider:
UAR_LLM__MODEL=openai/gpt-4o
OPENAI_API_KEY=sk-...

# OR
UAR_LLM__MODEL=groq/llama-3.3-70b-versatile
GROQ_API_KEY=gsk_...

# OR (local, no key needed)
UAR_LLM__MODEL=ollama/llama3.2
UAR_LLM__BASE_URL=http://localhost:11434

# MCP tools (optional)
TAVILY_API_KEY=tvly-...

# Database
DATABASE_URL=postgres://user:password@localhost:5432/uar
```

### Development Commands

```bash
# Rust backend
cargo run                    # dev server
cargo build --release        # production build
cargo test                   # unit + integration tests
cargo clippy                 # lint

# Frontend
bun run build                # build all assets
bun run dev                  # watch mode
bun run lint                 # ESLint + typecheck

# Full test suite (requires Docker)
./tools/test-all.sh --quick  # smoke tests
./tools/test-all.sh --full   # complete suite with coverage
./tools/test-all.sh --ci     # CI mode (sequential)
```

---

## Project Structure

```
src/
├── main.rs                  # Axum server entry point
├── lib.rs                   # AppState, module wiring
├── config.rs                # LlmConfig, AppConfig, Cli, load_llm_config()
├── normalized.rs            # NormalizedEvent enum
├── server.rs                # Route handlers, /api/models, /api/catalog
├── llm/
│   ├── catalog.rs           # ModelCatalog (compile-time, from build.rs)
│   ├── liter_driver.rs      # LiterLlmDriver: ChatCompletionChunk → NormalizedEvent
│   ├── orchestrator.rs      # Tool loop, streaming aggregation
│   ├── registry.rs          # ProviderRegistry + register_custom_provider()
│   └── router.rs            # ModelRouter: capability-based model selection
├── mcp/                     # MCP client registry
├── uar/
│   ├── domain/              # Agent artifacts, policy, events
│   ├── runtime/             # RunManager, actor system, skills, matching
│   ├── settings/            # SettingsManager (llm namespace + persistence)
│   └── persistence/         # Postgres / SurrealDB adapters
build.rs                     # Catalog builder: models.dev + liter-llm schemas
mcp.json                     # MCP server configuration
example.config.yaml          # Full configuration reference
```

---

## Code Quality

- **Zero warnings policy** — `cargo clippy` must pass clean
- **Rust 2024 edition** — latest language features
- **Extensive lints** — pedantic + perf + correctness + restriction lints in `Cargo.toml`
- **Structured logging** — `tracing` throughout, OpenTelemetry-compatible
- **mimalloc** — high-performance memory allocator

---

## CI/CD — Deployment to GKE

Pushing to the `deployment` branch triggers `.github/workflows/deploy.yml`:

1. Runs `cargo clippy`, `cargo fmt --check`, `bun run build`, `cargo test --lib`
2. Builds and pushes `tribehealth/universal-agent-runtime:deployment-<sha>` to Docker Hub
3. Authenticates to GKE and performs a rolling `kubectl set image` update
4. Waits for `kubectl rollout status`

### Required GitHub Secrets

| Secret | Description |
|---|---|
| `DOCKER_USERNAME` | Docker Hub username |
| `DOCKER_PASSWORD` | Docker Hub password or token |
| `GCP_SA_KEY` | Base64-encoded GCP service account JSON |
| `GCP_PROJECT_ID` | GCP project ID |
| `GKE_CLUSTER_NAME` | GKE cluster name |
| `GKE_CLUSTER_LOCATION` | GKE cluster location |

---

## Tauri Compatibility

- No CDN scripts — all assets served locally
- No API keys in the browser
- SSE works identically in webview
- Same codebase for web, desktop, and mobile

---

## Security

Found a vulnerability? **Please do not open a public issue.** Report it
privately via [GitHub private vulnerability reporting](https://github.com/Prometheus-AGS/universal-agent-runtime/security/advisories/new).
Our disclosure policy, response targets, supported-versions table, and EU CRA
posture are in [SECURITY.md](SECURITY.md). For usage questions and support
channels, see [SUPPORT.md](SUPPORT.md).

---

## Licensing

Dual-licensed:
- Open source: `AGPL-3.0-only` (see `LICENSE`)
- Commercial: separate terms for AGPL-incompatible usage (see `LICENSE-COMMERCIAL.md`)

Additional: [docs/licensing/LICENSING.md](docs/licensing/LICENSING.md) · [TRADEMARKS.md](TRADEMARKS.md) · [CONTRIBUTING.md](CONTRIBUTING.md)

---

## Summary

UAR demonstrates that it is possible to build deeply agentic, tool-first, streaming-native, HTML-centric, Tauri-compatible AI applications that work with **any of 142+ LLM providers** — without heavyweight SPA frameworks, without client-side secrets, and without sacrificing architectural clarity.
