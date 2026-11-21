# Provider Configuration Guide

UAR's LLM layer is powered by **[liter-llm](https://github.com/GQAdonis/liter-llm)**, a Rust-native universal LLM client. It supports 142+ providers through a single unified API and handles tool-call normalization across all wire formats automatically.

---

## Model Addressing

All providers and models are addressed in `provider/model` format:

```
openai/gpt-4o
anthropic/claude-sonnet-4
google/gemini-2.0-flash
groq/llama-3.3-70b-versatile
mistral/mistral-large-latest
ollama/llama3.2
```

This is the format used in:
- `UAR_LLM__MODEL` env var
- `llm.model` in `config.yaml`
- `--llm-model` CLI argument
- Chat API requests: `"model": "openai/gpt-4o"`

---

## Quick Configuration

### Native service bootstrap

Native installation reads provider credentials from the invoking shell once
and writes only an allowlisted service environment. It never copies the whole
profile and never writes literal keys into YAML.

| Provider seed | Canonical variable | Approved aliases, in order | Seeded models |
|---|---|---|---|
| Kimi Coding | `KIMI_API_KEY` | `KIMI_CODING_API_KEY`, `KIMI_CODING_KEY` | `kimi-for-coding/k3` |
| MiniMax | `MINIMAX_API_KEY` | `MINIMAX_KEY` | `minimax/MiniMax-M3` |
| Alibaba/Qwen | `DASHSCOPE_API_KEY` | `QWEN_API_KEY`, `QWEN_TOKEN_PLAN_API_KEY` | `alibaba/qwen3.8-max` |
| Z.AI | `ZAI_API_KEY` | none | `zai/glm-4.7`, `zai/glm-5.2` |
| Moonshot | `MOONSHOT_API_KEY` | none | `moonshotai/kimi-k2.5`, `moonshotai/kimi-k3` |

The local OpenAI-compatible proxy at `http://127.0.0.1:8181/v1` is seeded only
from an observed `/models` inventory. If inventory cannot be read, bootstrap
omits that provider instead of inventing names. RunPod and tool-only or
endpoint/model-less credentials are excluded. Existing YAML provider IDs and
database-backed provider/default-model choices are never replaced by refresh,
apart from the exact obsolete native Alibaba values documented below.

Alibaba's released flagship API identifier is `qwen3.8-max`; the retired
`qwen3.8-max-preview` is not seeded. Native refresh upgrades only the exact
phase-owned `qwen3-coder-plus` seed and exact selected
`alibaba/qwen3.7-max`. It also repairs the observed malformed
`QWEN_TOKENPLAN_API_KEY` YAML indirection to canonical `DASHSCOPE_API_KEY`.
Other Alibaba model selections, credential references, endpoints, and custom
provider blocks remain operator-owned.

### Proposed Apple first-run bootstrap

**Specification only (2026-11-15); not implemented or locally verified by this change.** A new supported Apple Silicon Mac with no saved model choice, explicit configuration or bootstrap opt-out should use the available on-device Apple `SystemLanguageModel` to assist setup. Check chip/OS, compiled adapter, Apple Intelligence and ready system assets; retain manual setup when unavailable. Supported iOS/iPadOS profiles require their own qualification. Existing selections always win.

The system-model availability API starts at macOS/iOS 26. The optional custom `LanguageModel` integration is a separate macOS 27 SDK/runtime path: `MLXLanguageModel` with loader-owned `weightsLocation`/`load`, `CoreAILanguageModel` with exported resources, or a bridge to flint-loader. Direct native inference is allowed only with declared model, hardware and capability support. Core AI FP4/FP8 activation/KV-cache presets require macOS 27.2+. These are eligibility floors, not dependency pins or claims of support for every model.

UAR remains the sole agent/tool execution authority. Flint-loader owns custom-model acquisition and load/unload leases; Apple owns system assets. Setup shows durable progress, reasons and cancel/retry/resume controls. Downloads, provider changes, cloud transitions (including Private Cloud Compute) and credential acquisition/use require explicit consent; no silent remote fallback. Generation switches never change embedding identities, vector dimensions or reindex policy.

The [direct provider specification](../openspec/specs/provider-model-settings-certification/spec.md) and [design with primary sources and future VLBV gates](../openspec/changes/apple-foundation-models-bootstrap/design.md) define the target contract. No existing configuration key enables it yet. Do not treat this section as an install or runtime instruction.

### Environment Variables (Recommended)

The simplest approach: set your provider's API key and the model you want.

```bash
# OpenAI
OPENAI_API_KEY=sk-...
UAR_LLM__MODEL=openai/gpt-4o

# Anthropic Claude
ANTHROPIC_API_KEY=sk-ant-...
UAR_LLM__MODEL=anthropic/claude-sonnet-4

# Google Gemini
GEMINI_API_KEY=...
UAR_LLM__MODEL=google/gemini-2.0-flash

# Groq (fast inference, free tier)
GROQ_API_KEY=gsk_...
UAR_LLM__MODEL=groq/llama-3.3-70b-versatile

# Mistral
MISTRAL_API_KEY=...
UAR_LLM__MODEL=mistral/mistral-large-latest

# Cohere
COHERE_API_KEY=...
UAR_LLM__MODEL=cohere/command-r-plus

# Together.ai
TOGETHER_API_KEY=...
UAR_LLM__MODEL=together/meta-llama/Llama-3-70b-chat-hf

# Perplexity
PERPLEXITY_API_KEY=...
UAR_LLM__MODEL=perplexity/llama-3.1-sonar-large-128k-online

# Local Ollama (no API key needed)
UAR_LLM__MODEL=ollama/llama3.2
UAR_LLM__BASE_URL=http://localhost:11434

# LM Studio (local, OpenAI-compatible)
UAR_LLM__MODEL=lmstudio/my-model
UAR_LLM__BASE_URL=http://localhost:1234/v1
```

---

## Configuration Precedence

Settings are merged from multiple sources. Higher priority overrides lower:

| Priority | Source | Example |
|---|---|---|
| **1 (highest)** | CLI flags | `--llm-model openai/gpt-4o --llm-api-key sk-...` |
| **2** | `UAR_LLM__*` env vars | `UAR_LLM__MODEL=openai/gpt-4o` |
| **3** | Legacy `LLM_*` env vars | `LLM_MODEL=gpt-4o` (backward compat) |
| **4** | Provider shortcut keys | `OPENAI_API_KEY=sk-...` |
| **5** | `config.yaml` `llm:` section | see YAML reference below |
| **6 (lowest)** | Compiled defaults | `openai/gpt-4o`, 60s timeout |

### Environment Variable Reference

```bash
# Primary (structured) env vars — highest env priority
UAR_LLM__MODEL=openai/gpt-4o          # provider/model format
UAR_LLM__API_KEY=sk-...               # API key for the provider
UAR_LLM__BASE_URL=http://localhost:11434  # override for local/proxy
UAR_LLM__TIMEOUT_SECS=60
UAR_LLM__MAX_RETRIES=3
UAR_LLM__COST_TRACKING=false
UAR_LLM__TRACING=true
UAR_LLM__BUDGET__GLOBAL_LIMIT=10.0    # USD spending cap

# Provider-specific shortcut keys (auto-mapped to llm.api_key)
OPENAI_API_KEY=sk-...
ANTHROPIC_API_KEY=sk-ant-...
GROQ_API_KEY=gsk_...
MISTRAL_API_KEY=...
COHERE_API_KEY=...
GEMINI_API_KEY=...
TOGETHER_API_KEY=...
PERPLEXITY_API_KEY=...

# Legacy vars (still supported, lower priority than UAR_LLM__*)
LLM_MODEL=gpt-4o
LLM_API_KEY=sk-...
LLM_BASE_URL=https://api.openai.com/v1
```

### CLI Arguments

```bash
./universal-agent-runtime \
  --llm-model openai/gpt-4o \
  --llm-api-key sk-... \
  --llm-base-url https://api.openai.com/v1 \
  --llm-protocol auto \
  --llm-budget-limit 5.0
```

---

## YAML Configuration Reference

```yaml
# config.yaml
llm:
  # Model in provider/model format (required)
  model: "openai/gpt-4o"

  # API key — prefer environment variables for security
  # api_key: "sk-..."

  # Base URL override — use for local endpoints or proxies
  # base_url: "http://localhost:11434"

  # Request timeout in seconds (default: 60)
  timeout_secs: 60

  # Maximum retry attempts on transient errors (default: 3)
  max_retries: 3

  # Enable/disable parallel tool calls (default: null = auto-detect)
  # parallel_tool_calls: true

  # Enable cost tracking via Tower middleware (default: false)
  cost_tracking: false

  # Enable OpenTelemetry tracing (default: true)
  tracing: true

  # Cooldown between requests in seconds (default: null = none)
  # cooldown_secs: 1

  # Health check interval in seconds (default: null = disabled)
  # health_check_secs: 30

  # Budget enforcement
  # budget:
  #   global_limit: 10.0       # USD
  #   enforcement: "hard"      # "hard" = reject, "soft" = warn
  #   model_limits:
  #     "openai/gpt-4o": 5.0

  # Rate limiting
  # rate_limit:
  #   rpm: 60                  # requests per minute
  #   tpm: 100000              # tokens per minute
```

---

## Supported Providers

The full catalog is embedded at build time from [models.dev](https://models.dev) and liter-llm. Query it at runtime:

```bash
# All providers with model counts and auth env vars
curl http://localhost:3001/api/catalog

# Full model capabilities, pricing, and limits per provider
curl http://localhost:3001/api/models
```

### Selected Providers

| Provider | Example model | Key env var | Notes |
|---|---|---|---|
| **OpenAI** | `openai/gpt-4o` | `OPENAI_API_KEY` | Tool calling, vision, streaming |
| **Anthropic** | `anthropic/claude-sonnet-4` | `ANTHROPIC_API_KEY` | Tool calling, vision, extended thinking |
| **Google** | `google/gemini-2.0-flash` | `GEMINI_API_KEY` | Multimodal, long context |
| **Groq** | `groq/llama-3.3-70b-versatile` | `GROQ_API_KEY` | Ultra-fast LPU inference |
| **Mistral** | `mistral/mistral-large-latest` | `MISTRAL_API_KEY` | European provider |
| **Cohere** | `cohere/command-r-plus` | `COHERE_API_KEY` | RAG-optimized |
| **Together.ai** | `together/meta-llama/Llama-3-70b` | `TOGETHER_API_KEY` | Open model hosting |
| **Perplexity** | `perplexity/llama-3.1-sonar-large-128k-online` | `PERPLEXITY_API_KEY` | Real-time search |
| **Ollama** | `ollama/llama3.2` | *(none)* | Local, privacy-preserving |
| **LM Studio** | `lmstudio/<model>` | *(none)* | Local, OpenAI-compatible |
| **OpenRouter** | `openrouter/openai/gpt-4o` | `OPENROUTER_API_KEY` | Multi-provider gateway |
| **Azure OpenAI** | `azure/<deployment>` | `AZURE_API_KEY` | Enterprise Azure |
| **AWS Bedrock** | `bedrock/anthropic.claude-3` | AWS credentials | AWS-native |
| **Vertex AI** | `vertex/gemini-2.0-flash` | GCP credentials | Google Cloud |
| **Fireworks** | `fireworks/llama-v3-70b-instruct` | `FIREWORKS_API_KEY` | Fast open models |
| **Deepseek** | `deepseek/deepseek-chat` | `DEEPSEEK_API_KEY` | Code + reasoning |

142+ providers total. All tool-call format differences are normalized automatically.

---

## Tool Calling

All providers route through the same `LiterLlmDriver`, which normalizes tool-call formats:

| Provider wire format | Normalized to |
|---|---|
| Anthropic `tool_use` / `tool_result` content blocks | OpenAI `tool_calls` / `tool` role messages |
| Google `functionCall` / `functionResponse` | OpenAI `tool_calls` / `tool` role messages |
| Mistral `tool_calls` (slightly different shape) | OpenAI `tool_calls` |
| Standard OpenAI `tool_calls` | Passed through unchanged |

You do not need to configure anything for tool calling — it works automatically for any provider that supports it. Capability data is included in the model catalog (`GET /api/models`).

---

## Multi-Provider Configuration

You can configure additional provider overrides (e.g., custom API keys per team, internal proxies) via the `providers` section in `config.yaml`. These supplement the compile-time catalog — the catalog provides model lists and defaults, your config provides API keys and overrides.

```yaml
providers:
  - id: "openai"
    display_name: "OpenAI (Production)"
    base_url: "https://api.openai.com/v1"
    api_key: "${OPENAI_API_KEY}"
    default_model: "gpt-4o"
    enabled: true

  - id: "groq"
    display_name: "Groq (Fast)"
    base_url: "https://api.groq.com/openai/v1"
    api_key: "${GROQ_API_KEY}"
    default_model: "llama-3.3-70b-versatile"
    enabled: true

  - id: "internal-proxy"
    display_name: "Internal LLM Proxy"
    base_url: "https://llm-proxy.corp.example.com"
    api_key: "${INTERNAL_PROXY_KEY}"
    default_model: "gpt-4o"
    enabled: true
```

Per-agent provider selection (in agent artifact YAML):

```yaml
policy:
  provider:
    default: { provider: "groq", model: "llama-3.3-70b-versatile" }
    fallbacks:
      - { provider: "openai", model: "gpt-4o-mini" }
      - { provider: "internal-proxy", model: "gpt-4o" }
```

### Provider REST API

Manage providers at runtime without restart:

| Method | Path | Description |
|---|---|---|
| `GET` | `/api/uar/providers` | List all configured providers |
| `GET` | `/api/uar/providers/{id}` | Get a single provider |
| `POST` | `/api/uar/providers` | Register a new provider override |
| `PUT` | `/api/uar/providers/{id}` | Update a provider |
| `DELETE` | `/api/uar/providers/{id}` | Remove a provider |
| `GET` | `/api/uar/providers/{id}/models` | List models for a provider |
| `POST` | `/api/uar/providers/{id}/default` | Set the default provider |

### Database persistence

When **`persistence`** is enabled in config (`persistence.provider`: `postgres`, or `surreal` / `surrealdb`, plus `persistence.database_url` and optional Surreal credentials), provider configuration is stored in the **application settings database** alongside other UAR settings:

1. **Initial load**: On startup, the settings DB is **seeded from the resolved app configuration** — the same merged sources as everywhere else (YAML `config.yaml`, CLI flags, `UAR_*` / environment variables, in the [precedence order](#configuration-precedence) above). Entries under `providers:` in YAML become rows keyed `provider.{id}`; the default provider id is also stored as `llm.default_provider` (derived from `llm.model`’s provider segment on first boot).

2. **Runtime source of truth**: After that bootstrap step, the running server **loads provider state from the database** into the in-process provider registry so routing and admin APIs match persisted data. Changes made through the Provider REST API (`POST`, `PUT`, `DELETE`, set default) are **written back to the database** and survive process restarts.

3. **Backend selection**: The same `persistence` block chooses **PostgreSQL** vs **SurrealDB** (embedded RocksDB / in-memory / or a remote Surreal server — see `database_url` and `surreal_user` / `surreal_pass` in config). No separate store is used for providers.

If persistence is **disabled**, provider API calls still update the in-memory registry for the lifetime of the process, but **nothing is written to a database**.

### Choosing which model answers

**The system default** is the default provider plus that provider's `default_model`. Change it in the console from the *System default model* panel at the top of **Providers** (or the command palette action *Change system default model*). The console saves the provider's `default_model` with `PUT /api/uar/providers/{id}`, then calls `POST /api/uar/providers/{id}/default`. The change applies to the next turn without a restart; the console shows it as `live` only after the runtime acknowledges.

- `POST /api/uar/providers/{id}/default` is the only writer of `llm.default_provider`. The settings API refuses that key with `422 unsupported_setting` (single PUT, bulk PUT and reset). A bulk settings save that includes it is refused as a whole.
- A disabled provider cannot become the default (`409`, `Provider '<id>' is disabled`); an unknown provider returns `404`.

**Which model a turn uses**, first match wins:

1. a one-turn override;
2. the conversation's model override;
3. the agent's explicit model (`policy.provider.default`, or the model in `extensions["uar.run_policy"]`);
4. the system default;
5. the configured fallback (`llm.model` from YAML or environment), used only when nothing above names a model and the system default cannot be served (for example, its provider was disabled after it became the default), and only if the fallback model itself is verified.

A model named by an override or an agent that cannot be served fails the turn with a visible error; it is never replaced by another model. A turn binds its model when it starts, so a default change never affects a reply already being written.

**Agents that keep their own model** do not follow the system default. The panel and the change result list them with their models and say how many agents follow the default. `GET /api/agents` exposes this as `runtime_agents[i].explicit_model`.

**Each answer shows a receipt**: the model the provider reported as serving (not the requested id), why it was used (`system default`, `this conversation`, `agent`, `this turn` or `configured fallback`) and, for the system default, when it became the default. Note: for `google_ai` and `vertex_ai`, the reported id can equal the requested id when the provider omits `modelVersion`.

**Model verification.** A turn and a conversation override save both require the model to be listed, enabled and verified (`POST /api/uar/providers/{id}/test` with `{ "model": "<model>" }`). Saving an override with a model its provider does not offer returns `422 model_not_offered` with the reason, and nothing is saved. Changing a provider's `default_model` does not invalidate its verifications. After upgrading to this release, verifications recorded by earlier versions are stale once: verify each model again before use.

Sub-agent model inheritance is unchanged and still pending an operator decision: a child run uses its own model if it names one, otherwise its parent's effective model.

---

## Model Routing

UAR can automatically select the best model for a given task based on the compile-time catalog:

```bash
curl -X POST http://localhost:3001/api/uar/route \
  -H 'Content-Type: application/json' \
  -d '{
    "needs_tools": true,
    "needs_vision": false,
    "min_context": 32000,
    "max_cost_per_1m_tokens": 3.0,
    "preferred_provider": "openai"
  }'
```

The router queries the embedded `ModelCatalog` — no network calls — and returns the best match from configured providers.

---

## Troubleshooting

### "Failed to load LLM configuration"

Ensure at minimum `UAR_LLM__MODEL` is set (or `LLM_MODEL` for legacy compat), and that an API key is available via the provider shortcut or `UAR_LLM__API_KEY`.

### Tool calls not working

1. Check `GET /api/models` to confirm `"tool_call": true` for your model
2. Check `GET /api/uar/providers` to confirm your provider is enabled
3. Tool-call support depends on the model — some Ollama models may not support it

### Local Ollama not connecting

```bash
# Confirm Ollama is running and accessible
curl http://localhost:11434/api/tags

# Config
UAR_LLM__MODEL=ollama/llama3.2
UAR_LLM__BASE_URL=http://localhost:11434
# No API key needed
```

### Viewing the full provider catalog

```bash
# All providers with auth env var hints
curl http://localhost:3001/api/catalog | jq '.providers[] | {id, display_name, auth_env_var, model_count}'

# All models with capability flags
curl http://localhost:3001/api/models | jq 'to_entries[] | {provider: .key, models: (.value.models | keys)}'
```
