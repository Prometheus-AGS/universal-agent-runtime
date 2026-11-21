## Context

Build items 3 and 4 of `analysis.md`, plus the round-2 warning "item 3 fixes the Model id
but not the divergent record shape (`providers-graph.ts:84` hard-codes `vision: false`)".
Scenario area `IDN`. In this document **source** means read at file:line during this Spec
stage, and **runtime** means observed by running something. Nothing was run (full-phase
verification boundary), so every defect below is source-derived.

### Verified at source

| Claim | Location |
|---|---|
| Qualified id built by two live writers | `features/models/model/models-graph.ts:22,45`; `models-store.ts:30,49` |
| Bare id built by a third writer; `vision` hard-coded | `features/providers/model/providers-graph.ts:78,84,86`; bare id from `features/models/api/models-api.ts:30-31` |
| The three loader functions have no caller | grep over `frontend/src` for `loadModelsForProvider`, `loadProvidersIntoGraph`, `loadModelsIntoGraph`: definitions and two doc comments only |
| `hydrateCatalog` is the live REST writer | `models-store.ts:26-52,86-90`; reached from `use-models-admin.ts:17-19` |
| Realtime snapshot writes the qualified id with the raw shape | `src/server/lifecycle_snapshot.rs:165-193`; replace-and-delete in `frontend/src/lib/realtime/uar-sse-adapter.ts:258-283`, applied at `:373-380` |
| The page reads flat fields the snapshot lacks (the graph's `upsert` shallow-merges, `entity-graph-core/dist/index.d.ts:358,401`, so a page that already hydrated keeps them) | `features/models/model/model-catalog-view.ts:26-48` (`context`, `cost_input`, `provider_name`, `provider_configured`, `vision`) |
| The snapshot fires on every connect | initial `subscription_established` reset with `requires_snapshot: true`: `src/uar/api/live.rs:81,112-118`; `models` is public: `src/uar/realtime/mod.rs:193-198` |
| `/api/live` adapters serve Postgres and remote Surreal | `frontend/src/entities/sync.ts:342-344` (analysis delta 1; not re-read beyond the cited lines) |
| Per-row writes are frozen in an inline legacy set | `scripts/check-frontend-boundaries.mjs:34-74` |
| The duplicate-cache rule matches four identifiers only | `scripts/check-frontend-boundaries.mjs:467-476` |
| The facade re-exports the raw graph store | `frontend/src/platform/entities/runtime.ts:10,18`; 28 non-test feature files import it (grep) |
| `batchGraphWrites`, `upsertEntities`, `setListIds` exist | `frontend/node_modules/@prometheus-ags/entity-graph-core/dist/index.d.ts:576-658` |
| Precedent for atomic domain ingestion | `frontend/src/platform/entities/teams/commands.ts:63,191,316` |
| `ConfiguredModel` is keyed by the qualified route and omits disabled models | `platform/entities/session-configuration/registration.ts:59-100` |
| One full-catalog fetch per provider detail panel | `features/providers/model/use-provider-models.ts:8-16`; `provider-models-store.ts:24-31` |

### UNVERIFIED

- Whether the gate currently passes. Grep puts the `providers-store.ts` row writes at lines
  71, 77 and 199, while the inline legacy set lists `:70`, `:76`, `:183`. If the keys are
  off, the gate already reports them. This is why the new baseline is keyed by file and
  rule, not by line.
- Whether the realtime manager applies a snapshot `upsert` as a merge or as a replace. The graph core's default is a shallow merge (`write` defaults to `upsert`, `index.d.ts:1562`). The baseline verdicts assume merge; the spec requires the same outcome either way.
- Whether `ConfiguredModel` (enabled models only) can replace the models page's configured
  rows, which today include every entry in `provider.models`.

## Rules applied (task-specific summary)

Vercel React Best Practices: `rerender-derived-state` and `rerender-defer-reads` (a control
subscribes to one primitive, not to a list; a callback-only read uses `getState` inside the
handler); `rerender-split-combined-hooks` (separate hooks for separate fields);
`rerender-no-inline-components`; `rendering-content-visibility` and the virtualization note
in candidate cand-003 for the 8k-row list; `bundle-barrel-imports` (the new domain is
imported through the existing `@/platform/entities` entry; model pickers use a named narrow
entry as `@/features/models/model-selector` does). Vercel Composition Patterns:
`state-decouple-implementation` (components know the hook interface, not the store) and
`state-context-interface` (the domain exposes state, actions and meta). Prometheus Entity
Management and the project rule: records that are persistent, shared or server-confirmed
(Provider, ProviderMeta, Model, ConfiguredProvider, ConfiguredModel, Thread) live in the
graph and reach components through typed hooks from `@/platform/entities`; domain actions
sequence mutations; transports own I/O; a fetched list is ingested by one atomic action
(`batchGraphWrites`), never one write per row; `useState` and transient Zustand are for
UI-local mechanics only. `entity-graph-optimize` memory note: the graph has no automatic
garbage collection, so the catalog action removes rows a refresh no longer lists.

## Decisions

**D1. One new platform domain.** Add `frontend/src/platform/entities/model-catalog/`
(proposed name) beside `session-configuration`, with `contracts.ts`, `domain.ts`,
`use-model-catalog.ts`, `registration.ts` and an `api/` folder, exported from
`platform/entities/index.ts`. It owns the `Provider`, `ProviderMeta` and `Model` catalog
entities. `ConfiguredProvider` and `ConfiguredModel` stay in `session-configuration`
(spec `session-configuration` owns them); this change only adds the missing
`useConfiguredProviders` hook there if the models page needs it. Rejected: extending
`session-configuration` (different spec, `authoritative: false` transports, session scope)
and keeping the logic in `features/models` (features cannot import each other except
through narrow entries, and `providers` needs it too).

**D2. Identity.** `catalogModelId(providerId, modelId) = providerId + "/" + modelId`,
exported once. The id is an opaque key. Model ids can contain `/` (for example
`features/agents/ui/agent-editor.tsx:71` splits route strings on the first slash for a
different purpose), so splitting an entity id is unsound; consumers read `provider_id` and
`model_id`. The Rust side already emits the same string, so no backend change is needed. A
new gate rule `feature-model-id-construction` flags a template-string key built outside the
domain.

**D3. One projection, applied at both entry points.** `projectCatalogModel` takes the REST
catalog entry (nested `limit`, `modalities`, `cost`) or a snapshot record (the same nested
fields plus `id`, `model_id`, `provider_id`) and returns the flat record. REST hydration
calls it directly. Snapshot ingestion calls it through an optional `projectRecord` hook on
`createUarSseAdapter` (`uar-sse-adapter.ts`), supplied for the `models` topic from
`UAR_TOPICS` (`lib/realtime/topics.ts:16`). `provider_name` and `provider_configured` are
absent from the raw snapshot record; the projection takes them from the `Provider` entities
already in the graph, with the provider id as the visible fallback. Rejected: changing
`lifecycle_snapshot.rs` to emit the flat shape (cross-domain, and REST would still need the
same projection). If the state owner removes the `models` subscription (analysis build item
14), the hook becomes unused but the projection requirement still holds for REST.

**D4. Atomic ingestion.** `ingestModelCatalog(payload)` runs synchronously inside one
`batchGraphWrites`: `upsertEntities("Model", rows)`, `removeEntity` for ids absent from the
payload, and `setListIds` for the list key. `ingestProviderRegistry(catalog, configured)`
does the same for `Provider`, `ProviderMeta` and the provider list. Transports fetch first
and call the action with parsed data, so a failed fetch writes nothing. A snapshot record
without `provider_id` or `model_id` aborts that ingestion with a visible error. The existing
adapter already returns `null` and marks lists stale for a duplicate or empty id
(`uar-sse-adapter.ts:258-271`); this change keeps that behavior.

**D5. Retire the duplicates.** Delete `provider-models-store.ts` and
`use-provider-models.ts`; the providers detail panel reads
`useCatalogModelsForProvider(providerId)` from the domain, which removes the per-panel
full-catalog fetch. Remove `ModelsState.configuredProviders` from `models-store.ts`; keep
`refreshing`, `busyModelKey` and `error` as transient workflow state. Delete the dead loaders
(`loadModelsIntoGraph`, `loadModelsForProvider`, `loadProvidersIntoGraph`) with
`models-graph.ts` and `providers-graph.ts`. `providers-store.ts` keeps its workflow fields;
its `hydrateProviders` moves into the domain. Add, default and remove for provider models
become domain actions that call the existing provider update API and re-ingest on success.
`use-provider-default.ts` reads `ProviderMeta` through a domain hook, so its two
`useGraphStore` selectors disappear.

**D6. Threads (flagged for Plan).** Two options: (a) the graph `Thread` entity is the single
owner and `thread-registry-store` keeps only ephemeral and active-thread state; (b) keep the
registry as owner and remove the graph `Thread` slice. Option (a) matches the project rule
and the existing `threads` topic. The PGlite write path (`getDbInstance().upsertThread`,
`stores/thread-registry-store.ts:60`) belongs to `uiux-local-first-persistence-worker`;
sequence that change first, or define the hook interface here and let it adopt it. Task
group 7 is separable and may move to a child phase.

**D7. Facade leak by staging.** `runtime.ts` stops re-exporting `useGraphStore` and
`graphStore` last, after every file in the baseline has moved. Until then the gate rule
`feature-raw-graph-access` enforces "no new use" against a baseline keyed by file and rule.
Migrated domains (models, providers) start at zero. The rule and baseline live in `scripts/`
(outside console ownership; see Unresolved).

## Interfaces and cross-change contracts

- **Provides** (proposed): `useCatalogModels()`, `useCatalogModel(id)`,
  `useCatalogModelsForProvider(providerId)`, `useCatalogProviders()`,
  `useProviderMetaField(field)`, `ingestModelCatalog`, `ingestProviderRegistry`,
  `catalogModelId`, `projectCatalogModel`. `uiux-schema-controls-and-pickers` consumes the
  model hooks for catalog-backed pickers; `uiux-system-model-live-default` consumes
  `ProviderMeta`.
- **File sharing hazards.** `features/providers/model/providers-store.ts` is also touched by
  `uiux-system-model-live-default` (`setDefault`, build item 12), and `lib/realtime/topics.ts`
  by `uiux-realtime-topics-and-replica-refresh`. Order this change first for
  `providers-store.ts` and keep its edit to moving `hydrateProviders`. One writer per file per
  Plan wave.
- **Console-A overlap.** About 8k `Model` entities sit in the persisted graph snapshot.
  `uiux-local-first-persistence-worker` (build item 5) decides whether catalog entities are
  persisted at all. This change works either way and does not decide it.

## Frozen oracle fixtures

Written now, before any implementation, so no expected value comes from code the
implementer writes or from the system under test (revision rule R1).

| Fixture | Path | Holds |
|---|---|---|
| Catalog identity oracle | `openspec/changes/uiux-model-identity-graph-ownership/fixtures/catalog-identity.json` | The SHA-256 of the committed catalog snapshot (`catalog/provider_catalog.json`, embedded by `build.rs:40-52`), and counts computed from that file alone: 343 providers, 8,170 (provider, model) pairs, 3,827 distinct bare model ids, 1,123 bare ids listed by two or more providers, 2,207 ids containing a slash. Bare-id keying would collapse 8,170 rows to 3,827. It also holds 65 expected sample rows (id, provider, model, context, tool_call, reasoning, vision, costs) and the per-provider lists for three shared ids. The context rule is `limits.context_window` if greater than 0, else 128000 (`src/server.rs:3949-3953`). |
| Collision config | `openspec/changes/uiux-model-identity-graph-ownership/fixtures/collision-config.yaml` | Two custom providers (`idn-gw-a`, `idn-gw-b`) listing the same bare model id `idn-shared-model` with different context, vision and tool flags, plus a slash id `idn-org/slash-model`. Expected rows are in the JSON fixture under `customProviderCollisionConfig`. With this config the catalog total is 8,173 and the provider count is 345. |
| Provider-model sequence | `openspec/changes/uiux-model-identity-graph-ownership/fixtures/provider-model-sequence.json` and `provider-model-config.yaml` | The frozen add, default, remove and external-add steps for provider `nano-gpt` (M0 to M2 are its first three catalog ids, digest-pinned), the expected end state (models {M1, M2}, default M1), the store query and read-back tool, the expected UI rows, and the three wrong builds (baseline, two mutants) with their expected failures. |
| Baseline revision | `f2c9e36efe088e06a17c5d880ada877e25f2ad52` (HEAD when this revision was written) | The pre-phase tree. Its frontend bundle is the negative control for SCN-IDN-003, -004 and -007, and the confirmed-verdict reference for every mandatory row in `fixtures/baseline-verdicts.json`. |
| Thresholds | this document | A long task above 200 ms fails (the project INP target). The unfiltered Models list renders at most 200 row elements while the page reports 8,173 models. |
| Gate negative fixtures | names frozen in B5 | The six fixture cases the architecture gate must reject. |

The snapshot digest is checked first: if the committed catalog changes, the fixture counts are
stale and the runner reports BLOCKED, not FAIL.

## Acceptance

Common real collaborators: the runtime binary with its production static bundle, a scratch
persistence directory per run, the collision config above, the repository stub model server
for provider upstreams, and Playwright Chromium. The stub is `src/bin/stub-llm.rs`, started as
`target/debug/stub-llm <fixtures.json>` with `STUB_LLM_PORT` set (its header comment and
`frontend/playwright.real-server.config.ts:35-41`, fixture `tests/bdd/fixtures/bdd-chat.json`).
No scenario here calls a paid model route. Identity comes from the existing external proxy;
tests never mint, carry or send credentials or JWTs. Ports are scratch ports, never the live
`1906` service. Unit, mocked and structural checks are supplemental only. A mandatory
scenario whose negative control did not run is BLOCKED, never PASS.

Common prerequisites (the lead adds the build commands): the runtime and `stub-llm` binaries;
for scenarios that name the baseline, a worktree created with
`scripts/worktree-new.sh uiux-idn-baseline --base f2c9e36efe088e06a17c5d880ada877e25f2ad52`
(under `~/.claude/worktrees/`, per the project rule) and that tree's production frontend build.
The backend is unchanged by this change, so the same runtime binary serves both bundles; how a
bundle is selected at run time is UNVERIFIED here and is bound by the lead.

### Behavior B1: The catalog has one provider-qualified identity

| Field | Required declaration |
|---|---|
| Canonical identity | `uar-uiux-architecture-2026-11` / `uiux-model-identity-graph-ownership` / `uiux-model-identity-graph-ownership#8.1` |
| Source identity | repository HEAD plus the working tree at the boundary; production bundle digest and catalog snapshot digest recorded |
| Production entry point | The operator opens `/admin/models` in the production UI served by the runtime started with the collision config |
| Real collaborators | runtime binary, the real `GET /api/models` payload (8,170 bundled pairs plus the two custom providers), browser graph store, PGlite persistence |
| Boundary exercised | HTTP `GET /api/models`; browser graph publication; rendered DOM (`models-count` and "of N in catalog" at `models-page.tsx:187-188`, `compare-checkbox-<key>` per row) |
| Observable result | The page reports 8,173 models in the catalog and 345 providers. Filtering by `claude-opus-5` shows one row per provider listed in `sharedBareIdSamples`. Filtering by `idn-shared-model` shows two rows, `idn-gw-a/idn-shared-model` with context 64,000 and vision, and `idn-gw-b/idn-shared-model` with context 32,000 and no vision. Filtering by `slash-model` shows one row whose model id is exactly `idn-org/slash-model` |
| Negative control | Mutant "bare id": the candidate at the implementation-complete commit, in a worktree, with `catalogModelId` changed to return the bare model id. It must FAIL the row-count assertion (8,173 would collapse to about 3,830). A mutant that passes makes the scenario BLOCKED. The independent reviewer writes the one-line patch after implementation |
| Isolation | scratch data dir, scratch port, fresh browser profile; cleanup by the Playwright global teardown |
| Prerequisites | runtime binary, `stub-llm`, catalog snapshot digest equal to the fixture; a the mutant worktree; runs on embedded Surreal, no remote store needed |
| Final local gate | from the repository root: `pnpm -C frontend exec playwright test -c playwright.real-server.config.ts e2e/model-catalog-identity.spec.ts`, run once after the phase integration build |
| Evidence | pending |
| Limitations | Reads the entity set through the rendered UI (the graph is not exposed on `window`). This is a regression guard: on the pre-phase tree the REST path already keys by the qualified id, so the baseline PASSES it (see `fixtures/baseline-verdicts.json`); the mutant, not the baseline, proves the runner can reject. Nondeterministic: 3 samples, each must pass |

### Behavior B2: A realtime snapshot leaves the models surface unchanged

| Field | Required declaration |
|---|---|
| Canonical identity | `uiux-model-identity-graph-ownership#8.2` |
| Source identity | as B1 |
| Production entry point | The operator keeps `/admin/models` open against a remote store while the page reconnects its live stream (a new connection runs the initial `subscription_established` snapshot, `src/uar/api/live.rs:81`) |
| Real collaborators | runtime with Postgres or remote Surreal persistence (the `/api/live` path), `GET /api/live`, `GET /api/live/models/snapshot` |
| Boundary exercised | SSE connect, reset event, snapshot fetch, graph change set, rendered rows |
| Observable result | After the reconnect, every one of the 65 frozen sample rows and the three custom rows shows the frozen context, tool, reasoning, vision and cost values, and the page still reports 8,173 models. Each row's displayed values also equal the same model's entry in the live `GET /api/models` response, whose `limit`, `modalities` and `cost` are read as the server's truth |
| Negative control | A snapshot record missing `model_id`, delivered on `GET /api/live/models/snapshot` (the harness fulfils that one response; every other request reaches the real server): the page keeps all 8,173 rows with unchanged values and an ingestion error is visible. This is a declared substituted boundary and its evidence is never promoted to a real-collaborator claim. The baseline lacks the visible ingestion error, so the baseline FAILS this control |
| Isolation | remote-store fixture owned by `uiux-realtime-topics-and-replica-refresh`; scratch database name and port; teardown drops it |
| Prerequisites | RTM's remote-store fixture; the `postgres-backend` build for Postgres (analysis delta 1); the baseline worktree and bundle (needed only to confirm the baseline verdict) |
| Final local gate | `pnpm -C frontend exec playwright test -c <remote-store Playwright config named by the lead> e2e/model-catalog-snapshot.spec.ts` |
| Evidence | pending |
| Limitations | The value assertions are a regression guard: the graph's shallow merge keeps REST-hydrated flat fields, so the baseline passes them; the new behavior is the visible rejection of a bad snapshot record. If the realtime change removes the `models` subscription, no snapshot writer exists and this scenario is BLOCKED; the requirement stays covered by B1. Embedded Surreal uses a different adapter (`/api/uar/sync/stream`) and is not covered. Nondeterministic: 5 samples, all must pass |

### Behavior B3: Loading the full catalog does not stall the page

| Field | Required declaration |
|---|---|
| Canonical identity | `uiux-model-identity-graph-ownership#8.3` |
| Source identity | as B1 |
| Production entry point | Navigate to `/admin/models`, then type in the model filter |
| Real collaborators | runtime with the bundled catalog (8,170 pairs plus the custom providers), browser |
| Boundary exercised | browser main thread during hydration; filter input latency |
| Observable result | The `PerformanceObserver` long-task list for the load and for the first filter keystroke contains no task above 200 ms; the filtered count updates after one keystroke |
| Negative control | The baseline bundle on the same machine must show at least one long task above 200 ms in at least 3 of the 5 samples. If it does not, the 200 ms threshold does not discriminate and the scenario is BLOCKED until the operator replaces the threshold. The baseline revision is frozen above; its measurement is taken by the runner at the boundary, because measuring now would be product verification before the implementation-complete inventory |
| Isolation | as B1 |
| Prerequisites | as B1 plus the baseline worktree and bundle |
| Final local gate | same spec file as B1 |
| Evidence | pending |
| Limitations | Timing depends on hardware; the receipt records the machine. A single graph publication is also enforced structurally by the gate (B5), a supplemental check. Nondeterministic: 5 samples, grader is the maximum long task per sample under 200 ms, all must pass |

### Behavior B4: Adding, defaulting and removing a provider model reaches the store, and no second cache goes stale

| Field | Required declaration |
|---|---|
| Canonical identity | `uiux-model-identity-graph-ownership#8.4` |
| Source identity | as B1, plus the digest of `fixtures/provider-model-sequence.json` |
| Production entry point | `/admin/models` with the scratch config `fixtures/provider-model-config.yaml` (provider `nano-gpt`, seeded with model M0 as default). The operator runs the frozen steps: add M1, set M1 as default, remove M0. A second browser page is left open on `/admin/models` and is never reloaded; the harness then adds M2 with `PUT /api/uar/providers/nano-gpt`, a write that does not go through the console |
| Real collaborators | runtime, the scratch embedded store, the host fixture tool `surrealql_store_runner` (`scripts/acceptance/fixtures/surrealql_store_runner.rs`, `[[example]]` at `Cargo.toml:155-158`), two browser pages |
| Boundary exercised | the console's provider update path; the persisted provider row; the live refresh of a page that did not initiate the change |
| Observable result | The expected end state is frozen in `provider-model-sequence.json`: models {M1, M2}, default M1, where M0, M1 and M2 are the first three catalog model ids of `nano-gpt` (all containing a slash). Truth is the frozen file, compared with two readers that are not the console under test. (1) The runtime is stopped and the persisted row `provider.nano-gpt` is read directly from the scratch store with `surrealql_store_runner` (stopped store, as the 2026-10-29 gotcha requires); its model ids as a set and `default_model` must equal the frozen end state. (2) The runtime is restarted and the UI must render exactly the configured-model rows for M1 and M2 (`configured-model-nano-gpt/<model>`, `models-page.tsx:579`), the default marker on M1, and no M0 row. After step 4 the second page shows the M2 row without a reload within 15 seconds |
| Negative control | Three wrong builds must each FAIL, and a wrong build that PASSes makes the scenario BLOCKED: (a) baseline `f2c9e36efe088e06a17c5d880ada877e25f2ad52`, built in a worktree from `scripts/worktree-new.sh`, fails at step 4 because its configured list is `useModelsStore.configuredProviders`, loaded once (`models-store.ts:86-90`), so the second page stays stale; (b) mutant "dropped remove": the candidate at the implementation-complete commit in a worktree, with the domain action that removes a provider model changed to update the graph and return without calling the provider update API, fails at the store read-back because M0 is still stored; (c) mutant "dropped add", the same for the add action, fails at step 1. The mutant patches are written by the independent reviewer after implementation, not by the implementer |
| Isolation | scratch data dir and port; store read only while the runtime is stopped; cleanup removes the scratch dir and the worktrees |
| Prerequisites | the `surrealql_store_runner` example compiled (it is not built by an ordinary compile, `Cargo.toml:157`); the baseline and mutant worktrees; the realtime change's provider-topic fix so a change made by another client reaches the second page (analysis delta 1: on Surreal a provider change invalidates `settings`, not `providers`); the runtime on embedded Surreal. The exact table and field names in the store query are UNVERIFIED (provider rows are `settings` rows keyed `provider.{id}`, `manager.rs:975,1050`) and the lead confirms them when binding the runner |
| Final local gate | `pnpm -C frontend exec playwright test -c playwright.real-server.config.ts e2e/provider-model-roundtrip.spec.ts` for the UI steps, then the store read-back by the runner; 3 samples |
| Evidence | pending |
| Limitations | The store is read only after the runtime stops, so it proves persistence of the final state, not each step. The 15 second wait is a proposal. Mutant construction depends on the final function names. Nondeterministic (realtime, rendering): 3 samples, all must pass |

### Behavior B5 (supplemental static gate, not an acceptance scenario): The architecture gate rejects the failure shapes and cannot grow

| Field | Required declaration |
|---|---|
| Canonical identity | `uiux-model-identity-graph-ownership#8.5` |
| Source identity | repository HEAD at the boundary; base ref `UAR_BOUNDARY_BASE_REF` or `origin/main` |
| Production entry point | `pnpm frontend:boundaries` (root `package.json:47`) |
| Real collaborators | the real gate script, the TypeScript compiler API, git merge-base, the negative-fixture directory |
| Boundary exercised | filesystem scan of `frontend/src`; `git show <base>:scripts/frontend-boundary-allowlist.txt` |
| Observable result | exit 0 on the tree; non-zero with the rule id and location for each negative fixture |
| Negative control | frozen fixture names: `store-uarprovider-array`, `store-providermodelrow-record`, `feature-imports-usegraphstore`, `template-string-model-key`, `baseline-entry-added`, `baseline-entry-stale`; each must make the gate exit non-zero with its rule id |
| Isolation | read-only on the tree; fixtures under `scripts/fixtures/frontend-boundary-cases/` |
| Prerequisites | git history available |
| Final local gate | `pnpm frontend:boundaries` from the repository root |
| Evidence | pending |
| Limitations | Static analysis. The spec packet says structural checks cannot establish acceptance, so this is a supplemental gate; behavior is proved by B1 to B4 and B7 |

### Behavior B6: A server-confirmed thread has one owner

| Field | Required declaration |
|---|---|
| Canonical identity | `uiux-model-identity-graph-ownership#8.6` |
| Source identity | as B1 |
| Production entry point | The chat sidebar: create a conversation, rename it on the server, delete it on the server |
| Real collaborators | runtime sessions API, realtime `threads` topic, PGlite, browser |
| Boundary exercised | `GET /api/uar/sessions`; sidebar DOM; local persisted thread row |
| Observable result | After a server-side rename the sidebar title equals the title in `GET /api/uar/sessions`; after a server-side delete the row is gone from the sidebar and from the local thread store after reload |
| Negative control | The same rename against the baseline bundle is the control: the registry-to-graph mirror hook must fail to apply a title change the new owner applies, or the scenario is BLOCKED (the baseline mirror may already pass; if so this scenario cannot reject and stays optional) |
| Isolation | as B1 |
| Prerequisites | as B1 |
| Final local gate | `pnpm -C frontend exec playwright test -c playwright.real-server.config.ts e2e/thread-ownership.spec.ts` |
| Evidence | pending |
| Limitations | Optional: task group 7 may move to a child phase, and the lead must then drop thread retirement from the functional scope. Nondeterministic: 3 samples, all must pass |

### Behavior B7: The Models list renders only the visible rows

| Field | Required declaration |
|---|---|
| Canonical identity | `uiux-model-identity-graph-ownership#8.7` |
| Source identity | as B1 |
| Production entry point | `/admin/models` with no filter, then scroll to the end of the list |
| Real collaborators | runtime with the bundled catalog and the collision config, browser |
| Boundary exercised | rendered DOM of the catalog list (`compare-checkbox-<key>` elements) |
| Observable result | With the page reporting 8,173 models, at most 200 catalog row elements are in the DOM at any time; scrolling to the end brings the last row into view; keyboard focus moves between rows without losing its place |
| Negative control | The baseline bundle must render more than 8,000 row elements for the same page (source: `models-page.tsx:326` maps every filtered row); if it renders 200 or fewer, the scenario cannot reject and is BLOCKED |
| Isolation | as B1 |
| Prerequisites | as B3 |
| Final local gate | same spec file as B1 |
| Evidence | pending |
| Limitations | The 200-element cap is a frozen proposal. Nondeterministic: 3 samples, all must pass |

## Scenario matrix rows

For every row: fixtures are the frozen files above plus a scratch data directory and scratch
port with the stub model server; substituted boundaries are none unless stated; restoration is
to stop the runtime and stub, delete the scratch directory, close the browser context and remove
the baseline worktree with `scripts/worktree-rm.sh`. Owner tasks refer to `tasks.md` of this
change. Nondeterministic rows carry a stochastic block.

| Field | SCN-IDN-001 | SCN-IDN-002 | SCN-IDN-003 |
|---|---|---|---|
| name | Catalog model identity survives shared bare model ids | Realtime snapshot keeps the models surface intact | Full catalog load keeps the page responsive |
| actor | Operator browsing models | Operator with the models page open on a remote store | Operator opening the models page |
| useful outcome | Every provider's model is shown once, under its own provider | Reconnect does not blank or distort rows | The page is usable while 8k models load |
| source requirement | `openspec:frontend-react-entity-guardrails#Catalog Model identity is provider-qualified and defined once` | `openspec:frontend-react-entity-guardrails#Catalog Model records have one shape for every writer` | `openspec:frontend-react-entity-guardrails#Provider and model catalog data enter the graph in one atomic action` |
| reason now | Two live identity builders and a dead bare-id writer; 1,123 bare ids are shared in the committed catalog | The snapshot runs on every connect and writes a different record shape (source-derived) | The 8k catalog was 16,360 writes in an earlier lesson (`prior-context.md`) |
| likelihood | likely (1,123 shared ids exist in the snapshot) | UNKNOWN (runtime unverified) | likely (per-row loop over 8,170 rows) |
| mandatory | true | true | true |
| nondeterministic | true | true | true |
| stochastic | samples 3; grader: page reports 8,173 models and every frozen sample row matches; stoppingRule fixed N, all must pass | samples 5; grader: every frozen row equal after reconnect and total unchanged; stoppingRule fixed N, all must pass | samples 5; grader: maximum long task under 200 ms; stoppingRule fixed N, all must pass; baseline must exceed 200 ms in at least 3 of 5 |
| production entry | UI `/admin/models` with the collision config | UI `/admin/models`, forced live-stream reconnect | UI `/admin/models`, filter input |
| real collaborators | runtime, real `/api/models`, browser, PGlite | runtime on Postgres or remote Surreal, `/api/live`, snapshot route | runtime, bundled catalog, browser |
| oracle | observation: models count text and rows; truth source: `catalog-identity.json` and `collision-config.yaml` (frozen); comparator: exact count and per-row field equality | observation: rows after reconnect; truth source: frozen sample rows plus the live `/api/models` entry for each row; comparator: field equality | observation: long-task durations; truth source: the frozen 200 ms threshold; comparator: maximum below threshold |
| negative control | input: mutant with `catalogModelId` returning the bare id; expected: row-count assertion fails (else BLOCKED) | input: snapshot record missing `model_id`; expected: rows unchanged and a visible ingestion error (baseline fails: no error shown) | input: baseline bundle; expected: long task above 200 ms in at least 3 of 5 samples (else BLOCKED) |
| fixtures | `fixtures/catalog-identity.json`, `fixtures/collision-config.yaml`, scratch dir and port, mutant worktree | same plus remote-store fixture and baseline worktree `~/.claude/worktrees/uiux-idn-baseline` | same as 001 plus baseline worktree |
| evidence required | Playwright report, `/api/models` digest, catalog snapshot digest, screenshots of the two `idn-shared-model` rows | report, snapshot request log, per-row comparison table for baseline and candidate | report, long-task lists for candidate and baseline, machine description |
| substituted boundaries | the one fulfilled snapshot response in the negative control only | none | none |
| limitations | graph not read directly; regression guard, the baseline passes | value assertions are a regression guard (shallow merge); needs the RTM fixture; embedded adapter excluded | hardware dependent |
| invalidation keys | `platform/entities/model-catalog/**`, `features/models/**`, `catalog/provider_catalog.json`, `src/server.rs` `/api/models` route | plus `lib/realtime/**`, `src/server/lifecycle_snapshot.rs`, `src/uar/api/live.rs` | plus `components/ui/**` list rendering, `models-page.tsx` |
| restoration | scratch dir removed | scratch database dropped | baseline worktree removed |
| owner tasks | `uiux-model-identity-graph-ownership#8.1` with 1.x, 2.x | `#8.2` with 3.x | `#8.3` with 2.x, 5.6 |

| Field | SCN-IDN-004 | SCN-IDN-005 | SCN-IDN-006 | SCN-IDN-007 |
|---|---|---|---|---|
| name | Provider model changes reach the store and no second cache goes stale | Gate rejects duplicate caches and raw graph access, and cannot grow | Thread list has one owner | Models list renders only visible rows |
| actor | Operator managing provider models | Developer running the local gate | Operator creating and renaming conversations | Operator scrolling the catalog |
| useful outcome | A dropped action or a stale duplicate store is caught | Regressions are caught before merge | No duplicate or stale thread rows | Page stays responsive with 8k rows |
| source requirement | `openspec:frontend-react-entity-guardrails#Feature code reaches provider and model records only through platform domain hooks` | `openspec:frontend-react-entity-guardrails#Raw graph access in feature code is limited to a shrink-only baseline` | `openspec:frontend-react-entity-guardrails#Server-mirrored thread records have one owner` | `openspec:frontend-react-entity-guardrails#Large catalog lists render only visible rows` |
| reason now | Retires `useModelsStore.configuredProviders` and `useProviderModelsStore` | 28 feature files import the raw graph store and the rule misses DTO names | The registry mirrors the graph through a raw subscription | `models-page.tsx:326` renders every filtered row |
| likelihood | likely | likely | UNKNOWN | likely |
| mandatory | true | false (supplemental static gate) | false (may move to a child phase) | true |
| nondeterministic | true | false | true | true |
| stochastic | samples 3; grader: store and UI equal the frozen end state and the second page updates; stoppingRule fixed N, all must pass | not applicable | samples 3; grader: sidebar equals sessions API; stoppingRule fixed N, all must pass | samples 3; grader: at most 200 row elements and last row reachable; stoppingRule fixed N, all must pass |
| production entry | UI `/admin/models` | `pnpm frontend:boundaries` | UI chat sidebar | UI `/admin/models`, scroll |
| real collaborators | runtime, scratch embedded store, `surrealql_store_runner`, two browser pages | gate script, TypeScript compiler, git | runtime sessions API, `threads` topic, PGlite, browser | runtime, bundled catalog, browser |
| oracle | observation: persisted provider row read from the stopped scratch store, and rendered configured rows; truth source: `provider-model-sequence.json` (frozen end state: models {M1, M2}, default M1); comparator: set equality of model ids, exact default, exact row set | observation: exit code and rule ids; truth source: the six frozen fixture names; comparator: set equality | observation: sidebar rows after a server rename; truth source: `GET /api/uar/sessions`; comparator: title and order equality | observation: count of row elements and last-row reachability; truth source: 8,173 models and the 200 cap, frozen; comparator: at most the cap |
| negative control | input: baseline build and two mutants (dropped remove, dropped add); expected: baseline fails at step 4 (stale second page), mutants fail at the store read-back (else BLOCKED) | input: the six frozen negative fixtures; expected: non-zero and rule id for each | input: baseline bundle rename; expected: baseline mirror differs, else BLOCKED | input: baseline bundle; expected: more than 8,000 row elements (else BLOCKED) |
| fixtures | `provider-model-sequence.json`, `provider-model-config.yaml`, scratch dir | `scripts/fixtures/frontend-boundary-cases/**` | scratch dir and port | collision config, baseline worktree |
| evidence required | report, store query output, row list for candidate, baseline and mutants | gate stdout and exit code | report, request log | report, DOM counts for candidate and baseline |
| substituted boundaries | none | none | none | none |
| limitations | store read after stop; 15 s wait is a proposal; mutants depend on final names | static analysis | slicing pending Plan (design D6) | cap is a frozen proposal |
| invalidation keys | `features/models/**`, `features/providers/**`, `platform/entities/model-catalog/**`, `src/uar/api/providers.rs`, `catalog/provider_catalog.json` | `scripts/check-frontend-boundaries.mjs`, `scripts/frontend-boundary-allowlist.txt`, `frontend/src/**` imports | `stores/thread-registry-store.ts`, `stores/use-thread-graph-sync.ts`, `src/uar/realtime/**` | `features/models/ui/models-page.tsx`, `components/ui/**` |
| restoration | scratch dir and worktrees removed | none | scratch dir removed | baseline worktree removed |
| owner tasks | `#8.4` with 4.x, 5.x | `#8.5` with 6.x | `#8.6` with 7.x | `#8.7` with 5.6 |

## Risks and trade-offs

- **A shared projection hides a backend mismatch.** If the Rust snapshot record changes, the
  projection must follow. Mitigation: it tolerates nested and flat inputs and rejects
  records without `provider_id` or `model_id` visibly.
- **Baseline growth by the new rule.** A new rule needs initial entries, which the existing
  allowlist forbids (`boundary-allowlist-growth`). The change records its baseline in the
  script's inline-set style (existing-debt precedent). The lead must confirm that this change
  counts as the architecture decision the allowlist file requires.
- **Models page parity.** Replacing the Zustand `configuredProviders` with graph entities
  could drop disabled models from the configured list (UNVERIFIED, see above). Plan decides
  whether to add a `ProviderModelConfig` projection.
- **Phase size.** 28 feature files in the baseline are not migrated here. The rule stops the
  leak from spreading; closing it is a later phase. This is the uncomfortable part: after
  this change the facade still leaks for every other domain, and the baseline only holds the
  line.

## Unresolved decisions for the lead and operator

1. Single-writer assignment for `scripts/check-frontend-boundaries.mjs`,
   `scripts/frontend-boundary-allowlist.txt`, `scripts/test-frontend-boundaries-negative.mjs`
   and `scripts/fixtures/frontend-boundary-cases/**` (outside `uar-console` paths).
2. Thread registry slicing (D6): same phase or child phase, and ordering against
   `uiux-local-first-persistence-worker`.
3. Whether the `models` topic subscription stays (analysis item 14). It changes B2, not the
   requirements.
4. Whether the new gate rule's initial baseline is accepted as the architecture decision the
   allowlist file requires.
5. Whether the 200 ms and 200-row thresholds stand once the baseline is measured at the boundary (a baseline that does not exceed them makes SCN-IDN-003 and -007 BLOCKED).
6. Contract level (lead): functional scope must drop thread retirement if SCN-IDN-006 stays optional, and SCN-IDN-005 is a supplemental gate, not an acceptance row.
7. A one-line comment fix at `src/server/lifecycle_snapshot.rs:179-182` (state owner).
