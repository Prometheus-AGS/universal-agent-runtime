# add-settings-fetcher-and-hook

## Why
Settings is the biggest page. Add the entity-graph scaffold first so the two-part migration can land cleanly.

## What changes
- New `frontend/src/entities/fetchers/settings.ts` — `loadSettingsIntoGraph()`.
- New `frontend/src/entities/hooks/use-settings-entity.ts` — `useSettingsEntity()` (named to avoid collision with existing legacy `use-settings.ts` until the latter retires).
- `settings-types-meta-store.ts` is NOT touched — field-schema metadata stays as a one-shot REST cache.

## Impact
Additive scaffolding.
