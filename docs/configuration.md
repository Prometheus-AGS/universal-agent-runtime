# Configuration Guide

UAR uses a hierarchical configuration system. Settings are merged from multiple sources in priority order (highest wins):

1. **CLI arguments** — flags passed to the binary
2. **Process/service environment** — including `--env-file` / `UAR_ENV_FILE`
3. **`UAR_LLM__*` env vars** — structured env vars for the liter-llm client
4. **Legacy `LLM_*` env vars** — backward-compatible single-var config
5. **Provider-shortcut keys** — `OPENAI_API_KEY`, `GROQ_API_KEY`, etc.
6. **`config.yaml`** — file-based configuration
7. **Compiled defaults** — safe defaults built into the binary

---

## 1. CLI Arguments

```bash
./universal-agent-runtime [OPTIONS]

Options:
      --env-file <PATH>           Service environment file [env: UAR_ENV_FILE]
  -c, --config <PATH>              Config file path [env: CONFIG_FILE]
      --port <PORT>                Server port [env: PORT]
      --jwt-required <BOOL>        Require JWT auth [env: JWT_REQUIRED]
      --rate-limit-enabled <BOOL>  Enable rate limiting [env: RATE_LIMIT_ENABLED]
      --llm-model <MODEL>          LLM model (provider/model format) [env: LLM_MODEL]
      --llm-api-key <KEY>          LLM API key [env: LLM_API_KEY]
      --llm-base-url <URL>         LLM base URL override [env: LLM_BASE_URL]
      --llm-protocol <PROTO>       LLM protocol hint [env: LLM_PROTOCOL]
      --llm-budget-limit <USD>     Global budget limit in USD [env: UAR_LLM__BUDGET__GLOBAL_LIMIT]
```

Example:
```bash
./universal-agent-runtime \
  --port 8080 \
  --llm-model openai/gpt-4o \
  --llm-api-key sk-...
```

---

## 2. Environment Variables

### LLM Configuration (liter-llm)

All LLM settings live under the `llm` namespace. Use `UAR_LLM__` prefix for structured access:

| Env Var | Config Key | Default | Description |
|---|---|---|---|
| `UAR_LLM__MODEL` | `llm.model` | `openai/gpt-4o` | Model in `provider/model` format |
| `UAR_LLM__API_KEY` | `llm.api_key` | — | API key for the provider |
| `UAR_LLM__BASE_URL` | `llm.base_url` | — | Base URL override |
| `UAR_LLM__TIMEOUT_SECS` | `llm.timeout_secs` | `60` | Request timeout |
| `UAR_LLM__MAX_RETRIES` | `llm.max_retries` | `3` | Retry attempts |
| `UAR_LLM__COST_TRACKING` | `llm.cost_tracking` | `false` | Enable cost tracking |
| `UAR_LLM__TRACING` | `llm.tracing` | `true` | Enable OTEL tracing |
| `UAR_LLM__BUDGET__GLOBAL_LIMIT` | `llm.budget.global_limit` | — | USD spending cap |

Provider-specific shortcut keys (automatically mapped to `llm.api_key` if no explicit key is set):

```bash
OPENAI_API_KEY=sk-...
ANTHROPIC_API_KEY=sk-ant-...
GROQ_API_KEY=gsk_...
MISTRAL_API_KEY=...
COHERE_API_KEY=...
GEMINI_API_KEY=...
TOGETHER_API_KEY=...
PERPLEXITY_API_KEY=...
```

Legacy vars (lower priority than `UAR_LLM__*`, kept for backward compatibility):

```bash
LLM_MODEL=gpt-4o
LLM_API_KEY=sk-...
LLM_BASE_URL=https://api.openai.com/v1
LLM_PROTOCOL=auto
```

### Server Configuration

| Env Var | Default | Description |
|---|---|---|
| `UAR_SERVER__PORT` | `1906` | Listening port |
| `UAR_SERVER__HOST` | `0.0.0.0` | Bind address |
| `PORT` | — | Alias for `UAR_SERVER__PORT` |
| `UAR_SERVER__GRPC_PORT` | `50051` | A2A v0.3 gRPC transport port (serves alongside the HTTP/JSON-RPC A2A endpoint) |
| `UAR_ENV_FILE` | — | Explicit dotenv file loaded before telemetry/configuration; unreadable or malformed files fail startup |
| `UAR_LOG_FILE` | — | Append operational tracing to this file; an open failure fails startup |

### Native service files

Native installers generate a least-privilege service environment instead of
sourcing a login profile on every start:

| Platform | YAML | Environment | Operational logs |
|---|---|---|---|
| macOS | `~/.uar/config.yaml` | `~/.uar/service.env` | `~/.prometheus/logs/universal-agent-runtime/` |
| Linux | `/etc/uar/config.yaml` | `/etc/uar/uar.env` | `/var/lib/uar/.prometheus/logs/` |
| Windows | `%ProgramData%\Prometheus\UniversalAgentRuntime\config\config.yaml` | `%ProgramData%\Prometheus\UniversalAgentRuntime\config\uar.env` | `%ProgramData%\Prometheus\UniversalAgentRuntime\.prometheus\logs\` |

An explicitly selected environment file loads before telemetry so
`UAR_LOG_FILE` controls the first operational trace. Process variables already
supplied by the supervisor remain authoritative. YAML supplies lower-priority
defaults; database-backed API/UI provider settings remain authoritative after
initial seed. Run the platform `refresh-credentials` entrypoint to regenerate
the allowlist and apply it through one service restart.

### Security

| Env Var | Default | Description |
|---|---|---|
| `UAR_SECURITY__JWT_REQUIRED` | `true` | Require JWT on API requests |
| `UAR_SECURITY__JWT_SECRET` | no safe default | HS256 signing secret; startup fails if required auth still uses the published fallback |
| `UAR_SECURITY__JWT_ISSUER` | — | Optional issuer; when set, tokens must contain the matching `iss` claim |
| `UAR_SECURITY__JWT_AUDIENCE` | — | Optional audience; when set, tokens must contain the matching `aud` claim |
| `UAR_SECURITY__JWT_VALIDATE_NBF` | `true` | Reject a token whose optional `nbf` claim is beyond jsonwebtoken's 60-second default clock-skew allowance |

When `jwt_required: true`, set a deliberate secret such as one generated by
`openssl rand -hex 32`; UAR refuses to start with its published fallback.
Setting `jwt_required: false` is an explicit anonymous deployment mode: requests
without valid credentials share the `anonymous` identity, so it does not provide
user or tenant isolation and should be limited to trusted local environments.

For local development against a `jwt_required: true` instance, see
[docs/dev-tools.md](dev-tools.md)'s `uar-jwt-proxy` — it mints and injects a
valid JWT automatically instead of requiring you to disable auth or mint
tokens by hand.

### Persistence

| Env Var | Default | Description |
|---|---|---|
| `UAR_PERSISTENCE__PROVIDER` | `postgres` | `postgres` or `surreal` |
| `UAR_PERSISTENCE__DATABASE_URL` | — | Database connection string |
| `DATABASE_URL` | — | Alias for `UAR_PERSISTENCE__DATABASE_URL` |
| `REDIS_URL` | `redis://localhost:6379` | Redis connection string |

### Memory System

| Env Var | Default | Description |
|---|---|---|
| `UAR_MEMORY__ENABLED` | `false` | Enable the memory subsystem |
| `UAR_MEMORY__PROVIDER` | `postgres` | Storage backend for memory |

---

## 3. Configuration File

UAR looks for `config.yaml` in:
1. Path from `--config` / `CONFIG_FILE`
2. `./config.yaml`
3. `~/.uar/config.yaml`

The file supports YAML (recommended), JSON, or TOML. See `example.config.yaml` for the full annotated reference.

### Minimal Example

```yaml
server:
  port: 3001

security:
  jwt_required: false

persistence:
  provider: postgres
  database_url: "postgres://user:password@localhost:5432/uar"

llm:
  model: "openai/gpt-4o"
  timeout_secs: 60
  max_retries: 3
  tracing: true
```

### Full LLM Section

```yaml
llm:
  # Model in provider/model format; support depends on the provider tier
  model: "openai/gpt-4o"

  # API key — prefer environment variables
  # api_key: "sk-..."

  # Base URL override for local or proxy endpoints
  # base_url: "http://localhost:11434"

  # Protocol hint: "auto" | "chat" | "responses" (default: "auto")
  # protocol: auto

  # Enable/disable parallel tool calls (null = auto-detect per provider)
  # parallel_tool_calls: true

  # Timeouts and retries
  timeout_secs: 60
  max_retries: 3

  # Cooldown between requests (seconds)
  # cooldown_secs: 1

  # Background health check interval (seconds)
  # health_check_secs: 30

  # Cost tracking (requires Tower middleware)
  cost_tracking: false

  # OpenTelemetry tracing
  tracing: true

  # Budget enforcement
  # budget:
  #   global_limit: 10.0      # USD; hard-stop when exceeded
  #   enforcement: "hard"     # "hard" = reject | "soft" = warn
  #   model_limits:
  #     "openai/gpt-4o": 5.0

  # Rate limiting
  # rate_limit:
  #   rpm: 60                 # requests per minute
  #   tpm: 100000             # tokens per minute

# Provider overrides — supplement the committed catalog snapshot
# providers:
#   - id: "openai"
#     display_name: "OpenAI"
#     base_url: "https://api.openai.com/v1"
#     api_key: "${OPENAI_API_KEY}"
#     default_model: "gpt-4o"
#     enabled: true
```

---

## 4. Multi-Provider Setup

The embedded `ModelCatalog` comes from the committed, checksummed snapshot in
`catalog/provider_catalog.json`. It provides discovery metadata; an entry does
not imply certified execution support. The `providers:` section in
`config.yaml` adds runtime API-key configuration and overrides. See the
[provider tiers](product-support-matrix.md#provider-tiers).

```yaml
providers:
  - id: "openai"
    base_url: "https://api.openai.com/v1"
    api_key: "${OPENAI_API_KEY}"
    default_model: "gpt-4o"
    enabled: true

  - id: "groq"
    base_url: "https://api.groq.com/openai/v1"
    api_key: "${GROQ_API_KEY}"
    default_model: "llama-3.3-70b-versatile"
    enabled: true
```

At runtime, `ProviderRegistry.register_custom_provider()` merges your config with catalog data — model lists, display names, and base URLs from the catalog fill in any gaps you leave blank.

See [docs/PROVIDER_CONFIGURATION.md](PROVIDER_CONFIGURATION.md) for the full provider reference.

---

## 5. MCP Tools Configuration

Configure MCP (Model Context Protocol) tool servers in `mcp.json`:

```json
{
  "mcpServers": {
    "time": {
      "command": "npx",
      "args": ["-y", "@mcpcentral/mcp-time"]
    },
    "tavily": {
      "url": "https://mcp.tavily.com/mcp/?tavilyApiKey=${TAVILY_API_KEY}",
      "env": {
        "TAVILY_API_KEY": "${TAVILY_API_KEY}"
      }
    }
  }
}
```

Tools are auto-namespaced (`time::now`, `tavily::search`) and available to every LLM call.

---

## 6. Settings Persistence

All configuration can also be viewed and modified at runtime via the Settings API (backed by the database):

| Method | Path | Description |
|---|---|---|
| `GET` | `/api/uar/settings` | List all setting namespaces |
| `GET` | `/api/uar/settings/{namespace}` | Get all settings in a namespace |
| `PUT` | `/api/uar/settings/{namespace}/{key}` | Update a single setting |

The `llm` namespace exposes all `LlmConfig` fields as persisted settings. Changes take effect on the next request cycle.

The admin UI at `/admin` provides a graphical settings editor for all namespaces.
