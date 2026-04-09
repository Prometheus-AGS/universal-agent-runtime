# UAR CLI Arguments Reference

All command-line flags accepted by the UAR binary. Defined in the `Cli` struct in `src/config.rs`.

**Precedence**: CLI args override everything — config file AND environment variables.

**Usage**:
```bash
uar --port 8080 --llm-model anthropic/claude-opus-4 --jwt-required false
```

---

## All Flags

| Flag | Env Var | Type | Default | Description |
|------|---------|------|---------|-------------|
| `--config <PATH>` | `CONFIG_FILE` | string | — | Path to `config.yaml`. Overrides default search paths |
| `--port <PORT>` | `PORT` | u16 | `3000` | HTTP port to listen on |
| `--jwt-required <BOOL>` | `JWT_REQUIRED` | bool | `true` | Require JWT on all API requests |
| `--rate-limit-enabled <BOOL>` | `RATE_LIMIT_ENABLED` | bool | `true` | Enable token-bucket rate limiter |
| `--timeout-disabled <BOOL>` | `TIMEOUT_DISABLED` | bool | `false` | Disable request timeout middleware |
| `--external-cache-enabled <BOOL>` | `EXTERNAL_CACHE_ENABLED` | bool | `false` | Enable Redis response cache |
| `--llm-model <MODEL>` | `LLM_MODEL` | string | `"openai/gpt-4o"` | Default LLM in `provider/model` format |
| `--llm-api-key <KEY>` | `LLM_API_KEY` | string | — | API key for default provider |
| `--llm-base-url <URL>` | `LLM_BASE_URL` | string | — | Override provider endpoint URL |
| `--llm-protocol <PROTO>` | `LLM_PROTOCOL` | string | `"auto"` | Protocol: `auto` \| `chat` \| `responses` |
| `--llm-budget-limit <USD>` | `UAR_LLM__BUDGET__GLOBAL_LIMIT` | f64 | — | Monthly spend cap in USD |

---

## Common Launch Patterns

### Local development (no auth)
```bash
uar \
  --port 3000 \
  --jwt-required false \
  --rate-limit-enabled false \
  --llm-model openai/gpt-4o
```

### Production with config file
```bash
uar --config /etc/uar/config.yaml
```

### With candle-vllm local inference
```bash
uar \
  --llm-model candle-vllm/llama-3-8b-instruct \
  --llm-base-url http://localhost:3000 \
  --llm-protocol chat \
  --timeout-disabled true
```

### Docker / container
```bash
docker run -e UAR_PERSISTENCE__DATABASE_URL=postgres://... \
           -e UAR_SECURITY__JWT_SECRET=... \
           -e UAR_LLM__MODEL=openai/gpt-4o \
           -e UAR_LLM__API_KEY=sk-... \
           -p 3000:3000 \
           prometheus-ags/uar:latest
```

### Cloud Anthropic with budget limit
```bash
uar \
  --llm-model anthropic/claude-sonnet-4 \
  --llm-budget-limit 100.0
```

---

## Config File Search Path

When `--config` is not provided, UAR searches in order:
1. `./config.yaml` (current working directory)
2. `~/.uar/config.yaml` (home directory)
3. Compiled defaults only (no file)
