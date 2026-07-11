---
sidebar_position: 3
title: Configuration Reference
---

# Configuration Reference

UAR is configured from several layered sources. This page documents every
setting, how the layers combine, and the environment-variable naming
convention.

## The `UAR_*__*` nesting convention

Configuration is a nested structure (`server`, `security`, `persistence`,
`llm`, and so on). Environment variables map onto that structure with the
`UAR_` prefix and a **double underscore (`__`) as the nesting separator**:

```
UAR_<SECTION>__<KEY>=<value>
```

Examples:

| Env var | Sets |
|---|---|
| `UAR_SERVER__PORT=1906` | `server.port` |
| `UAR_SERVER__HOST=0.0.0.0` | `server.host` |
| `UAR_SECURITY__JWT_REQUIRED=true` | `security.jwt_required` |
| `UAR_PERSISTENCE__PROVIDER=surreal` | `persistence.provider` |
| `UAR_LLM__MODEL=openai/gpt-4o` | `llm.model` |
| `UAR_MEMORY__ENABLED=true` | `memory.enabled` |

The single underscore after `UAR` separates the prefix; the `__` separates
nesting levels. So `UAR_SERVER__GRPC_PORT` → `server.grpc_port`.

## Precedence order

When the same setting is supplied by more than one source, the **highest**
priority wins:

| Priority | Source | Example |
|---|---|---|
| 1 (highest) | CLI arguments | `--llm-model openai/gpt-4o`, `--port 1906` |
| 2 | `UAR_*__*` structured env vars | `UAR_LLM__MODEL=openai/gpt-4o` |
| 3 | Legacy `LLM_*` env vars | `LLM_MODEL=gpt-4o` |
| 4 | Provider shortcut env vars | `OPENAI_API_KEY=sk-...` |
| 5 | `config.yaml` (`llm:`, `server:`, …) | see YAML section |
| 6 (lowest) | Compiled defaults | `server.port = 1906`, `llm.model = openai/gpt-4o` |

A few short CLI env vars are honored as convenience overrides and are applied at
the CLI tier (priority 1): `PORT` / `--port`, `JWT_REQUIRED` /
`--jwt-required`, `RATE_LIMIT_ENABLED`, `TIMEOUT_DISABLED`, and
`EXTERNAL_CACHE_ENABLED`. For example, `PORT=8080` is equivalent to
`--port 8080` and overrides `UAR_SERVER__PORT`.

## Config file selection

A YAML config file is optional. UAR resolves one in this order:

1. `--config <path>` (or `CONFIG_FILE=<path>`).
2. `./config.yaml` in the working directory, if it exists.
3. `~/.uar/config.yaml`, if it exists.

The repository ships ready-to-use example files at its root:

- `config.embedded.yaml` — embedded SurrealDB (single machine / development).
- `config.remote.surreal.yaml` — remote SurrealDB server.
- `config.remote.postgres.yaml` — PostgreSQL (+ pgvector).

Run with an explicit file:

```bash
CONFIG_FILE=config.embedded.yaml cargo run
```

## Server

| Env var | YAML key | Default | Notes |
|---|---|---|---|
| `UAR_SERVER__PORT` (or `PORT` / `--port`) | `server.port` | `1906` | HTTP listen port. |
| `UAR_SERVER__HOST` | `server.host` | `0.0.0.0` | Bind address. |
| `UAR_SERVER__GRPC_PORT` | `server.grpc_port` | `50051` | A2A v0.3 gRPC transport port. |
| `UAR_SERVER__SHUTDOWN_TIMEOUT_SECS` | `server.shutdown_timeout_secs` | `30` | Graceful shutdown budget. |
| `UAR_SERVER__LOG_FORMAT` | `server.log_format` | `json` | `json` \| `compact` \| `pretty`. |

## Security

| Env var | YAML key | Default | Notes |
|---|---|---|---|
| `UAR_SECURITY__JWT_REQUIRED` (or `JWT_REQUIRED` / `--jwt-required`) | `security.jwt_required` | `true` | When true, protected endpoints require a valid JWT (401 otherwise). |
| `UAR_SECURITY__JWT_SECRET` | `security.jwt_secret` | `fallback_secret_change_in_production` | HMAC secret. **Always override in production.** Generate with `openssl rand -base64 64`. Redacted from logs. |
| `UAR_SECURITY__SETTINGS_MUTATION_AUTH_REQUIRED` | `security.settings_mutation_auth_required` | `true` | When true, `PUT`/`POST`/`DELETE` on `/api/uar/settings` require the `X-UAR-Admin-Key` header. Set `false` for trusted local dev only. |
| `CREDENTIAL_ENCRYPTION_KEY` | — | *(unset)* | Optional. Enables multi-tenant per-user provider credentials, encrypted at rest with AES-256-GCM. Must be 32 ASCII bytes or 64 hex chars. Leave unset for single-tenant/self-hosted operation. |

## LLM (liter-llm)

The active model is addressed as `provider/model` (e.g. `openai/gpt-4o`,
`anthropic/claude-sonnet-4`, `groq/llama-3.3-70b-versatile`,
`ollama/llama3.2`).

| Env var | YAML key | Default | Notes |
|---|---|---|---|
| `UAR_LLM__MODEL` (or `LLM_MODEL` / `--llm-model`) | `llm.model` | `openai/gpt-4o` | Default model in `provider/model` form. |
| `UAR_LLM__API_KEY` (or `LLM_API_KEY` / `--llm-api-key`) | `llm.api_key` | *(unset)* | API key for the default provider. Redacted from logs. |
| — | `llm.api_key_env` | *(unset)* | Name of an env var to read the key from (indirection so the secret stays out of `config.yaml`). |
| `UAR_LLM__BASE_URL` (or `LLM_BASE_URL` / `--llm-base-url`) | `llm.base_url` | *(unset)* | Override base URL (local proxies, Ollama, LM Studio). |
| `UAR_LLM__PROTOCOL` (or `LLM_PROTOCOL` / `--llm-protocol`) | `llm.protocol` | `auto` | `auto` \| `chat` \| `responses`. |
| `UAR_LLM__TIMEOUT_SECS` | `llm.timeout_secs` | `60` | Per-request timeout. |
| `UAR_LLM__MAX_RETRIES` | `llm.max_retries` | `3` | Retries on 429/5xx. |
| `UAR_LLM__COST_TRACKING` | `llm.cost_tracking` | `false` | Per-request cost tracking. |
| `UAR_LLM__TRACING` | `llm.tracing` | `true` | OpenTelemetry tracing spans. |
| `--llm-budget-limit` (`UAR_LLM__BUDGET__GLOBAL_LIMIT`) | `llm.budget.global_limit` | *(unset)* | Global spend cap (USD). |

### Provider shortcut keys

If `UAR_LLM__API_KEY` / `LLM_API_KEY` is not set, UAR maps well-known
provider-specific env vars to both the default key and a per-provider key used
for the registry's "configured" status:

| Env var | Provider id |
|---|---|
| `OPENAI_API_KEY` | `openai` |
| `ANTHROPIC_API_KEY` | `anthropic` |
| `GROQ_API_KEY` | `groq` |
| `MISTRAL_API_KEY` | `mistral` |
| `COHERE_API_KEY` | `cohere` |
| `GEMINI_API_KEY` | `google` |
| `TOGETHER_API_KEY` | `together` |
| `PERPLEXITY_API_KEY` | `perplexity` |

Local providers (`ollama/*`, `lmstudio/*`) need no key — set `UAR_LLM__BASE_URL`
to reach them, e.g. `http://localhost:11434` for Ollama.

### `config.yaml` `llm:` section

```yaml
llm:
  model: "openai/gpt-4o"        # provider/model format
  # api_key: "sk-..."           # prefer an env var instead
  # api_key_env: "MY_SECRET"    # or read the key from a named env var
  # base_url: "http://localhost:11434"
  protocol: "auto"
  timeout_secs: 60
  max_retries: 3
  cost_tracking: false
  tracing: true
```

## Persistence

UAR stores application state (sessions, skills, knowledge bases, settings) in
either SurrealDB (embedded or remote) or PostgreSQL.

| Env var | YAML key | Default | Notes |
|---|---|---|---|
| `UAR_PERSISTENCE__PROVIDER` | `persistence.provider` | *(required)* | `surreal` \| `postgres`. **No default** — must be set at boot. |
| `UAR_PERSISTENCE__DATABASE_URL` | `persistence.database_url` | *(required)* | Connection string (see below). |
| `UAR_PERSISTENCE__VECTOR_DIMENSION` | `persistence.vector_dimension` | `1536` | Must match your embedding model (e.g. `1536` for OpenAI `text-embedding-3-small`, `384` for BGE-small). |
| `UAR_PERSISTENCE__EXTERNAL_CACHE_ENABLED` | `persistence.external_cache_enabled` | `false` | Enable Redis-backed session cache. |
| `UAR_PERSISTENCE__REDIS_URL` (or `REDIS_URL`) | — | *(unset)* | Redis URL when external cache is enabled. |
| `UAR_PERSISTENCE__SURREAL_USER` | `persistence.surreal_user` | `root` | Username for **remote** SurrealDB (`ws`/`wss`/`http`/`https`). Ignored for embedded URLs. |
| `UAR_PERSISTENCE__SURREAL_PASS` | `persistence.surreal_pass` | `root` | Password for remote SurrealDB. Redacted from logs. |
| `UAR_PERSISTENCE__SURREAL_NS` | `persistence.surreal_ns` | `uar` | SurrealDB namespace. |
| `UAR_PERSISTENCE__SURREAL_DB` | `persistence.surreal_db` | `uar` | SurrealDB database name. |

### `database_url` forms

| Provider | Example URL | Mode |
|---|---|---|
| `surreal` | `surrealkv://./data/uar-db` | Embedded on-disk (SurrealKV). No separate process. |
| `surreal` | `rocksdb://./data/uar-db` | Embedded on-disk. `rocksdb://` is normalized to `surrealkv://`. |
| `surreal` | `memory` / `mem` | In-memory (ephemeral; testing only). |
| `surreal` | `http://127.0.0.1:8000` / `wss://host/rpc` | Remote SurrealDB server (requires `surreal_user` / `surreal_pass`). |
| `postgres` | `postgres://uar:changeme@localhost:5432/uar` | PostgreSQL (+ pgvector). |

:::note
`rocksdb://` and `surrealkv://` refer to the **same** embedded engine. UAR
rewrites `rocksdb://` to `surrealkv://` internally, so the two are
interchangeable in configuration. A bare path (no scheme) is treated as
`surrealkv://<path>`.
:::

:::info PostgreSQL requires a build feature
Selecting `persistence.provider = postgres` requires the binary to be built
with the `postgres-backend` Cargo feature. The default binary is built with
embedded SurrealDB only; without the feature, booting with `provider = postgres`
fails with a message telling you to rebuild with `postgres-backend` or switch to
`surreal`.
:::

## Memory system (optional)

Opt-in durable agent memory backed by surreal-memory + SurrealDB/SurrealKV.

| Env var | YAML key | Default |
|---|---|---|
| `UAR_MEMORY__ENABLED` | `memory.enabled` | `false` |
| `UAR_MEMORY__DB_PATH` | `memory.db_path` | `./data/memory.db` |
| `UAR_MEMORY__EMBEDDING_PROVIDER` | `memory.embedding_provider` | `openai` |
| `UAR_MEMORY__EMBEDDING_MODEL` | `memory.embedding_model` | `text-embedding-3-small` |
| `UAR_MEMORY__AUTO_CAPTURE` | `memory.auto_capture` | `true` |
| `UAR_MEMORY__INJECT_CONTEXT` | `memory.inject_context` | `true` |
| `UAR_MEMORY__MCP_HTTP_ENABLED` | `memory.mcp_http_enabled` | `true` |
| `UAR_MEMORY__MCP_HTTP_PATH` | `memory.mcp_http_path` | `/mcp/memory` |
| `UAR_MEMORY__SURREAL_ENDPOINT` | `memory.surreal_endpoint` | *(embedded)* |
| `UAR_MEMORY__SURREAL_USER` / `UAR_MEMORY__SURREAL_PASS` | `memory.surreal_user` / `_pass` | *(unset)* |

## Resilience (rate limiting, timeouts, retries)

| Env var | YAML key | Default |
|---|---|---|
| `UAR_RESILIENCE__RATE_LIMIT_ENABLED` (or `RATE_LIMIT_ENABLED`) | `resilience.rate_limit_enabled` | `true` |
| `UAR_RESILIENCE__REQUESTS_PER_SECOND` | `resilience.requests_per_second` | `10.0` |
| `UAR_RESILIENCE__BURST_SIZE` | `resilience.burst_size` | `20.0` |
| `UAR_RESILIENCE__TIMEOUT_DISABLED` (or `TIMEOUT_DISABLED`) | `resilience.timeout_disabled` | `false` |
| `UAR_RESILIENCE__REQUEST_TIMEOUT_MS` | `resilience.request_timeout_ms` | `30000` |
| `UAR_RESILIENCE__STREAM_START_TIMEOUT_MS` | `resilience.stream_start_timeout_ms` | `15000` |
| `UAR_RESILIENCE__RETRIES_ENABLED` | `resilience.retries_enabled` | `true` |
| `UAR_RESILIENCE__RETRY_MAX_ATTEMPTS` | `resilience.retry_max_attempts` | `3` |

## Optional feature toggles

| Env var | YAML key | Default | Feature |
|---|---|---|---|
| `UAR_FAILOVER__ENABLED` (`--failover-enabled`) | `failover.enabled` | `false` | Runtime model failover. |
| `UAR_NATIVE_TOOLS__FILE_TOOLS_ENABLED` | `native_tools.file_tools_enabled` | `false` | `file_read`/`file_write`/`file_patch` native tools. |
| `UAR_NATIVE_TOOLS__WEB_FETCH_ENABLED` | `native_tools.web_fetch_enabled` | `false` | `web_fetch` native tool. |
| `UAR_NATIVE_TOOLS__TERMINAL_EXEC_ENABLED` | `native_tools.terminal_exec_enabled` | `false` | `terminal_exec` native tool. |
| `UAR_SKILL_EVOLUTION__ENABLED` | `skill_evolution.enabled` | `false` | Post-run reflection → skills. |
| `UAR_ACP__ENABLED` (`--acp-enabled`) | `acp.enabled` | `false` | ACP JSON-RPC endpoint. |
| `UAR_ACP__PATH` (`--acp-path`) | `acp.path` | `/acp` | ACP path prefix. |

## Tools / integrations

| Env var | Purpose |
|---|---|
| `TAVILY_API_KEY` | Web-search MCP server (referenced by `mcp.json`). |
| `UAR_BUILTIN_SKILLS_DIR` | Primary builtin skills directory. |
| `UAR_EXTRA_BUILTIN_SKILL_DIRS` | Additional builtin skill dirs (colon-separated). |
| `UAR_MODELS_DIR` | Directory for model files (tokenizer, embeddings). |

## MCP tools (`mcp.json`)

MCP servers are configured separately in `mcp.json`. Both stdio and HTTP servers
are supported; tools are namespaced automatically (`time::now`,
`tavily::search`).

```json
{
  "mcpServers": {
    "time": { "command": "npx", "args": ["-y", "@mcpcentral/mcp-time"] },
    "tavily": { "url": "https://mcp.tavily.com/mcp/?tavilyApiKey=${TAVILY_API_KEY}" }
  }
}
```

## Minimal working example

```bash
# LLM: pick a provider and give it a key
UAR_LLM__MODEL=openai/gpt-4o
OPENAI_API_KEY=sk-...

# Persistence: required at boot
UAR_PERSISTENCE__PROVIDER=surreal
UAR_PERSISTENCE__DATABASE_URL=rocksdb://./data/uar-db

# Security
UAR_SECURITY__JWT_SECRET=$(openssl rand -base64 64)
UAR_SECURITY__JWT_REQUIRED=false   # local dev only
```

See **[Installation](./installation)** for the full first-run checklist.
