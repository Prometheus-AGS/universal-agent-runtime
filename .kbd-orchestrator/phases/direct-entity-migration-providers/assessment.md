# Assessment — `direct-entity-migration-providers`

**Date:** 2026-05-27
**Tool:** claude-code (`/kbd-assess`)
**Replaces:** prior bridge-only freshness for the `Provider` entity
**Project source of truth:** `.kbd-orchestrator/`
**Prior phase:** `full-frontend-entity-mgmt-migration` (reflect_complete, 70% goal achievement)

---

## 1. Phase goal

Pilot the **bridge-retirement** pattern with the `Provider` entity. Replace the indirection (`useProvidersAdminStore` ← `useGraphBridge` ← entity graph) with direct `useEntity*` + service-mutation calls so the page reads from the graph and mutates via service APIs without an intermediate Zustand store. Deliver:

1. Same UI behaviour and visual appearance as today.
2. Cross-tab freshness via SSE-fed graph (no degradation vs. bridge).
3. Optimistic mutations on `setDefault` via direct `upsertEntity` patches (matching what the bridge already provides).
4. Net **deletion** of `frontend/src/stores/providers-admin-store.ts` and `frontend/src/hooks/use-providers-admin.ts`.
5. Repeatable migration playbook for the other 4 cross-cutting entities (Agents, Models, Skills, Settings).

---

## 2. Current state inventory

### 2.1 Provider consumers (the surprise — there's only one)

Grep across `frontend/src` for `useProvidersAdmin` returns exactly **one consumer**: [providers-page.tsx](frontend/src/admin/pages/providers-page.tsx). Notable absences:

- `session-config-panel.tsx` — does NOT reference providers today.
- `agent-selector.tsx` — does NOT reference providers (reads agents instead).
- Header chips — render model + agent badges; no provider chip.

The "cross-cutting" categorization from the prior phase's assessment was over-broad for Providers — they're effectively isolated. This makes the migration smaller than expected and a clean template for entities that genuinely DO have cross-view consumers (Models, Agents).

### 2.2 Existing entity scaffolds (already in tree)

- `frontend/src/entities/fetchers/providers.ts` exports `loadProvidersIntoGraph()` which merges `fetchCatalog()` + `fetchConfiguredProviders()` into a `Provider` graph entity (with `configured: boolean` flag).
- `frontend/src/entities/hooks/use-providers.ts` exports `useProviders(searchTerm?, filter?)` returning a `useEntityView<ProviderEntity>` with sort (configured-first, then alphabetical), search across id+display_name, and filter chain.
- `loadProvidersIntoGraph()` is already invoked from `providers-page.tsx:43` on mount.

### 2.3 The redundancy that this phase removes

Two pipelines run in parallel today on the Providers page:

1. **Graph pipeline** (already correct): `loadProvidersIntoGraph()` → `upsertEntity("Provider", …)`. SSE deliveries upsert via the realtime adapter.
2. **Store pipeline** (redundant): `useProvidersAdminStore.load()` fetches the same catalog/configured pair, stores it in Zustand, and the bridge re-runs it on every SSE event.

The page reads from the *store*. We want to read from the *graph*.

### 2.4 Hook surface that needs replacing

`useProvidersAdmin()` returns 12 fields. Mapping each to a direct equivalent:

| Field | Replacement |
|-------|-------------|
| `catalog` | `useProviders()` (returns all `Provider` entities, sorted) |
| `configured` | derived: `catalog.filter(p => p.configured)` |
| `defaultId` | new local hook reading `Configured.default_id` from a small `ProviderMeta` entity, OR keep as a one-shot fetch with SSE invalidation |
| `loading` | `useProviders()` exposes a loading flag |
| `error` | local component state |
| `saving` | local component state |
| `removing` | local component state (string id) |
| `load` | `loadProvidersIntoGraph` (already called on mount) |
| `configureProvider` | `services/providers-api.ts::configureProvider` + optimistic `upsertEntity` |
| `setDefault` | `services/providers-api.ts::setDefaultProvider` + optimistic graph patch on a `ProviderMeta` singleton entity |
| `removeProvider` | `services/providers-api.ts::deleteProvider` + optimistic `removeEntity` |
| `clearError` | local component state setter |

### 2.5 The `defaultId` puzzle

The graph stores per-provider entities; `defaultId` is a single-cell global value backed by `/api/uar/providers` response shape. Options:

- **A. Singleton entity.** Upsert a single `ProviderMeta` entity with id `"current"` whose `default_id` field tracks the value. Cheap and works with the existing graph + SSE machinery. Recommended.
- **B. Derive from per-provider flag.** Add `is_default: boolean` to `Provider` rows. Requires backend response-shape change.

Default to (A). One additional fetch+upsert in `loadProvidersIntoGraph`.

### 2.6 Backend / realtime state

- `Provider` already has its `providers` topic enrolled in `EntityTopic::ALL`. SSE delivery verified working.
- The topic backs the SurrealDB `providers` table — but UAR's `Provider` shape today is a **catalog merge** computed at request time, not a raw row. Live-query on `providers` table won't capture catalog changes (those are immutable per build). Configured providers (the writable subset) WILL stream.
- This is acceptable: the catalog half is static; only the configured half mutates.

---

## 3. Definition of done

| # | Criterion | Verification |
|---|-----------|--------------|
| A1 | `providers-page.tsx` reads exclusively from `useProviders()` + a tiny `useProviderDefault()` hook. Zero `useProvidersAdmin` references in the page. | `git grep useProvidersAdmin frontend/src` returns no matches |
| A2 | `frontend/src/hooks/use-providers-admin.ts` is **deleted**. | file absent |
| A3 | `frontend/src/stores/providers-admin-store.ts` is **deleted**. | file absent |
| A4 | All three Provider mutations (`configureProvider`, `setDefault`, `removeProvider`) call the service directly and apply optimistic graph patches with rollback on failure. | code review + manual rollback smoke |
| A5 | Two-tab smoke: configure a provider in tab A → graph upsert visible in tab B within 200 ms; remove provider in tab A → row disappears in tab B without refresh. | manual |
| A6 | The existing `useGraphBridge` call in `useProvidersAdmin` is gone (because the hook itself is gone). No regression in other admin pages — only the Provider bridge is removed. | code review |
| A7 | Page UI matches current pixel-for-pixel for the configured-list, default-badge, filter chips, and remove-confirm dialog. | screenshot diff |
| A8 | Provider `Settings` (other admin pages) continue to work because their bridges are untouched. | manual sweep |
| A9 | Net frontend LOC delta is negative (store + hook deletions exceed any new code). | git diff stat |
| A10 | New playbook section in [`docs/migration-stale-data-audit.md`](docs/migration-stale-data-audit.md) describes the Provider migration end-to-end so the next 4 cross-cutting entities can follow it. | doc updated |

---

## 4. Gap analysis

| ID | Gap | Severity | Notes |
|----|-----|----------|-------|
| G1 | No `useProviderDefault()` hook today; the page needs to read `defaultId` from somewhere graph-backed. | **High** | Introduce a `ProviderMeta` singleton entity or pass a one-shot value down. (A) above. |
| G2 | `useProviders()` returns a `useEntityView<ProviderEntity>` shape, not the flat array the page currently iterates. | Med | Page renders are minor adjustments — extract `.items` from the view result. |
| G3 | Optimistic mutations need explicit `upsertEntity` / `removeEntity` calls; no `useEntityCRUD` wrapper authored for Provider yet. | Med | Library exports `useEntityCRUD`; can either use it directly or hand-roll patches (~5 lines per mutation). Hand-roll for the pilot; promote to `useEntityCRUD` if a pattern emerges. |
| G4 | The provider page uses `catalog.length` and `configured.length` for the subtitle counts. With graph reads, these become `providers.filter(p => p.configured).length` etc. | Low | Simple derivation. |
| G5 | `clearError` is a no-op once the store is gone — local component state replaces it. | Low | Trivial. |
| G6 | The page currently has 487 LOC. Direct migration WILL keep that count roughly stable (mutations move from store to component, but reads consolidate). | Low | Net delta in this file likely ±20 LOC; the wins are in the deleted files. |
| G7 | No regression test exists for "two tabs, mutation propagates". The audit doc and inline smoke are the only checkpoints today. | Med | Punt to the `vitest-contract-test-suite` phase queued next; do not block this pilot on it. |
| G8 | Bundle size impact unknown. Stores were tree-shakeable; the page now imports `useGraphStore`+`useEntityView` directly (already imported elsewhere). Likely neutral. | Low | Inspect after build. |
| G9 | The `bridge` reference in [`docs/migration-stale-data-audit.md`](docs/migration-stale-data-audit.md) must flip from `bridged` → `direct` for the Provider row when this lands. | Low | Documentation hygiene. |

---

## 5. Sequencing recommendation

1. **G1 — `useProviderDefault()` + `ProviderMeta` singleton.** Tiny hook + fetcher addition. No page change yet.
2. **G2 — providers-page reads from `useProviders()` + `useProviderDefault()`.** Pure render switch; keep mutations going through the old store temporarily so the page is still functional after this step.
3. **Mutations migration.** Move `configureProvider` / `setDefault` / `removeProvider` from the store action calls to direct service calls + optimistic graph patches.
4. **Local UI state.** Move `saving` / `removing` / `error` to component state.
5. **Delete `use-providers-admin.ts` and `providers-admin-store.ts`.** Verify no other consumers (grep).
6. **Update audit doc** + commit.
7. **Manual smoke** (two-tab propagation, remove rollback, default badge instant flip).

Each step compiles independently — bail out at any point without leaving the page broken.

---

## 6. Open questions for the user before planning

1. **`ProviderMeta` singleton (recommended) vs. backend response-shape change to add `is_default` to each Provider row?** Singleton is contained but introduces a new entity type; the response-shape change is more idiomatic but ripples into backend tests + Postgres provider.
2. **Optimistic strategy for `configureProvider`** (the create path): apply the new provider row immediately, mark it as `configured: true`, rollback on failure? Or wait for server confirmation (non-optimistic for creates, matching the global rule)? Defaults say non-optimistic for creates.
3. **Audit-doc update format**: keep the table row's status as one of {bridged, direct, …}, or add a per-row "migration date" column to track when the bridge was retired?
4. **Pilot scope**: stop after Providers ships, evaluate, then plan Agents/Models/Skills/Settings separately — or batch all 5 into a single phase now that the pattern is proven?

---

## 7. Progress signal

Completed kbd-assess — direct-entity-migration-providers
