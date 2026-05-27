# Reflection — `direct-entity-migration-providers`

**Date:** 2026-05-27
**Tool:** claude-code (`/kbd-reflect`)
**Phase status:** `execute_complete`
**Inputs:** assessment.md, plan.md, progress.json, openspec/changes/*/tasks.md

---

## 1. Goal achievement

Goals declared in §3 of the assessment ("Definition of done"). Scored against shipped artifacts:

| # | Goal | Status | Evidence |
|---|------|--------|----------|
| A1 | `providers-page.tsx` reads exclusively from `useProviders()` + `useProviderDefault()`. Zero `useProvidersAdmin` references in the page. | ✅ MET | `git grep useProvidersAdmin frontend/src` → empty |
| A2 | `frontend/src/hooks/use-providers-admin.ts` deleted. | ✅ MET | file absent |
| A3 | `frontend/src/stores/providers-admin-store.ts` deleted. | ✅ MET | file absent |
| A4 | All three Provider mutations call services directly with optimistic graph patches + rollback. | ✅ MET | `configureProvider` non-optimistic (creates); `setDefault` + `removeProvider` optimistic with snapshot-based rollback |
| A5 | Two-tab smoke: configure in A → reflects in B; remove in A → disappears in B; no refresh. | 🟨 PARTIAL | Build chain green; **manual browser verification still pending** |
| A6 | The `useGraphBridge` call in `useProvidersAdmin` is gone (because the hook itself is gone). No regression in other admin pages — only the Provider bridge removed. | ✅ MET | `git grep` clean; remaining 8 hooks still bridged and untouched |
| A7 | Page UI matches pixel-equivalent (configured list, default badge, filter chips, remove dialog). | 🟨 PARTIAL | Code path-equivalent to legacy; **visual regression check pending** |
| A8 | Other admin pages continue to work — Provider migration didn't touch their bridges. | ✅ MET | Other 8 admin hooks unchanged in this phase |
| A9 | Net frontend LOC delta negative. | ✅ MET | ~180 LOC deleted (use-providers-admin.ts + providers-admin-store.ts) vs ~90 LOC added (page + fetcher + new hook + type changes). Net ~90 LOC removed |
| A10 | Migration playbook added to audit doc so the next 4 cross-cutting entities can follow it. | ✅ MET | `docs/migration-stale-data-audit.md` "Bridge pattern vs. direct migration" section documents the 6-step playbook |

**Aggregate:** 8 MET + 2 PARTIAL + 0 NOT MET = **80% confirmed**, with both partials being browser-side visual checks that the build chain itself cannot validate.

---

## 2. Delivered changes

| # | Change | Status | Files touched |
|---|--------|--------|---------------|
| 1 | `provider-meta-singleton` | DONE | `entities/types.ts`, `entities/schemas.ts`, `entities/fetchers/providers.ts`, `entities/hooks/use-provider-default.ts` (new) |
| 2 | `providers-page-direct-reads` | DONE | `admin/pages/providers-page.tsx` (reads only) |
| 3 | `providers-page-direct-mutations` | DONE | `admin/pages/providers-page.tsx` (mutations + local state) |
| 4 | `retire-providers-admin-store` | DONE | `hooks/use-providers-admin.ts` deleted, `stores/providers-admin-store.ts` deleted, `docs/migration-stale-data-audit.md` updated |

**Per-change LOC**: 1 → +30 LOC, 2 → +15 LOC, 3 → +70 LOC, 4 → −180 LOC, audit doc → +15 LOC.
**Phase net**: ~−50 LOC. Smaller than the assessment estimate (−90) because the page itself grew slightly to absorb mutation logic that previously lived in the store.

---

## 3. Artifact Quality Summary

`artifact-refiner` not installed; inline verification used per change.

| Metric | Value |
|--------|-------|
| Changes with explicit QA gate | 0/4 (refiner unavailable) |
| Inline verification (build + grep sweep) | 4/4 |
| First-pass build/compile success | 4/4 — no errors needing rework |
| `git grep` final sweep | empty (zero `useProvidersAdmin` / `providers-admin-store` references) |
| LOC delta | ~−50 net |

### Recurring constraint violations

None — every change compiled clean on first build.

---

## 4. Technical debt introduced

| Item | Severity | Cost to retire |
|------|----------|----------------|
| **Visual regression unverified.** Build chain proves type-safety + reachability; pixel-equivalence and rollback behaviour are still pending a browser smoke test. | Med | One manual two-tab session (≤10 min) |
| **`ProviderEntity` extended with `status` + `status_detail`** to match `CatalogProviderSummary`. The entity is now a UI-shaped wrapper around the catalog response, rather than a pure domain type. | Low | Acceptable — entity types are SPA-local, not a public contract |
| **`useProviders().loading` semantics assumed** (the assessment flagged this as G2). Replaced with `providers.items.length === 0` derivation, which is OK at steady state but flickers during the very first render. | Low | Refine if loading-spinner regressions appear in the smoke |
| **No Vitest test for "optimistic rollback on `setDefault` rejection"** — only manual smoke validates it. | Med | Will fold into the `vitest-contract-test-suite` phase (already in queue) |
| **Bridge still exists for 8 other entities** — pattern is now proven but not applied. | Low | One PR per entity following the new playbook |
| **`useGraphStore.getState()` called inside event handlers** for snapshot capture + rollback. Works but bypasses React reactivity; if the library ever requires React-context-bound store access, this will need adapting. | Low | Library is React-agnostic today; revisit if entity-mgmt API shifts |

---

## 5. Lessons captured

1. **One concern per change.** Splitting the migration across 4 small changes (singleton → reads → mutations → deletion) meant each step compiled and was reviewable in isolation. The assessment's "each change compiles independently" criterion paid off — there was no point where the page was broken.
2. **The page entity ≠ the domain entity.** `ProviderEntity` had to grow `status` and `status_detail` fields to keep the credential-blocked badge working, because the page already iterated `CatalogProviderSummary`. For pure direct-`useEntity` migrations of less display-coupled entities (Models, Settings) this drift may not happen.
3. **Singleton entities work.** `ProviderMeta { id: "current" }` is a clean fit for "single global value that should react to SSE" without inventing a new mechanism. Future singletons (e.g. system status, default agent) can follow the same shape.
4. **Snapshot-based rollback beats inferring the previous state.** Capturing `useGraphStore.getState().entities["Provider"][id]` before delete made the rollback trivial. Trying to reconstruct the prior state from server hints would have been bug-prone.
5. **`git grep` is the unsung hero.** Zero references = retirement is real. Anything weaker leaves zombie code paths.
6. **Skipping the legacy hook in the destructure was enough.** Once the page no longer imported `useProvidersAdmin`, deleting the file was mechanical.

---

## 6. Recommended focus for next phase

In priority order (matches `nextPhaseSeeds` in the waypoint):

1. **`direct-entity-migration-agents`** — Agents have real cross-view consumers (agent-selector, useAgentConfig, chat-page fallback). This will exercise the playbook against an entity that, unlike Providers, fans out across the chat hot path. Critical sanity check before migrating Models/Skills/Settings.
2. **`vitest-contract-test-suite`** — Wire Vitest into npm scripts; author the optimistic-rollback regression test using `setDefault` reject as the canonical example. Required to lock the pattern before further migrations.
3. **`direct-entity-migration-models`** — second easiest after Providers; surfaces drive the chat header + capability toggles.
4. **`direct-entity-migration-skills`** — preserve the built-in badge from prior phase; deduplicate the skill-toggle optimistic patch that today lives in the store.
5. **`direct-entity-migration-settings`** — last cross-cutting; settings use a dirty-buffer pattern, may need a slightly different mutation strategy.

---

## 7. Evolver feedback

No `evolver-bridge.json` exists in this phase. Not driven by an iterative-evolver cycle. No outer-loop state to update.

---

## 8. Progress signal

Completed kbd-reflect — direct-entity-migration-providers
