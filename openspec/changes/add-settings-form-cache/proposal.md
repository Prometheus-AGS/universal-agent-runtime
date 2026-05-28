# add-settings-form-cache

## Why
The retired `settings-store` held per-namespace `dirty` / `saving` / `error` state in Zustand. The new direct hook needs the same per-namespace persistence across component re-mounts — but without a Zustand store. A small module-level cache with `useSyncExternalStore` is the right tool.

## What changes
- New `frontend/src/hooks/settings-form-cache.ts` exporting `getDirty`, `setDirty`, `clearDirty`, `setSaving`, `subscribe`.
- Module-level `Map<namespace, DirtyState>` + per-namespace listener set.
- Pure utility; no consumers in this change.

## Impact
Additive. Tests still pass.
