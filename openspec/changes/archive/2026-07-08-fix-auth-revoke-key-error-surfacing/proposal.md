## Why

`auth-keys-store.ts`'s `revokeKey` action swallows errors in an empty `catch {}` block, unlike every other mutation on the same page (`load`, `createKey`), which both set `error: (e as Error).message` on failure. A failed API-key revocation currently fails silently in the UI — the user sees no indication anything went wrong, and the key may still appear revoked in their mental model when it isn't. Found during `uar-production-ready-uiux-2026-07`'s assessment while auditing every admin page for real-vs-facade behavior.

## What Changes

- `revokeKey`'s catch block sets `error: (e as Error).message` before clearing `revoking`, matching the exact pattern already used by `load` and `createKey` in the same store.
- No other behavior changes — `auth-page.tsx` already renders `error` via `<AdminError error={error} />`; no template/rendering changes needed.

## Capabilities

### New Capabilities

- `auth-key-management`: covers the admin auth/API-key CRUD page's error-surfacing behavior — previously undocumented as a capability.

### Modified Capabilities

(none)

## Impact

- `frontend/src/stores/auth-keys-store.ts` only (one function).
- No backend changes — this is a frontend error-surfacing gap, not a missing API capability (`deleteAuthKey` already returns/throws real errors; the store just discarded them).
