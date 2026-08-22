## Why

Assessment H1/H6/M2/O2: RAG retrieval injects chunks invisibly (no SSE
events, so the UI cannot show KB hits), ingestion status stays pending
forever on SurrealDB 3.x, and the chat path bypasses the hardened
retrieval pipeline.

## What Changes

- Add knowledge-base identity to the existing RAG citation event, preserving
  document identity and the existing chat source-badge rendering.
- Surface SurrealDB statement errors from the ingestion status write and prove
  the indexed transition on embedded SurrealDB and PostgreSQL.
- Route chat retrieval through the existing hardened retrieval pipeline and
  strengthen the kb-retrieval BDD feature to assert visible source provenance.

## Capabilities
### New Capabilities
- `rag-provenance`

## Impact
Run manager retrieval, normalized/SSE events, Surreal persistence, focused
provider tests, and the existing chat-retrieval BDD feature.
