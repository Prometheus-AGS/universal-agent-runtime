# @prometheus-ags/prometheus-entity-management — public API reference

Generated from `src/index.ts`. Use this when scaffolding imports or explaining capabilities. Types are TypeScript-only unless noted.

**Runtime exports ledger:** Agent skills that must match shipped code should align with `library-exports.json` (export names from the built ESM bundle). After changing `src/index.ts` exports, run `pnpm run refresh:exports` at the repo root and commit `prometheus-entity-skills/_shared/references/library-exports.json`. CI runs `pnpm run verify:skills`.

---

## Core (`graph.ts`)

| Export | Kind | Description |
|--------|------|-------------|
| `useGraphStore` | hook/store | Zustand store for the normalized graph: `entities`, `patches`, `lists`, `entityStates`, and mutation actions. Prefer hooks in components; `getState()` in non-React code. |
| `GraphState` | type | Full store shape: canonical entities, UI patches, per-entity fetch state, list slots keyed by query key. |
| `EntityState` | type | Per-entity cache metadata: `isFetching`, `lastFetched`, `error`, `stale`. |
| `EntitySyncMetadata` | type | Optional sync/provenance metadata stored beside canonical rows (`synced`, `origin`, `updatedAt`). |
| `EntitySnapshot` | type | Sync-aware entity view returned by snapshot reads and graph query helpers (`$synced`, `$origin`, `$updatedAt`). |
| `ListState` | type | List slot: ordered **`ids`** only (not row copies), pagination cursors, `total`, fetch flags, page metadata. |
| `EntityType` | type | String key partitioning the graph (e.g. `"Post"`). |
| `EntityId` | type | Primary key string for an entity within its `EntityType`. |
| `QueryKey` | type | Stable string key used for list slots. |
| `SyncOrigin` | type | Provenance enum for sync metadata: `server`, `client`, or `optimistic`. |

---

## Graph runtime extensions

| Export | Kind | Description |
|--------|------|-------------|
| `queryOnce` | function | One-shot graph query helper: single row or list snapshot with local filter/sort/include support. |
| `selectGraph` | function | Alias for `queryOnce` when using the API as a projection helper. |
| `createGraphTransaction` | function | Explicit optimistic graph write transaction with rollback/commit semantics. |
| `createGraphAction` | function | Wraps optimistic transaction + async execution into one reusable mutation action. |
| `createGraphEffect` | function | Lightweight enter/update/exit workflow helper driven by graph query results. |
| `createGraphTool` | function | Small AI-facing helper that injects graph query/export context into a typed function. |
| `createSchemaGraphTool` | function | Schema-aware graph tool helper that injects schema lookup and schema-aware export helpers. |
| `exportGraphSnapshot` | function | Serialize graph data for prompt/context building or external workflows. |
| `GraphActionRecord` | type | Serializable pending action record (`id`, `key`, `input`, `enqueuedAt`). |
| `GraphActionEvent` | type | Event union emitted when graph actions are enqueued or settled. |
| `GraphQueryOptions` | type | Options for `queryOnce`: `type`, ids/list key, local predicates, includes, and projections. |
| `GraphIncludeMap` | type | Nested include/projection map for graph snapshot expansion. |
| `GraphIncludeRelation` | type | Relation resolver for snapshot includes (`field`, `array`, `list`, `resolver`). |
| `GraphTransaction` | type | Transaction surface for graph write orchestration. |
| `GraphActionOptions` | type | Async action config wrapping optimistic writes, success, and rollback behavior. |
| `GraphEffectHandle` | type | Disposable handle returned by `createGraphEffect`. |
| `GraphEffectOptions` | type | Enter/update/exit effect configuration over graph query results. |
| `GraphEffectEvent` | type | Update payload for graph effects. |
| `GraphSnapshotExportOptions` | type | Serialization options for `exportGraphSnapshot`. |
| `GraphToolContext` | type | Helper context injected into `createGraphTool` handlers. |
| `SchemaGraphToolContext` | type | Extended tool context with schema lookup and schema-aware export functions. |

---

## Schema-driven entities and markdown

| Export | Kind | Description |
|--------|------|-------------|
| `registerEntityJsonSchema` | function | Register a JSON Schema for an entity type and optional JSON column / schema id. |
| `registerRuntimeSchema` | function | Register or replace a runtime-generated schema (for AI/A2UI or other dynamic metadata). |
| `getEntityJsonSchema` | function | Resolve the active schema by `entityType`, `field`, or `schemaId`. |
| `buildEntityFieldsFromSchema` | function | Generate entity field descriptors from JSON Schema for dynamic views and editors. |
| `useSchemaEntityFields` | hook | Resolve registered schema config and memoize generated field descriptors. |
| `exportGraphSnapshotWithSchemas` | function | Serialize graph data with schema metadata for AI or external consumers. |
| `MarkdownFieldRenderer` | component | Built-in safe markdown renderer for schema-driven or manual field descriptors. |
| `MarkdownFieldEditor` | component | Built-in markdown editor with preview for schema-driven fields. |
| `renderMarkdownToHtml` | function | Escapes and renders a constrained markdown subset to HTML. |
| `JsonSchemaObject` | type | Lightweight JSON Schema shape used by the schema registry and generators. |
| `EntityJsonSchemaConfig` | type | Registration shape for static/runtime schemas. |
| `SchemaFieldDescriptor` | type | Schema-derived field descriptor compatible with entity sheets. |
| `BuildEntityFieldsFromSchemaOptions` | type | Input options for field generation (`schema`, optional `rootField`). |
| `GraphSnapshotWithSchemasOptions` | type | Serialization options for graph export with schemas. |

---

## Local-first runtime

| Export | Kind | Description |
|--------|------|-------------|
| `startLocalFirstGraph` | function | Starts graph hydration, persistence, pending action replay, and sync-status tracking over a host storage adapter. |
| `hydrateGraphFromStorage` | function | Restore a persisted graph snapshot into the Zustand graph. |
| `persistGraphToStorage` | function | Persist the current graph snapshot plus pending action metadata. |
| `useGraphSyncStatus` | hook | Read serializable sync status for browser/PWA/Tauri-style hosts. |
| `GraphPersistenceAdapter` | type | Storage adapter contract: `get`, `set`, optional `remove`. |
| `PersistedGraphActionRecord` | type | Re-exported persisted action metadata shape used by the local-first runtime. |
| `GraphSyncStatus` | type | Status payload: phase, online state, sync state, pending count, hydration/persistence timestamps, error. |
| `GraphSnapshotPayload` | type | Persisted JSON payload containing graph snapshot and pending actions. |
| `StartLocalFirstGraphOptions` | type | Runtime bootstrap options for storage, replay, and online-source integration. |
| `LocalFirstGraphRuntime` | type | Returned runtime handle with `ready`, `persistNow`, `hydrate`, `dispose`, and `getStatus`. |

---

## Engine (`engine.ts`)

| Export | Kind | Description |
|--------|------|-------------|
| `configureEngine` | function | App-wide engine options: base fetch, default `staleTime`, retry behavior, etc. |
| `serializeKey` | function | Stable string key from a query key array (for `lists` map and dedupe). |
| `fetchEntity` | function | Loads/refreshes a single entity through dedupe + graph writes (used by hooks internally). |
| `fetchList` | function | Loads/refreshes a list query: normalizes rows into entities and stores **IDs** under the list key. |
| `dedupe` | function | Process-global promise deduplication for in-flight identical requests. |
| `startGarbageCollector` | function | Starts optional TTL-based cleanup of unused graph data (when configured). |
| `stopGarbageCollector` | function | Stops the garbage collection loop. |
| `EngineOptions` | type | Configuration object for `configureEngine`. |
| `EntityQueryOptions` | type | Options for single-entity fetch pipeline (normalizer, subscriber behavior, etc.). |
| `ListQueryOptions` | type | Options for list fetch (pagination mode, normalizer, merge strategy). |
| `ListFetchParams` | type | Cursor/page parameters passed to list fetch implementations. |
| `ListResponse` | type | Normalized list result shape expected from adapters (ids + meta). |

---

## Hooks — REST (`hooks.ts`)

| Export | Kind | Description |
|--------|------|-------------|
| `useEntity` | hook | Subscribe to one entity by type/id; triggers fetch/SWR via engine; returns merged entity + status. |
| `useEntityList` | hook | Subscribe to a list query key; resolves `ids` to entities from the graph; handles pagination helpers. |
| `useEntityMutation` | hook | Orchestrates create/update/delete with graph upserts/removals and optional invalidation hooks. |
| `useEntityAugment` | hook | Read/write **patches** for an entity (selection, loading flags, etc.) visible to all subscribers. |
| `useSuspenseEntity` | hook | Suspense variant: throws a promise while loading; requires non-null id; use inside `<Suspense>` boundaries. |
| `useSuspenseEntityList` | hook | Suspense variant for list queries; same graph semantics as `useEntityList`. |

---

## DevTools (`devtools.ts`)

| Export | Kind | Description |
|--------|------|-------------|
| `useGraphDevTools` | hook | Development-oriented subscription to graph stats / introspection (useful for debugging, not for production UI). |

---

## View layer (`view/`)

| Export | Kind | Description |
|--------|------|-------------|
| `useEntityView` | hook | Virtual collection over the graph: local / remote / hybrid completeness, filter/sort/search, optional fetch integration. |
| `FilterSpec` | type | Top-level filter: flat `FilterClause[]` **or** nested `FilterGroup` (`and` / `or`). |
| `FilterGroup` | type | Recursive boolean group of clauses and sub-groups. |
| `FilterClause` | type | Atomic predicate: `field`, `op`, optional `value`, optional `predicate` for `custom` ops. |
| `FilterOperator` | type | Supported comparison operators (`eq`, `in`, `contains`, `custom`, …). |
| `SortSpec` | type | Ordered array of `SortClause` (multi-key sort). |
| `SortClause` | type | One sort key: `field`, `direction`, optional `nulls`, optional `comparator`. |
| `SortDirection` | type | `"asc"` \| `"desc"`. |
| `ViewDescriptor` | type | `filter?`, `sort?`, `search?` — everything needed to describe a view for local or remote execution. |
| `CompletenessMode` | type | `"local"` \| `"remote"` \| `"hybrid"` — how much can be answered from the graph vs server. |
| `toRestParams` | function | Compile `ViewDescriptor` to flat REST query string params. |
| `toSQLClauses` | function | Compile to parameterized SQL `where` / `orderBy` fragments + `params` array. |
| `toGraphQLVariables` | function | Produce a Hasura/Postgraphile-style variable object from a view (starting point for wiring). |
| `toPrismaWhere` | function | JSON-serializable Prisma `where` object from filters (pair with server route using Prisma). |
| `toPrismaOrderBy` | function | JSON-serializable Prisma `orderBy` from `SortSpec`. |
| `flattenClauses` | function | Normalize nested `FilterGroup` trees to a flat `FilterClause[]` for simple compilers. |
| `hasCustomPredicates` | function | Returns true if any clause uses `op === "custom"` (cannot be pushed to generic REST/SQL). |
| `applyView` | function | Apply filter/sort/search in **local** JS against in-memory entities (evaluator). |
| `compareEntities` | function | Multi-key comparator implementing `SortSpec` (stable ordering semantics). |
| `matchesFilter` | function | Test one entity against a `FilterSpec`. |
| `matchesSearch` | function | Multi-field substring search helper aligned with view search config. |
| `checkCompleteness` | function | Determine whether graph data is complete enough for local vs remote handling for a descriptor. |
| `UseEntityViewResult` | type | Return type of `useEntityView`. |
| `UseEntityViewOptions` | type | Options for `useEntityView`. |
| `ViewFetchParams` | type | Compiled params passed to optional remote fetch in view hook. |

---

## CRUD (`crud/`)

| Export | Kind | Description |
|--------|------|-------------|
| `useEntityCRUD` | hook | Unified list + detail + create/edit **edit buffer** (React state) + save/cancel + dirty tracking + optional optimistic patch. |
| `registerSchema` | function | Register `EntitySchema` for an `EntityType` (relations + invalidation hints). |
| `getSchema` | function | Lookup registered schema or `null`. |
| `cascadeInvalidation` | function | After mutation success, mark related lists/entities stale from schema graph (FK moves, collections, M2M). |
| `readRelations` | function | Resolve relation placeholders for UI: joins graph reads for belongs-to / has-many / many-to-many. |
| `CRUDOptions` | type | Configuration for `useEntityCRUD` (type, keys, fetchers, etc.). |
| `CRUDState` | type | Full CRUD hook state snapshot type. |
| `CRUDMode` | type | Current UI mode enum for CRUD flow. |
| `DirtyFields` | type | Map of which fields diverge from canonical entity. |
| `EntitySchema` | type | Declares `type`, optional `relations`, optional `globalListKeys`. |
| `RelationDescriptor` | type | Union of relation shapes below. |
| `BelongsToRelation` | type | FK edge: `foreignKey`, `targetType`, optional `invalidateTargetLists`. |
| `HasManyRelation` | type | Inverse collection via `foreignKey` + `listKeyPrefix(parentId)`. |
| `ManyToManyRelation` | type | Id array on entity + `listKeyPrefix` for partner lists. |
| `CascadeContext` | type | `{ type, id, previous, next, op }` passed into cascade logic after mutations. |

---

## Realtime (`adapters/`)

| Export | Kind | Description |
|--------|------|-------------|
| `RealtimeManager` | class | Coalesces incoming `ChangeSet`s and applies them to the graph on a flush window (default ~16ms). |
| `getRealtimeManager` | function | Singleton accessor; configure `flushInterval`, status callbacks, etc. |
| `resetRealtimeManager` | function | Testing / HMR helper to tear down singleton state. |
| `createWebSocketAdapter` | function | Build a `RealtimeAdapter` from a WebSocket URL/protocol. |
| `createSupabaseRealtimeAdapter` | function | Adapter for Supabase Realtime channels. |
| `createConvexAdapter` | function | Adapter for Convex subscription-style feeds. |
| `createGraphQLSubscriptionAdapter` | function | Adapter bridging GraphQL subscriptions to `ChangeSet`s. |
| `ManagerOptions` | type | Options for realtime manager construction. |
| `WebSocketAdapterOptions` | type | WebSocket adapter configuration. |
| `SupabaseAdapterOptions` | type | Supabase-specific adapter options. |
| `RealtimeAdapter` | type | Interface: subscribe/unsubscribe, emit normalized changes. |
| `SyncAdapter` | type | Optional sync-oriented adapter surface. |
| `ChangeSet` | type | Batch of entity changes for one flush tick. |
| `EntityChange` | type | Single create/update/delete payload inside a changeset. |
| `ChangeOperation` | type | Operation discriminator for a change. |
| `ChannelConfig` | type | Channel subscription metadata. |
| `AdapterStatus` | type | Connection health enum / union. |
| `UnsubscribeFn` | type | Teardown function returned from subscriptions. |
| `SubscriptionConfig` | type | Subscription parameters. |

---

## GraphQL (`graphql/`)

| Export | Kind | Description |
|--------|------|-------------|
| `createGQLClient` | function | Factory returning a configured `GQLClient` instance. |
| `GQLClient` | class | Executes operations, normalizes entity-shaped responses into the graph. |
| `normalizeGQLResponse` | function | Map arbitrary GQL JSON into entity inserts keyed by `EntityDescriptor`s. |
| `executeGQL` | function | One-shot execution helper using client config. |
| `useGQLEntity` | hook | Graph-backed single entity with GQL document + variables. |
| `useGQLList` | hook | Graph-backed list via GQL with normalization into entities + list ids. |
| `useGQLMutation` | hook | Mutation helper that updates graph + handles errors. |
| `useGQLSubscription` | hook | Subscribe to server pushes and forward into graph/realtime pipeline as appropriate. |
| `GQLClientConfig` | type | Endpoint, headers, fetcher, default type map. |
| `GQLError` | type | Structured GraphQL error. |
| `GQLResponse` | type | Typed response envelope. |
| `EntityDescriptor` | type | Describes how to normalize a GQL selection into `EntityType` + id field + nested fragments. |
| `GQLEntityOptions` | type | Options for `useGQLEntity`. |
| `GQLListOptions` | type | Options for `useGQLList`. |

---

## Prisma adapter (`adapters/prisma.ts`)

| Export | Kind | Description |
|--------|------|-------------|
| `createPrismaEntityConfig` | function | Build config bridging Prisma models to entity types and list keys. |
| `prismaRelationsToSchema` | function | Derive `EntitySchema` relation entries from Prisma relation metadata. |
| `toPrismaInclude` | function | Compute `include` object for nested fetches from descriptors. |
| `PrismaEntityConfigOptions` | type | Options for Prisma entity config factory. |

*Note:* `toPrismaWhere` and `toPrismaOrderBy` are also exported from `view/types` (re-exported from `prisma-compile`) for view-driven Prisma queries.

---

## Local-first (`adapters/electricsql.ts`)

| Export | Kind | Description |
|--------|------|-------------|
| `createElectricAdapter` | function | ElectricSQL / shape stream adapter emitting `ChangeSet`s compatible with `RealtimeManager`. |
| `useLocalFirst` | hook | Orchestrates local-first sync + graph hydration patterns with PGlite/Electric. |
| `usePGliteQuery` | hook | Run SQL against embedded PGlite and reflect results into React + graph workflows. |
| `ElectricAdapterOptions` | type | Shape URLs, table maps, auth, etc. |
| `ElectricTableConfig` | type | Per-table sync configuration. |
| `UseLocalFirstResult` | type | Hook result: status, connection, helpers. |

---

## UI (`ui/`)

| Export | Kind | Description |
|--------|------|-------------|
| `EntityTable` | component | Opinionated data table: pagination, inline edit hooks, integration with column defs. |
| `InlineCellEditor` | component | Cell-level editor used by `EntityTable` for editable columns. |
| `EntityDetailSheet` | component | Read-only side sheet for one entity. |
| `EntityFormSheet` | component | Create/edit sheet bound to field descriptors and submit handlers. |
| `Sheet` | component | Primitive sheet/drawer wrapper used by the entity sheets. |
| `SortHeader` | component | Table header control wiring sort state to `SortSpec` / table APIs. |
| `selectionColumn` | function | Column factory for row selection checkboxes + graph patch patterns. |
| `textColumn` | function | Text column with optional filter metadata. |
| `numberColumn` | function | Numeric column + filter type hints. |
| `dateColumn` | function | Date/datetime column. |
| `enumColumn` | function | Enum / select-style column. |
| `booleanColumn` | function | Boolean toggle column. |
| `actionsColumn` | function | Row actions menu column. |
| `EntityColumnMeta` | type | Metadata for filters and entity-aware table features. |
| `ColumnFilterType` | type | Drives toolbar filter control selection. |
| `ActionItem` | type | Declarative row action definition. |
| `FieldDescriptor` | type | Describes one form field for `EntityFormSheet`; now supports dotted field paths for JSON-column-backed editors. |
| `FieldType` | type | Field editor type enum / union, including `json` and `markdown`. |

---

## Import map (package name)

```ts
import {
  useGraphStore,
  useEntity,
  useEntityList,
  useEntityView,
  registerSchema,
  // …
} from "@prometheus-ags/prometheus-entity-management";
```

In this monorepo’s examples, the package may resolve via TypeScript path alias to `src/index.ts` during development.
