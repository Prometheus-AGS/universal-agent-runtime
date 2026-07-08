## 1. Fix

- [x] 1.1 In `auth-keys-store.ts`'s `revokeKey`, set `error: (e as Error).message` in the catch block before clearing `revoking`, matching `load`/`createKey`'s pattern.

## 2. Verify

- [x] 2.1 Confirm `git status --short` shows only `auth-keys-store.ts` changed.
- [x] 2.2 Confirm `pnpm run build` (frontend) still succeeds.
