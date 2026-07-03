## 1. Backend

- [x] 1.1 `GET /api/models` includes a `benchmarks` array per model
      (`src/server.rs`, sourced from `crate::llm::benchmarks::scores_for`).

## 2. Frontend types + data flow

- [x] 2.1 `CatalogModelBenchmark`/`BenchmarkDimension` types + `CatalogModel.benchmarks`
      (`frontend/src/types/index.ts`).
- [x] 2.2 `benchmarks` carried into the `Model` graph entity
      (`frontend/src/entities/fetchers/models.ts`).

## 3. Compare UI

- [x] 3.1 Compare checkbox per catalog row (`ModelRow`), bounded to
      `MAX_COMPARE = 4` selections.
- [x] 3.2 Sticky compare bar (running selection, remove/clear, "compare (N)").
- [x] 3.3 `CompareDialog`: side-by-side table — context window, cost,
      capabilities, one row per benchmark dimension present in the
      selection — with best-value highlighting per row.

## 4. Verify

- [x] 4.1 `cargo check --lib` + `cargo test --lib` (330/330) green for the
      backend `benchmarks` field; confirmed via live curl that
      `anthropic/claude-haiku-4-5` returns a real sourced score.
- [x] 4.2 `bun run build` (frontend) green.
- [x] 4.3 Live server smoke test: Playwright screenshot of `/admin/models`
      — page renders, zero console errors. Catalog list itself is empty in
      this environment due to the pre-existing `useEntityList` hook drift
      (see proposal.md) — not a defect in this change's own code, but it
      does mean the compare checkboxes/dialog could not be interactively
      exercised with real data this pass.

## 5. Follow-ups (disclosed, not this pass)

- [ ] Pre-existing `useEntityList`/`useEntity` hook API drift (see
      proposal.md scope note) blocks interactive verification of the
      catalog list (and therefore the compare feature) against a live
      server until fixed.
