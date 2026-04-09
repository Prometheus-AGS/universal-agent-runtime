# TurboQuant KV-Cache Compression Guide

TurboQuant (Google Research, ICLR 2026) compresses the transformer attention KV-cache to as few as 3 bits per coordinate, achieving 6–8x memory reduction with virtually zero accuracy loss.

**Source**: `turboquant-rs/README.md` + `candle-vllm/example.models.yaml` `kvcache_compression` field.

---

## What is the KV-Cache?

Every token in a conversation is stored in a key-value (KV) cache in GPU memory. Without compression:
- Each token entry: 2 × layers × d_model × 2 bytes (FP16)
- A 70B model with 128-layer transformer at 8192 context: ~4GB for KV alone

As context grows, KV-cache competes with model weights for VRAM. At 32K+ context windows, this becomes the primary bottleneck.

---

## How TurboQuant Works

Two complementary algorithms from Google Research:

1. **PolarQuant** (AISTATS 2026): Randomized Hadamard rotation + Lloyd-Max scalar quantization. MSE-optimal vector quantization — all bits go to minimizing reconstruction error.

2. **QJL** (AAAI 2025): 1-bit residual correction using Quantized Johnson-Lindenstrauss transform for unbiased inner-product estimation.

**Result**: 3-bit KV-cache with ~9.8x compression ratio and <0.1% quality degradation on standard benchmarks.

---

## candle-vllm Configuration

Configured per-model in `models.yaml`:

```yaml
kvcache_compression:
  bits: 3          # 2, 3, or 4
  policy:
    # Choose ONE of the three policy types:

    # Option A: Always compress
    always: {}

    # Option B: Compress when context exceeds threshold
    threshold_tokens: 4096    # Compress once context > 4096 tokens

    # Option C: Compress when GPU KV-block pressure exceeds threshold
    memory_pressure:
      free_block_pct: 0.20    # Compress when <20% of KV blocks free
```

---

## Bits Decision Table

| Headroom | Bits | Policy | Compression | Quality Impact |
|----------|------|--------|------------|----------------|
| >30% VRAM free | disabled | — | 1× (none) | 0% |
| 15–30% free | 4 | `memory_pressure: {free_block_pct: 0.20}` | ~4× | <0.05% |
| 5–15% free | **3 (recommended)** | `threshold_tokens: 4096` | **~7×** | **<0.1%** |
| <5% free | 3 | `always` | ~7× | <0.1% |
| Critical (<2%) | 2 | `always` | ~16× | ~1% (warn) |

**Apple Silicon Special Case**: Use a higher threshold due to fast unified memory access:

| Apple Silicon | Bits | Policy |
|--------------|------|--------|
| 15–30% free | 4 | `threshold_tokens: 8192` |
| 5–15% free | 3 | `threshold_tokens: 8192` |
| <5% free | 3 | `threshold_tokens: 4096` |

---

## Memory Math

### How much context can I fit?

```
# KV-cache memory per token (rough)
kv_per_token_bytes = 2 × num_layers × d_model × 2 (FP16)

# With turboquant (3-bit)
kv_per_token_compressed = kv_per_token_bytes / 7

# Available KV memory
available_kv_mb = total_vram_gb × 1024 - model_vram_gb × 1024 - os_overhead_mb

# Max context without compression
max_context_tokens = available_kv_mb × 1024 × 1024 / kv_per_token_bytes

# Max context with 3-bit turboquant
max_context_turboquant = max_context_tokens × 7
```

### Practical examples

| Model | VRAM | Normal max context | With 3-bit TurboQuant |
|-------|------|------------------|-----------------------|
| Llama-3.1-8B on RTX 4090 (24GB) | 24GB | ~32K tokens | ~200K tokens |
| Llama-3.1-70B on A100 80GB | 80GB | ~16K tokens | ~100K tokens |
| Qwen3-8B on M3 Max 64GB | 64GB | ~80K tokens | 128K+ native |
| Phi-4 14B on A100 40GB | 40GB | ~24K tokens | ~150K tokens |

---

## How to Configure (Step-by-Step)

### Step 1: Calculate VRAM headroom

```bash
# GPU
available_vram_gb = total_vram_gb           # e.g., 24
model_vram_gb = model_params_b * 2          # 8B model = 16GB
kvcache_vram_gb = configured_mem_mb / 1024  # from models.yaml params.mem
headroom_pct = (available_vram - model_vram) / available_vram
```

### Step 2: Choose bits and policy

Use the decision table above.

### Step 3: Add to models.yaml

```yaml
models:
  - name: my-model
    hf_id: Qwen/Qwen3-8B-Instruct
    params:
      dtype: bf16
      mem: 14336
      max_num_seqs: 32
      device_ids: [0]
    kvcache_compression:
      bits: 3
      policy:
        threshold_tokens: 4096
```

### Step 4: Test

Start candle-vllm and send a long-context request. Check logs for compression activation:
```
[turboquant] KV compression activated at token 4096 (3-bit, ratio=7.1x)
```

---

## Quality Trade-offs

| Bits | Compression Ratio | Quality Loss | When to use |
|------|-----------------|--------------|-------------|
| 4 | ~4× | <0.05% | Tight but not critical VRAM |
| 3 | ~7–8× | <0.1% | **Recommended for most cases** |
| 2 | ~16× | ~0.5–1% | Extreme memory pressure only |

**Note**: Quality loss is measured on standard LLM benchmarks (MMLU, HumanEval, etc.). For most practical tasks, the difference is imperceptible.

---

## Policy Trade-offs

| Policy | When compression starts | Best for |
|--------|------------------------|---------|
| `disabled` | Never | VRAM is plentiful |
| `threshold_tokens: N` | Context > N tokens | Predictable, saves quality on short turns |
| `memory_pressure: {free_block_pct: P}` | When KV blocks run low | Adaptive, maximizes short-context quality |
| `always` | Every token | Tight VRAM, maximum context capacity |

---

## Integration Notes

- TurboQuant is implemented in `turboquant-rs` (pure Rust, zero dependencies beyond `thiserror`)
- It's integrated directly into candle-vllm's PagedAttention engine
- No retraining required — applies to any supported model
- The compression/decompression happens lazily before each forward pass
- 3-bit compression adds ~2–3% latency overhead on modern GPUs (negligible vs compute)
