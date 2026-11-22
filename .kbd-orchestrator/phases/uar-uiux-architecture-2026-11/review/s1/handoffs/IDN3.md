# IDN3 handoff: OpenSpec tasks 3.1 and 3.2 (uiux-model-identity-graph-ownership)

Writer: uar-console. Nothing was run (verification boundary closed). All claims below are source inspection.

## Files changed
- frontend/src/lib/realtime/uar-sse-adapter.ts: new exported types `UarSnapshotRecordProjection`, `UarSnapshotOutcome` (~:47-53); new options `projectRecord`, `onSnapshotOutcome` (~:68-74); destructured in `createUarSseAdapter` (~:343-344); projection step + outcome calls in the snapshot `.then` (~:390-412).
- frontend/src/lib/realtime/topics.ts: `UarTopicDefinition` interface, `UAR_TOPICS` retyped from `as const` to `readonly UarTopicDefinition[]` (only `topic`/`entityType` are read elsewhere: entities/sync.ts:69), `models` entry carries `projectRecord: (record) => projectCatalogModel(record)` and `onSnapshotOutcome`; `createAllUarAdapters` forwards both options. Other topics unchanged (options optional).
- frontend/src/platform/entities/model-catalog/contracts.ts (:17-24): `MODEL_CATALOG_SNAPSHOT_LOAD_ID = "models-snapshot"`, added to `ModelCatalogLoadId`.
- frontend/src/platform/entities/model-catalog/domain.ts (~:54-73): `recordModelCatalogSnapshotRejected(error)` (reuses `recordLoadFailure`) and `recordModelCatalogSnapshotAccepted()`; each is one `batchGraphWrites`.
- frontend/src/platform/entities/model-catalog/index.ts: export lines for the three new symbols (re-exported through `@/platform/entities` by `export *`).
- frontend/src/features/models/ui/models-ingestion-error.tsx (new): `ModelsIngestionError`, subscribes only to `useModelCatalogLoadError("models-snapshot")`; renders `role="alert"` + `data-testid="models-ingestion-error"`: "Model catalog update rejected: <error>. Showing the previous catalog." The error text is the typed `missing_model_id` message, which contains "model_id".
- frontend/src/features/models/ui/models-page.tsx: import + one `<ModelsIngestionError />` line right after the existing ERR-MODELS bar (~:219).

## Flow of a rejected record
Reset/initial -> `fetchSnapshot` -> complete snapshot -> `records.map(projectRecord)` -> `projectCatalogModel` throws `ModelCatalogIngestionError("missing_model_id")` -> adapter catch: `recovering=false`, `markListsStale()` (same as the existing null path), `onSnapshotOutcome({ok:false,error})` and return BEFORE `buildAuthoritativeSnapshotChangeSet` and `handler` -> topics.ts callback -> `recordModelCatalogSnapshotRejected` writes only the `ModelCatalogLoadState["models-snapshot"]` entity -> `ModelsIngestionError` re-renders and shows the alert.

## Prior entities preserved
Projection runs over the whole array before any change set exists; a throw returns before `handler`, so no Model upsert or delete is issued. Only the load-state entity changes. A later good snapshot calls `recordModelCatalogSnapshotAccepted` (clears the error). A separate load id (not "models") is used so a REST `ingestModelCatalog` (which writes `models` load state with error null) cannot clear the snapshot error before the user or test sees it.

## Rules applied (summary)
Vercel React best practices: `rerender-split-combined-hooks` and `rerender-derived-state` led to a tiny subscriber component with one primitive subscription (a nullable string), so the 8k-row page shell does not rerender when the error changes; no setState in render; no effects. Composition patterns: no boolean-prop growth, the adapter takes optional function options instead of topic-specific branching. Entity skills and typescript.md layering: the failure is domain state (graph entity written by a domain action in one `batchGraphWrites`), read by a domain hook from `@/platform/entities`; the component imports no store, transport or graph package; the realtime adapter stays I/O and calls a domain action through a callback supplied at the topic table. No `any`, no console, no new dependencies; the projection is optional and applies only to `models`.

## Unresolved decisions / risks
- Embedded-store path (entities/sync.ts:235, `createEmbeddedSseAdapter`) also calls `buildAuthoritativeSnapshotChangeSet` with no projection; not in my ownership and not covered by the negative control (remote mode). Needs its own task if embedded `Model` rows must be projected.
- Live per-record create/update events for `models` are not projected (task text covers snapshots only).
- Server snapshot records (src/server/lifecycle_snapshot.rs:165-187) always carry `provider_id`, `model_id`, `id = provider/model`, so valid snapshots should project; projected `id` is `catalogModelId` and should equal the server id. Projection overwrites `provider_name`/`provider_configured` from the graph Provider (or the provider id fallback) on each snapshot.
- `uar-sse-adapter.test.ts` was not touched; no unit test added for the new options (tests are authored by others / boundary closed).
- Task 3.3 (Rust comment) remains a coordination item.

## Unverified (everything)
Not typechecked, linted, unit-tested or run in Playwright, including `e2e/model-catalog-identity.spec.ts` SCN-IDN-001 negative control. Open points a typecheck should confirm: `CatalogModel extends Record<string, unknown>` makes `projectCatalogModel` assignable to `UarSnapshotRecordProjection`; `@/platform/entities` value imports in topics.ts introduce no import cycle (the adapter already imports `useGraphStore` from it).
