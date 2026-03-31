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
  --llm-base-url https://api.openai.com \
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
    base_url: "https://api.openai.com"
    api_key: "${OPENAI_API_KEY}"
    default_model: "gpt-4o"
    enabled: true

  - id: "groq"
    display_name: "Groq (Fast)"
    base_url: "https://api.groq.com/openai"
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
