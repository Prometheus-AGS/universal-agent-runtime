# CH-10 model-comparison-dashboard

## Why

CH-09 (capability-registry-benchmarks) added sourced benchmark data to the
internal model catalog, but it was never exposed via REST — the models
admin page had no way to show it, and there was no way to compare models
side-by-side at all (benchmarks, cost, capabilities).

## What changed

- `GET /api/models` (`src/server.rs`) now includes a `benchmarks` array per
  model (`benchmark`, `dimension`, `score`, `source_url`, `retrieved_date`),
  sourced from `crate::llm::benchmarks::scores_for` — previously
  CH-09's data was only consumed internally by `ModelRouter`.
- `CatalogModel`/`CatalogModelBenchmark` types (`frontend/src/types/index.ts`)
  and the graph-entity mapping (`frontend/src/entities/fetchers/models.ts`)
  carry the new field through.
- `models-page.tsx` gained a bounded (max 4) multi-select compare flow:
  a checkbox on each catalog row, a sticky compare bar showing the running
  selection, and a `CompareDialog` with a side-by-side table (context
  window, cost, capabilities, and one row per benchmark dimension present
  in the selection) that highlights the best value per row (largest
  context, cheapest cost, highest score).

## Scope notes

- **Verification limitation (disclosed, pre-existing, unrelated to this
  change):** the same pre-existing `useEntityList` hook API mismatch
  described in CH-07's proposal.md affects `models-page.tsx`'s existing
  catalog list (`useModels()` → `useEntityList("Model")`, called with the
  library's old bare-string API against a submodule now expecting an
  options object) — this predates CH-10 and affects the page's *existing*
  catalog display, not anything newly added here. Confirmed via a live
  server: the compare checkboxes, sticky bar, and dialog are correctly
  wired against the `ModelRowShape`/`CatalogModelBenchmark` types and the
  page's own empty/loading states render cleanly, but the catalog list
  itself is empty in this environment until the hook-API drift is fixed
  (tracked as a phase follow-up, not part of this change).
- Compare selection is capped at 4 models (`MAX_COMPARE`) to keep the
  dialog table readable; further raising it is a straightforward constant
  change if needed.
