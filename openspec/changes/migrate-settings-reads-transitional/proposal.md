# migrate-settings-reads-transitional

## Why
Settings page is 3334 LOC. Splitting reads from mutations keeps each PR reviewable. This change is the deliberately-transitional commit: page reads from the graph; old store still owns writes.

## What changes
- All read sites in `settings-page.tsx` switch to `useSettingsEntity()`.
- Mutations CONTINUE to go through the legacy `settings-store`. Reads use the graph; writes flow through the store and the SSE event eventually reconciles the graph row. No double-source.

## Impact
Transitional: two state systems coexist for one commit. The next change retires the store. Manual smoke: edit a field → save → refresh → value persists.
