# Pluggable embedding backends for RAG and memory

## Purpose

Allow UAR to use multiple embedding providers (local and hosted) for RAG and memory retrieval without changing the downstream RAG pipeline.

## ADDED Requirements

### Requirement: Common embedding backend trait
A trait `EmbeddingBackend` MUST exist with `embed`, `vector_dimension`, and `backend_name` methods. All downstream RAG and memory code MUST consume this trait rather than directly depending on FastEmbed.

#### Scenario: VectorMatcher creates vectors for a query
- **WHEN** a query or document needs to be embedded
- **THEN** `VectorMatcher` calls `backend.embed(...)`
- **AND** receives vectors of the configured dimension
- **AND** does not depend on FastEmbed types directly

### Requirement: Runtime backend selection
The active backend MUST be selected by `llm.embedding.backend` (config file or `UAR_LLM__EMBEDDING__BACKEND` env var). When missing or unknown, the system MUST fall back to `fastembed`.

#### Scenario: Operator selects OpenAI embeddings
- **WHEN** `llm.embedding.backend = "openai"` is configured
- **THEN** UAR uses the OpenAI Embeddings API
- **AND** the RAG pipeline continues to work without code changes

### Requirement: FastEmbed default backend
FastEmbed MUST remain the default backend and MUST be available when the `local-models` feature is enabled. Existing `UAR_MODELS_DIR` and `models.models_dir` configuration continues to apply.

#### Scenario: No embedding backend is configured
- **WHEN** `llm.embedding.backend` is not set
- **THEN** UAR uses FastEmbed with the existing default model
- **AND** local inference is preserved

### Requirement: Candle local backend (feature-gated)
When the `candle-embeddings` feature is enabled, a `candle` backend MUST be available. It MUST load a small embedding model from disk and produce vectors without ONNX Runtime.

#### Scenario: Candle feature is enabled
- **WHEN** the crate is built with `--features candle-embeddings`
- **AND** `llm.embedding.backend = "candle"` is configured
- **THEN** UAR loads a Candle model and embeds locally
- **AND** the build does not require FastEmbed/ONNX

### Requirement: OpenAI hosted backend
When `llm.embedding.backend = "openai"`, UAR MUST call the OpenAI `/v1/embeddings` endpoint. It MUST support `text-embedding-3-small` and `text-embedding-3-large`, and MUST read the API key from `llm.embedding.api_key` or the env var named by `llm.embedding.api_key_env`.

#### Scenario: OpenAI backend is configured
- **WHEN** `llm.embedding.backend = "openai"` and a valid API key is provided
- **THEN** UAR sends batched texts to OpenAI
- **AND** returns the embeddings to the RAG pipeline

### Requirement: Voyage hosted backend
When `llm.embedding.backend = "voyage"`, UAR MUST call the Voyage `/v1/embeddings` endpoint. It MUST support `voyage-3` and `voyage-3-lite`.

#### Scenario: Voyage backend is configured
- **WHEN** `llm.embedding.backend = "voyage"` and a valid API key is provided
- **THEN** UAR sends batched texts to Voyage
- **AND** returns the embeddings to the RAG pipeline

### Requirement: Cohere hosted backend
When `llm.embedding.backend = "cohere"`, UAR MUST call the Cohere `/embed` endpoint. It MUST support `embed-english-v3` and `embed-multilingual-v3`, and MUST set `input_type` based on the call context.

#### Scenario: Cohere backend is configured
- **WHEN** `llm.embedding.backend = "cohere"` and a valid API key is provided
- **THEN** UAR sends batched texts to Cohere with the correct `input_type`
- **AND** returns the embeddings to the RAG pipeline

### Requirement: Unified error handling
All backends MUST return `EmbeddingError`. Errors from hosted providers MUST be logged with the backend name and HTTP status without leaking the API key.

#### Scenario: OpenAI returns a 401
- **WHEN** the OpenAI backend receives a 401
- **THEN** `EmbeddingError::RequestFailed` is returned
- **AND** the log message contains the backend name and status but not the API key

### Requirement: Capability evidence
The product support matrix MUST document each backend, the models it supports, the required feature flags, and whether it works offline.

#### Scenario: A reviewer checks supported backends
- **WHEN** reading `docs/product-support-matrix.json`
- **THEN** each backend has a capability entry with model names, feature flags, and offline/online status
