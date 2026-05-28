# Reflection — `settings-store-retirement`

**Date:** 2026-05-27
**Tool:** claude-code (`/kbd-reflect`)
**Backend:** OpenSpec
**Status:** reflect_complete

---

## 1. Goal achievement

**Phase goal:** Retire `settings-store.ts` (the only remaining Zustand store on a write path for entity data) and move dirty/saving/error state into a module-level form cache consumed via `useSyncExternalStore`. Zero call-site changes on the 3334 LOC `settings-page.tsx`.

| # | DoD criterion | Verdict |
|---|---|---|
| C1 | `settings-store.ts` deleted; `useSettingsStore` grep empty | **MET** |
| C2 | `useSettings()` contract unchanged | **MET** — settings-page.tsx built without modification |
| C3 | dirty/conflicts/saving/error work per-namespace | **MET** — module-level `Map<namespace, DirtyState>` |
| C4 | `saveAll` posts dirty payload + reflects server response | **MET** |
| C5 | SSE conflict semantics preserved | **MET** — derived via `useMemo` from `dirty` vs `graphView.values` |
| C6 | Non-page listeners still receive `uar:settings-changed` | **MET** — `entities/sync.ts:292` emits; bus unchanged |
| C7 | `initSettingsRealtimeBridge` removed from `main.tsx` | **MET** |
| C8 | Tests ≥ 36/36 | **EXCEEDED** — 40/40 (4 new tests added) |
| C9 | Build clean | **MET** |
| C10 | Audit row flipped to `direct` | **MET** — plus new "Form-cache pattern" playbook section |
| C11 | Net LOC negative | **MET** — see §3 |

**Goal achievement: 100%** (all 11 criteria MET).

---

## 2. Delivered changes

| # | Change ID | Status | Effort |
|---|---|---|---|
| 1 | `add-settings-form-cache` | DONE | ~80 LOC pure utility |
| 2 | `rewrite-use-settings-hook` | DONE | ~150 LOC + 4 contract tests |
| 3 | `retire-settings-store-and-bus-source` | DONE | −242 LOC store + 6 lines from main.tsx |
| 4 | `audit-doc-flip-settings-direct` | DONE | doc-only |

---

## 3. Code shape

| Metric | Value |
|---|---|
| Frontend net LOC (this phase) | **~−40** (−242 store + −6 main.tsx − 6 use-settings.ts; +80 form-cache, +150 new use-settings, +75 test) |
| Stores retired | 1 (`settings-store.ts`) — was the LAST Zustand store on a write path for entity data |
| New contract tests | 4 (`settings-form-cache.test.tsx`) → 40/40 |
| Files touched | 6 |

---

## 4. Artifact Quality Summary

| Metric | Value |
|---|---|
| Changes with QA (artifact-refiner) | 0/4 (project hasn't configured refiner) |
| First-pass pass rate | n/a |

Substitute gates: `pnpm test ≥ 36/36` (achieved 40/40), `pnpm build` clean, `git grep useSettingsStore frontend/` empty, `git grep initSettingsRealtimeBridge frontend/` empty.

---

## 5. Technical debt introduced

**None.** The migration:
- Adds a contract test where previously there was none for the settings dirty/save flow.
- Reduces architectural surface (one fewer Zustand store on a write path).
- Documents a reusable pattern (Form-cache) in the audit playbook for future pages with edit/save semantics.

The only residual: `settings-types-meta-store.ts` (25 LOC) intentionally stays — it caches field-schema metadata, not per-row entities. Tagged as a non-entity cache in the audit.

---

## 6. Lessons captured for the knowledge base

1. **`useSyncExternalStore` is the missing primitive between Zustand and module cache.** It gives module-level mutable state React-managed re-renders without pulling in a state-management dependency. For per-namespace ephemeral state (dirty markers, draft form values), this is lighter than a custom store.

2. **Contract preservation enables low-risk retirement of big stores.** Keeping `UseSettingsReturn` byte-identical meant the 3334 LOC `settings-page.tsx` required zero call-site changes. The entire diff lived in the hook + cache + 1 line in main.tsx. A 3334 LOC review surface collapsed to a ~250 LOC review surface.

3. **Conflicts can be synthesised, not stored.** The old store had a `conflicts: Record<string, unknown>` field maintained alongside `dirty`. The new hook derives conflicts via `useMemo(() => diff(dirty, graphValues), …)`. One source of truth, half the state.

4. **Module-level caches don't break SSR or tests.** The `Map<string, DirtyState>` is initialized at import; the `__resetForTests` helper lets vitest restart it between tests. No global-singleton anxiety.

5. **The form-cache pattern is reusable.** Any page with a "type-then-save" workflow (skill editor's dialog, agent memory config) can lift the same cache. Captured in the audit's Direct Migration Playbook.

6. **`change-bus` survived as a thin facade.** Two listeners (`chat-stream-store`, `provider-models-store`) needed the `uar:settings-changed` event signal but didn't need the store. Keeping the bus + moving its emission point to the SSE adapter preserved their contract.

---

## 7. Cross-phase status — entity migration scoreboard

After this phase, the bridge-pattern transition is essentially **done** for the admin surface:

| Entity | Status (post-phase) |
|---|---|
| Provider | direct ✅ |
| Agent | direct ✅ |
| Model | direct ✅ |
| Skill | direct ✅ |
| Memory | direct ✅ |
| CompilerSession | direct ✅ |
| KnowledgeBase + Document | direct (compat hook; visual redesign deferred) |
| **Setting** | **direct ✅ (this phase)** |
| Tool | bridged (deferred — `tool-mcp-status-push-channels`) |
| Thread | pending (sidebar not wired) |
| ApiKey | non-realtime (intentional) |
| McpStatus | non-realtime (polling) |

`useGraphBridge` has exactly ONE consumer remaining (`use-tools-discovery.ts`). The next phase finishes the bridge story.

---

## 8. Recommended next phase

`tool-mcp-status-push-channels` — wire SSE push for the Tool topic, migrate Tool to direct, then delete `use-graph-bridge.ts` entirely. After that, the only entities outside the direct pattern are intentional non-realtime ones (ApiKey, McpStatus) and the pending Thread topic wiring.

Secondary candidates from the waypoint:
- `knowledge-page-aesthetic-pass` — visual-only follow-up on the 782 LOC page
- `ci-frontend-tests` — wire 40/40 + grep gates into CI
- Browser smoke walkthrough — still owed for two phases now

---

## 9. Progress signal

Reflection complete. Advance with `/kbd-new-phase tool-mcp-status-push-channels`.
