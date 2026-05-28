# migrate-settings-mutations-and-retire-stores

## Why
Complete the settings migration: direct mutations + retire the store + apply the aesthetic. Heaviest change in the phase.

## What changes
- Every mutation in `settings-page.tsx` goes through `optimisticUpsert("Setting", id, patch, …)`.
- Delete `frontend/src/stores/settings-store.ts` (242 LOC).
- `settings-types-meta-store.ts` REMAINS — schemas are not entities.
- Apply terminal aesthetic across the entire settings page; reuse shared components.
- Playwright screenshot at `screenshots/settings-page.png`.
- Flip `Setting` row to `direct` in audit doc.

## Impact
End-state: settings page reads + writes via graph; only the field-schema metadata stays as a non-graph cache.
