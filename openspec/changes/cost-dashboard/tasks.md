## 1. Backend gap-closing (prerequisite)

- [x] 1.1 `NormalizedEvent::BudgetAlert` mapped in `to_runtime_entity_event`
      (`src/uar/api/sse.rs`) → `runtime.budget_alert` SSE event.
- [x] 1.2 `RuntimeBudgetAlertEntity` type (`frontend/src/entities/types.ts`)
      + `registerSchema` (`schemas.ts`) + `EVENT_TYPE_MAP` entry
      (`runtime-ingest.ts`).
- [x] 1.3 `RuntimeRunEntity` gained `cost_usd_estimate`/`input_tokens`/
      `output_tokens`/`total_tokens` fields.

## 2. Page

- [x] 2.1 `CostDashboardPage` — stat tiles, spend-over-time chart (recharts
      + `ChartContainer`), per-model breakdown, budget alerts list.
- [x] 2.2 Registered as a new "Cost" admin nav section (Infrastructure
      group) + `PAGE_MAP` entry.
- [x] 2.3 Follows `docs/admin-aesthetic-spec.md`: terminal tokens, shared
      `<EmptyFrame>`/`<LoadingCursor>` components.

## 3. Verify

- [x] 3.1 `cargo check --lib` + `cargo test --lib` (330/330) green for the
      backend piece.
- [x] 3.2 `bun run build` (frontend) green.
- [x] 3.3 Live server smoke test (local `cargo build --bin`, embedded
      SurrealDB, `UAR_SECURITY__JWT_REQUIRED=false` for local access):
      Playwright screenshot of `/admin/cost` — renders correctly, all four
      empty states show (`0 priced runs`, `no priced runs yet`, `no model
      breakdown yet`, `no budget alerts`), zero console errors.
- [ ] 3.4 Loaded-state (non-empty) screenshot — not captured this pass; no
      priced runs existed on the fresh verification server (no LLM calls
      were made). The empty-state rendering plus the code review of the
      chart/breakdown/alert-list logic stand in for this pass.

## 4. Follow-ups (disclosed, not this pass)

- [ ] Durable spend history (current data is session-scoped/in-memory).
- [ ] Pre-existing `useEntityList`/`useEntity` hook API drift across
      `use-models.ts`/`use-compiler-sessions.ts`/`use-mcp-status.ts`/
      `use-memory.ts`/`use-settings-entity.ts` (see proposal.md scope note)
      — unrelated to this change, found while verifying it.
