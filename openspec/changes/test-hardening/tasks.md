## 1. Gate + assertion hardening

- [x] 1.1 bdd-chat.yml -> blocking (removed continue-on-error on the suite step).
- [x] 1.2 live-integration.yml test tier -> blocking; Matrix check stays advisory;
      header comment corrected.
- [x] 1.3 rag.spec.ts: remove failure-is-a-pass; assert non-empty response + no
      error state; reference the verified rag_ingest_then_retrieve integration test.
- [x] 1.4 Deferred + disclosed: full browser upload->search e2e + broad vitest
      store coverage (need live e2e harness; RAG has verified integration coverage).

## 2. Verify + bookkeeping

- [ ] 2.1 On push, confirm bdd-chat + live-integration (now blocking) are GREEN
      on real CI (else the flip breaks main).
- [ ] 2.2 Commit, push, archive; update phase state.
