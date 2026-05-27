# Plan — `direct-entity-migration-models`

**Date:** 2026-05-27
**Tool:** claude-code (`/kbd-plan`)
**Backend:** OpenSpec (detected at `openspec/`)
**Assessment input:** `.kbd-orchestrator/phases/direct-entity-migration-models/assessment.md`
**Scope:** 6 entities (Tools deferred) — Model, Skill, KnowledgeBase+Document, Setting, Memory, CompilerSession — plus per-page UI rewrite, Playwright screenshot harness, design-skill installs.

---

## Decisions locked (from /kbd-assess answers)

| # | Decision |
|---|----------|
| Q1 | Install `impeccable` + `ui-ux-pro-max` skills; combine with already-installed `frontend-design` |
| Q2 | Aesthetic: **retro-futuristic / terminal** — CRT callbacks, mono display, accent green/amber, scanline textures |
| Q3 | Tool bridge stays put; Tools migration deferred to `tool-mcp-status-push-channels` |
| Q4 | UI pass cadence: **per-page** — each migration change includes its polish |
| Q5 | Settings split: **2 changes** (reads first, then mutations + store retirement) |
| Q6 | Screenshot regression: **Playwright headless** per redesigned page → `.kbd-orchestrator/phases/direct-entity-migration-models/screenshots/<page>.png` |

---

## Aesthetic Pin (the one decision per phase, not per page)

To honour Q4 (per-page polish) without re-deciding the aesthetic 6 times, this plan pins a single visual contract that every page MUST satisfy. The contract goes into `docs/admin-aesthetic-spec.md` in change-001 and is the canonical source for change-003 onward.

### Visual contract — retro-futuristic / terminal

- **Display font:** `JetBrains Mono` or `Berkeley Mono` (mono-display). NOT Inter / Roboto / Arial / Space Grotesk.
- **Body font:** same family for atmosphere consistency. Sentence-case headlines, lowercase chips.
- **Palette:**
  - bg: `--terminal-bg` near-black (`#0a0d0c`)
  - fg: `--terminal-fg` cool gray (`#c8d4cf`)
  - accent-primary: phosphor green (`#7fffa1`)
  - accent-warning: amber CRT (`#ffb347`)
  - accent-error: signal red (`#ff5d5d`)
  - lines: thin (`1px solid color-mix(in oklab, var(--terminal-fg) 18%, transparent)`)
- **Surface texture:** subtle scanline overlay via repeating linear-gradient at 2px, 6% opacity; optional grain SVG mask. Solid colors only where data legibility wins (tables, form fields).
- **Density:** 13 px body, 12 px chips, 20 px headlines. Generous left-aligned forms over centered modals.
- **Loading state:** flicker-cursor placeholder, not a generic skeleton. Use `▍` glyph blinking at 600 ms.
- **Empty state:** ASCII art frame + a single-sentence prompt + one action button.
- **Error state:** red bar pinned top of card, monospaced error code prefix.
- **Motion:** all transitions ≤180 ms ease-out; no fade-throughs. Page transitions: 4 px upward slide + opacity. NO purple gradients.
- **A11y:** every interactive element keyboard-focusable; focus ring is 2 px solid phosphor-green at 100% opacity (no outline:none).

The contract is small enough to grep in PR review: search for banned fonts in newly-touched CSS, search for `outline: none`, search for `linear-gradient.*purple`.

---

## Ordered change list (12 changes)

| # | OpenSpec change ID | Title | Recommended agent | Verification |
|---|---|---|---|---|
| 1 | `setup-design-skills-and-playwright` | Install impeccable + ui-ux-pro-max plugins; add `@playwright/test`; author `docs/admin-aesthetic-spec.md`; create `screenshots/` dir | claude (small ops) | `pnpm playwright --version` succeeds; `~/.claude/plugins/installed_plugins.json` contains the two new entries |
| 2 | `add-models-fetcher-and-hook` | `frontend/src/entities/fetchers/models.ts` + `frontend/src/entities/hooks/use-models.ts` | claude | `pnpm test` 36/36 stays green; new exports compile |
| 3 | `migrate-models-page-direct-and-redesign` | Switch reads to `useModels`; direct mutations via `optimisticUpsert/Remove`; delete `models-browse-store.ts`; apply aesthetic contract; Playwright screenshot | claude | grep + diff + screenshot present + 36/36 |
| 4 | `migrate-compiler-page-direct-and-redesign` | smallest page; redesign + retire `compiler-sessions-store.ts` | claude | same gate |
| 5 | `add-memory-fetcher-and-hook` | scaffolding for Memory | claude | compile + tests |
| 6 | `migrate-memory-page-direct-and-redesign` | medium page; retire `memory-admin-store.ts` | claude | grep + diff + screenshot + 36/36 |
| 7 | `migrate-skills-page-direct-and-redesign` | `useSkills` exists already; rewire reads, swap mutations, retire `skills-admin-store.ts` | claude | grep + diff + screenshot + 36/36 |
| 8 | `migrate-knowledge-page-direct-and-redesign` | largest non-settings page; retire `knowledge-admin-store.ts` | claude | grep + diff + screenshot + 36/36 |
| 9 | `add-settings-fetcher-and-hook` | scaffolding for Setting; do not touch `settings-types-meta-store.ts` (stays as one-shot REST cache) | claude | compile + tests |
| 10 | `migrate-settings-reads-transitional` | Settings page reads from graph; mutations still flow through the old store (no double-source) | claude | manual: open settings, edit a field, refresh, value persists; 36/36 |
| 11 | `migrate-settings-mutations-and-retire-stores` | Mutations direct via optimistic helpers; delete `settings-store.ts`; apply aesthetic contract; Playwright screenshot | claude | grep + diff + screenshot + 36/36 |
| 12 | `delete-use-graph-bridge` | Final cleanup: `git grep useGraphBridge` empty; delete `frontend/src/lib/realtime/use-graph-bridge.ts`; flip all 6 rows in `docs/migration-stale-data-audit.md` from `bridged` → `direct`; bridge section moved to a "historical" appendix | claude | `git grep useGraphBridge frontend/` empty |

Each change runs `pnpm --filter ./frontend test` and `pnpm --filter ./frontend build`. Aesthetic gate runs additionally on changes 3, 4, 6, 7, 8, 11.

---

## Per-change synopsis

### 1. `setup-design-skills-and-playwright`

- Install `pbakaus-impeccable` and `ui-ux-pro-max` marketplaces/plugins via `/plugin marketplace add` + `/plugin install`.
- Add `@playwright/test` as a frontend devDependency.
- Add scripts: `pnpm --filter ./frontend test:visual` runs `playwright test --grep @visual`; `pnpm --filter ./frontend screenshot:<page>` captures a single page.
- Author `docs/admin-aesthetic-spec.md` containing the visual contract above plus a token table referenced by `frontend/src/app.css`.
- Create empty `.kbd-orchestrator/phases/direct-entity-migration-models/screenshots/` directory.
- Add CSS tokens (`--terminal-bg`, `--terminal-fg`, `--phosphor`, `--amber`, `--signal-red`) into `frontend/src/app.css` under a `:root[data-admin-theme="terminal"]` selector so they're scoped to admin views.

### 2. `add-models-fetcher-and-hook`

```ts
// frontend/src/entities/fetchers/models.ts
export async function loadModelsIntoGraph(): Promise<void> {
  const res = await fetch("/api/admin/models");
  const rows = (await res.json()) as ModelEntity[];
  const graph = useGraphStore.getState();
  rows.forEach((m) => graph.upsertEntity("Model", m.id, m));
}

// frontend/src/entities/hooks/use-models.ts
export function useModels() {
  return useEntityList<ModelEntity>("Model");
}
```

### 3. `migrate-models-page-direct-and-redesign`

- Swap `useModelsBrowse()` → `useModels()` at every read site.
- Mutations (refresh, set-favorite) wrap `optimisticUpsert("Model", id, patch, () => api.fn(...))`.
- Delete `frontend/src/stores/models-browse-store.ts`.
- Apply the aesthetic contract: replace Tailwind `bg-gray-…` defaults with terminal tokens; mono headline; flicker-cursor loading; ASCII empty state.
- Capture Playwright PNG at `.kbd-orchestrator/phases/direct-entity-migration-models/screenshots/models-page.png` (1440×900).

### 4. `migrate-compiler-page-direct-and-redesign`

Tiny page (59 LOC) — quick win. Same playbook as models. Likely also the place to standardize the empty-state ASCII frame component, since it'll be reused.

### 5. `add-memory-fetcher-and-hook`

Mirrors change-2; `MemoryEntity` shape comes from existing `entities/types.ts`.

### 6. `migrate-memory-page-direct-and-redesign`

Standard playbook. Memory page has search affordances — keep them; just rewire the data source.

### 7. `migrate-skills-page-direct-and-redesign`

`useSkills` already exists. Just rewire reads, retire the admin store, redesign. The optimistic toggle helper from earlier phases should now go through `optimisticUpsert` for consistency.

### 8. `migrate-knowledge-page-direct-and-redesign`

Largest non-Settings page (782 LOC). Has 2 entity types (`KnowledgeBase`, `Document`). Document upload mutation needs care — the optimistic insert should mark `status: "uploading"` and let the SSE event update to `ready` / `failed`.

### 9. `add-settings-fetcher-and-hook`

Scaffolding only. `settings-types-meta-store.ts` is NOT migrated — it serves field-schema metadata, not per-row entities, and stays as a one-shot REST cache.

### 10. `migrate-settings-reads-transitional`

The page reads from `useSettings()`. The store still owns writes. This is the deliberately-transitional commit so a regression here is bisectable. Verify: edit a field; save; refresh; value persists.

### 11. `migrate-settings-mutations-and-retire-stores`

Switch mutations to direct service calls + `optimisticUpsert`. Delete `settings-store.ts`. Apply aesthetic contract. This is the heaviest change in the phase; budget for review time.

### 12. `delete-use-graph-bridge`

After every consumer migrates, the bridge has zero callers. Delete the file. Flip 6 audit rows. Move the bridge section in the audit to a "Historical: the bridge pattern" appendix for context.

---

## Aesthetic gate (changes 3, 4, 6, 7, 8, 11)

Each redesign change must pass this checklist before its OpenSpec proposal is archived:

- [ ] No `font-family.*(Inter|Roboto|Arial|Space Grotesk)` in newly authored CSS.
- [ ] No `outline:\s*none` in newly authored CSS.
- [ ] No `linear-gradient.*purple` anywhere in the page's tree.
- [ ] Loading state uses the flicker-cursor component, not a generic skeleton.
- [ ] Empty state uses the ASCII frame component.
- [ ] Error state uses the monospaced-error-code prefix.
- [ ] Keyboard tab order verified manually (or via Playwright `keyboard.press("Tab")` loop).
- [ ] Focus ring is visible at 2 px phosphor-green on every interactive element.
- [ ] Playwright screenshot committed under `screenshots/`.
- [ ] `impeccable` skill's product-mode `audit` command run; output stored at `.kbd-orchestrator/phases/direct-entity-migration-models/audits/<page>.md`.

---

## Verification matrix

| Gate | Where | When |
|------|-------|------|
| `pnpm --filter ./frontend test` ≥ 36/36 | every change | always |
| `pnpm --filter ./frontend build` clean | every change | always |
| `git grep useGraphBridge frontend/` empty | change-12 only | end of phase |
| `git grep useGraphStore.getState frontend/src/admin/pages` empty | preserved from prior phase | every change |
| Playwright screenshot present | changes 3, 4, 6, 7, 8, 11 | per change |
| Aesthetic checklist | changes 3, 4, 6, 7, 8, 11 | per change |
| `docs/migration-stale-data-audit.md` row flipped | changes 3, 4, 6, 7, 8, 11 | per change |
| Audit doc moves bridge to "historical" appendix | change-12 | end of phase |

---

## OpenSpec change scaffolding

12 directories already exist under `openspec/changes/` for some of these IDs from prior planning. The plan-phase will (re-)scaffold any missing `proposal.md` + `tasks.md`. Each tasks file follows the standard structure: Imports, Reads, Mutations, Aesthetic, Verification.

---

## Risk register

| Risk | Mitigation |
|------|------------|
| Aesthetic drift across 6 redesigns | Visual contract pinned in change-1 as a doc; aesthetic gate checklist on every redesign change |
| Settings rewrite balloons | 2-change split + explicit "transitional" commit between them |
| Playwright slows CI | `test:visual` is a separate script; not wired into the default `pnpm test` gate yet |
| `impeccable` audit produces noisy "fix me" lists | Audits are advisory, not blocking; checklist is the gate |
| Tools deferral causes confusion | Audit doc explicitly retains Tools row as `bridged` with a forward link to the push-channel phase |
| Memory page search regression | Manual search smoke required during change-6 verification |

---

## Estimated effort

| Change | Effort |
|--------|--------|
| 1 (setup) | S |
| 2, 5, 9 (scaffolds) | XS each |
| 3, 4 (models, compiler) | S each |
| 6, 7 (memory, skills) | M each |
| 8 (knowledge) | L |
| 10 (settings reads) | M |
| 11 (settings mutations + redesign) | XL |
| 12 (bridge delete + audit) | S |

Total estimate: ~3–4 focused sessions.

---

## Next step

`/kbd-execute direct-entity-migration-models` — starts at change-1 (`setup-design-skills-and-playwright`).
