# Plan — `direct-entity-migration-providers`

**Date:** 2026-05-27
**Tool:** claude-code (`/kbd-plan`)
**Backend:** OpenSpec (detected at `openspec/`)
**Assessment input:** `.kbd-orchestrator/phases/direct-entity-migration-providers/assessment.md`

---

## Decisions locked (defaults applied)

| Q | Answer |
|---|--------|
| Q1 — `defaultId` storage | **`ProviderMeta` singleton** entity (id `"current"`, field `default_id`). Contained; no backend response-shape change. |
| Q2 — optimistic `configureProvider` (create) | **Non-optimistic.** Matches the global "high-frequency only" rule; creates are rare and have higher rejection rates. |
| Q3 — audit-doc format | **Status-only.** No migration-date column; git history is authoritative. |
| Q4 — pilot scope | **Stop after Providers.** Validate the pattern, capture lessons, then plan Agents/Models/Skills/Settings as their own phases. |

---

## Ordered change list (4 changes)

| # | Change ID | Title | Depends on |
|---|-----------|-------|------------|
| 1 | `provider-meta-singleton` | New `ProviderMeta` singleton + `useProviderDefault()` hook; update `loadProvidersIntoGraph()` to upsert it | — |
| 2 | `providers-page-direct-reads` | `providers-page.tsx` reads from `useProviders()` + `useProviderDefault()`; mutations still go through the old store temporarily | 1 |
| 3 | `providers-page-direct-mutations` | Move `configureProvider` / `setDefault` / `removeProvider` to direct service calls with optimistic graph patches (`setDefault` only) | 2 |
| 4 | `retire-providers-admin-store` | Delete `use-providers-admin.ts` and `providers-admin-store.ts`; update audit doc | 3 |

Each change compiles independently; bailing out after any step leaves the page functional.

---

## Per-change synopsis

### 1. `provider-meta-singleton`
- Add `ProviderMeta` to `frontend/src/entities/types.ts`:
  ```ts
  export interface ProviderMetaEntity {
    id: "current";
    default_id: string | null;
  }
  ```
- Register schema in `frontend/src/entities/schemas.ts`: `registerSchema({ type: "ProviderMeta" });`
- Extend `loadProvidersIntoGraph()` to also upsert the singleton:
  ```ts
  upsertEntity("ProviderMeta", "current", { id: "current", default_id: configured.default_id ?? null });
  ```
- Add `frontend/src/entities/hooks/use-provider-default.ts`:
  ```ts
  export function useProviderDefault(): string | null {
    return useGraphStore((s) => (s.entities["ProviderMeta"]?.["current"] as ProviderMetaEntity | undefined)?.default_id ?? null);
  }
  ```
- No page change yet. Acceptance: graph contains one `ProviderMeta` row after page mount.

### 2. `providers-page-direct-reads`
- Swap the page's `useProvidersAdmin()` destructuring so reads come from `useProviders()` + `useProviderDefault()`. Mutations still call through the legacy store hook for safety:
  ```ts
  const providersView = useProviders();
  const defaultId = useProviderDefault();
  const { configureProvider, setDefault, removeProvider, saving, removing, error, clearError } = useProvidersAdmin();
  ```
- Derive `catalog`/`configured` from `providersView.items`.
- Subtitle counts derived from filtered length.
- Acceptance: page renders pixel-equivalent; `loading` derives from the view; no functional regression.

### 3. `providers-page-direct-mutations`
- Replace each mutation call:
  - `configureProvider` → direct `services/providers-api.ts::configureProvider` call + post-success `loadProvidersIntoGraph()` refresh.
  - `setDefault` → direct call + optimistic `upsertEntity("ProviderMeta", "current", { default_id: id })` with rollback on rejection.
  - `removeProvider` → direct call + optimistic `removeEntity("Provider", id)` with re-upsert on rejection.
- Local component state replaces `saving`, `removing`, `error`.
- `clearError` becomes a `setError(null)` setter.
- Acceptance: all three mutations work; rollback verified by forcing a server reject.

### 4. `retire-providers-admin-store`
- Delete `frontend/src/hooks/use-providers-admin.ts`.
- Delete `frontend/src/stores/providers-admin-store.ts`.
- Verify zero remaining references via `git grep`.
- Flip the `Provider` row in [`docs/migration-stale-data-audit.md`](docs/migration-stale-data-audit.md) from `bridged` to `direct`. Update the bridge section to note the retirement.
- Acceptance: `git grep useProvidersAdmin frontend/src` returns no matches; SPA build still clean; manual smoke green.

---

## Risk register

| Risk | Mitigation |
|------|------------|
| Page renders subtly different after the read swap (e.g. sort order, filter behaviour drifts) | Change 2 keeps mutations on the legacy path so we can verify reads in isolation before touching writes |
| Optimistic `setDefault` patch races with SSE confirmation | `upsertEntity` is idempotent — second write with same value is a no-op. Race is benign |
| `removeProvider` rollback fails because we don't have the original entity on hand | Capture full entity snapshot via `useGraphStore.getState().entities["Provider"][id]` before delete |
| Other admin pages indirectly read provider state through some hook I missed | `git grep -E "providers-admin-store\|useProvidersAdmin"` before change 4 |
| `useProviders()` view hook's loading semantics differ from store loading | Audit `useEntityView`'s `loading` field; if missing, derive from "items.length === 0 && !hydrated" |
| Pilot scope creep — somebody asks for Models/Agents in the same PR | Decision Q4 locked: stop after Providers; new phase per entity |

---

## Acceptance gate before phase reflect

Before running `/kbd-reflect`:
1. `pnpm --filter ./frontend build` clean.
2. `git grep -nE "useProvidersAdmin|providers-admin-store" frontend/src` empty.
3. Manual two-tab smoke: configure → both tabs reflect; remove → both tabs reflect; set-default → default badge flips instantly.
4. Audit doc updated.

---

## Sources

- [Assessment](.kbd-orchestrator/phases/direct-entity-migration-providers/assessment.md) — §2 inventory, §4 gap analysis.
- Existing infra: [`entities/fetchers/providers.ts`](frontend/src/entities/fetchers/providers.ts), [`entities/hooks/use-providers.ts`](frontend/src/entities/hooks/use-providers.ts), [`lib/realtime/use-graph-bridge.ts`](frontend/src/lib/realtime/use-graph-bridge.ts) (about to become unused for this entity).

---

## Progress signal

Completed kbd-plan — direct-entity-migration-providers
