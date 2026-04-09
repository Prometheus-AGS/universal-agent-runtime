---
name: uar-config-wizard
description: >
  Configuration assistant for Universal Agent Runtime (UAR). Covers all modes:
  interactive wizard (guided Q&A → tailored config.yaml + .env), advisor
  (explain options, generate annotated templates), validator (check existing
  config.yaml / .env for errors), migration assistant (upgrade LLM_* / legacy
  env vars to UAR_* prefix), Kubernetes config generation, model selection
  advisor (hardware profiling + live models.dev search via Tavily + candle-vllm
  catalog + turboquant KV-cache compression sizing), and full-stack bundle
  generator (UAR config.yaml + candle-vllm models.yaml + .env + quickstart.sh
  + optional K8s manifests). Generates immediately-runnable configs for every
  supported deployment scenario including local candle-vllm inference.
allowed-tools: file_system code_interpreter web_search tavily
---

# UAR Configuration Wizard

A PMPO-driven configuration assistant for the Universal Agent Runtime. Handles the full configuration lifecycle — from first-run guided setup through Docker/Kubernetes production deployment.

## Modes

### `/uar-wizard` — Interactive guided setup
Asks targeted questions about your deployment scenario, LLM provider, database backend, security requirements, and optional features. Produces a tailored `config.yaml` and `.env` in one pass.

### `/uar-config` — Main entry point (auto-routes)
Analyzes intent and routes to the appropriate mode automatically. Use this if you're unsure which command to run.

### `/uar-validate` — Validate existing files
Checks an existing `config.yaml` and/or `.env` for syntax errors, missing required fields, type mismatches, conflicting settings, and deprecated keys.

### `/uar-migrate` — Legacy env var migration
Converts `LLM_*`, `LLM_API_KEY`, `LLM_BASE_URL`, and other legacy environment variables to the `UAR_LLM__*` and `UAR_*` prefix convention used by UAR 2.x+.

### `/uar-k8s-config` — Kubernetes config generation
Generates a K8s `Secret` (sensitive keys: JWT secret, API keys, DB passwords) and `ConfigMap` (non-sensitive settings) from an existing config or wizard output.

### `/uar-model-select` — AI model selection advisor
Five-phase guided model selection using live web research:
1. **Hardware profile** — GPU type/count/VRAM, RAM, platform (CUDA/Metal/CPU)
2. **Requirements** — context window, vision, tool-calling, reasoning, latency vs quality tradeoff
3. **Live catalog search** — Tavily queries to `models.dev`, HuggingFace, liter-llm docs
4. **Scoring + ranking** — VRAM fit × capability × speed × cost weighted rubric
5. **TurboQuant configuration** — `kvcache_compression.bits` (2/3/4) + policy based on VRAM headroom

Produces: recommended model list with rationale + candle-vllm `models.yaml` entry + UAR provider config snippet.

### `/uar-stack` — Full stack bundle (one command)
Runs wizard + model-select in sequence and emits a complete, immediately-runnable bundle:
- `config.yaml` — UAR with candle-vllm registered as a `providers[]` entry
- `candle-vllm-models.yaml` — model definition with turboquant `kvcache_compression`
- `.env` — all secrets and keys
- `quickstart.sh` — copy-pastable CLI to launch both services
- `k8s/` manifests (optional)

## Configuration Sections Covered

| Section | Key options |
|---------|------------|
| `server` | port, host |
| `security` | jwt_required, jwt_secret |
| `resilience` | rate limiting, timeouts, retries with backoff |
| `persistence` | provider (postgres/surreal), database_url, redis |
| `file_processing` | provider (auto/unstructured/mistral/kreuzberg), upload limits |
| `unstructured` | api_url, api_key |
| `mistral_ocr` | api_key, ocr_model |
| `kreuzberg` | ocr_enabled, ocr_backend, languages, DPI, table/metadata extraction |
| `vision` | model, auto_detect |
| `models` | models_dir, vector_threshold |
| `knowledge_bases` | default KB, named KBs, embedding providers, chunking strategies |
| `intent_classifier` | backend, topk, accept_threshold, margin_threshold |
| `llm` | model (provider/model format), api_key, base_url, budget, cache, rate_limit |
| `providers` | multi-provider config for per-agent selection |
| `memory` | SurrealDB-backed memory, embedding config, auto_capture, context injection |

## LLM Provider Support

142+ providers via liter-llm unified `provider/model` addressing:

- **Cloud**: OpenAI, Anthropic, Google Gemini, Groq, Mistral, Cohere, Together, Perplexity, OpenRouter
- **Local**: Ollama, LM Studio, custom OpenAI-compatible proxies
- **Enterprise**: Azure OpenAI, AWS Bedrock, Vertex AI

## Execution Model (PMPO Loop)

### Startup
1. **Resolve provider** — Determine state backend (`scripts/state-resolve-provider.sh`)
2. **Init/resume state** — Load or create named config session (`scripts/state-init.sh`)
3. **Mode routing** — Route to wizard, advise, validate, migrate, or generate phase

### Phase Loop
1. **Clarify** (`prompts/wizard.md` or `prompts/advise.md`) — Gather requirements
2. **Generate** (`prompts/generate.md`) — Produce config artifacts
3. **Validate** (`prompts/validate.md`) — Check output for correctness
4. **Persist** — Write validated state and output files

### Phase Hooks
After each phase: checkpoint (`scripts/state-checkpoint.sh`) + dispatch (`scripts/workflow-dispatch.sh`)

## State Management

State is persisted to `.config-wizard/` in the project root:

```
.config-wizard/
  registry.json              # Maps session_name → state path
  sessions/
    {session_name}/
      state.json             # Current session state
      checkpoints/           # Mid-session snapshots
      output/                # Generated config files
      history/               # Previous iterations
```

## Deployment Scenarios

The wizard supports five canonical scenarios:

| Scenario | What gets generated |
|----------|-------------------|
| `local-dev` | Minimal config.yaml + .env, JWT disabled, Postgres local |
| `docker-compose` | Full .env matching docker-compose.prod.yml vars |
| `kubernetes` | K8s Secret + ConfigMap YAML manifests |
| `ollama-local` | config.yaml with base_url pointing to Ollama |
| `multi-provider` | providers[] array + per-agent routing policy examples |

## Quick Start

- `/uar-config` — Auto-detect what you need and get started
- `/uar-wizard` — Guided first-time setup
- `/uar-validate` — Check an existing config for errors
- `/uar-migrate` — Upgrade legacy env vars to UAR_* format
- `/uar-k8s-config` — Generate Kubernetes manifests
