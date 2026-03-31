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
    └── router.rs         # ModelRouter
                            route(requirements) → best available model
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

## Testing

```bash
# Unit + integration (no network)
SKIP_MODEL_BUILD=1 SKIP_FRONTEND_BUILD=1 SKIP_CATALOG_BUILD=1 cargo test

# With real LLM (requires env vars)
LLM_MODEL=openai/gpt-4o LLM_API_KEY=sk-... cargo test --test uar_integration

# Full suite (requires Docker)
./tools/test-all.sh --full
```
