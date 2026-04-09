# UAR Configuration Reference

Complete reference for all `config.yaml` keys. Extracted from `src/config.rs`.

**Precedence**: CLI args → `UAR_*` env vars → `LLM_*` legacy vars → provider shortcuts → `config.yaml` → compiled defaults.

---

## server

| Key | Env Var | CLI Flag | Default | Type | Description |
|-----|---------|----------|---------|------|-------------|
| `server.port` | `UAR_SERVER__PORT` | `--port` | `3000` | u16 | HTTP listening port |
| `server.host` | `UAR_SERVER__HOST` | — | `"0.0.0.0"` | string | Bind address |

---

## security

| Key | Env Var | CLI Flag | Default | Type | Description |
|-----|---------|----------|---------|------|-------------|
| `security.jwt_required` | `UAR_SECURITY__JWT_REQUIRED` | `--jwt-required` | `true` | bool | Require JWT Bearer token on all requests |
| `security.jwt_secret` | `UAR_SECURITY__JWT_SECRET` | — | `"fallback_secret_change_in_production"` | string | HMAC signing secret. **Generate with `openssl rand -base64 64`** |

---

## resilience

| Key | Env Var | CLI Flag | Default | Type | Description |
|-----|---------|----------|---------|------|-------------|
| `resilience.rate_limit_enabled` | `UAR_RESILIENCE__RATE_LIMIT_ENABLED` | `--rate-limit-enabled` | `true` | bool | Enable token-bucket rate limiter |
| `resilience.requests_per_second` | `UAR_RESILIENCE__REQUESTS_PER_SECOND` | — | `10.0` | f64 | Steady-state refill rate |
| `resilience.burst_size` | `UAR_RESILIENCE__BURST_SIZE` | — | `20.0` | f64 | Max burst above steady rate |
| `resilience.timeout_disabled` | `UAR_RESILIENCE__TIMEOUT_DISABLED` | `--timeout-disabled` | `false` | bool | Disable request timeout middleware |
| `resilience.request_timeout_ms` | `UAR_RESILIENCE__REQUEST_TIMEOUT_MS` | — | `30000` | u64 | Per-request timeout (ms) |
| `resilience.stream_start_timeout_ms` | `UAR_RESILIENCE__STREAM_START_TIMEOUT_MS` | — | `15000` | u64 | Timeout waiting for first stream chunk (ms) |
| `resilience.retries_enabled` | `UAR_RESILIENCE__RETRIES_ENABLED` | — | `true` | bool | Enable automatic retries |
| `resilience.retry_max_attempts` | `UAR_RESILIENCE__RETRY_MAX_ATTEMPTS` | — | `3` | u32 | Max retry attempts |
| `resilience.retry_base_delay_ms` | `UAR_RESILIENCE__RETRY_BASE_DELAY_MS` | — | `1000` | u64 | Initial retry delay (ms) |
| `resilience.retry_backoff_multiplier` | `UAR_RESILIENCE__RETRY_BACKOFF_MULTIPLIER` | — | `2.0` | f64 | Exponential backoff multiplier |
| `resilience.retry_max_delay_ms` | `UAR_RESILIENCE__RETRY_MAX_DELAY_MS` | — | `10000` | u64 | Max retry delay cap (ms) |
| `resilience.retry_jitter_mode` | `UAR_RESILIENCE__RETRY_JITTER_MODE` | — | `"full"` | string | Jitter mode: `"full"` \| `"none"` |
| `resilience.retry_respect_retry_after` | `UAR_RESILIENCE__RETRY_RESPECT_RETRY_AFTER` | — | `true` | bool | Honor `Retry-After` response header |
| `resilience.retryable_http_statuses` | — | — | `[408,425,429,500,502,503,504]` | array | HTTP status codes that trigger retry |
| `resilience.retry_budget_ms` | `UAR_RESILIENCE__RETRY_BUDGET_MS` | — | `20000` | u64 | Total retry time budget (ms) |

---

## persistence

| Key | Env Var | CLI Flag | Default | Type | Description |
|-----|---------|----------|---------|------|-------------|
| `persistence.provider` | `UAR_PERSISTENCE__PROVIDER` | — | `"postgres"` | string | DB backend: `"postgres"` \| `"surreal"` |
| `persistence.database_url` | `UAR_PERSISTENCE__DATABASE_URL` | — | — | string | **Required.** Connection string |
| `persistence.external_cache_enabled` | `UAR_PERSISTENCE__EXTERNAL_CACHE_ENABLED` | `--external-cache-enabled` | `false` | bool | Enable Redis external cache |
| `persistence.redis_url` | `UAR_PERSISTENCE__REDIS_URL` | — | — | string | Redis connection string (if cache enabled) |
| `persistence.vector_dimension` | `UAR_PERSISTENCE__VECTOR_DIMENSION` | — | `1536` | u32 | Embedding dimension (match your embedding model) |

---

## file_processing

| Key | Env Var | Default | Description |
|-----|---------|---------|-------------|
| `file_processing.provider` | `UAR_FILE_PROCESSING__PROVIDER` | `"auto"` | `"auto"` \| `"unstructured"` \| `"mistral"` \| `"kreuzberg"` |
| `file_processing.upload_dir` | `UAR_FILE_PROCESSING__UPLOAD_DIR` | `/tmp/uar-uploads` | Temp upload directory |
| `file_processing.max_files_per_prompt` | — | `10` | Max files per request |
| `file_processing.max_file_size` | — | `52428800` (50MB) | Max single file size (bytes) |
| `file_processing.max_total_size` | — | `104857600` (100MB) | Max total size per request (bytes) |

---

## unstructured

| Key | Env Var | Default | Description |
|-----|---------|---------|-------------|
| `unstructured.api_url` | `UAR_UNSTRUCTURED__API_URL` | — | Unstructured API endpoint |
| `unstructured.api_key` | `UNSTRUCTURED_API_KEY` | — | API key |

---

## mistral_ocr

| Key | Env Var | Default | Description |
|-----|---------|---------|-------------|
| `mistral_ocr.api_key` | `MISTRAL_API_KEY` | — | Mistral API key for OCR |
| `mistral_ocr.ocr_model` | — | `"mistral-ocr-latest"` | OCR model ID |

---

## kreuzberg

| Key | Default | Description |
|-----|---------|-------------|
| `kreuzberg.ocr_enabled` | `false` | Enable OCR via Kreuzberg |
| `kreuzberg.ocr_backend` | `"tesseract"` | `"tesseract"` \| `"easyocr"` \| `"paddleocr"` |
| `kreuzberg.ocr_languages` | `["en"]` | OCR language codes |
| `kreuzberg.dpi` | `300` | Scan DPI for PDFs |
| `kreuzberg.extract_tables` | `false` | Extract tables from documents |
| `kreuzberg.extract_metadata` | `true` | Extract document metadata |

---

## vision

| Key | Env Var | Default | Description |
|-----|---------|---------|-------------|
| `vision.model` | `UAR_VISION__MODEL` | — | Vision model ID (`provider/model`) |
| `vision.auto_detect` | `UAR_VISION__AUTO_DETECT` | `true` | Auto-detect images in messages |

---

## models

| Key | Env Var | Default | Description |
|-----|---------|---------|-------------|
| `models.models_dir` | `UAR_MODELS_DIR` | `"src/uar/runtime/matching/models"` | Directory for intent classification models |
| `models.vector_threshold` | — | `0.75` | Cosine similarity threshold for intent matching |

---

## knowledge_bases

Configure knowledge base backends for RAG (Retrieval-Augmented Generation).

```yaml
knowledge_bases:
  default: "my_kb"
  items:
    - id: "my_kb"
      provider: "pgvector"          # pgvector | surreal | qdrant
      connection_string: "${DATABASE_URL}"
      embedding_provider: "openai"  # openai | cohere | local
      embedding_model: "text-embedding-3-small"
      chunk_size: 1000
      chunk_overlap: 200
```

---

## intent_classifier

| Key | Default | Description |
|-----|---------|-------------|
| `intent_classifier.backend` | `"vector"` | `"vector"` \| `"llm"` \| `"hybrid"` |
| `intent_classifier.topk` | `3` | Top-K candidates |
| `intent_classifier.accept_threshold` | `0.85` | Minimum confidence to accept match |
| `intent_classifier.margin_threshold` | `0.10` | Min margin between top-2 matches |

---

## llm

| Key | Env Var | CLI Flag | Default | Type | Description |
|-----|---------|----------|---------|------|-------------|
| `llm.model` | `UAR_LLM__MODEL` \| `LLM_MODEL` | `--llm-model` | `"openai/gpt-4o"` | string | **Required.** `provider/model` format. See `models.dev` |
| `llm.api_key` | `UAR_LLM__API_KEY` \| `LLM_API_KEY` | `--llm-api-key` | — | string | Default provider API key |
| `llm.base_url` | `UAR_LLM__BASE_URL` \| `LLM_BASE_URL` | `--llm-base-url` | — | string | Override provider endpoint. Use for candle-vllm/Ollama/proxies |
| `llm.protocol` | `UAR_LLM__PROTOCOL` \| `LLM_PROTOCOL` | `--llm-protocol` | `"auto"` | string | `"auto"` \| `"chat"` \| `"responses"`. Use `"chat"` for candle-vllm |
| `llm.timeout_secs` | `UAR_LLM__TIMEOUT_SECS` | — | `60` | u64 | Request timeout (s). Increase for slow local models |
| `llm.max_retries` | `UAR_LLM__MAX_RETRIES` | — | `3` | u32 | Max retries on 429/5xx |
| `llm.cost_tracking` | `UAR_LLM__COST_TRACKING` | — | `false` | bool | Track per-request costs |
| `llm.tracing` | `UAR_LLM__TRACING` | — | `true` | bool | OpenTelemetry tracing spans |
| `llm.parallel_tool_calls` | — | — | — | bool | Allow parallel tool calls (provider-dependent) |
| `llm.cooldown_secs` | — | — | — | u64 | Cooldown after transient errors (s) |
| `llm.health_check_secs` | — | — | — | u64 | Health check interval (s) |

### llm.cache

| Key | Default | Description |
|-----|---------|-------------|
| `llm.cache.enabled` | `false` | Enable Tower middleware response cache |
| `llm.cache.ttl_secs` | `300` | Cache entry TTL (s) |
| `llm.cache.max_size` | `1000` | Max cached responses |

### llm.budget

| Key | Default | Description |
|-----|---------|-------------|
| `llm.budget.global_limit` | — | Monthly spend limit in USD |
| `llm.budget.per_request_limit` | — | Per-request spend limit in USD |
| `llm.budget.alert_threshold` | `0.8` | Alert at this fraction of limit |

### llm.rate_limit

| Key | Default | Description |
|-----|---------|-------------|
| `llm.rate_limit.requests_per_minute` | — | Per-provider RPM cap |
| `llm.rate_limit.tokens_per_minute` | — | Per-provider TPM cap |

---

## providers

Multi-provider array. Each entry overrides the default `llm` config for specific agents.

```yaml
providers:
  - id: "openai"                   # Identifier used in provider/model format
    display_name: "OpenAI"
    base_url: ""                    # Empty = use provider default
    api_key: "${OPENAI_API_KEY}"
    protocol: auto                  # auto | chat | responses
    default_model: "gpt-4o"
    enabled: true
    models:
      - id: "gpt-4o"
        context_window: 128000
        supports_vision: true
        supports_tools: true

  - id: "candle-vllm"              # Local inference
    display_name: "Local candle-vllm"
    base_url: "http://localhost:3000"
    api_key: ""                     # No auth by default
    protocol: chat                  # MUST be chat
    default_model: "llama-3-8b"
    enabled: true
```

---

## memory

Requires SurrealDB (embedded or server mode via `surreal-memory` crate).

| Key | Env Var | Default | Description |
|-----|---------|---------|-------------|
| `memory.enabled` | `UAR_MEMORY__ENABLED` | `false` | Enable SurrealDB-backed memory |
| `memory.db_path` | `UAR_MEMORY__DB_PATH` | `"./data/memory.db"` | Embedded SurrealDB path |
| `memory.surreal_endpoint` | `UAR_MEMORY__SURREAL_ENDPOINT` | — | Server-mode SurrealDB WebSocket URL |
| `memory.embedding_provider` | `UAR_MEMORY__EMBEDDING_PROVIDER` | `"openai"` | `"openai"` \| `"cohere"` \| `"local"` |
| `memory.embedding_model` | `UAR_MEMORY__EMBEDDING_MODEL` | `"text-embedding-3-small"` | Embedding model |
| `memory.auto_capture` | `UAR_MEMORY__AUTO_CAPTURE` | `true` | Auto-extract memories after each turn |
| `memory.inject_context` | `UAR_MEMORY__INJECT_CONTEXT` | `true` | Inject relevant memories into system prompt |
| `memory.max_context_tokens` | `UAR_MEMORY__MAX_CONTEXT_TOKENS` | `2000` | Token budget for injected context |
| `memory.vector_weight` | — | `0.7` | Hybrid search: vector score weight |
| `memory.bm25_weight` | — | `0.3` | Hybrid search: BM25 score weight |
| `memory.mcp_http_enabled` | — | `true` | Expose memory via MCP HTTP |
| `memory.mcp_http_path` | — | `"/mcp/memory"` | MCP HTTP mount path |
| `memory.namespace` | — | `"uar"` | SurrealDB namespace |
| `memory.database` | — | `"memory"` | SurrealDB database name |
