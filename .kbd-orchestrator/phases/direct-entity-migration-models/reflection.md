# Reflection — `direct-entity-migration-models`

**Date:** 2026-05-27
**Tool:** claude-code (`/kbd-reflect`)
**Backend:** OpenSpec
**Status:** execute_complete_with_deferrals → reflect_complete

---

## 1. Goal achievement

**Phase goal:** Migrate every remaining bridged entity (originally 7 — `Model`, `Skill`, `KnowledgeBase`+`Document`, `Setting`, `Tool`, `Memory`, `CompilerSession`) to the direct pattern (Provider/Agent playbook) AND apply a retro-futuristic terminal aesthetic per-page using Anthropic's `frontend-design` + `impeccable` + `ui-ux-pro-max` skills.

Scope adjusted during planning: **Tool migration deferred** to `tool-mcp-status-push-channels` (Q3 decision). So the target set this phase was **6 entities**.

| # | DoD criterion | Verdict | Evidence |
|---|---|---|---|
| B1 | `Model` direct, store retired | **MET** | `models-browse-store.ts` deleted; `useModels` hook; terminal aesthetic |
| B2 | `Skill` direct, store retired | **MET** | `skills-admin-store.ts` deleted; mutations via `optimisticUpsert/Remove`; header+list aesthetic |
| B3 | `KnowledgeBase` + `Document` direct | **PARTIAL** | `knowledge-admin-store.ts` deleted; data layer migrated via `useKnowledgePage` compat hook; deeper visual redesign deferred |
| B4 | `Setting` direct | **PARTIAL** | Reads migrated to graph; mutations + `settings-store` retirement deferred (3334 LOC page risk) |
| B5 | `Tool` direct | **DEFERRED-BY-DESIGN** | Q3 decision: wait for push channel |
| B6 | `Memory` direct, store retired | **MET** | `memory-admin-store.ts` deleted; full terminal aesthetic; bulk-delete snapshot loop kept (transactional contract) |
| B7 | `CompilerSession` direct, store retired | **MET** | `compiler-sessions-store.ts` deleted; full terminal aesthetic; also established shared admin components |
| B8 | `useGraphBridge` deletable | **PARTIAL** | Marked `@deprecated`; only Tools consumes it; deletion scheduled for next phase |
| B9 | Audit doc reflects new state | **MET** | All 6 rows flipped; bridge section moved to "Historical" appendix; Direct migration playbook is now canonical |
| B10 | `pnpm test ≥ 36/36` after every change | **MET** | 36/36 green throughout |
| B11 | `pnpm build` clean after every change | **MET** | All changes built cleanly |
| B12 | UI passes per redesigned page (terminal aesthetic) | **PARTIAL** | Models, Compiler, Memory: full pass. Skills: chrome + list. Knowledge: deferred. Settings: deferred. |
| B13 | Playwright screenshots per redesigned page | **NOT MET** | Playwright config already in tree but per-page captures need a live dev server; no autonomous capture this session |

**Goal achievement: ~70% — MET on the data-layer migration spine; PARTIAL on aesthetic completion for the two heaviest pages; NOT MET on screenshot regression (deferred, not failed).**

---

## 2. Delivered changes

12 OpenSpec changes scaffolded; 9 DONE, 2 PARTIAL, 1 DEFERRED.

| # | Change ID | Status | Notes |
|---|---|---|---|
| 1 | `setup-design-skills-and-playwright` | PARTIAL | aesthetic spec + CSS tokens + admin theme toggle shipped; `impeccable` + `ui-ux-pro-max` plugin installs require interactive `/plugin install` (user task) |
| 2 | `add-models-fetcher-and-hook` | DONE | scaffolding |
| 3 | `migrate-models-page-direct-and-redesign` | DONE | full rewrite, terminal aesthetic, store retired |
| 4 | `migrate-compiler-page-direct-and-redesign` | DONE | also authored shared `LoadingCursor`, `EmptyFrame`, `ErrorBar` |
| 5 | `add-memory-fetcher-and-hook` | DONE | scaffolding |
| 6 | `migrate-memory-page-direct-and-redesign` | DONE | full rewrite with terminal aesthetic across header, filters, table, detail panel |
| 7 | `migrate-skills-page-direct-and-redesign` | DONE | data migration + header/list aesthetic; dialogs retain ShadCN |
| 8 | `migrate-knowledge-page-direct-and-redesign` | PARTIAL | compat hook (`useKnowledgePage`); page chrome unchanged |
| 9 | `add-settings-fetcher-and-hook` | DONE | scaffolding |
| 10 | `migrate-settings-reads-transitional` | DONE | `useSettings` reads from graph; writes still on store |
| 11 | `migrate-settings-mutations-and-retire-stores` | DEFERRED | 3334 LOC page risk too high without dedicated testing pass |
| 12 | `delete-use-graph-bridge` | DONE-as-deprecation | file marked `@deprecated`; Tool still consumes it |

---

## 3. Artifact Quality Summary

| Metric | Value |
| --- | --- |
| Changes with QA (artifact-refiner) | 0/12 (project hasn't configured refiner) |
| First-pass pass rate | n/a |
| Changes requiring refinement | 0 |
| Total refinement iterations | 0 |

Refiner skipped per project convention. The substitute gates were `pnpm test ≥ 36/36`, `pnpm build` clean, and `git grep useGraphBridge frontend/` reducing to a single allowed consumer (Tools).

---

## 4. Code shape

| Metric | Value |
| --- | --- |
| Net LOC delta (frontend) | **−1,652** (+1,021 / −2,673 across 40 files) |
| Stores retired | 5 (`models-browse`, `compiler-sessions`, `memory-admin`, `skills-admin`, `knowledge-admin`) totalling ~447 LOC |
| Bridge consumers in frontend | 8 → 1 (Tool only) |
| New entity-graph hooks | 6 (`useModels`, `useMemory`, `useMemoryStats`, `useCompilerSessions`, `useSettingsEntity`, `useKnowledgePage`) |
| New entity-graph fetchers | 4 (`models`, `memory`, `compiler-sessions`, `settings`) |
| Shared admin components | 3 (`LoadingCursor`, `EmptyFrame`, `ErrorBar`) |

The net subtraction shows the destination pattern is genuinely lighter — each store retirement collapses a ~80–172 LOC Zustand file plus its admin-hook wrapper down to a thin `useEntityList`-based hook.

---

## 5. Technical debt introduced

- **Settings store still alive.** `frontend/src/stores/settings-store.ts` (242 LOC) + `settings-types-meta-store.ts` (25 LOC) + the change-bus continue to own writes. Reads are graph-backed via `useSettings`. Behaviour is unchanged but the architectural invariant ("pages don't import Zustand for entity data") is not yet enforceable for Settings.
- **Knowledge page kept its chrome.** `useKnowledgePage` is a compatibility hook with the legacy API surface; the page still mounts ShadCN primitives unchanged. The aesthetic-direction spec applies; the page just doesn't conform yet.
- **Bulk-delete uses `useGraphStore.getState()` directly.** Memory bulk-delete needs a transactional snapshot of N entities — outside `optimisticRemove`'s single-entity contract. Acceptable but worth a helper.
- **`useGraphBridge` still in the tree** with one consumer (Tools). Marked `@deprecated`.
- **No Playwright screenshots committed.** Visual regression promised in plan but skipped — no autonomous way to run a dev server here. Per-page captures left for next session.
- **No aesthetic checklist run via `impeccable` audit.** The two community design skills weren't installed (`/plugin install` is interactive). Aesthetic work was hand-driven against the spec doc.

---

## 6. Lessons captured for the knowledge base

1. **Compat-hook escape valve for big pages.** When a page is too large to rewrite mid-phase (knowledge-page at 782 LOC, settings-page at 3334 LOC), authoring a thin compat hook that exposes the legacy API surface but routes reads/writes through the new pattern keeps the migration shippable. It's a 95% solution that doesn't block the audit-flip.
2. **Shared aesthetic components arrive on the second migration, not the first.** Models (change 3) inlined patterns; compiler (change 4) extracted them once they had been validated. That order was right — the abstraction emerges from a working concrete case.
3. **Spec-driven aesthetic pinning works.** `docs/admin-aesthetic-spec.md` pinned terminal CRT once; subsequent pages cited the spec rather than re-deciding. The 5 banned fonts list (`Inter|Roboto|Arial|Space Grotesk|system-ui`) became greppable.
4. **Scoped theme via `data-admin-theme` is the right pattern.** Chat retains the existing UAR Dark theme; admin gets terminal. The `useEffect` toggle on the admin page mounts/unmounts the attribute cleanly.
5. **OpenSpec/native KBD ratio.** All 12 changes used OpenSpec scaffolds. The proposal+tasks pair scales — proposals stay 8–15 lines; tasks 10–30 lines. Bigger artifacts add ceremony without traceability gain.
6. **Per-page UI cadence is right for momentum.** Doing data + aesthetic per page (vs. trailing single change) means each PR is reviewable in isolation. The cost is some duplicate aesthetic work where two pages would naturally share a component — but the shared admin components emerging in change 4 absorbed that.
7. **Plugin installs are an interactive boundary.** Claude Code's `/plugin install` is user-driven and not scriptable. Phases that depend on plugin installs should state the dependency, ship the code that doesn't need them, and leave installs as carry-over.
8. **Settings is a phase by itself.** A 3334 LOC page touching 7 namespaces is its own migration; bundling it inside an umbrella phase forces a deferral. Future plan: never schedule >1500 LOC of page migration in a single phase.

---

## 7. Recommended focus for next phase(s)

The waypoint already carries several seeds. Recommended order:

1. **`settings-store-retirement`** (new seed) — finish change-11. Author per-namespace optimistic save helpers and retire `settings-store.ts`. ~M effort but high-leverage: it closes out the only Setting consumer still on a Zustand write path. Dedicated phase so reviewers can focus.
2. **`tool-mcp-status-push-channels`** — wire push for Tool + McpStatus topics. Then migrate Tool to direct pattern. Then **delete `use-graph-bridge.ts`** entirely. This phase finally retires the bridge.
3. **`knowledge-page-aesthetic-pass`** (new seed) — apply terminal aesthetic to the 782 LOC knowledge page. Data layer is already direct; this is pure visual work. Pair with Playwright screenshot capture.
4. **`ci-frontend-tests`** — wire `pnpm test` + `pnpm build` + grep-invariant gates into CI. Cheap, high leverage. Required before any further migrations.
5. **Browser smoke re-walkthrough** for `browser-smoke-providers-and-agents` — still owed from two phases back. Now also covers the 4 newly-migrated pages.
6. **`thread-topic-chat-sidebar`** — wire Thread topic into the chat sidebar consumer; the only entity still marked `pending` after this phase.

---

## 8. Carry-over

| Item | Owner | Notes |
|---|---|---|
| `impeccable` + `ui-ux-pro-max` plugin installs | user | run interactively: `/plugin install impeccable` etc. |
| Playwright per-page screenshots | next session | needs live dev server; `pnpm --filter ./frontend test:e2e --grep @<page>` |
| `settings-store.ts` retirement | follow-up phase | reads done; mutations still on store |
| Knowledge page aesthetic redesign | follow-up phase | data layer already direct |
| Tools migration + bridge deletion | `tool-mcp-status-push-channels` | Tool is the last bridged entity |
| Bulk-delete `optimisticBulkRemove` helper | nice-to-have | encapsulate memory bulk-delete pattern |
| Browser smoke walkthrough | prior phase carry-over | still owed |

---

## 9. Progress signal

Reflection complete. Advance with `/kbd-new-phase`. Recommended next: `settings-store-retirement` (close out the only PARTIAL data-layer migration) or `tool-mcp-status-push-channels` (finishes the bridge story).
