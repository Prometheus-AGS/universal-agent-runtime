# liter-llm Integration Guide

UAR's LLM layer is powered by **[liter-llm](https://github.com/GQAdonis/liter-llm)** — a Rust-native universal LLM client library that provides a single, unified API surface across 142+ providers.

---

## Why liter-llm?

Before liter-llm, UAR maintained separate drivers per protocol:
- `ChatCompletionsDriver` for OpenAI Chat Completions
- `ResponsesDriver` for the OpenAI Responses API
- A `Provider` enum for routing

Each new provider required a new driver, and tool-call normalization had to be done per-provider in the UAR codebase.

With liter-llm, **all of that is gone**. One `LiterLlmDriver` handles everything:

| What was required | What it is now |
|---|---|
| Separate driver per API protocol | Single `LiterLlmDriver` |
| Manual tool-call format conversion | Automatic via liter-llm `Provider` trait |
| Provider enum with 5+ variants | `provider/model` string (e.g. `openai/gpt-4o`) |
| `load_llm_settings()` env-var parsing | `load_llm_config(cli)` from layered sources |
| Hard-coded provider list | 142+ providers from `schemas/providers.json` |

---

## Architecture

### Driver

`LiterLlmDriver` (in `src/llm/liter_driver.rs`) wraps `liter_llm::DefaultClient` and implements the UAR `LlmDriver` trait:

```rust
pub struct LiterLlmDriver {
    client: Arc<DefaultClient>,
    model: String,
    parallel_tool_calls: Option<bool>,
}

impl LlmDriver for LiterLlmDriver {
    async fn stream(
        &self,
        req: LlmRequest,
    ) -> anyhow::Result<Pin<Box<dyn Stream<Item = anyhow::Result<NormalizedEvent>> + Send>>> {
        // ...
    }
}
```

The driver:
1. Converts UAR's JSON messages to `liter_llm::Message` (via `serde_json::from_value`)
2. Converts UAR's JSON tool schemas to `liter_llm::ChatCompletionTool`
3. Calls `client.chat_stream(req)` — liter-llm handles provider routing, auth, and wire format
4. Iterates `ChatCompletionChunk` events and maps them to `NormalizedEvent`

### ChatCompletionChunk → NormalizedEvent Mapping

```
chunk.choices[i].delta.content  → NormalizedEvent::MessageDelta { text }

chunk.choices[i].delta.tool_calls[j]
    → NormalizedEvent::ToolCallDelta {
          call_index, id, name, arguments_delta
      }
    (accumulated in ToolAccum per call_index)

chunk.choices[i].finish_reason == ToolCalls
    → NormalizedEvent::ToolCallComplete {
          call_index, id, name, arguments_json
      }
    (one per accumulated tool call)

chunk.usage
    → NormalizedEvent::Usage {
          prompt_tokens, completion_tokens, total_tokens
      }

(at end of stream)
    → NormalizedEvent::Done

(on error)
    → NormalizedEvent::Error { message, code }
```

All provider-specific tool-call formats (Anthropic `tool_use`, Google `functionCall`, Mistral blocks) are normalized by liter-llm *before* the chunk reaches `LiterLlmDriver`. UAR sees only the OpenAI shape.

### ClientConfig Construction

`config::build_client_config(llm: &LlmConfig) -> liter_llm::ClientConfig`:

```rust
pub fn build_client_config(llm: &LlmConfig) -> liter_llm::ClientConfig {
    let api_key = llm.api_key.clone()
        .or_else(|| env::var("LLM_API_KEY").ok())
        .unwrap_or_default();

    let mut builder = liter_llm::ClientConfigBuilder::new(api_key)
        .timeout(Duration::from_secs(llm.timeout_secs))
        .max_retries(llm.max_retries);

    if let Some(ref url) = llm.base_url {
        builder = builder.base_url(url.clone());
    }

    builder.build()
}
```

### Orchestrator

`Orchestrator::new()` always constructs a `LiterLlmDriver` — there is no protocol dispatch:

```rust
pub fn new(
    llm_config: LlmConfig,
    mcp: Arc<McpRegistry>,
    native_skills: Arc<NativeSkillRegistry>,
) -> anyhow::Result<Self> {
    let client_config = build_client_config(&llm_config);
    let driver: Arc<dyn LlmDriver> = Arc::new(LiterLlmDriver::new(
        client_config,
        llm_config.model.clone(),
        llm_config.parallel_tool_calls,
    )?);
    Ok(Self { llm_config, mcp, driver, native_skills })
}
```

The tool loop runs up to `MAX_TOOL_ITERATIONS = 10` times, feeding tool results back into the conversation until the model produces a final text response.

---

## LlmConfig

`LlmConfig` (in `src/config.rs`) is the unified configuration struct:

```rust
pub struct LlmConfig {
    /// Model in provider/model format (e.g. "openai/gpt-4o")
    pub model: String,
    /// API key for the provider
    pub api_key: Option<String>,
    /// Base URL override (for local endpoints or proxies)
    pub base_url: Option<String>,
    /// Protocol hint: "auto" | "chat" | "responses"
    pub protocol: String,
    /// Enable/disable parallel tool calls (None = auto)
    pub parallel_tool_calls: Option<bool>,
    /// Request timeout in seconds (default: 60)
    pub timeout_secs: u64,
    /// Retry attempts (default: 3)
    pub max_retries: u32,
    /// Cache configuration (Tower middleware)
    pub cache: Option<LlmCacheConfig>,
    /// Budget enforcement
    pub budget: Option<LlmBudgetConfig>,
    /// Rate limiting
    pub rate_limit: Option<LlmRateLimitConfig>,
    /// Enable cost tracking (default: false)
    pub cost_tracking: bool,
    /// Enable OpenTelemetry tracing (default: true)
    pub tracing: bool,
    /// Cooldown between requests (seconds)
    pub cooldown_secs: Option<u64>,
    /// Health check interval (seconds)
    pub health_check_secs: Option<u64>,
}
```

### Loading LlmConfig

```rust
// In application startup (src/main.rs):
let cli = Cli::parse();
let app_config = AppConfig::load_with_cli(cli)?;
let llm_config: LlmConfig = app_config.llm;

// Or as a standalone function:
let llm_config: LlmConfig = load_llm_config(cli)?;
```

`load_llm_config` merges all layers (defaults → YAML → env → CLI) and returns just the `LlmConfig` portion of `AppConfig`.

---

## Compile-Time Model Catalog

### How It Works

`build.rs` runs before every compilation and:

1. Locates `liter-llm/schemas/providers.json` (sibling directory or Cargo git cache)
2. Fetches `https://models.dev/api.json` (skipped if `SKIP_MODEL_BUILD=1`)
3. Merges: liter-llm provider routing data + models.dev model capabilities
4. Writes the result to `$OUT_DIR/provider_catalog.json`
5. `catalog.rs` embeds it via `include_str!` into a `LazyLock<ModelCatalog>` singleton

### ModelCatalog API

```rust
let catalog = ModelCatalog::global();

// All providers
for provider in catalog.all_providers() {
    println!("{}: {} models", provider.id, provider.models.len());
}

// Look up by ID
let openai = catalog.provider("openai");
let gpt4o  = catalog.model("openai", "gpt-4o");

// Filter by capability
let capable = catalog.models_with_capabilities(&CapabilityFilter {
    needs_tool_call: true,
    needs_vision: Some(false),
    min_context_window: Some(32_000),
    ..Default::default()
});
```

### ProviderInfo Fields

```rust
pub struct ProviderInfo {
    pub id: String,              // "openai"
    pub display_name: String,    // "OpenAI"
    pub base_url: Option<String>,
    pub auth: Option<ProviderAuth>,  // env_var: "OPENAI_API_KEY"
    pub endpoints: Vec<String>,      // ["chat/completions", ...]
    pub model_prefixes: Vec<String>, // for routing
    pub models: Vec<ModelInfo>,
}

pub struct ModelInfo {
    pub id: String,                    // "gpt-4o"
    pub name: String,                  // "GPT-4o"
    pub capabilities: ModelCapabilities,
    pub modalities: Modalities,        // input: ["text", "image"], ...
    pub limits: ModelLimits,           // context_window, max_output
    pub cost: Option<ModelCost>,       // input/output per 1M tokens
    pub open_weights: bool,
}

pub struct ModelCapabilities {
    pub tool_call: bool,
    pub reasoning: bool,
    pub structured_output: bool,
    pub attachment: bool,          // file/image input
    pub temperature: bool,
    pub streaming: bool,
}
```

---

## ProviderRegistry

`ProviderRegistry` (in `src/llm/registry.rs`) manages runtime API-key configuration. It integrates with the `ModelCatalog` for enrichment.

### Seeding from LlmConfig

```rust
// Called at startup — extracts provider from "openai/gpt-4o"
// and enriches with catalog data (display name, base URL, model list)
registry.seed_from_llm_config(&app_config.llm).await;
```

### Registering Custom Providers

```rust
// Auto-enriched: if "groq" is in the catalog, models/display_name/base_url
// are pulled from catalog data. User config (api_key) takes precedence.
registry.register_custom_provider(ProviderConfig {
    id: "groq".to_string(),
    api_key: Some(groq_api_key),
    ..Default::default()
}).await?;

// Fully custom (not in catalog):
registry.register_custom_provider(ProviderConfig {
    id: "internal-proxy".to_string(),
    display_name: "Internal LLM Proxy".to_string(),
    base_url: "https://llm.corp.example.com".to_string(),
    api_key: Some(proxy_key),
    default_model: Some("gpt-4o".to_string()),
    enabled: true,
    ..Default::default()
}).await?;
```

### Resolving to LlmConfig

```rust
// For driver construction:
let llm_config = registry
    .resolve_to_llm_config("openai", "gpt-4o")
    .await
    .ok_or_else(|| anyhow!("Provider not configured"))?;

// From agent policy (with fallbacks):
let llm_config = registry
    .resolve_llm_config_from_policy(&agent.policy.provider)
    .await;
```

---

## ModelRouter

`ModelRouter` (in `src/llm/router.rs`) selects the optimal available model without network calls.

### Route Requirements

```rust
pub struct RouteRequirements {
    pub needs_tools: bool,
    pub needs_vision: bool,
    pub min_context: Option<u64>,
    pub max_cost_per_1m_tokens: Option<f64>,
    pub preferred_provider: Option<String>,
}
```

### API

```bash
POST /api/uar/route
Content-Type: application/json

{
  "needs_tools": true,
  "needs_vision": false,
  "min_context": 32000,
  "max_cost_per_1m_tokens": 3.0,
  "preferred_provider": "openai"
}
```

Response:
```json
{
  "provider_id": "openai",
  "model_id": "gpt-4o",
  "full_model": "openai/gpt-4o",
  "reasoning": "Matched preferred provider with tool support and 128000 token context"
}
```

---

## Adding a New Provider

liter-llm handles new providers in its own `schemas/providers.json`. If you need a provider not yet in liter-llm:

1. Add it to `liter-llm/schemas/providers.json` (and push to the liter-llm fork)
2. It will appear in UAR's catalog on the next build
3. Register its API key at runtime via `register_custom_provider()` or `config.yaml`

No UAR code changes required for new providers.

---

## Build-Time Flags

| Flag | Effect |
|---|---|
| `SKIP_CATALOG_BUILD=1` | Skip catalog generation; use empty stub |
| `SKIP_MODEL_BUILD=1` | Skip `models.dev` HTTP fetch; use liter-llm data only |
| `SKIP_FRONTEND_BUILD=1` | Skip frontend asset compilation |

These are useful for fast iteration:

```bash
SKIP_MODEL_BUILD=1 SKIP_FRONTEND_BUILD=1 SKIP_CATALOG_BUILD=1 cargo check
```

---

## Migration from Legacy Drivers

If you forked UAR before the liter-llm migration, here's what changed:

| Was | Is now |
|---|---|
| `ChatCompletionsDriver` | `LiterLlmDriver` |
| `ResponsesDriver` | `LiterLlmDriver` |
| `Provider` enum | `provider/model` string |
| `LlmSettings` struct | `LlmConfig` struct |
| `load_llm_settings()` | `load_llm_config(cli)` |
| `LLM_BASE_URL` + `LLM_MODEL` (manual compose) | `UAR_LLM__MODEL=openai/gpt-4o` |
| `LlmProtocol::Chat` / `::Responses` dispatch | No dispatch — one driver |
| Empty provider list | 142+ providers from compile-time catalog |

Environment variables `LLM_MODEL`, `LLM_API_KEY`, `LLM_BASE_URL`, `LLM_PROTOCOL` are still supported for backward compatibility — they map to `LlmConfig` fields with lower priority than `UAR_LLM__*` vars.
