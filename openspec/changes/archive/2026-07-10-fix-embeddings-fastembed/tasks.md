## 1. Dependency + assets

- [x] 1.1 Add `fastembed` 5.x to `Cargo.toml` with offline-only features (no
      hf-hub runtime download path); `cargo check` records build-time/size
      impact and confirms the pinned `ort` rc coexists with the workspace.
- [x] 1.2 Add the canonical `BAAI/bge-small-en-v1.5` model `config.json` to
      `src/uar/runtime/matching/models/` (required by
      `UserDefinedEmbeddingModel`; repo has tokenizer files but no model
      config).

## 2. Core rewrite

- [x] 2.1 Rewire `VectorMatcher` internals: load
      `TextEmbedding::try_new_from_user_defined` from on-disk assets in
      `initialize()`; replace the burn placeholder path in `embed_batch` with
      real inference inside `spawn_blocking`; delete the
      "generic placeholder mode" warn. Public API unchanged (per design D3).
- [x] 2.2 `embedding_provider` honesty (design D5): unknown values log a loud
      warning naming the value + fallback; `"fastembed"` remains default.
- [x] 2.3 Stale-index self-identification (design D4): zero-norm stored
      embeddings hit during search log an explicit error naming the KB and
      pointing at re-ingestion.

## 3. Tests

- [x] 3.1 Unit tests: non-zero norms; near-duplicate pair similarity >
      unrelated pair; empty-batch; offline path (no network fixture).
- [x] 3.2 Full `cargo test --lib` + integration suites green; fix fallout.
- [x] 3.3 Live verification: boot server, ingest fixture doc, direct
      `POST /api/knowledge/{id}/search` returns the phrase-bearing chunk.
- [x] 3.4 bdd-chat suite locally: `chat-kb-retrieval.feature` passes
      UNWEAKENED → 6/6. Check the 2 `#[ignore]`d live-integration cases —
      un-ignore any that were gated on this bug.

## 4. Docs + bookkeeping

- [x] 4.1 Update `docs/BDD_SCENARIOS.md` (scenario 2 → PASS, bug note →
      fixed) and add the re-ingestion upgrade note where the docs-site change
      will pick it up.
- [x] 4.2 Commit, push, confirm bdd-chat.yml goes 6/6 green on real CI;
      update phase progress.json/waypoint; `openspec validate --strict` +
      archive.
