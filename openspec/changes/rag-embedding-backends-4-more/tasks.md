## 1. Design the embedding trait
- [x] 1.1 Add `src/uar/rag/embeddings/mod.rs` defining `EmbeddingBackend` trait:
  - `async fn embed(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, EmbeddingError>`
  - `fn vector_dimension(&self) -> usize`
  - `fn backend_name(&self) -> &str`
- [x] 1.2 Add `EmbeddingError` enum with `BackendUnavailable`, `RequestFailed`, `DimensionMismatch`, `InvalidInput`.
- [x] 1.3 Add `EmbeddingConfig` struct under `llm` config section.

## 2. FastEmbed backend (default, refactor)
- [x] 2.1 Move existing FastEmbed usage from `VectorMatcher` / `IngestService` into `src/uar/rag/embeddings/fastembed.rs`.
- [x] 2.2 Implement `EmbeddingBackend` for `FastEmbedBackend`.
- [x] 2.3 Ensure `vector_dimension` matches `config.models.vector_dimension` / `persistence.vector_dimension`.
- [x] 2.4 Add a unit test that verifies FastEmbed embeddings round-trip through the trait.

## 3. Candle local backend
- [x] 3.1 Add `candle-core`, `candle-nn`, `candle-transformers`, and `tokenizers` dependencies behind a `candle-embeddings` feature.
- [x] 3.2 Implement `CandleEmbeddingBackend` in `src/uar/rag/embeddings/candle.rs`.
- [x] 3.3 Load a small BGE-style model from `models_dir` on first call; cache the model in the backend struct.
- [x] 3.4 Verify `cargo check --no-default-features --features server-full,candle-embeddings` compiles.

## 4. OpenAI hosted backend
- [x] 4.1 Implement `OpenAiEmbeddingBackend` in `src/uar/rag/embeddings/openai.rs` using `reqwest`.
- [x] 4.2 Support `text-embedding-3-small` and `text-embedding-3-large` models.
- [x] 4.3 Read API key from `llm.embedding.api_key` or `llm.embedding.api_key_env`.
- [x] 4.4 Add a stub test for the OpenAI request shape and API key resolution.

## 5. Voyage hosted backend
- [x] 5.1 Implement `VoyageEmbeddingBackend` in `src/uar/rag/embeddings/voyage.rs` using `reqwest`.
- [x] 5.2 Support `voyage-3` and `voyage-3-lite` models.
- [x] 5.3 Read API key from `llm.embedding.api_key` or `llm.embedding.api_key_env`.
- [x] 5.4 Add a stub test for the Voyage request shape and API key resolution.

## 6. Cohere hosted backend
- [x] 6.1 Implement `CohereEmbeddingBackend` in `src/uar/rag/embeddings/cohere.rs` using `reqwest`.
- [x] 6.2 Support `embed-english-v3` and `embed-multilingual-v3` models.
- [x] 6.3 Handle Cohere's `input_type` parameter (`search_document` by default; configurable via `model` config for other modes).
- [x] 6.4 Add a stub test for the Cohere request shape and API key resolution.

## 7. Config and wiring
- [x] 7.1 Add `llm.embedding` section to `AppConfig` with backend selection, model, API key, base URL, vector dimension, and batch size.
- [x] 7.2 In `start_server`, build `Arc<dyn EmbeddingBackend>` based on `llm.embedding.backend` and store it in `AppState`.
- [x] 7.3 Replace direct `VectorMatcher` / `IngestService` FastEmbed usage with the backend from `AppState`. (`MemoryService` continues to use surreal-memory's embedding service; no direct FastEmbed usage remains in RAG/matching.)
- [x] 7.4 Update `docs/product-support-matrix.json` with backend capability evidence.

## 8. Verification
- [x] 8.1 Run `cargo check --no-default-features --features server-full`.
- [x] 8.2 Run `cargo check --no-default-features --features server-full,candle-embeddings`.
- [x] 8.3 Run `cargo test --no-default-features --features server-full --lib embeddings` and ensure green.
- [x] 8.4 Run `openspec validate --strict --changes rag-embedding-backends-4-more` and confirm validity.
- [x] 8.5 Mark Change 15 implementation complete in `progress.json` and update `current-waypoint.json`.

## Notes
- Fixed pre-existing `JsonSchema` derive gaps on `ClassifierConfig`, `ClassifierBackend`, `ProviderConfig`, `ProtocolSetting`, `ModelConfig`, and `ContextStrategy` surfaced by `AppConfig` schema generation.
- Fixed pre-existing `SecretString` usage in `middleware.rs` and `settings/manager.rs` by explicitly calling `expose_secret()`.
- FastEmbed module is gated behind the `local-models` feature; `candle` module is gated behind `candle-embeddings`.
