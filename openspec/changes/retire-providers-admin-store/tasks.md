## 1. Delete files

- [x] 1.1 `rm frontend/src/hooks/use-providers-admin.ts`
- [x] 1.2 `rm frontend/src/stores/providers-admin-store.ts`

## 2. Sweep

- [x] 2.1 `git grep -nE "useProvidersAdmin|providers-admin-store" frontend/src` returns empty.

## 3. Audit doc

- [x] 3.1 Flipped `Provider` row in `docs/migration-stale-data-audit.md` from `bridged` → `direct`.
- [x] 3.2 Added playbook description in the bridge section: 6-step process for the remaining 8 bridged entities.

## 4. Verification

- [x] 4.1 `pnpm --filter ./frontend build` clean.
- [x] 4.2 UAR restarted; SPA serves new bundle.
- [ ] 4.3 Manual two-tab smoke green across all three mutations — pending.
- [x] 4.4 `git diff --stat`: 2 files deleted (use-providers-admin.ts + providers-admin-store.ts ≈ 180 LOC removed); page + fetcher + new hook ≈ 90 LOC added. Net: ~90 LOC reduction.
