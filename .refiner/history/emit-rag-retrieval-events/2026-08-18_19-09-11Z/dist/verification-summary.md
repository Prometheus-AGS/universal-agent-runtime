# RAG retrieval provenance verification summary

Scope: `server-full` on macOS, Chromium through the BDD dev stack, embedded
SurrealKV, and PostgreSQL 17 with pgvector 0.8.6 where named. Results transfer
to no other profile, browser, database version, or platform.

- Provenance: PASS locally. The AG-UI mapping preserves KB ID, document ID, and
  document name; the browser displays the source badge and hover-card filename.
- Chat pipeline: PASS locally. The manager uses `RagRetrievalPipeline`; six focused
  tests cover decomposition, deduplication, verification, post-score limiting, and the
  structured `rag.retrieval.decision` metadata and fields.
- SurrealKV status: PASS locally. A pending document was read back indexed after
  the checked statement completed.
- PostgreSQL status: PASS locally. The lifecycle passed against an isolated
  PostgreSQL 17 database with pgvector.
- TypeScript/browser fixture: PASS locally. Typecheck and ESLint exit 0; the
  exact scenario passes 1/0 after observed fixture timing/locator corrections.
- Rust Tier 0: PASS within the named baseline. Check exits 0 with three known
  warnings; scoped Clippy exits 0 with 571 warnings. No warning-free claim.
- Tier timing: full phase Tier 2 remains deferred until all active-phase changes
  are complete.

Independent artifact review: PASS. The artifact critic and independent artifact
judge accepted the exact candidate hashes and refreshed receipts. The artifact
converged with 4/4 constraints satisfied.
