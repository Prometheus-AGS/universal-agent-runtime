# UAR Configuration Wizard

Configuration assistant for the Universal Agent Runtime. Generates `config.yaml`, `.env`, `candle-vllm-models.yaml`, CLI examples, and Kubernetes manifests for every deployment scenario.

## Quick Start

| Command | What it does |
|---------|-------------|
| `/uar-config` | Auto-detect intent and route to the right mode |
| `/uar-wizard` | Guided first-time setup Q&A |
| `/uar-validate` | Check existing config.yaml / .env for errors |
| `/uar-migrate` | Upgrade LLM_* env vars to UAR_* format |
| `/uar-k8s-config` | Generate Kubernetes Secret + ConfigMap |
| `/uar-model-select` | AI model advisor: hardware profile → live search → turboquant config |
| `/uar-stack` | Full stack bundle: UAR + candle-vllm + .env + quickstart.sh |

## Model Selection (`/uar-model-select`)

Guides you through:
1. **Hardware profile** — GPU VRAM, RAM, platform (CUDA/Metal/CPU)
2. **Requirements** — context window, vision, tool-calling, reasoning
3. **Live model research** — searches `models.dev`, HuggingFace, and liter-llm docs via Tavily
4. **VRAM-fit scoring** — ranks models by suitability for your hardware
5. **TurboQuant settings** — recommends `kvcache_compression.bits` (2/3/4) and policy

Produces: `candle-vllm-models.yaml` entry + UAR `providers[]` snippet.

## Full Stack Bundle (`/uar-stack`)

One command to configure the entire UAR + candle-vllm stack:

```
Output:
  config.yaml               UAR configuration
  candle-vllm-models.yaml   Local model definitions with turboquant
  .env                      All secrets and keys
  quickstart.sh             Launch commands for both services
  k8s/                      Kubernetes manifests (optional)
```

## candle-vllm Integration

UAR connects to candle-vllm via the OpenAI-compatible API. Configure in `config.yaml`:

```yaml
providers:
  - id: "candle-vllm"
    display_name: "Local candle-vllm"
    base_url: "http://your-host:3000"
    protocol: chat
    default_model: "llama-3-8b-instruct"
    enabled: true

llm:
  model: "candle-vllm/llama-3-8b-instruct"
  base_url: "http://your-host:3000"
  protocol: chat
```

## TurboQuant KV-Cache Compression

Applied per-model in `candle-vllm-models.yaml`:

```yaml
kvcache_compression:
  bits: 3          # 2=highest compression, 4=best quality
  policy:
    threshold_tokens: 4096   # only compress once context exceeds this
```

| VRAM Headroom | bits | policy |
|--------------|------|--------|
| >30% free | disabled | — |
| 15–30% free | 4 | `memory_pressure: {free_block_pct: 0.20}` |
| 5–15% free | 3 | `threshold_tokens: 4096` |
| <5% free | 3 | `always` |

## Supported Config Sections

`server` · `security` · `resilience` · `persistence` · `file_processing` · `unstructured` · `mistral_ocr` · `kreuzberg` · `vision` · `models` · `knowledge_bases` · `intent_classifier` · `llm` · `providers` · `memory`

## State Management

Sessions are tracked in `.config-wizard/sessions/{name}/`:
- `state.json` — current session
- `checkpoints/` — mid-session snapshots
- `output/` — generated files
- `history/` — previous iterations
