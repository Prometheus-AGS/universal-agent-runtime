> Slice S1 plan and task model assignments: `.kbd-orchestrator/phases/uar-uiux-architecture-2026-11/plan.md` ("Task model assignments", rows keyed by this change ID). Tasks not listed there are outside S1.

## 1. Platform model-catalog domain (identity and record shape)

- [ ] 1.1 Create `frontend/src/platform/entities/model-catalog/contracts.ts` with `catalogModelId(providerId, modelId)`, the `CatalogModel` record type (fields listed in the spec requirement "Catalog Model records have one shape for every writer") and the `Provider` and `ProviderMeta` record types now in `frontend/src/entities/types.ts`.
- [ ] 1.2 Add `projectCatalogModel` that accepts a REST catalog entry or a realtime snapshot record, derives `vision` from `modalities.input`, takes `provider_name` and `provider_configured` from the graph's `Provider` entities with the provider id as fallback, and rejects a record without `provider_id` or `model_id` with a typed ingestion error. Reuse the null-safe field handling already in `hydrateCatalog` (`features/models/model/models-store.ts:26-52`) as the baseline.
- [ ] 1.3 Create `registration.ts` for the domain and register the `Provider`, `ProviderMeta` and `Model` schemas and relations currently in `frontend/src/entities/schemas.ts:19-45`; keep the relation `Provider.models -> Model` on `provider_id`.
- [ ] 1.4 Create the domain `api/` transport (fetch `/api/models`, `/api/uar/providers`, the provider catalog) by moving `features/models/api/models-api.ts` and `features/providers/api/providers-api.ts` functions that the domain needs; leave a feature-level re-export only where a non-migrated caller still imports it, and list each such caller in the changed-path inventory.
- [ ] 1.5 Export the domain from `frontend/src/platform/entities/index.ts` without exporting the raw graph store through it.

## 2. Atomic ingestion actions

- [ ] 2.1 Implement `ingestModelCatalog(payload)` in `domain.ts`: one synchronous `batchGraphWrites` (from `@prometheus-ags/entity-graph-core`, as `platform/entities/teams/commands.ts:63` does) that calls `upsertEntities("Model", rows)`, removes `Model` entities absent from the payload and sets the list ids.
- [ ] 2.2 Implement `ingestProviderRegistry(catalog, configured)` the same way for `Provider`, `ProviderMeta` (default provider id and that provider's `default_model`) and the provider list ordering now set at `features/providers/model/providers-store.ts:95`.
- [ ] 2.3 Implement `loadModelCatalog()` and `loadProviderRegistry()` actions that fetch through the domain transport and call the ingestion actions only after the fetch succeeds; a failed fetch leaves the graph unchanged and sets the domain error state.
- [ ] 2.4 Implement the per-provider health ingestion (`RuntimeProviderHealth`, per-row loop at `providers-store.ts:199`) as one batch in the same domain.

## 3. Realtime snapshot projection

- [ ] 3.1 Add an optional `projectRecord` option to `createUarSseAdapter` in `frontend/src/lib/realtime/uar-sse-adapter.ts` and apply it to each snapshot record before `buildAuthoritativeSnapshotChangeSet` (`:258-283`). A record the projection rejects makes the snapshot ingestion fail visibly and keep the previous entities (the existing `null` path at `:372-379`).
- [ ] 3.2 Supply `projectCatalogModel` for the `models` entry of `UAR_TOPICS` (`frontend/src/lib/realtime/topics.ts:16`). Coordinate this one-line edit with `uiux-realtime-topics-and-replica-refresh`, which also edits `topics.ts`; one writer per Plan wave.
- [ ] 3.3 Ask the state owner to correct the comment at `src/server/lifecycle_snapshot.rs:179-182` so it names the new `catalogModelId` instead of `models-graph.ts`. This is a coordination item, not an edit by this change.

## 4. Retire the Zustand duplicates and dead loaders

- [ ] 4.1 Delete `features/providers/model/provider-models-store.ts` and `use-provider-models.ts`; replace their one consumer (`features/providers/ui/providers-page.tsx:356`) with `useCatalogModelsForProvider(providerId)`.
- [ ] 4.2 Remove `configuredProviders` from `features/models/model/models-store.ts`; keep `refreshing`, `busyModelKey` and `error` as transient workflow state. Read configured providers and models through `useConfiguredModels` and a new `useConfiguredProviders` in `platform/entities/session-configuration` (decide the disabled-model handling flagged as UNVERIFIED in design.md before removing the old path).
- [ ] 4.3 Turn `addModel`, `setDefaultModel` and `removeModel` into domain actions that call the existing provider update API and re-ingest on success, restoring the previous graph state and surfacing the error on failure. Keep the request and response contract of `updateProvider` unchanged.
- [ ] 4.4 Delete `features/models/model/models-graph.ts` and `features/providers/model/providers-graph.ts` (no callers, see design.md) and `use-providers.ts`'s `useProviderModelsEntity` (no caller).
- [ ] 4.5 Replace `hydrateProviders` in `features/providers/model/providers-store.ts` with a call to `loadProviderRegistry`; keep `saving`, `removingId`, `error` and the health flags as workflow state. Land this before `uiux-system-model-live-default` edits `setDefault` in the same file.
- [ ] 4.6 Refresh the configured provider and model entities when a provider changes elsewhere: the domain reloads `ConfiguredProvider` and `ConfiguredModel` on the realtime provider-change invalidation, so a second open page updates without a reload and no feature store caches the list. Depends on the provider-topic fix in `uiux-realtime-topics-and-replica-refresh`; one writer for `lib/realtime/**` per Plan wave.

## 5. Consumers read through domain hooks

- [ ] 5.1 Add the hooks `useCatalogModels`, `useCatalogModel`, `useCatalogModelsForProvider`, `useCatalogProviders` and `useProviderMetaField` to `use-model-catalog.ts`, each subscribing to one field or one derived primitive.
- [ ] 5.2 Move `features/models/model/use-models.ts` and `use-models-admin.ts` onto the domain hooks; the page keeps `projectModelRows` for display only.
- [ ] 5.3 Move `features/providers/model/use-provider-default.ts` onto `useProviderMetaField` so it no longer imports `useGraphStore`.
- [ ] 5.4 Split any shell component that subscribes to a list and also renders a single-field control into independent subscriber components (Vercel `rerender-split-combined-hooks`), starting with the provider detail panel and the models configured-rows section.
- [ ] 5.5 Replace the raw `useGraphStore` selector in `features/cost/ui/cost-dashboard-page.tsx:28` with a domain hook, or record it as a baseline entry if its entity belongs to a domain outside this change.
- [ ] 5.6 Virtualize the catalog list on the Models page (`features/models/ui/models-page.tsx:326` maps every filtered row today) with `@tanstack/react-virtual`, following the existing use at `features/chat/ui/run-trace-timeline.tsx:2`; keep the compare checkboxes, `data-testid` keys (`compare-checkbox-<key>`), keyboard focus order and the `models-count` text. Owned here; this change owns the 8k-row page.

## 6. Architecture gate (single-writer assignment needed for `scripts/**`)

- [ ] 6.1 Widen `duplicate-graph-owned-cache` in `scripts/check-frontend-boundaries.mjs` (rule at `:467-476`) to the entity and DTO names in the modified requirement; match a Zustand store whose state type or initial value references them, not only a bare identifier.
- [ ] 6.2 Add rule `feature-model-id-construction`: a template-string or concatenation key of the form provider-id slash model-id outside `platform/entities/model-catalog/`.
- [ ] 6.3 Add rule `feature-raw-graph-access` for imports of `useGraphStore`, `graphStore` and raw write functions from `@/platform/entities` in `frontend/src/features/**` and `frontend/src/stores/**`, with a baseline keyed by file path and rule, shrink-only, and growth and stale-entry detection matching the allowlist file's semantics. Seed the baseline from the 28 feature files found by grep, excluding every file under `features/models` and `features/providers`.
- [ ] 6.4 Remove the `legacyPerRowGraphWriteLocations` entries for the files this change migrates (`models-graph.ts:45`, `models-store.ts:49`, `providers-graph.ts:36,51,86`, `providers-store.ts:70,76,183`) and re-key the remaining entries by file and rule if the lead accepts that change; the gate's own line numbers and the current set disagree for `providers-store.ts` (design.md, UNVERIFIED).
- [ ] 6.5 Add negative fixtures under `scripts/fixtures/frontend-boundary-cases/` for the six cases in design.md B5 and register their expected rule ids in `scripts/test-frontend-boundaries-negative.mjs`.
- [ ] 6.6 After the last baseline file has migrated (or in the phase that does it), remove the `useGraphStore` and `graphStore` re-export from `platform/entities/runtime.ts`. Until then keep it and rely on 6.3.

## 7. Server-mirrored thread ownership (separable; Plan may move it to a child phase)

- [ ] 7.1 Add a platform thread domain hook (`useThreads`, `useThread(id)`) over the graph `Thread` entity and its realtime `threads` topic.
- [ ] 7.2 Reduce `stores/thread-registry-store.ts` to ephemeral threads and the active thread id; read title, `updated_at` and the persisted flag from the graph hook.
- [ ] 7.3 Delete `stores/use-thread-graph-sync.ts` and its mount in `hooks/use-app-bootstrap.ts:7`; replace the registry-to-PGlite write path through the interface that `uiux-local-first-persistence-worker` defines.

## 8. Boundary acceptance (runs only at the phase boundary)

These tasks describe what the final integration gate exercises. They must not be run early. Expected values come only from the frozen fixtures listed in design.md (`fixtures/catalog-identity.json`, `fixtures/collision-config.yaml`, the baseline revision and the frozen thresholds). Provider upstreams use the stub model server (`target/debug/stub-llm`); no scenario calls a paid model route. A mandatory scenario whose negative control does not run is BLOCKED.

- [ ] 8.1 SCN-IDN-001: start the runtime with the collision config and open `/admin/models`; assert 8,173 models and 345 providers, one row per provider for the frozen shared ids, the two `idn-shared-model` rows with their frozen context and vision values, and the slash id intact; run the mutant with the bare-id `catalogModelId` and require it to fail the count. Gate: `pnpm -C frontend exec playwright test -c playwright.real-server.config.ts e2e/model-catalog-identity.spec.ts`, 3 samples.
- [ ] 8.2 SCN-IDN-002: on a remote store, force a live-stream reconnect and compare every frozen sample row and the live `/api/models` entry; deliver one snapshot record missing `model_id` and assert unchanged rows plus a visible ingestion error. 5 samples. Uses the realtime change's remote-store fixture; BLOCKED, not skipped, if it is unavailable.
- [ ] 8.3 SCN-IDN-003: record long tasks over the catalog load and the first filter keystroke for the candidate and for the baseline bundle; require the candidate under 200 ms in every sample and the baseline over 200 ms in at least 3 of 5. 5 samples.
- [ ] 8.4 SCN-IDN-004: with `fixtures/provider-model-config.yaml`, run the frozen steps in `fixtures/provider-model-sequence.json` (add M1, default M1, remove M0 in the console; add M2 over REST while a second page stays open). Stop the runtime and read the persisted row `provider.nano-gpt` with `surrealql_store_runner`; compare model ids (as a set) and default with the frozen end state (models {M1, M2}, default M1). Restart and assert the UI renders exactly the frozen rows. Run the same steps against the baseline worktree (must fail at step 4) and against the two mutants (dropped remove, dropped add; must fail at the store read-back or step 1); a wrong build that passes makes the scenario BLOCKED. Gate: the Playwright spec `e2e/provider-model-roundtrip.spec.ts` under `playwright.real-server.config.ts` plus the store read-back, 3 samples.
- [ ] 8.5 SCN-IDN-005 (supplemental static gate): run `pnpm frontend:boundaries`; confirm exit 0 on the tree and the expected rule id for each of the six frozen negative fixtures.
- [ ] 8.6 SCN-IDN-006 (optional; only if group 7 is in scope): rename and delete a conversation on the server and compare the sidebar with `GET /api/uar/sessions`. Gate: `pnpm -C frontend exec playwright test -c playwright.real-server.config.ts e2e/thread-ownership.spec.ts`, 3 samples.
- [ ] 8.7 SCN-IDN-007: with the bundled catalog and the collision config, assert at most 200 catalog row elements are in the DOM while the page reports 8,173 models, the last row is reachable by scrolling and by keyboard, and the baseline bundle renders more than 8,000 rows (else BLOCKED). 3 samples.

## 9. Prerequisites for the boundary runner (the lead adds the commands)

- [ ] 9.1 Runtime binary and `stub-llm` binary compiled under the serialized single-writer rule.
- [ ] 9.2 Baseline worktree created with `scripts/worktree-new.sh uiux-idn-baseline --base f2c9e36efe088e06a17c5d880ada877e25f2ad52` (under `~/.claude/worktrees/`) and its frontend bundle produced; removed with `scripts/worktree-rm.sh` after the run.
- [ ] 9.3 Remote-store fixture and the `postgres-backend` feature, supplied by the realtime change, for SCN-IDN-001's negative control and SCN-IDN-002.
- [ ] 9.4 `surrealql_store_runner` example compiled (`Cargo.toml:155-158`; an ordinary compile does not build examples) under the serialized single-writer rule.
- [ ] 9.5 Mutant worktrees `uiux-idn-mutant-remove`, `uiux-idn-mutant-add` and `uiux-idn-mutant-bareid` (the last changes `catalogModelId` to return the bare model id, for SCN-IDN-001), created from the implementation-complete commit with `scripts/worktree-new.sh`; the one-function patches are written by the independent reviewer after implementation (not by the implementer), and the worktrees are removed with `scripts/worktree-rm.sh` after the run.
