## Why

Backend RAG now embeds and retrieves correctly, but the React Knowledge journey and its failure/realtime behavior are not fully certified and its hook owns service I/O.

## What Changes

- Move knowledge I/O into an owning store/domain action.
- Certify KB create, upload, index, ranked search, chat grounding, delete, retry, auth, and reconciliation.

## Capabilities
### New Capabilities
- `knowledge-rag-product-certification`

## Impact
Knowledge React feature, store/service, RAG APIs, BDD/Playwright tests.
