# local-embedding-inference Specification

## Purpose
TBD - created by archiving change fix-embeddings-fastembed. Update Purpose after archive.
## Requirements
### Requirement: Real Local Embedding Inference
`VectorMatcher::embed_batch` SHALL produce real BGE-small-en-v1.5 embeddings
(384-dim, L2-normalized) computed locally from the repository's on-disk model
and tokenizer assets, with no network access at runtime and no placeholder
values.

#### Scenario: Embeddings are non-degenerate and discriminative
- **WHEN** two semantically different texts and one near-duplicate pair are
  embedded
- **THEN** all vectors have non-zero norm, and the near-duplicate pair's
  cosine similarity exceeds the unrelated pair's

#### Scenario: Offline operation
- **WHEN** the server runs without outbound network access
- **THEN** embedding inference still succeeds using only on-disk assets

### Requirement: Knowledge-Base Retrieval Returns Real Matches
Knowledge-base search SHALL return ranked matches for queries semantically
matching ingested document content.

#### Scenario: Exact-phrase document is retrievable
- **WHEN** a document containing a distinctive phrase is ingested and indexed,
  and that phrase is queried via `POST /api/knowledge/{id}/search`
- **THEN** the response contains at least one result whose chunk includes the
  phrase

#### Scenario: Chat RAG injects retrieved content
- **WHEN** an agent scoped to that knowledge base is asked a question
  answerable only from the ingested document
- **THEN** the outgoing LLM request's system prompt contains the retrieved
  content (the existing `chat-kb-retrieval.feature` scenario passes without
  modification)

### Requirement: Embedding Provider Config Honesty
The knowledge-base `embedding_provider` configuration value SHALL reflect the
engine actually used; unsupported values SHALL produce a loud warning and
documented fallback rather than silent acceptance.

#### Scenario: Unknown provider value warns
- **WHEN** a knowledge base is configured with an unrecognized
  `embedding_provider`
- **THEN** a warning naming the value and the fallback engine is logged and
  retrieval still functions

### Requirement: Stale Zero-Vector Index Self-Identification
Search over an index containing zero-norm stored embeddings SHALL log an
explicit error identifying the index as stale rather than silently returning
empty results.

#### Scenario: Legacy zero-vector rows detected
- **WHEN** a search executes against stored embeddings with zero norm
- **THEN** an error log names the knowledge base and directs the operator to
  re-ingest per the upgrade guide

