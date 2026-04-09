# UAR Deployment Patterns

Five canonical deployment scenarios with recommended config for each.

---

## 1. Local Development

**Use case**: Single developer, no production traffic, fastest iteration.

**Characteristics**:
- JWT disabled for simplicity
- Rate limiting disabled
- Local PostgreSQL (Docker)
- Cloud LLM via API key in `.env`

```yaml
# config.yaml
server:
  port: 3000
  host: "127.0.0.1"
security:
  jwt_required: false
resilience:
  rate_limit_enabled: false
  retries_enabled: true
persistence:
  provider: postgres
  database_url: "postgres://uar:uar@localhost:5432/uar_dev"
llm:
  model: "openai/gpt-4o"
  timeout_secs: 60
```

```bash
# .env
OPENAI_API_KEY=sk-...
UAR_PERSISTENCE__DATABASE_URL=postgres://uar:uar@localhost:5432/uar_dev
UAR_SECURITY__JWT_REQUIRED=false
```

---

## 2. Docker Compose

**Use case**: Self-hosted server with docker-compose, staging or small production.

```yaml
# config.yaml
server:
  port: 3000
  host: "0.0.0.0"
security:
  jwt_required: true
  jwt_secret: "<GENERATE: openssl rand -base64 64>"
resilience:
  rate_limit_enabled: true
  requests_per_second: 20.0
  burst_size: 50.0
persistence:
  provider: postgres
  database_url: "postgres://uar:${POSTGRES_PASSWORD}@postgres:5432/uar"
  external_cache_enabled: true
  redis_url: "redis://redis:6379"
llm:
  model: "openai/gpt-4o"
  timeout_secs: 60
```

```bash
# .env (docker-compose picks this up automatically)
POSTGRES_PASSWORD=<REPLACE: your secure postgres password>
UAR_SECURITY__JWT_SECRET=<GENERATE: openssl rand -base64 64>
OPENAI_API_KEY=sk-...
```

---

## 3. Kubernetes

**Use case**: Cluster deployment, multi-replica, production scale.

Split sensitive config into Secret, non-sensitive into ConfigMap.

**K8s Secret** (base64-encoded values):
```yaml
apiVersion: v1
kind: Secret
metadata:
  name: uar-secrets
type: Opaque
stringData:
  jwt_secret: "<GENERATE: openssl rand -base64 64>"
  database_url: "postgres://uar:password@postgres-service:5432/uar"
  llm_api_key: "sk-..."
```

**K8s ConfigMap**:
```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: uar-config
data:
  UAR_SERVER__PORT: "3000"
  UAR_SECURITY__JWT_REQUIRED: "true"
  UAR_RESILIENCE__RATE_LIMIT_ENABLED: "true"
  UAR_RESILIENCE__REQUESTS_PER_SECOND: "50.0"
  UAR_LLM__MODEL: "openai/gpt-4o"
  UAR_LLM__PROTOCOL: "auto"
```

**Deployment env references**:
```yaml
env:
  - name: UAR_SECURITY__JWT_SECRET
    valueFrom:
      secretKeyRef:
        name: uar-secrets
        key: jwt_secret
  - name: UAR_PERSISTENCE__DATABASE_URL
    valueFrom:
      secretKeyRef:
        name: uar-secrets
        key: database_url
  - name: UAR_LLM__API_KEY
    valueFrom:
      secretKeyRef:
        name: uar-secrets
        key: llm_api_key
```

---

## 4. candle-vllm Stack

**Use case**: UAR + local LLM inference via candle-vllm. No cloud API required.

**Architecture**:
```
User → UAR (port 3001) → candle-vllm (port 3000) → loaded model
```

```yaml
# config.yaml
server:
  port: 3001
security:
  jwt_required: true
providers:
  - id: "candle-vllm"
    display_name: "Local candle-vllm"
    base_url: "http://localhost:3000"    # candle-vllm URL
    protocol: chat
    default_model: "llama-3-8b-instruct"
    api_key: ""
    enabled: true
llm:
  model: "candle-vllm/llama-3-8b-instruct"
  base_url: "http://localhost:3000"
  protocol: chat
  timeout_secs: 120    # Local inference can be slower
  max_retries: 1       # Reduce retries for local
```

**candle-vllm models.yaml** (separate file for candle-vllm):
```yaml
default_model: llama-3-8b-instruct
models:
  - name: llama-3-8b-instruct
    hf_id: meta-llama/Llama-3.1-8B-Instruct
    params:
      dtype: bf16
      mem: 14336         # KV cache memory: ~14GB
      max_num_seqs: 32
      block_size: 16
      device_ids: [0]
      temperature: 0.7
      top_p: 0.9
    kvcache_compression:
      bits: 3
      policy:
        threshold_tokens: 4096
```

---

## 5. Multi-Provider (Hybrid Cloud + Local)

**Use case**: Multiple LLM providers with per-agent routing. Primary = candle-vllm, fallback = OpenAI.

```yaml
# config.yaml
providers:
  - id: "candle-vllm"
    display_name: "Local candle-vllm"
    base_url: "http://localhost:3000"
    protocol: chat
    default_model: "llama-3-8b-instruct"
    enabled: true
  - id: "openai"
    display_name: "OpenAI (fallback)"
    base_url: ""
    api_key: "${OPENAI_API_KEY}"
    protocol: auto
    default_model: "gpt-4o"
    enabled: true
  - id: "anthropic"
    display_name: "Anthropic (premium)"
    api_key: "${ANTHROPIC_API_KEY}"
    protocol: responses
    default_model: "claude-sonnet-4"
    enabled: true
llm:
  model: "candle-vllm/llama-3-8b-instruct"   # Default: local
  base_url: "http://localhost:3000"
  protocol: chat
```

Per-agent provider routing is configured in the skill/agent definition using `ProviderPolicy`:
```yaml
# In agent skill definition
provider_policy:
  default:
    provider: candle-vllm
    model: llama-3-8b-instruct
  fallbacks:
    - provider: openai
      model: gpt-4o
    - provider: anthropic
      model: claude-sonnet-4
```
