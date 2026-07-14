## Why

Assessment H1/H6/M2/O2: RAG retrieval injects chunks invisibly (no SSE
events, so the UI cannot show KB hits), ingestion status stays pending
forever on SurrealDB 3.x, and the chat path bypasses the hardened
retrieval pipeline.

## What Changes

- Emit citation/KB-hit events from the retrieval path with KB/document provenance.
- Fix the ingestion status write; render KB hits in the chat UI.
- Adopt the hardened retrieval pipeline for chat (or document divergence);
  make the kb-retrieval BDD feature assert real provenance (O2).

## Capabilities
### New Capabilities
- `rag-provenance`

## Impact
Run manager retrieval, SSE mapping, ingestion worker, chat UI, BDD feature.
