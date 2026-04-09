# candle-vllm Model Catalog

Supported model architectures in the Prometheus fork of candle-vllm. Source of truth: `candle-vllm/README.md` + `candle-vllm/crates/candle-vllm-core/src/`.

---

## Architecture Support Table

| # | Architecture | Tool Calling | Vision | Reasoning | GGUF | Notes |
|---|-------------|-------------|--------|-----------|------|-------|
| 1 | **Llama 3/3.1/3.2/3.3** | ✅ | ❌ | ❌ | ✅ | Fastest. 1B/3B/8B/70B variants |
| 2 | **Mistral / Ministral** | ✅ | ❌ | ❌ | ✅ | Use BF16/FP16 only (no FP8) |
| 3 | **Phi-3 / Phi-4** | ✅ | ✅ (Phi-3.5-Vision) | ❌ | ✅ | Small VRAM, strong quality/size ratio |
| 4 | **Qwen2.5 / Qwen3** | ✅ | ✅ (VL variants) | ✅ (thinking mode) | ✅ | 0.5B–72B, strong code + tools |
| 5 | **Yi** | ✅ | ❌ | ❌ | ✅ | 6B/34B |
| 6 | **StableLM** | ❌ | ❌ | ❌ | ❌ | 3B, lightweight |
| 7 | **Gemma-2 / Gemma-3** | ✅ | ✅ (Gemma-3) | ❌ | ✅ | Google models, 9B/27B |
| 8 | **DeepSeek-R1-Distill-Qwen** | ❌ | ❌ | ✅✅✅ | ✅ | Reasoning-only, 14B/32B |
| 9 | **DeepSeek-R1-Distill-Llama** | ❌ | ❌ | ✅✅✅ | ✅ | Reasoning-only, 8B/70B |
| 10 | **DeepSeek V2/V3/R1** | ✅ | ❌ | ✅ | ✅ (AWQ) | MoE, large, multi-GPU |
| 11 | **QwQ-32B** | ✅ | ❌ | ✅✅✅ | ✅ | Best reasoning + tools, 32B |
| 12 | **GLM-4** | ✅ | ✅ | ❌ | ✅ | 9B |
| 13 | **Qwen2 MoE** | ✅ | ❌ | ❌ | ✅ | 14B active params |
| 14 | **Qwen3 MoE** | ✅ | ❌ | ✅ | ✅ | 32B, fast |

---

## VRAM Requirements by Model Size

Assumes single GPU, no turboquant. Add ~20% for KV-cache headroom.

| Model | BF16/FP16 VRAM | Q4_K_M GGUF VRAM | Recommended dtype |
|-------|---------------|-----------------|-------------------|
| 0.5B | ~1GB | ~0.3GB | f16 |
| 1B | ~2GB | ~0.6GB | f16 or GGUF |
| 1.5B | ~3GB | ~0.8GB | f16 or GGUF |
| 3B | ~6GB | ~1.7GB | f16 or GGUF |
| 7B / 8B | ~14–16GB | ~4.5GB | bf16 (24GB+) or GGUF |
| 13B | ~26GB | ~7GB | GGUF on <32GB |
| 14B | ~28GB | ~8GB | bf16 (40GB+) or GGUF |
| 32B | ~64GB | ~18GB | GGUF on <80GB |
| 70B | ~140GB | ~40GB | Multi-GPU BF16 or GGUF |

---

## Quantization Options

| Format | candle-vllm param | VRAM | Quality | Notes |
|--------|------------------|------|---------|-------|
| BF16 full | `dtype: bf16` | 2×params GB | Best | Requires sufficient VRAM |
| FP16 full | `dtype: f16` | 2×params GB | Best | Alternative to BF16 |
| FP32 | `dtype: f32` | 4×params GB | Best | CPU only |
| ISQ Q4K (in-situ) | `isq: q4k` | ~0.5×params GB | Good | Applied at load time |
| ISQ Q8 | `isq: q8` | ~1×params GB | Very good | Good balance |
| GGUF Q4_K_M | `weight_file: *.gguf` | ~0.5×params GB | Good | Pre-quantized HF file |
| GGUF Q2_K | `weight_file: *.gguf` | ~0.25×params GB | Fair | Memory-constrained only |
| GPTQ/Marlin 4-bit | auto-detected | ~0.5×params GB | Good | Faster than GGUF |

---

## Model Entry Reference (models.yaml format)

### BF16 full precision (24GB GPU)
```yaml
- name: qwen3-8b
  hf_id: Qwen/Qwen3-8B-Instruct
  params:
    dtype: bf16
    mem: 14336             # KV cache in MB (~14GB)
    max_num_seqs: 32
    block_size: 16
    device_ids: [0]
    temperature: 0.7
    top_p: 0.9
    prefill_chunk_size: 8192
```

### GGUF Q4_K_M (8GB GPU)
```yaml
- name: llama-3-8b-q4k
  hf_id: bartowski/Meta-Llama-3.1-8B-Instruct-GGUF
  weight_file: Meta-Llama-3.1-8B-Instruct-Q4_K_M.gguf
  params:
    dtype: f16
    mem: 4096
    max_num_seqs: 16
    block_size: 64
    device_ids: [0]
```

### With TurboQuant (tight VRAM, 3-bit, threshold policy)
```yaml
- name: llama3-70b-compressed
  hf_id: meta-llama/Llama-3-70B-Instruct
  params:
    dtype: bf16
    mem: 72000
    max_num_seqs: 8
    block_size: 16
    device_ids: [0]
    prefill_chunk_size: 8192
    kvcache_compression:
      bits: 3
      policy:
        threshold_tokens: 4096
```

### Multi-GPU (4× A100 80GB)
```yaml
- name: llama3-70b-multigpu
  hf_id: meta-llama/Llama-3-70B-Instruct
  params:
    dtype: bf16
    mem: 16384
    max_num_seqs: 512
    device_ids: [0, 1, 2, 3]
    multithread: true
```

### Vision (proxy mode)
```yaml
- name: phi-3.5-vision
  hf_id: microsoft/Phi-3.5-vision-instruct
  params:
    dtype: f16
    mem: 6144
    max_num_seqs: 16
    device_ids: [0]
  capabilities:
    vision_mode: proxy
    vision_proxy:
      hf_id: microsoft/Phi-3.5-vision-instruct
      prompt_template: "Describe this image in detail:"
```

---

## Parking Lot Scheduler (per-model overrides)

```yaml
parking_lot:
  pool:
    worker_threads: 4         # Inference worker threads
  limits:
    max_units: 4096           # Max KV-cache blocks
    max_queue_depth: 100      # Max queued requests
    timeout_secs: 300         # Request timeout
  queue:
    backend: "memory"         # memory | postgres | yaque
    persistence: false
  mailbox:
    backend: "memory"
    retention_secs: 3600
```

---

## HuggingFace Model IDs (Quick Reference)

| Alias | HuggingFace ID | Size | Best for |
|-------|---------------|------|---------|
| Llama-3.1-8B | `meta-llama/Llama-3.1-8B-Instruct` | 8B | General purpose + tools |
| Llama-3.2-3B | `meta-llama/Llama-3.2-3B-Instruct` | 3B | Lightweight + tools |
| Llama-3.3-70B | `meta-llama/Llama-3.3-70B-Instruct` | 70B | Best Llama quality |
| Mistral-7B | `mistralai/Mistral-7B-Instruct-v0.3` | 7B | Fast + tools |
| Ministral-8B | `mistralai/Ministral-8B-Instruct-2410` | 8B | Fast + tools |
| Phi-4 | `microsoft/Phi-4` | 14B | Strong reasoning, tools |
| Phi-3.5-mini | `microsoft/Phi-3.5-mini-instruct` | 3.8B | Small, fast, tools |
| Phi-3.5-vision | `microsoft/Phi-3.5-vision-instruct` | 4.1B | Vision + tools |
| Qwen3-8B | `Qwen/Qwen3-8B-Instruct` | 8B | Tools + reasoning + code |
| Qwen3-32B | `Qwen/Qwen3-32B-Instruct` | 32B | High quality reasoning |
| Qwen3-0.6B | `Qwen/Qwen3-0.6B` | 0.6B | Ultra-lightweight |
| QwQ-32B | `Qwen/QwQ-32B` | 32B | Best reasoning + tools |
| Gemma-3-9B | `google/gemma-3-9b-it` | 9B | Quality balance |
| DeepSeek-R1-8B | `deepseek-ai/DeepSeek-R1-Distill-Llama-8B` | 8B | Reasoning only |
| GLM-4-9B | `THUDM/glm-4-9b-chat` | 9B | Tools + vision |

---

## Choosing dtype for candle-vllm

| Scenario | dtype |
|---------|-------|
| NVIDIA GPU (Ampere+, sm80+) | `bf16` |
| NVIDIA GPU (older) | `f16` |
| Apple Silicon M1/M2 | `f16` |
| Apple Silicon M3/M4+ | `bf16` |
| CPU only | `f32` |
| GGUF model | `f16` (loads automatically correct precision) |
