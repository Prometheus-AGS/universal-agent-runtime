## Why

Goal 4 (test coverage credible for production claims): two proven-green suites
ran as advisory (`continue-on-error: true`), so a regression would not fail CI;
and `tests/e2e/rag.spec.ts` was a visibility-only smoke that treated a failed or
empty response as a pass.

## What Changes

- Flip `bdd-chat.yml` to BLOCKING (6/6 green on real CI since
  fix-embeddings-fastembed).
- Flip the `live-integration.yml` recorded-backend test tier to BLOCKING (green
  on real CI); keep the Matrix presence check advisory (governance, not a code
  regression).
- Tighten `tests/e2e/rag.spec.ts`: remove the "graceful failure is a pass"
  anti-pattern; require a real non-empty assistant response and zero error
  states. The deterministic upload->ingest->search->retrieval assertion remains
  covered by the `rag_ingest_then_retrieve` integration test.

## Capabilities

### New Capabilities
- `test-credibility`: load-bearing suites gate CI (blocking), and e2e specs
  assert real outcomes rather than mere visibility.

## Impact

CI: two suites become blocking. One e2e spec strengthened. KBD: change 8/9.
Deferred (disclosed): a full browser upload->search e2e and broad vitest store
coverage need the live e2e harness to author+verify; RAG already has verified
integration coverage.
