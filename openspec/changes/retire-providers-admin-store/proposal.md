## Why

After the previous three changes, `providers-page.tsx` reads from the entity graph and mutates via direct service calls. `frontend/src/hooks/use-providers-admin.ts` and `frontend/src/stores/providers-admin-store.ts` are now orphaned code paths still maintained for backward compatibility. Deleting them retires the bridge for the `Provider` entity, completes the pilot, and establishes the playbook the next 4 cross-cutting entities will follow.

## What Changes

- **Delete** `frontend/src/hooks/use-providers-admin.ts`.
- **Delete** `frontend/src/stores/providers-admin-store.ts`.
- Verify no remaining references via `git grep -nE "useProvidersAdmin|providers-admin-store" frontend/src` — must return zero.
- Update [`docs/migration-stale-data-audit.md`](../../../docs/migration-stale-data-audit.md): flip the `Provider` row from `bridged` → `direct`. Add a one-line note in the bridge section explaining that direct migration is the pattern going forward.
- No other file changes — the bridge helper (`use-graph-bridge.ts`) stays in tree because the other 7 admin hooks still depend on it.

## Acceptance

- `git grep -nE "useProvidersAdmin|providers-admin-store" frontend/src` → empty.
- `pnpm --filter ./frontend build` clean.
- Manual two-tab smoke: configure, set-default, remove — all propagate ≤200 ms across tabs.
- Audit doc updated.
- Net LOC delta is negative for the migration (two deleted files > new additions across the prior 3 changes).
