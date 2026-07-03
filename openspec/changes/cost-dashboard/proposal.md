# CH-07 cost-dashboard

## Why

CH-06 (cost-budgets-backend) wired spend aggregation and a `BudgetAlert` event
into the runtime, but there was no admin UI surface for it — an operator had
to read Prometheus `uar_cost_budget_spent_usd` directly to see spend or
budget status.

## What changed

- New `CostDashboardPage` (`frontend/src/admin/pages/cost-dashboard-page.tsx`),
  registered as a new "Cost" admin nav item (Infrastructure group,
  `admin-shell.tsx` + `admin-page.tsx` PAGE_MAP).
- Reads the shared entity graph directly (`useGraphStore`, same low-level
  pattern as `runtime-console-page.tsx` — not the higher-level
  `useEntityList` hook; see scope note below): stat tiles (total spend,
  priced runs, budget warnings, budgets exceeded), a spend-over-time bar
  chart (first real consumer of the previously-unused `recharts` +
  `ChartContainer` scaffolding), a per-model spend breakdown, and a budget
  alerts list.
- `NormalizedEvent::BudgetAlert` (CH-06) is now mapped into the entity-graph
  SSE surface (`to_runtime_entity_event` in `src/uar/api/sse.rs`, new
  `runtime.budget_alert` → `RuntimeBudgetAlert` entity type) alongside its
  existing `agui.budget.alert` legacy mapping — it previously only reached
  the legacy AG-UI surface, not the Runtime Console entity graph the new
  page needs.
- `RuntimeRunEntity` (`frontend/src/entities/types.ts`) gained
  `cost_usd_estimate`/`input_tokens`/`output_tokens`/`total_tokens` fields —
  present in the wire payload since CH-06 shipped, but untyped until now.
- Follows `docs/admin-aesthetic-spec.md` (terminal/CRT theme): raw
  `hsl(var(--terminal-fg))`-style tokens, the shared `<EmptyFrame>`/
  `<LoadingCursor>` components (introduced in a later change than
  `models-page.tsx`, so this new page uses them directly rather than
  inlining local equivalents).

## Scope notes

- **Session-scoped, not historical**: like the rest of the Runtime Console,
  data comes from the in-memory entity graph fed by SSE — there is no
  persisted run-history REST endpoint. Spend/alerts shown cover runs
  completed since the page was last loaded, not an all-time ledger. A
  durable roll-up (SurrealDB/Postgres) is a reasonable follow-up, matching
  CostBudgetTracker's own module doc ("durable roll-ups can layer on top by
  subscribing to the emitted events").
- **Verification limitation (disclosed, pre-existing, unrelated to this
  change):** while verifying this page and CH-10 against a live server, a
  version mismatch was found between this repo's entity-graph hook call
  sites and the `@prometheus-ags/prometheus-entity-management` submodule
  (which was *uninitialized* — empty — for this entire phase until this
  pass; initializing and building it surfaced the mismatch for the first
  time). The higher-level `useEntityList`/`useEntity` hooks now take an
  options object; several existing call sites (`use-models.ts`,
  `use-compiler-sessions.ts`, `use-mcp-status.ts`, `use-memory.ts`,
  `use-settings-entity.ts`) still call them with the old bare-string API and
  silently get empty results. This page deliberately uses the lower-level
  `useGraphStore` selector directly (the same pattern `runtime-console-page.tsx`
  already uses) and is unaffected — confirmed rendering correctly (empty
  state) against a live local server. Fixing the broader hook-API drift is
  out of scope here; flagged as a phase follow-up.
