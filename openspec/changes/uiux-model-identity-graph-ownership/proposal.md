## Why

Phase `uar-uiux-architecture-2026-11` found that provider and model records are
not owned by one place in the console. The facts below were re-read at source in
this Spec stage (grep and file reads only; nothing was run).

- **Three copies of the `Model` projection, two id formats.** The qualified id
  `${providerId}/${modelId}` is built at `frontend/src/features/models/model/models-graph.ts:22,45`
  and `frontend/src/features/models/model/models-store.ts:30,49`. The bare catalog id is
  built at `frontend/src/features/providers/model/providers-graph.ts:78,86` from
  `modelsRowsForProvider` (`frontend/src/features/models/api/models-api.ts:30-31`, the
  `Object.entries` key). `providers-graph.ts:84` also hard-codes `vision: false`.
  Two providers that list the same bare model id would overwrite each other under the
  bare format.
- **The bare writer is dead code today.** `loadModelsForProvider`,
  `loadProvidersIntoGraph` and `loadModelsIntoGraph` have no caller anywhere in
  `frontend/src` (grep for the three names finds only their definitions and two comments).
  The live Model writers are `models-store.ts` `hydrateCatalog` (reached through
  `useModelsAdmin().load`, `use-models-admin.ts:17-19`) and the realtime snapshot. The hazard
  is latent, not active, and fixing the id alone would leave the dead writer in place.
- **The realtime snapshot is a fourth writer with a different record shape.**
  `src/server/lifecycle_snapshot.rs:165-193` builds each record from the raw catalog model
  JSON plus `id`, `model_id`, `provider_id`. The id is qualified (the Rust comment names
  `models-graph.ts` as the owner of that format). The shape is not: raw records carry nested
  `limit`, `modalities`, `cost` and no `context`, `cost_input`, `provider_name` or
  `provider_configured`. The graph core's `upsert` is a shallow merge
  (`entity-graph-core/dist/index.d.ts:358,401`), so on a page that already hydrated from REST
  the flat fields survive and the raw nested fields are added beside them: one entity then
  carries two shapes. A model written first by a snapshot has no flat fields until REST runs,
  and `projectModelRows` (`features/models/model/model-catalog-view.ts:24-48`) would render
  `context` 0, cost 0 and `vision` false for it. The visible effect on a page that has
  hydrated is therefore small; the defect is two writers disagreeing about the record. This is
  source-derived; runtime behavior is unverified.
- **That snapshot runs on every connect, not only after a change.** `/api/live` opens each
  public topic with a `subscription_established` reset that carries `requires_snapshot: true`
  (`src/uar/api/live.rs:81,112-118`). `models` is a public topic
  (`src/uar/realtime/mod.rs:193-198`). On Postgres or remote Surreal the frontend registers
  the `/api/live` adapters (`frontend/src/entities/sync.ts:342-344`), so the Model set is
  replaced by the snapshot on every page load and reconnect, and every Model the snapshot
  lacks is deleted (`frontend/src/lib/realtime/uar-sse-adapter.ts:258-283`, applied at
  `:373-380`).
- **Per-row graph writes.** Both live writers loop over about 8k catalog models and call
  `upsertEntity` once per row. They are listed in the frozen
  `legacyPerRowGraphWriteLocations` set (`scripts/check-frontend-boundaries.mjs:34-74`:
  `models-graph.ts:45`, `models-store.ts:49`, `providers-graph.ts:36,51,86`,
  `providers-store.ts:70,76,183`). The project rule is one atomic ingestion action.
  `batchGraphWrites` with `upsertEntities` already exists in the installed graph core
  (`frontend/node_modules/@prometheus-ags/entity-graph-core/dist/index.d.ts:576-658`) and is
  used by the teams domain (`frontend/src/platform/entities/teams/commands.ts:63`).
- **Zustand stores still hold graph-owned records.** `useModelsStore.configuredProviders`
  is a `UarProvider[]` (`models-store.ts:10,90`). `useProviderModelsStore.rowsByProvider`
  holds catalog rows per provider (`features/providers/model/provider-models-store.ts:11-37`).
  The gate's `duplicate-graph-owned-cache` rule only matches the identifiers
  `ConfiguredProvider|ConfiguredModel|AgentSession|AgentSessionDraft`
  (`check-frontend-boundaries.mjs:467-476`), so neither store trips it.
- **The facade leaks the raw graph store.** `frontend/src/platform/entities/runtime.ts`
  re-exports `useGraphStore` and `graphStore`. 28 non-test feature files import one of them
  (grep), including a component, `features/cost/ui/cost-dashboard-page.tsx:3,28`. The gate
  has no rule for it.
- **Threads have a second owner.** `frontend/src/stores/thread-registry-store.ts` mirrors the
  graph `Thread` slice through `stores/use-thread-graph-sync.ts`, which subscribes to the raw
  graph store.

## What Changes

- **One identity and one record shape for catalog `Model` entities.** A single
  `catalogModelId(providerId, modelId)` and a single `projectCatalogModel` define the id and
  the record. The id is `${provider_id}/${model_id}`, is never parsed back into its parts, and
  every writer (REST hydration, realtime snapshot, any future writer) goes through them.
  Dead writers are deleted, not aligned.
- **One new platform domain for provider and model catalog entities**, under
  `@/platform/entities`, with typed hooks and graph actions. It ingests the catalog and
  provider registry in one `batchGraphWrites` call each, including removal of rows a
  snapshot no longer lists.
- **Retire the Zustand duplicates of graph-owned provider and model records**
  (`useModelsStore.configuredProviders`, `useProviderModelsStore`). Workflow state
  (busy, error, refreshing) stays transient.
- **Close the facade leak by staging.** Raw `useGraphStore` and `graphStore` stop being
  available to feature code for the domains this change migrates, and the gate rejects new
  raw graph access in feature code against a shrink-only baseline of the remaining files.
- **Tighten the gate's duplicate-cache rule** to cover `Provider`, `ProviderMeta` and `Model`
  entities and their DTO shapes (`UarProvider`, `ProviderModelRow`, `CatalogModelsResponse`,
  `ProviderEntity`, `ModelEntity`).
- **Server-mirrored thread records get one owner** (graph `Thread` entity behind a platform
  hook); client-only ephemeral thread state stays transient. Slicing and PGlite write-path
  coordination are Plan decisions (see design.md, decision D6).

- **Virtualize the Models page list** (it maps every filtered row today, `models-page.tsx:326`), so 8k rows do not mount at once.
- **Frozen oracles.** Expected values for the acceptance scenarios live in `fixtures/` (catalog counts and sample rows computed from the committed catalog snapshot, and a collision config), written before implementation.

No backend change. The Rust snapshot already emits the qualified id.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `frontend-react-entity-guardrails`: the architecture gate's duplicate-cache rule widens
  to provider and model entities; new requirements fix catalog Model identity, record shape,
  atomic ingestion, raw-graph-access limits and single ownership of mirrored thread records.

## Impact

- Frontend source (owner `uar-console`): `frontend/src/platform/entities/**` (new domain),
  `frontend/src/features/{models,providers}/**`, `frontend/src/lib/realtime/uar-sse-adapter.ts`
  (record projection hook), `frontend/src/entities/{schemas,types}.ts`,
  `frontend/src/stores/{thread-registry-store,use-thread-graph-sync}.ts`, and every feature
  file listed in the raw-graph-access baseline.
- Outside console ownership; needs a single-writer assignment from the lead:
  `scripts/check-frontend-boundaries.mjs`, `scripts/frontend-boundary-allowlist.txt`,
  `scripts/test-frontend-boundaries-negative.mjs`, `scripts/fixtures/frontend-boundary-cases/**`.
  `src/server/lifecycle_snapshot.rs` is not edited by this change; its comment at `:179-182`
  points at `models-graph.ts`, which this change deletes, so a one-line comment fix belongs to
  the state owner.
- No dependency pins change. `@prometheus-ags/entity-graph-core` is already imported directly
  inside `platform/entities` (teams domain precedent).

## Related open changes

Not modified here; the operator decides in Plan (decision 7).

| Open change | State | Suggested disposition | Reason |
|---|---|---|---|
| `migrate-models-page-direct-and-redesign` | 0 of 16 tasks | supersede by this change | Plans a `useModels()` migration and `optimisticUpsert/Remove` on the models page; this change owns the Model entity and its hooks, and the redesign belongs to the design changes. |
| `add-models-fetcher-and-hook` | 0 of 5 tasks | archive | Targets `entities/fetchers/models.ts`; the hook already lives at `features/models/model/use-models.ts` and the fetcher is superseded by the new domain. |
| `migrate-providers-models-agents-to-entity-mgmt` | 0 of 13 tasks | supersede (provider and model parts); agents, skills, settings parts remain for their owners | Same entities, older architecture (`services/entities/`). |
| `retire-providers-admin-store` | 8 of 9 tasks | continue to archive after its last task | Nearly done; the remaining store (`providers-store.ts`) is addressed here. |
| `providers-page-direct-reads`, `providers-page-direct-mutations` | 8 of 10, 9 of 12 | archive as superseded | The pages already read and mutate through the graph; the residual per-row writes are removed here. |
| `backend-extend-realtime-topics` | 9 of 11 | continue (owner `uiux-realtime-topics-and-replica-refresh`) | Topic coverage; this change only fixes the Model record contract on the `models` topic. |
| `contract-bridge-refetch`, `delete-use-graph-bridge` | 0 of 6, 0 of 10 | continue, unrelated | Concern `useGraphBridge`, not the Model identity. Listed so the operator sees the overlap with the raw-graph-access baseline. |
| `admin-agent-provider-first-model-picker` | empty shell (`.openspec.yaml` only) | archive, successor `uiux-schema-controls-and-pickers` | No artifacts; the picker work is specified there. |
