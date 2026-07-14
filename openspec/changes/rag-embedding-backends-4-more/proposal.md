## Why

UAR currently uses FastEmbed as the local embedding backend for RAG and memory. The 2026-07-13 release-readiness assessment identified pluggable embedding backends as a grade-A gap: production deployments need the ability to use hosted providers (OpenAI, Cohere, Voyage) and a local Candle backend without rebuilding the binary.

Change 15 adds four Tier-1 embedding backends behind a unified provider trait and selects the active backend via `UAR_LLM__EMBEDDING__BACKEND` (and the config file). FastEmbed remains the default local backend so existing behavior is preserved.

## What Changes

- Introduce a pluggable `EmbeddingBackend` trait in `src/uar/rag/embeddings/`.
- Add four backend implementations:
  - `fastembed` (local, default) — the existing behavior, refactored behind the trait.
  - `candle-embeddings` (local) — `candle` crate, runs small embedding models on CPU/GPU without ONNX Runtime.
  - `openai-embeddings` (hosted) — `text-embedding-3-small` / `text-embedding-3-large` via `reqwest`.
  - `voyage` (hosted) — `voyage-3` / `voyage-3-lite` via `reqwest`.
  - `cohere` (hosted) — `embed-english-v3` / `embed-multilingual-v3` via `reqwest`.
- Add `llm.embedding` config section with `backend`, `model`, `api_key`, `api_key_env`, `base_url`, `vector_dimension`, and `batch_size`.
- Refactor existing call sites (`VectorMatcher`, `IngestService`, `MemoryService`) to consume `Arc<dyn EmbeddingBackend>` instead of directly depending on `fastembed`.
- Add backend-selection tests and a round-trip smoke test for each provider.
- Update `docs/product-support-matrix.json` with capability evidence for each backend.

## Capabilities

### New Capabilities

- `embedding-provider-pluggable`: runtime-selectable embedding backends with a common trait.

## Impact

- **No breaking change to existing deployments**: FastEmbed stays the default.
- **New deployments can use hosted embeddings** without installing ONNX Runtime.
- **Local CPU-only deployments can use Candle** instead of FastEmbed/ONNX.
- **The same `VectorMatcher` and RAG pipeline work unchanged**; only the embedding model source changes.

## Out of scope

- Quantization, distillation, or fine-tuning of embedding models.
- Async batching beyond simple per-request batching exposed by the provider APIs.
- Migration of existing vectors between backends (a future re-indexing feature).
