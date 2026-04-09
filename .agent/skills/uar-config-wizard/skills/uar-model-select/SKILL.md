---
name: uar-model-select
version: 1.0.0
description: >
  AI model selection advisor for UAR + candle-vllm deployments. Five-phase
  guided flow: hardware profiling (GPU/VRAM/RAM/platform), use-case
  requirements (context, tools, vision, reasoning), live Tavily web research
  on models.dev + HuggingFace + liter-llm docs, VRAM-fit scoring + ranking,
  and TurboQuant KV-cache compression configuration. Produces a recommended
  model list with rationale, a candle-vllm models.yaml entry, and UAR
  provider config snippet.
triggers:
  keywords:
    - "what model should i use"
    - "choose a model"
    - "model selection"
    - "best model for my hardware"
    - "candle-vllm model"
    - "recommend a model"
    - "model advisor"
    - "/uar-model-select"
  when_to_use: >
    Use when the user is unsure which LLM to run, particularly for local
    candle-vllm inference. Combines live web research with hardware profiling
    and TurboQuant optimization.
---

# UAR Model Selection Advisor

I will help you choose the best AI model for your hardware and use case, then configure it optimally with TurboQuant KV-cache compression.

## Five phases

### Phase 1 — Hardware Profile
I ask about your GPU (type, VRAM), RAM, and platform (NVIDIA CUDA, Apple Silicon, CPU-only). For Apple Silicon, I account for unified memory shared between CPU, GPU, and OS.

### Phase 2 — Requirements
What do you need? Context window size, tool/function calling (required for UAR agents), vision, reasoning capability, speed vs quality priority.

### Phase 3 — Live Research
I search `models.dev`, HuggingFace, and liter-llm provider docs via Tavily to get current model information. If web search is unavailable, I use the static catalog in `references/candle-vllm-catalog.md`.

### Phase 4 — Scoring + Ranking
Each candidate model is scored:
```
score = VRAM-fit(35%) + capability-match(30%) + speed(20%) + cost(15%)
```
I present the top 3 recommendations with full reasoning.

### Phase 5 — TurboQuant Configuration
Based on your VRAM headroom, I recommend KV-cache compression settings:
- **>30% free**: disabled (no compression needed)
- **15–30%**: 4-bit + memory pressure policy
- **5–15%**: 3-bit + threshold_tokens: 4096 (recommended sweet spot)
- **<5%**: 3-bit + always (maximum compression, ~7x memory reduction)

## Output

- Top 3 recommended models with rationale
- `candle-vllm-models.yaml` entry for the selected model
- UAR `providers[]` config snippet
- TurboQuant `kvcache_compression` block with explanation

## Entry point

On invocation, load `prompts/model-select.md`.
Invoke subagent: `agents/model-advisor.md`
Required tools: `web_search` (Tavily), `code_interpreter` (VRAM math), `file_system` (state)
