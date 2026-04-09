# UAR Environment Variables Reference

All `UAR_*` prefixed environment variables. Loaded via the `config` crate with prefix `UAR_`, separator `__`.

**Format**: `UAR_SECTION__KEY=value` maps to `section.key` in `config.yaml`.

---

## Server

| Variable | Default | Example |
|----------|---------|---------|
| `UAR_SERVER__PORT` | `3000` | `UAR_SERVER__PORT=8080` |
| `UAR_SERVER__HOST` | `0.0.0.0` | `UAR_SERVER__HOST=127.0.0.1` |

## Security

| Variable | Default | Example |
|----------|---------|---------|
| `UAR_SECURITY__JWT_REQUIRED` | `true` | `UAR_SECURITY__JWT_REQUIRED=false` |
| `UAR_SECURITY__JWT_SECRET` | `fallback_secret...` | `UAR_SECURITY__JWT_SECRET=$(openssl rand -base64 64)` |

## Resilience

| Variable | Default | Example |
|----------|---------|---------|
| `UAR_RESILIENCE__RATE_LIMIT_ENABLED` | `true` | `UAR_RESILIENCE__RATE_LIMIT_ENABLED=false` |
| `UAR_RESILIENCE__REQUESTS_PER_SECOND` | `10.0` | `UAR_RESILIENCE__REQUESTS_PER_SECOND=50.0` |
| `UAR_RESILIENCE__BURST_SIZE` | `20.0` | `UAR_RESILIENCE__BURST_SIZE=100.0` |
| `UAR_RESILIENCE__TIMEOUT_DISABLED` | `false` | `UAR_RESILIENCE__TIMEOUT_DISABLED=true` |
| `UAR_RESILIENCE__REQUEST_TIMEOUT_MS` | `30000` | `UAR_RESILIENCE__REQUEST_TIMEOUT_MS=120000` |
| `UAR_RESILIENCE__RETRY_MAX_ATTEMPTS` | `3` | `UAR_RESILIENCE__RETRY_MAX_ATTEMPTS=5` |

## Persistence

| Variable | Default | Example |
|----------|---------|---------|
| `UAR_PERSISTENCE__PROVIDER` | `postgres` | `UAR_PERSISTENCE__PROVIDER=surreal` |
| `UAR_PERSISTENCE__DATABASE_URL` | — | `UAR_PERSISTENCE__DATABASE_URL=postgres://user:pass@localhost/uar` |
| `UAR_PERSISTENCE__EXTERNAL_CACHE_ENABLED` | `false` | `UAR_PERSISTENCE__EXTERNAL_CACHE_ENABLED=true` |
| `UAR_PERSISTENCE__REDIS_URL` | — | `UAR_PERSISTENCE__REDIS_URL=redis://localhost:6379` |
| `UAR_PERSISTENCE__VECTOR_DIMENSION` | `1536` | `UAR_PERSISTENCE__VECTOR_DIMENSION=1024` |

## LLM

| Variable | Legacy Equivalent | Default | Example |
|----------|------------------|---------|---------|
| `UAR_LLM__MODEL` | `LLM_MODEL` | `openai/gpt-4o` | `UAR_LLM__MODEL=candle-vllm/llama-3-8b` |
| `UAR_LLM__API_KEY` | `LLM_API_KEY` | — | `UAR_LLM__API_KEY=sk-...` |
| `UAR_LLM__BASE_URL` | `LLM_BASE_URL` | — | `UAR_LLM__BASE_URL=http://localhost:3000` |
| `UAR_LLM__PROTOCOL` | `LLM_PROTOCOL` | `auto` | `UAR_LLM__PROTOCOL=chat` |
| `UAR_LLM__TIMEOUT_SECS` | — | `60` | `UAR_LLM__TIMEOUT_SECS=120` |
| `UAR_LLM__MAX_RETRIES` | — | `3` | `UAR_LLM__MAX_RETRIES=2` |
| `UAR_LLM__COST_TRACKING` | — | `false` | `UAR_LLM__COST_TRACKING=true` |
| `UAR_LLM__BUDGET__GLOBAL_LIMIT` | — | — | `UAR_LLM__BUDGET__GLOBAL_LIMIT=50.0` |

## Provider API Key Shortcuts

These are auto-detected and mapped to `llm.api_key` if no explicit key is set:

| Variable | Provider |
|----------|---------|
| `OPENAI_API_KEY` | openai |
| `ANTHROPIC_API_KEY` | anthropic |
| `GROQ_API_KEY` | groq |
| `MISTRAL_API_KEY` | mistral |
| `COHERE_API_KEY` | cohere |
| `GEMINI_API_KEY` | google |
| `TOGETHER_API_KEY` | together |
| `PERPLEXITY_API_KEY` | perplexity |

## Vision

| Variable | Default | Example |
|----------|---------|---------|
| `UAR_VISION__MODEL` | — | `UAR_VISION__MODEL=openai/gpt-4o` |
| `UAR_VISION__AUTO_DETECT` | `true` | `UAR_VISION__AUTO_DETECT=false` |

## Models (Intent Classification)

| Variable | Default | Example |
|----------|---------|---------|
| `UAR_MODELS_DIR` | `src/uar/runtime/matching/models` | `UAR_MODELS_DIR=/opt/uar/models` |

## Memory

| Variable | Default | Example |
|----------|---------|---------|
| `UAR_MEMORY__ENABLED` | `false` | `UAR_MEMORY__ENABLED=true` |
| `UAR_MEMORY__DB_PATH` | `./data/memory.db` | `UAR_MEMORY__DB_PATH=/var/lib/uar/memory.db` |
| `UAR_MEMORY__SURREAL_ENDPOINT` | — | `UAR_MEMORY__SURREAL_ENDPOINT=ws://localhost:8000/rpc` |
| `UAR_MEMORY__EMBEDDING_PROVIDER` | `openai` | `UAR_MEMORY__EMBEDDING_PROVIDER=local` |
| `UAR_MEMORY__EMBEDDING_MODEL` | `text-embedding-3-small` | `UAR_MEMORY__EMBEDDING_MODEL=text-embedding-3-large` |
| `UAR_MEMORY__AUTO_CAPTURE` | `true` | `UAR_MEMORY__AUTO_CAPTURE=false` |
| `UAR_MEMORY__MAX_CONTEXT_TOKENS` | `2000` | `UAR_MEMORY__MAX_CONTEXT_TOKENS=4000` |

## Legacy Variables (Deprecated — still work, lower priority)

| Legacy Var | UAR Equivalent | Notes |
|-----------|---------------|-------|
| `LLM_API_KEY` | `UAR_LLM__API_KEY` | Works in UAR 2.x but discouraged |
| `LLM_MODEL` | `UAR_LLM__MODEL` | Must use `provider/model` format |
| `LLM_BASE_URL` | `UAR_LLM__BASE_URL` | Same value |
| `LLM_PROTOCOL` | `UAR_LLM__PROTOCOL` | Same value |
| `PORT` | `UAR_SERVER__PORT` | Bare CLI env var |
| `JWT_REQUIRED` | `UAR_SECURITY__JWT_REQUIRED` | Bare CLI env var |
| `RATE_LIMIT_ENABLED` | `UAR_RESILIENCE__RATE_LIMIT_ENABLED` | Bare CLI env var |
| `EXTERNAL_CACHE_ENABLED` | `UAR_PERSISTENCE__EXTERNAL_CACHE_ENABLED` | Bare CLI env var |
