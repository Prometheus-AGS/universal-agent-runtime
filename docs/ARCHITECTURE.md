# Architecture Overview

## System Design

UAR is built around four core ideas:

1. **liter-llm as the unified LLM layer** — one driver, 142+ providers, automatic tool-call normalization
2. **Compile-time model intelligence** — capabilities, pricing, and limits are baked in at build time, not fetched at runtime
3. **MCP-first tool execution** — tools are always on, server-controlled, and protocol-agnostic
4. **Normalized event streaming** — all providers produce the same typed event stream regardless of wire format

---

## LLM Layer

### liter-llm Integration

The legacy `ChatCompletionsDriver`, `ResponsesDriver`, and `Provider` enum have been replaced by a single `LiterLlmDriver` backed by `liter_llm::DefaultClient`.

```
LlmConfig (merged from CLI + env + YAML + defaults)
    │
    ▼
build_client_config()
    │
    ▼
liter_llm::ClientConfig
    │
    ▼
LiterLlmDriver
    │
    ├── liter_llm::DefaultClient.chat_stream(req)
    │       │
    │       └── ChatCompletionChunk stream
    │               (Anthropic tool_use, Google functionCall,
    │                Mistral tool_calls, all → OpenAI shape)
    │
    └── NormalizedEvent stream
            message.delta
            tool_call.delta
            tool_call.complete
            usage
            done
            error
```

### Compile-Time Model Catalog

`build.rs` fetches and merges two sources before every binary build:

1. **`liter-llm/schemas/providers.json`** — 142 providers with routing data (`name`, `base_url`, `auth env_var`, `endpoints`, `model_prefixes`, `param_mappings`)
2. **`models.dev/api.json`** — rich model metadata per provider (capabilities, pricing, context limits, modalities)

The merged result is written to `$OUT_DIR/provider_catalog.json` and embedded via `include_str!` into the `ModelCatalog` singleton. No network calls at runtime — the catalog is baked in.

```rust
// Access at any time — zero cost after first parse
let catalog = ModelCatalog::global();
let provider = catalog.provider("openai");
let model    = catalog.model("openai", "gpt-4o");
```

### ModelRouter

`ModelRouter` queries the embedded catalog to select the best available model based on capability requirements — tool support, vision, minimum context window, cost ceiling, preferred provider:

```
POST /api/uar/route
  { needs_tools: true, min_context: 32000, max_cost_per_1m_tokens: 3.0 }
    │
    ▼
ModelRouter.route(requirements, configured_providers)
    │   (no network calls — pure catalog lookup)
    ▼
RouteResult { provider_id, model_id, full_model, reasoning }
```

### ProviderRegistry

`ProviderRegistry` is the runtime-managed index of configured provider API keys and overrides. It seeds from two sources:

1. **`seed_from_llm_config()`** — extracts `provider/model` from `LlmConfig` and enriches the entry with display name, base URL, and model list from the `ModelCatalog`
2. **`register_custom_provider()`** — adds user-defined custom providers; if the provider ID is in the catalog, model lists and defaults are auto-populated from catalog data

```rust
// Auto-enriched — model list + display name from catalog
registry.register_custom_provider(ProviderConfig {
    id: "groq".to_string(),
    api_key: Some(groq_key),
    ..Default::default()  // catalog fills in the rest
}).await?;
```

### Provider Health & Failover

`ProviderHealthMonitor` (`src/llm/health.rs`) tracks per-provider consecutive-failure counts and puts a provider into a cooldown window once `FailoverConfig.error_threshold` consecutive failures are recorded. Both `ModelRouter::route` and `ProviderRegistry::resolve_to_llm_config` consult `is_available()` before selecting a provider — a provider currently in cooldown is excluded from selection entirely, not just deprioritized, so a struggling provider can't keep winning ties against healthy alternatives:

```rust
health_monitor.record_failure("openai").await;  // increments consecutive_errors
// ... after error_threshold consecutive failures:
health_monitor.is_available("openai").await;    // false until cooldown_secs elapses
```

The monitor is shared (`Arc`) across `ProviderRegistry`, `ModelRouter`, and every `Orchestrator` instance, so a failure observed on one run's driver call immediately affects routing decisions for the next run. Failover itself (opt-in via `FailoverConfig.enabled`) falls back through `FailoverConfig.fallback_models` using the configured `strategy` (default: `Priority`).

### Prompt Dialect Engine

Different model families have documented, web-verified preferences for how prompts and reasoning are expressed at the API layer — treating every model as an interchangeable black box leaves capability on the table. `PromptDialect::detect(model_id)` (`src/llm/prompt_dialect.rs`) classifies a model into one of six dialects (`AnthropicXml`, `OpenAiJson`, `KimiMarkdown`, `GlmThinking`, `QwenHybrid`, `MiniMaxStructured`) plus a `Generic` fallback, purely from substring matching on the model id — no network call, no catalog lookup. `PromptDialectEngine::request_params(model_id, DialectRequest)` then emits the extra request-body parameters that dialect wants (reasoning-persistence toggles, structured-output hints, thinking-effort levels) as a JSON object the driver merges into the outbound request. Only web-verified parameters are encoded — nothing here is guessed.

### Cost & Budget Tracking

`src/uar/runtime/cost_budget.rs` (CH-06) adds the *accounting* layer on top of the existing opt-in cost estimation (`LlmConfig.cost_tracking`, `estimate_cost`, the `uar_llm_cost_usd` metric): aggregate spend per `BudgetScope` (`Run` / `Task` / `Session` / `Agent` / `Global`), a configurable `BudgetLimit { limit_usd, warn_at }` per scope, and threshold-crossing events so the UI and operators can react before a runaway burns the budget. It's in-memory and lock-light — a single-process aggregator suitable for the runtime hot path; durable roll-ups (SurrealDB/Postgres) are intentionally out of scope and would layer on top by subscribing to the emitted events. The cost-dashboard admin page (CH-07) is the current read surface for this data; only the *global* budget limit is configurable today (per-agent/per-task configuration is tracked debt, not yet built).

---

## Agent Spec v2 & Conformance (`uar-spec-v2-and-polish` phase)

`AgentDescriptorIR` (`src/uar/compiler/ir.rs`) gained five additive v2 sections — `model_requirements`, `prompt_dialect`, `rag_configuration`, `context_strategy`, `api_harness` — each deliberately mirroring an existing runtime type so a compiled agent's *declared* needs can be checked against what the runtime actually does:

| v2 IR section | Mirrors |
|---|---|
| `model_requirements` | `llm::router::RouteRequirements` |
| `prompt_dialect` | `llm::prompt_dialect::DialectRequest` (+ optional explicit override) |
| `rag_configuration` | the in-process RAG hardening knobs (decomposition/verification/audit) |
| `context_strategy` | `uar::context::strategy::ContextStrategy`, variant-for-variant, including `Auto` |
| `api_harness` | transport protocol + `stream_mode` selection |

All five are `#[serde(default)]` and excluded from the original 15-section completeness contract, so v1.1 documents still parse and compile unchanged (`s08_emit` only bumps the emitted `schema` to `uar-agent-descriptor/v2` when at least one v2 section is non-default; otherwise it stays `/v1` — descriptive metadata, not a compile gate).

**Conformance harness** (`src/uar/compiler/conformance.rs`): `check_conformance(ir, router)` drives the *real* runtime functions — `ModelRouter::route`, `PromptDialect::detect`, `apply_strategy` — against a descriptor's declared `model_requirements`/`prompt_dialect`/`context_strategy`, proving each is satisfiable at load time and honored at run time rather than re-implementing their logic. A section left at its parsed default is reported `NotDeclared` (indistinguishable from "the author wrote nothing here" — `#[serde(default)]` produces the same value either way), not a pass/fail verdict.

**Template library** (`templates/*.agent.md`): four worked examples — `coding` (tools+reasoning-required), `vision` (vision-required, deliberately leaves context strategy at `Auto`), `terminal` (tools+structured-output-required, small sliding window), `data` (RAG fully enabled, `Hierarchical` context strategy) — each compiles+signs cleanly via `universal-agent-runtime compile templates/<name>.agent.md` (`Command::Compile`, `src/uar/compiler/cli.rs`) and is regression-guarded by `tests/agent_templates_test.rs`. CI compiles+signs all four as a release artifact (`.github/workflows/release.yml`'s `compile-agent-templates` job → `agent-templates.tar.gz`).

---

## Chat Serving Path

```
Client
  POST /api/chat/completion
  { model, messages, stream, stream_mode }
      │
      ▼
Axum Handler (src/server.rs)
  - resolves provider/model via ProviderRegistry
  - resolves/creates session UUID
  - starts run via RunManager
      │
      ▼
RunManager
  - appends user turn to SessionStore
  - executes Orchestrator
      │
      ▼
Orchestrator (tool loop)
  ├── LiterLlmDriver.stream(req)
  │       └── liter-llm → NormalizedEvent stream
  │               (tool_call.delta accumulation)
  │               (tool_call.complete detection)
  │
  ├── McpRegistry.call_tool(name, args)  [on tool_call.complete]
  │       └── NormalizedEvent::ToolResult
  │
  └── loop until stop or max iterations
      │
      ▼
NormalizedEvent stream
      │
      ├── stream=false: aggregate → chat.completion JSON
      │
      └── stream=true: map events → SSE chunks
              stream_mode="openai"  → chat.completion.chunk
              stream_mode="agui"    → agui.* named events
              stream_mode="dual"    → both
```

---

## Model Resolution Rules

1. **Plain model** (`"gpt-4o"`) — validates only against the default provider's model catalog; `404 Unknown model` if not found
2. **Scoped model** (`"openai/gpt-4o"`) — validates provider + model tuple; `404 Unknown model` if either is unknown

The `ModelCatalog` is the source of truth. The `ProviderRegistry` adds runtime API-key configuration on top.

---

## Endpoints Reference

### Chat

| Method | Path | Description |
|---|---|---|
| `POST` | `/api/chat/completion` | Primary chat endpoint |
| `POST` | `/v1/chat/completions` | OpenAI-compatible alias |

### Catalog & Discovery

| Method | Path | Description |
|---|---|---|
| `GET` | `/api/models` | Full model catalog (providers + models + capabilities + pricing) |
| `GET` | `/api/catalog` | Provider summary (counts, auth env vars) |
| `POST` | `/api/uar/route` | Capability-based model routing |
| `GET` | `/api/uar/providers` | Runtime-configured provider overrides |
| `GET` | `/api/uar/discovery/agents` | Registered agents |
| `GET` | `/api/uar/discovery/tools` | MCP + native tools |
| `GET` | `/api/uar/discovery/skills` | Registered skills |

### Settings

| Method | Path | Description |
|---|---|---|
| `GET` | `/api/uar/settings` | All setting namespaces (including `llm`) |
| `PUT` | `/api/uar/settings/llm/{key}` | Update a single LLM setting at runtime |

---

## NormalizedEvent Model

All providers emit the same typed stream regardless of wire format:

| Event | Meaning |
|---|---|
| `stream.start` | New streaming response begins |
| `message.delta` | Incremental assistant text |
| `thinking.delta` | Reasoning/CoT output (when exposed by model) |
| `reasoning.delta` | Chain-of-thought output |
| `tool_call.delta` | Partial tool invocation (accumulate by `call_index`) |
| `tool_call.complete` | Fully assembled tool call with ID, name, and JSON args |
| `tool_result` | Result of tool execution |
| `usage` | Token counts (prompt, completion, total) |
| `error` | Error with message and optional code |
| `done` | Stream complete |

These are also mirrored as `agui.*` events (AG-UI protocol) for future compatibility:
`agui.message.delta`, `agui.tool_call.delta`, `agui.tool_call.complete`, `agui.tool_result`, `agui.usage`, `agui.done`, etc.

---

## Source Layout

```
src/
├── config.rs             # LlmConfig, AppConfig, Cli, load_llm_config()
│                           build_client_config() → liter_llm::ClientConfig
├── lib.rs                # AppState (includes model_router: Arc<ModelRouter>)
├── main.rs               # Server boot, ProviderRegistry seeding
├── normalized.rs         # NormalizedEvent enum (all event types)
├── server.rs             # Route handlers
│   ├── api_models()      # GET /api/models — catalog-backed
│   ├── api_catalog()     # GET /api/catalog — provider summary
│   └── api_route_model() # POST /api/uar/route — ModelRouter dispatch
│
└── llm/
    ├── mod.rs            # Public re-exports
    ├── catalog.rs        # ModelCatalog (embedded at build time)
    │                       ModelInfo, ProviderInfo, CapabilityFilter
    ├── liter_driver.rs   # LiterLlmDriver
    │                       ChatCompletionChunk → NormalizedEvent mapping
    │                       ToolAccum: streaming tool-call accumulator
    ├── orchestrator.rs   # Orchestrator: tool loop, up to 10 iterations
    ├── registry.rs       # ProviderRegistry
    │                       seed_from_llm_config() — catalog-enriched
    │                       register_custom_provider() — with auto-enrichment
    │                       resolve_to_llm_config() — for driver construction
    ├── router.rs         # ModelRouter
    │                       route(requirements) → best available model
    ├── health.rs         # ProviderHealthMonitor — failover cooldown tracking
    └── prompt_dialect.rs # PromptDialect::detect, PromptDialectEngine

# uar/compiler/ — UAR-AGENT-MD compiler (agent spec v2)
src/uar/compiler/
    ├── ir.rs             # AgentDescriptorIR (15 v1.1 sections + 5 v2 sections)
    ├── parser.rs         # Markdown → IR
    ├── stages/           # 8-stage compile pipeline (s01_frontmatter .. s08_emit)
    ├── pipeline.rs       # compile() orchestrator → CompiledDescriptor
    ├── conformance.rs    # check_conformance() — declared v2 sections vs. real runtime
    ├── cli.rs            # `compile` subcommand (compile+sign a .agent.md file)
    └── signing.rs        # LocalKeyProvider (Ed25519)

src/uar/runtime/cost_budget.rs  # BudgetScope/BudgetLimit — per-scope spend aggregation
templates/                      # coding/vision/terminal/data .agent.md worked examples
```

---

## Build Pipeline

```
cargo build
    │
    └── build.rs
        ├── SKIP_CATALOG_BUILD=1?  → write empty stub
        │
        ├── find liter-llm/schemas/providers.json
        │   (sibling dir or Cargo git checkout cache)
        │
        ├── fetch models.dev/api.json (HTTP, ~1MB)
        │   (SKIP_MODEL_BUILD=1 skips this for offline dev)
        │
        ├── merge: liter-llm providers + models.dev metadata
        │   (models.dev enriches each provider with model capabilities,
        │    pricing, context limits, and modalities)
        │
        └── write $OUT_DIR/provider_catalog.json
            └── embedded via include_str! into ModelCatalog
```

The catalog is rebuilt on every clean build. For faster iteration, set `SKIP_CATALOG_BUILD=1` to use the empty stub (catalog will have no providers).

---

## Architectural Decisions

Standing decisions carried forward from `uar-next-harness`'s planning, re-affirmed (not re-litigated) during this phase:

- **D-A — RAG stays in-process.** Query decomposition, retrieval verification, and audit events (CH-11) are hardened directly in `src/uar/rag/pipeline.rs`, not extracted into a separate Knowledge Service. Revisit only if RAG load or team ownership genuinely requires a separate deployable.
- **D-B — MemPalace stays off by default.** The `memory-palace` Cargo feature (`surreal-memory/palace`) is not in UAR's default feature set: it pulls in `mempalace-core` → `rusqlite` (bundled SQLite) → `libsqlite3-sys`, which conflicts with the newer `libsqlite3-sys` versions pulled in by other consumers (e.g. `sqlx 0.8`'s transitive closure). See `Cargo.toml`'s `memory-palace` feature comment for the exact version conflict. UAR itself does not need it; opt in explicitly via `--features memory-palace` only if you understand the conflict and have resolved it for your build.
- **D-C — LibreFang integration is scoped to the UAR side only.** UAR exposes the zero-code `provider_urls` seam (an OpenAI-compatible `/v1/chat/completions` endpoint LibreFang can target with no UAR-specific code) — see `docs/librefang-integration.md`. No LibreFang-side code lives in this repository.
- **D-D — Git-sourced dependency pins are deliberate, not technical debt.** `rmcp`, `surreal-memory`, and `prometheus_parking_lot` are pinned to specific commit SHAs; `kreuzberg` is pinned to a stable release tag (`v4.9.8`) rather than a raw SHA. None float on a branch — `surreal-memory` was moved off `branch = "main"` to a fixed `rev` in `uar-post-dependabot-followup-2026-07` after that phase found it was the one pin actively undermining this decision's own "reproducible builds" rationale. Full rationale, current pins, and the upgrade SOP live in `docs/DEPENDENCY_MANAGEMENT.md` — this is not duplicated here.
- **D-E — Runtime tool provisioning is a separate concern from process supervision, and does not cover skill compilation.** `src/uar/orchestrator/provisioning.rs` resolves "does this executable exist yet, and if not, obtain it" (Adopt → native package manager → git-install → prebuilt binary) for MCP stdio server commands. `process_supervisor.rs` solves a different problem — adopting an already-running TCP-listening service — and has no current call site; the two are not merged. `surreal-memory` and `liter-llm` are linked Rust libraries used in-process, not spawnable targets for either module. The 5 skill-compilation toolchains (Rust/Node/Python/Go/wasmtime) have ready-to-use provisioning recipes (`skill_toolchain_specs()`) but no active caller — UAR has no code path that compiles a skill from source today; the Dockerfile keeps those toolchains resident for manual, human-driven skill authoring inside the container. Full detail in `docs/PROVISIONING.md`.

---

## Testing

```bash
# Unit + integration (no network)
SKIP_MODEL_BUILD=1 SKIP_FRONTEND_BUILD=1 SKIP_CATALOG_BUILD=1 cargo test

# With real LLM (requires env vars)
LLM_MODEL=openai/gpt-4o LLM_API_KEY=sk-... cargo test --test uar_integration

# Full suite (requires Docker)
./tools/test-all.sh --full
```
