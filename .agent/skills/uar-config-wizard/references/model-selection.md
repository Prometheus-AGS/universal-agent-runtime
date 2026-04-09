# Model Selection Guide

Scoring rubric, hardware profiles, and decision trees for the `/uar-model-select` advisor.

---

## Scoring Rubric

```
composite_score = (vram_fit × 0.35) + (capability_match × 0.30) + (speed_tps × 0.20) + (cost_score × 0.15)
```

### VRAM Fit Score (0.0–1.0)

```
full_precision_vram_gb = params_b × 2          # BF16 / FP16
q4k_vram_gb            = params_b × 0.5        # GGUF Q4_K_M
q2k_vram_gb            = params_b × 0.25       # GGUF Q2_K

fits_bf16  → score = 1.00   (full_precision_vram ≤ available × 0.80)
fits_q4k   → score = 0.75   (q4k_vram ≤ available × 0.85)
fits_q2k   → score = 0.35   (q2k_vram ≤ available × 0.85, warn quality)
cpu_offload→ score = 0.10   (won't fit GPU, but fits RAM — very slow)
won't_fit  → score = 0.00   (exclude from recommendations)
```

### Capability Match Score (0.0–1.0)

```
required_capabilities = user-selected set from: text, tools, vision, reasoning, code

score = (capabilities_model_supports ∩ required_capabilities) / |required_capabilities|

Example: user needs [tools, vision, text], model supports [tools, text] → 2/3 = 0.67
```

### Speed Score (0.0–1.0)

Normalized from candle-vllm benchmark tokens/second (A100 BF16 reference):

| tokens/sec | Speed score |
|-----------|-------------|
| >150 | 1.00 |
| 100–150 | 0.85 |
| 60–100 | 0.70 |
| 30–60 | 0.50 |
| <30 | 0.25 |
| local/unknown | 0.50 (neutral) |

### Cost Score (0.0–1.0, inverted — free = 1.0)

| Cost tier | Score |
|-----------|-------|
| Free / local | 1.00 |
| <$1/M tokens | 0.85 |
| $1–5/M tokens | 0.65 |
| $5–15/M tokens | 0.40 |
| >$15/M tokens | 0.15 |

---

## Hardware Profiles

### Apple Silicon Unified Memory

Apple Silicon shares VRAM with system RAM. Reserve headroom for OS + runtime:

| Chip | Total Memory | Available for Model+KV |
|------|-------------|----------------------|
| M1 8GB | 8GB | ~5.5GB |
| M1/M2 16GB | 16GB | ~12GB |
| M2 Pro 32GB | 32GB | ~26GB |
| M3 Max 64GB | 64GB | ~55GB |
| M3 Max 128GB | 128GB | ~112GB |
| M4 Pro 48GB | 48GB | ~42GB |

**Note**: For Apple Silicon, use `dtype: f16` or GGUF quantized models. BF16 works on M3/M4+.

### NVIDIA CUDA

| GPU | VRAM | Available (model) |
|-----|------|-------------------|
| RTX 3060 12GB | 12GB | ~10GB |
| RTX 3090/4090 24GB | 24GB | ~20GB |
| RTX 4080 16GB | 16GB | ~14GB |
| A10G 24GB | 24GB | ~20GB |
| A100 40GB | 40GB | ~34GB |
| A100 80GB | 80GB | ~68GB |
| H100 80GB | 80GB | ~68GB |
| H100 NVL 94GB | 94GB | ~80GB |

### CPU-Only

| RAM | Available | Max recommended model |
|-----|-----------|----------------------|
| 16GB | ~10GB | 7B Q4_K_M (very slow) |
| 32GB | ~22GB | 13B Q4_K_M (slow) |
| 64GB | ~50GB | 34B Q4_K_M (usable) |

CPU inference is 10–50x slower than GPU. Only recommend for experimentation.

---

## Decision Tree

```
User wants local inference?
├── Yes
│   ├── Hardware: Apple Silicon
│   │   ├── 8GB  → Models ≤3B (Phi-3.5 mini, Qwen2.5-1.5B, SmolLM2-1.7B)
│   │   ├── 16GB → Models ≤7B (Mistral-7B, Llama-3.2-3B Q4, Qwen2.5-7B Q4)
│   │   ├── 32GB → Models ≤13B (Llama-3.1-8B BF16, Mistral-7B BF16)
│   │   ├── 48GB → Models ≤27B (Llama-3-70B Q2, Qwen3-14B BF16)
│   │   └── 64GB+→ Models ≤70B (Llama-3-70B Q4, Qwen3-32B)
│   │
│   ├── Hardware: NVIDIA GPU
│   │   ├── 8–12GB  → 7B Q4_K_M, Phi-4-mini, Qwen2.5-7B GGUF
│   │   ├── 16–24GB → 7B BF16, 13B Q4, Phi-4 full, Mistral-7B BF16
│   │   ├── 40–48GB → 34B BF16, 70B Q4, DeepSeek-R1-Distill-Qwen-32B
│   │   └── 80GB+   → 70B BF16, QwQ-32B BF16, DeepSeek V3/R1 (multi-GPU)
│   │
│   └── Hardware: CPU-only
│       └── ≤7B GGUF Q4_K_M only (warn: slow)
│
└── No (cloud API)
    ├── Need tools + quality → openai/gpt-4o or anthropic/claude-sonnet-4
    ├── Need speed + free → google/gemini-2.0-flash
    ├── Need reasoning → anthropic/claude-opus-4 or openai/o3
    ├── Need low cost → groq/llama-3.3-70b or google/gemini-2.0-flash
    └── Need vision → openai/gpt-4o, anthropic/claude-*, google/gemini-*
```

---

## Minimum Requirements for UAR Tool Calling

UAR agent skills require **tool/function calling** support. Confirm before recommending:

| Model | Tool Calling |
|-------|-------------|
| Llama 3.1/3.2/3.3 | ✅ |
| Mistral 7B+ | ✅ |
| Phi-3.5/4 | ✅ |
| Qwen2.5/3 | ✅ |
| Gemma-3 | ✅ |
| GLM-4 | ✅ |
| DeepSeek-R1-Distill | ❌ (reasoning-only) |
| QwQ-32B | ✅ |
| All major cloud APIs | ✅ |

---

## Recommended Starting Points by Use Case

| Use case | Local | Cloud |
|----------|-------|-------|
| General agent + tools | Qwen3-8B (16GB+) | openai/gpt-4o |
| Code generation | Qwen2.5-Coder-7B | openai/gpt-4o |
| Long document analysis | Qwen3-8B + TurboQuant | anthropic/claude-sonnet-4 |
| Vision + tools | Phi-3.5-Vision or Qwen2.5-VL | openai/gpt-4o |
| Fast low-VRAM | Phi-4-mini (4GB) | groq/llama-3.1-8b-instant |
| Reasoning / CoT | QwQ-32B (20GB Q4) | anthropic/claude-opus-4 |
| Privacy-sensitive | Any local model | N/A |
