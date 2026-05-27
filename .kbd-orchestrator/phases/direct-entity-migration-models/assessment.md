# Assessment — `direct-entity-migration-models`

**Date:** 2026-05-27
**Tool:** claude-code (`/kbd-assess`)
**Prior phase:** `use-optimistic-patch-helper-extraction` (reflect_complete, 100% MET)
**Scope expansion:** The user requested that this phase cover **every remaining bridged entity** (Models, Skills, Knowledge, Settings, Tools, Memory, CompilerSession) — not just Models — and that the UI rewrites pull in Anthropic's `frontend-design` + community `ui-ux-pro-max` / `impeccable` skills so we don't ship generic AI-slop UI.

---

## 1. Phase goal

Two-part umbrella phase:

**Part A — Direct-entity migrations.** Convert every bridged page (`Model`, `Skill`, `Knowledge`+`Document`, `Setting`, `Tool`, `Memory`, `CompilerSession`) from the `Zustand store + useGraphBridge` interim pattern to the **destination pattern** already used by Providers + Agents:

1. Page reads via `useEntity*` hooks from `@prometheus-ags/prometheus-entity-management`.
2. Mutations call services directly, wrapped in the now-shared `optimisticUpsert` / `optimisticRemove` helpers (`@/lib/realtime/optimistic`).
3. Local UI flags (`saving`, `removing`, `error`) move to `useState`.
4. The Zustand admin store + admin hook is deleted; `git grep` confirms zero references.
5. `docs/migration-stale-data-audit.md` flips the row from `bridged` → `direct`.

Each page is one OpenSpec change; verification gate after each: `pnpm --filter ./frontend test` (≥36/36) + `pnpm --filter ./frontend build` clean + `git grep "useGraphBridge.*<EntityType>"` empty.

**Part B — UI quality pass via design skills.** Where a page is being rewritten anyway, run the rewrite through Anthropic's `frontend-design` skill (already installed at `~/.claude/skills/frontend-design/`) and the community `impeccable` skill in **product mode** (design SERVES the product: dashboards, admin tools), borrowing patterns from `ui-ux-pro-max`. The goal is a coherent visual direction across the admin surface — not "make it pretty" but "make it intentional." Constraint: no scope creep into routing/IA changes; only the visual + interaction layer of pages that are already being touched for Part A.

The realtime-via-SSE substrate is unchanged; the migrations are pure refactors that shorten the data path from `SSE → graph → bridge → store → page` to `SSE → graph → page`.

---

## 2. Current state inventory

### 2.1 Migration status across all entities

| Entity | Topic | Page | Pattern | Store LOC | Page LOC |
|--------|-------|------|---------|-----------|----------|
| `Provider` | `providers` | providers-page | **direct** ✅ | (retired) | 576 |
| `Agent` | `agents` | agents-page + agent-selector | **direct** ✅ | (retired) | 580 |
| `Model` | `models` | models-page | bridged | 41 | 216 |
| `Skill` | `skills` | skills-page | bridged | 102 | 469 |
| `KnowledgeBase` + `Document` | `knowledge_bases`, `knowledge_documents` | knowledge-page | bridged | 172 | 782 |
| `Setting` | `settings` | settings-page | bridged | 242 + 25 (meta) | 3334 |
| `Tool` | (push deferred; graph still receives via REST refresh) | tools-page | bridged | 46 | 115 |
| `Memory` | `memory` | memory-page | bridged | 98 | 451 |
| `CompilerSession` | `compiler_sessions` | compiler-page | bridged | 49 | 59 |
| `ApiKey` | (intentionally none) | auth-page | non-realtime | 60 | 136 |
| `McpStatus` | (push deferred) | runtime-console (health tab) | polled | 41 | 450 |
| `Thread` | `threads` | chat sidebar | pending | — | — |

**Migration target this phase: 7 bridged entities** (rows 3–9). `ApiKey`, `McpStatus`, `Thread` are tracked separately in other waypoint seeds (`tool-mcp-status-push-channels`, etc.) and are explicitly **out of scope**.

### 2.2 Shared infrastructure (all present, no changes needed)

- `frontend/src/lib/realtime/optimistic.ts` — `optimisticUpsert` + `optimisticRemove`.
- `frontend/src/entities/hooks/` — already has `use-agents`, `use-providers`, `use-provider-default`, `use-knowledge`, `use-skills`, `use-tools`. Missing: `use-models`, `use-settings`, `use-memory`, `use-compiler-sessions` (must be added per-entity).
- `frontend/src/entities/fetchers/` — already has `agents`, `providers`, `knowledge`, `skills`, `tools`. Missing: `models`, `settings`, `memory`, `compiler-sessions`.
- `frontend/src/lib/realtime/use-graph-bridge.ts` — to be deleted *only after every consumer is migrated*; not this phase.
- Contract tests (5 in `optimistic-rollback.test.tsx`, 4 in `graph-propagation.test.tsx`, etc.) — already locked; serve as regression gates.

### 2.3 UI skills available locally

| Skill | Installed | Role |
|-------|-----------|------|
| `~/.claude/skills/frontend-design/` | ✅ | Anthropic official; bans Inter/Roboto/Arial/Space Grotesk; pushes a committed aesthetic direction |
| `~/.claude/skills/brand-guidelines/` | ✅ | Project brand pinning |
| `~/.claude/skills/canvas-design/` | ✅ | Layout/spacing primitives |
| `~/.claude/skills/web-artifacts-builder/` | ✅ | Self-contained HTML/CSS artifacts (less relevant for SPA work) |
| `impeccable` (pbakaus marketplace) | ❌ not yet installed | Extends frontend-design with `brand` vs `product` mode |
| `ui-ux-pro-max` (community) | ❌ not yet installed | 50+ UI styles, 97 palettes, 99 UX guidelines |

**Plan-phase decision needed:** install `impeccable` + `ui-ux-pro-max` now, or proceed with the installed Anthropic `frontend-design` alone. Recommendation: install both before Part B work begins — they're additive and quick. If the user prefers to keep the skill surface minimal, `frontend-design` alone is sufficient to avoid generic-AI aesthetics.

### 2.4 Settings page is the elephant

`settings-page.tsx` is **3334 LOC** — roughly 6× the next-largest page. It likely contains many sub-sections that each pull from settings + settings-types-meta. Direct migration will need to be carefully phased to avoid one monolithic OpenSpec change. **Recommendation:** split the settings migration into 2 changes — (1) replace reads, (2) replace mutations — and consider deferring the UI rewrite to a separate phase to keep this one shippable.

---

## 3. Definition of done

| # | Criterion | Verification |
|---|-----------|--------------|
| B1 | `Model`: `useModels` hook + `entities/fetchers/models.ts`; models-page reads via hook; mutations direct; `models-browse-store.ts` deleted; audit row flipped to `direct` | grep + diff |
| B2 | `Skill`: same shape; `skills-admin-store.ts` deleted | grep + diff |
| B3 | `KnowledgeBase` + `Document`: `useKnowledge` + `useDocuments` consumed directly; `knowledge-admin-store.ts` deleted | grep + diff |
| B4 | `Setting`: reads + mutations direct; `settings-store.ts` + `settings-types-meta-store.ts` deleted | grep + diff |
| B5 | `Tool`: `useTools` consumed directly; `tools-discovery-store.ts` deleted | grep + diff |
| B6 | `Memory`: reads + mutations direct; `memory-admin-store.ts` deleted | grep + diff |
| B7 | `CompilerSession`: reads + mutations direct; `compiler-sessions-store.ts` deleted | grep + diff |
| B8 | `useGraphBridge` is no longer imported by any remaining hook/page; the helper file itself is deleted | `git grep useGraphBridge` empty |
| B9 | `docs/migration-stale-data-audit.md` reflects the new state (7 rows flipped to `direct`; `bridge pattern` section moved to "historical") | file diff |
| B10 | `pnpm --filter ./frontend test` ≥ 36/36 after every change (no regressions; new tests welcome) | output |
| B11 | `pnpm --filter ./frontend build` clean after every change | output |
| B12 | UI rewrite passes: each touched page commits to a single aesthetic direction; no Inter/Roboto/Arial/Space Grotesk in newly authored CSS; design tokens use existing `app.css` variables; loading + empty + error states all designed (not default skeleton) | manual review using `frontend-design` skill heuristics |
| B13 | At least one screenshot (or rendered HTML snapshot) per redesigned page lives at `.kbd-orchestrator/phases/direct-entity-migration-models/screenshots/<page>.png` for future-phase regression checks | files present |

---

## 4. Gap analysis

### 4.1 Direct migration

- **All 7 entities are already realtime-fed** via the SSE adapter → graph bridge. The migrations are mechanical: add fetcher/hook (where missing), rewire page reads, swap mutations to optimistic helpers, delete store. No backend changes needed.
- **Tool topic has no push channel yet** — bridge currently triggers refetch via REST. After direct migration, `useTools()` reads from the graph; the graph stays fresh because the existing `tools-discovery` REST refresh runs on bridge events. *Caveat:* once we delete the bridge, we lose the auto-refresh trigger for Tools. Either (a) keep the Tool bridge in place until the push channel ships in a later phase, or (b) move the refresh trigger into a lightweight `useEffect` polling shim. Plan-phase decision.
- **Settings** has a meta-store (`settings-types-meta-store.ts`) backing field schemas — *not* a per-row entity. It should stay as a one-shot REST cache or migrate to its own non-graph hook; do not try to graph-ify it.

### 4.2 UI quality

- The admin surface today uses Tailwind utility classes + ShadCN-style primitives in `frontend/src/components/ui/`. The `frontend-design` skill will likely steer us toward a more committed typography + color direction. Risk: divergence from the existing chat surface, which has its own aesthetic. Mitigation: scope this phase to **admin pages only**; do not touch the chat or runtime-console UI.
- Loading skeletons today are generic. Empty states are sparse. Error banners are inconsistent (some pages, not others). The redesign should standardize these three states across all 7 pages.
- A11y is unaudited. Keyboard nav + ARIA labels should be a checklist item in plan-phase, not deferred.

### 4.3 Risk areas

- **Settings page** (3334 LOC) is the only realistic candidate for slipping the schedule. Sub-splitting is mandatory.
- **Test regression**: contract tests don't cover Settings, Memory, Compiler, Tools. Authoring new contract tests for each is out of scope (separate phase: `contract-tests-second-pass`), but each migration must at minimum *not break* the 36/36.
- **Bridge deletion timing**: `useGraphBridge` is shared infrastructure; deleting it before the last consumer migrates would brick the previous one. Sequencing must put bridge deletion **dead last** (after all 7 entities are direct).

---

## 5. Sequencing recommendation

12 changes, ordered easy → hard, with the heaviest Settings work split:

1. `add-models-fetcher-and-hook` — net-additive scaffolding for Model entity.
2. `migrate-models-page-to-direct` — reads + mutations; delete `models-browse-store.ts`.
3. `migrate-tools-page-to-direct` — smallest page; delete `tools-discovery-store.ts` (and accept the polling-shim tradeoff or keep the REST refresh trigger via `useEffect`).
4. `migrate-compiler-page-to-direct` — small page + small store.
5. `migrate-memory-page-to-direct` — medium page; needs `useMemory` hook + fetcher first.
6. `migrate-skills-page-to-direct` — already has `useSkills`; mostly mutation rewires.
7. `migrate-knowledge-page-to-direct` — the largest non-settings page; document optimistic patches are non-trivial.
8. `add-settings-fetcher-and-hook` — scaffolding only.
9. `migrate-settings-reads` — half of the settings page; mutations stay on the old store temporarily (no double-source: the store keeps writing through the existing path; the page reads from the graph). This is a transitional commit.
10. `migrate-settings-mutations-and-retire-store` — the other half; delete `settings-store.ts` + `settings-types-meta-store.ts`.
11. `delete-use-graph-bridge` — after all consumers are direct.
12. `ui-pass-admin-pages` — single change running `frontend-design`-driven polish across the 7 redesigned pages: typography commitment, palette commitment, loading/empty/error state standardization, a11y pass.

Each change runs the pnpm test + build gate. Audit doc is updated incrementally.

---

## 6. Open questions for the user before planning

1. **Skill installation.** Install `impeccable` (`pbakaus-impeccable` marketplace) and `ui-ux-pro-max` (community marketplace) before the UI pass, or proceed with only `frontend-design` (already installed)?
2. **Aesthetic direction.** Pick one (or let me propose during planning):
   - editorial/magazine (refined, generous spacing, serif display)
   - brutalist/raw (monospace dominance, harsh edges)
   - retro-futuristic (CRT/terminal callbacks fit the UAR brand)
   - keep-current (don't redesign — just do the data migration)
3. **Tool bridge deletion.** When we cut the `useGraphBridge` for Tools, options are: (a) drop in a 15-second `setInterval` polling shim until the push channel ships, (b) defer the Tools direct migration to the `tool-mcp-status-push-channels` phase, or (c) accept staler tool discovery until push is wired.
4. **UI pass timing.** Run the UI rewrite **per page** (every migration change includes its own polish) or as a single trailing `ui-pass-admin-pages` change at the end? Per-page is safer (smaller diffs); trailing is cheaper (one aesthetic decision, applied consistently).
5. **Settings split.** Confirm the 2-change settings split (reads, then mutations) is acceptable, or do you want it as one big change?
6. **Screenshot regression.** Worth wiring? Adds a Playwright/headless-chromium dep but enables visual regression on future redesigns.

---

## 7. Progress signal

Assessment complete. Awaiting answers to §6 before invoking `/kbd-plan direct-entity-migration-models`.

---

## Sources

- [Frontend Design | Claude Code Skills](https://claudemarketplaces.com/skills/anthropics/skills/frontend-design)
- [Impeccable: The missing upgrade to Anthropic's frontend skill](https://impeccable.style/)
- [Top 8 Claude Skills for UI/UX Engineers — Snyk](https://snyk.io/articles/top-claude-skills-ui-ux-engineers/)
- [The 18 Best Claude Code Skills for UI/UX Design](https://pasqualepillitteri.it/en/news/576/claude-code-skills-design-uiux-guide)
- [Best Claude Code Skills to Try in 2026 — Firecrawl](https://www.firecrawl.dev/blog/best-claude-code-skills)
- [Claude Code UI/UX Design Skills Compared — Software Thug](https://www.softwarethug.com/posts/claude-code-ui-design-skills-compared/)
- [impeccable Marketplace — Claude Code Plugin Hub](https://www.claudepluginhub.com/marketplaces/pbakaus-impeccable)
- [Introducing Claude Design by Anthropic Labs](https://www.anthropic.com/news/claude-design-anthropic-labs)
