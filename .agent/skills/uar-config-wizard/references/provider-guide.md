# LLM Provider Setup Guide

UAR uses liter-llm for unified `provider/model` addressing across 142+ providers. This guide covers the most common providers.

**Model format**: always `provider/model` — e.g., `openai/gpt-4o`, `anthropic/claude-sonnet-4`, `candle-vllm/llama-3-8b`.

---

## Cloud Providers

### OpenAI

```yaml
llm:
  model: "openai/gpt-4o"
  api_key: "${OPENAI_API_KEY}"    # or set OPENAI_API_KEY env var
  protocol: auto
```

Popular models: `gpt-4o`, `gpt-4o-mini`, `o1`, `o3`, `o4-mini`

### Anthropic

```yaml
llm:
  model: "anthropic/claude-sonnet-4"
  api_key: "${ANTHROPIC_API_KEY}"
  protocol: responses    # Anthropic supports both; responses preferred
```

Popular models: `claude-opus-4`, `claude-sonnet-4`, `claude-haiku-4`

### Google Gemini

```yaml
llm:
  model: "google/gemini-2.5-pro"
  api_key: "${GEMINI_API_KEY}"
  protocol: auto
```

Popular models: `gemini-2.5-pro`, `gemini-2.5-flash`, `gemini-2.0-flash`

### Groq (fast inference)

```yaml
llm:
  model: "groq/llama-3.3-70b-versatile"
  api_key: "${GROQ_API_KEY}"
  protocol: chat
```

Popular models: `llama-3.3-70b-versatile`, `llama-3.1-8b-instant`, `mixtral-8x7b-32768`, `qwen-qwq-32b`

### Mistral AI

```yaml
llm:
  model: "mistral/mistral-large-latest"
  api_key: "${MISTRAL_API_KEY}"
  protocol: chat
```

Popular models: `mistral-large-latest`, `mistral-small-latest`, `codestral-latest`

### Cohere

```yaml
llm:
  model: "cohere/command-r-plus"
  api_key: "${COHERE_API_KEY}"
  protocol: auto
```

### Together AI

```yaml
llm:
  model: "together/meta-llama/Llama-3.3-70B-Instruct-Turbo"
  api_key: "${TOGETHER_API_KEY}"
  protocol: chat
```

### OpenRouter (multi-provider routing)

```yaml
llm:
  model: "openrouter/anthropic/claude-sonnet-4"
  api_key: "${OPENROUTER_API_KEY}"
  base_url: "https://openrouter.ai/api/v1"
  protocol: chat
```

---

## Local / Self-Hosted Providers

### candle-vllm (Prometheus fork)

OpenAI-compatible local inference engine. Supports Llama, Mistral, Phi, Qwen, Gemma, DeepSeek, and more.

```yaml
providers:
  - id: "candle-vllm"
    display_name: "Local candle-vllm"
    base_url: "http://localhost:3000"    # Your candle-vllm instance URL
    api_key: ""                           # No auth by default
    protocol: chat                        # MUST be chat
    default_model: "your-model-alias"
    enabled: true

llm:
  model: "candle-vllm/your-model-alias"
  base_url: "http://localhost:3000"
  protocol: chat
  timeout_secs: 120    # Increase for large models
```

**Required**: candle-vllm must be running with your model loaded. See `references/candle-vllm-catalog.md`.

### Ollama

```yaml
llm:
  model: "ollama/llama3.2"
  base_url: "http://localhost:11434"
  protocol: chat
  api_key: ""
```

Verify Ollama is running: `curl http://localhost:11434/api/tags`

### LM Studio

```yaml
llm:
  model: "lm-studio/local-model"
  base_url: "http://localhost:1234/v1"
  protocol: chat
  api_key: "lm-studio"    # LM Studio accepts any non-empty string
```

### Custom OpenAI-Compatible Endpoint

Any server implementing the OpenAI chat completions API:

```yaml
llm:
  model: "custom/my-model"
  base_url: "https://my-proxy.example.com/v1"
  api_key: "${MY_PROXY_API_KEY}"
  protocol: chat
```

---

## Enterprise Providers

### Azure OpenAI

```yaml
llm:
  model: "azure/gpt-4o"
  api_key: "${AZURE_OPENAI_API_KEY}"
  base_url: "https://my-resource.openai.azure.com"
  protocol: chat
```

Also set: `AZURE_API_VERSION=2024-10-21`

### AWS Bedrock

```yaml
llm:
  model: "bedrock/anthropic.claude-3-5-sonnet-20241022-v2:0"
  protocol: chat
```

Uses AWS credentials from environment: `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, `AWS_REGION`

### Vertex AI

```yaml
llm:
  model: "vertex_ai/gemini-2.5-pro"
  protocol: auto
```

Uses GCP application default credentials.

---

## Provider API Key Environment Variables

| Provider | Variable |
|----------|---------|
| OpenAI | `OPENAI_API_KEY` |
| Anthropic | `ANTHROPIC_API_KEY` |
| Google / Gemini | `GEMINI_API_KEY` |
| Groq | `GROQ_API_KEY` |
| Mistral | `MISTRAL_API_KEY` |
| Cohere | `COHERE_API_KEY` |
| Together | `TOGETHER_API_KEY` |
| Perplexity | `PERPLEXITY_API_KEY` |
| OpenRouter | `OPENROUTER_API_KEY` |
| Azure OpenAI | `AZURE_OPENAI_API_KEY` |

Provider shortcuts map to `llm.api_key` if `UAR_LLM__API_KEY` is not set.

---

## Choosing Between Providers for UAR

| Use case | Recommended provider |
|----------|---------------------|
| Best overall quality | `anthropic/claude-opus-4` or `openai/gpt-4o` |
| Fastest response time | `groq/llama-3.3-70b-versatile` |
| Free tier / experimentation | `google/gemini-2.0-flash` |
| Privacy / no cloud | `candle-vllm/*` or `ollama/*` |
| Tool calling required | All major providers support it; candle-vllm supports it for Llama/Mistral/Qwen/Phi |
| Long context (128K+) | `anthropic/claude-*`, `google/gemini-*`, `openai/gpt-4o`, or `qwen/qwen3-*` via candle-vllm |
| Reasoning / math | `anthropic/claude-opus-4`, `openai/o3`, `deepseek/deepseek-r1`, `candle-vllm/qwq-32b` |
