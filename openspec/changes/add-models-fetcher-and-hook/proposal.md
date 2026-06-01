# add-models-fetcher-and-hook

## Why
Models is currently bridged via `models-browse-store`. Direct migration needs an entity-graph fetcher + `useModels` hook first.

## What changes
- New `frontend/src/entities/fetchers/models.ts` — `loadModelsIntoGraph()`.
- New `frontend/src/entities/hooks/use-models.ts` — `useModels()` returning `useEntityList<ModelEntity>("Model")`.
- `ModelEntity` already exists in `entities/types.ts`.

## Impact
Net-additive. No consumer rewires yet.
