## MODIFIED Requirements

### Requirement: Architecture validation rejects the observed failure shapes
The local frontend architecture gate SHALL reject synchronous state setters in a
component render body, feature-owned loops that issue one graph mutation per row,
direct entity-package imports outside the platform facade, and duplicate feature
caches for the graph-owned configured Provider, Model, AgentSession, or
AgentSessionDraft records. The duplicate-cache rule SHALL cover the catalog
`Provider`, `ProviderMeta` and `Model` entities and the data-transfer shapes that
carry the same records (`UarProvider`, `ProviderModelRow`, `CatalogModelsResponse`,
`ProviderEntity`, `ModelEntity`, `ConfiguredProvider`, `ConfiguredModel`), whatever
the store variable is named. It SHALL allow UI-local state and event-driven updates
that do not duplicate business entities.

#### Scenario: A forbidden fixture is checked
- **WHEN** a fixture contains any named forbidden pattern
- **THEN** the local gate exits non-zero with a deterministic rule identifier and source location

#### Scenario: A valid narrow subscriber is checked
- **WHEN** a component obtains one field through a platform domain hook and mutates it only from an event handler
- **THEN** the local gate accepts the component

#### Scenario: The gate's negative controls run
- **WHEN** the validation fixtures intentionally contain each forbidden pattern
- **THEN** every negative fixture is observed to fail before the gate is accepted

#### Scenario: A store caches provider rows under a DTO name
- **WHEN** a feature file creates a Zustand store whose state holds `UarProvider[]`, `ProviderModelRow[]` or `Record<string, ProviderModelRow[]>`
- **THEN** the gate reports `duplicate-graph-owned-cache` with the source location of the first matching type reference

#### Scenario: A store holds only workflow state for the same feature
- **WHEN** a feature store holds only busy, error, refreshing or saving flags and no provider or model record shape
- **THEN** the gate accepts it

## ADDED Requirements

### Requirement: Catalog Model identity is provider-qualified and defined once
A catalog `Model` entity SHALL be identified by `${provider_id}/${model_id}`, built by one
exported function in the platform provider-and-model domain. Every writer of `Model`
entities SHALL obtain the id from that function or from the same value the server emits.
Code SHALL NOT split a `Model` entity id to recover its parts.

#### Scenario: Two providers list the same bare model id
- **WHEN** the catalog lists model id `m` under providers `a` and `b`
- **THEN** the graph holds two `Model` entities, `a/m` and `b/m`
- **AND** each carries its own `provider_id`, `provider_name` and `provider_configured`
- **AND** the models surface renders both rows

#### Scenario: A model id contains a slash
- **WHEN** a provider lists model id `org/model-x`
- **THEN** the entity id is `<provider_id>/org/model-x`
- **AND** the entity's `model_id` field is exactly `org/model-x`
- **AND** no consumer derives either field by splitting the entity id

#### Scenario: A feature file builds a model key on its own
- **WHEN** a feature file other than the provider-and-model domain constructs a `Model` entity id by template string
- **THEN** the architecture gate reports rule `feature-model-id-construction` with the source location

#### Scenario: Every hydration path has run
- **WHEN** REST hydration and a realtime snapshot have both written the Model slice
- **THEN** every Model entity id starts with its own `provider_id` followed by `/`
- **AND** no Model entity is keyed by a bare catalog model id

### Requirement: Catalog Model records have one shape for every writer
Every `Model` entity record SHALL be produced by one projection function that accepts a
REST catalog entry or a realtime snapshot record and returns the same flat record.
Capability flags SHALL be derived from the catalog entry and never hard-coded. Applying
a realtime snapshot SHALL NOT change what the models surface renders for a model whose
catalog entry is unchanged.

#### Scenario: A snapshot follows REST hydration
- **WHEN** the models surface has hydrated from REST and a realtime snapshot for the `models` topic then replaces the Model set with unchanged catalog data
- **THEN** every row shows the same context window, cost, capability flags and provider name as before the snapshot

#### Scenario: A catalog entry accepts image input
- **WHEN** a catalog model lists `image` among its input modalities
- **THEN** its `vision` field is true regardless of which writer produced the record

#### Scenario: A snapshot record is missing an optional field
- **WHEN** a snapshot record lacks `family` or `benchmarks`
- **THEN** the projection supplies the documented default and the record is still ingested

#### Scenario: A snapshot record is missing its identity fields
- **WHEN** a snapshot record lacks `provider_id` or `model_id`
- **THEN** the ingestion fails with a visible error and the previous Model entities remain
- **AND** the record is not stored under a guessed id

### Requirement: Provider and model catalog data enter the graph in one atomic action
Hydrating the model catalog SHALL publish all `Model` rows, and the removal of rows the
new catalog no longer lists, in one atomic graph publication. Hydrating the provider
registry SHALL do the same for `Provider`, `ProviderMeta` and the provider list. Feature
code SHALL NOT issue one graph write per row.

#### Scenario: The full catalog is loaded
- **WHEN** the models surface loads a catalog of about 8,000 models
- **THEN** the Model slice is published once for that load
- **AND** the page stays interactive while it loads

#### Scenario: A catalog refresh drops a model
- **WHEN** a later catalog response no longer lists a model that the graph holds
- **THEN** the same publication removes that model's entity

#### Scenario: The catalog fetch fails
- **WHEN** the catalog request fails after a previous successful load
- **THEN** the previously loaded Model entities remain
- **AND** the surface reports the failure through its error hook

### Requirement: Feature code reaches provider and model records only through platform domain hooks
Components and feature hooks SHALL read `Provider`, `ProviderMeta`, `Model`,
`ConfiguredProvider` and `ConfiguredModel` records through typed hooks exported from
`@/platform/entities` and change them through domain actions from the same entry. A
feature SHALL NOT hold a Zustand store whose state contains these records or their
DTO shapes; busy, error and refreshing flags MAY stay transient.

#### Scenario: A model is added to a provider
- **WHEN** an operator adds a catalog model to a configured provider on the models surface
- **THEN** the change goes through a platform domain action
- **AND** the provider's configured model list reflects it after the server confirms
- **AND** a failed save restores the previous state and shows the error

#### Scenario: A provider model change is persisted
- **WHEN** an operator adds, defaults and removes provider models through the models surface
- **THEN** the persisted provider record, read from the store, lists exactly the resulting models and default
- **AND** a restarted runtime renders exactly those rows

#### Scenario: A change made by another client appears without reload
- **WHEN** another client changes a provider's model list while the models surface is open and not reloaded
- **THEN** the surface shows the new list without a reload
- **AND** no feature-owned cache keeps showing the old list

#### Scenario: Two independent controls read provider data
- **WHEN** one control reads a provider's default model and another reads its enabled flag
- **THEN** they subscribe through separate hooks in separate components
- **AND** changing the enabled flag does not re-render the default-model control

#### Scenario: A provider card needs its models
- **WHEN** a provider detail panel lists that provider's catalog models
- **THEN** it reads them through a domain hook filtered by `provider_id`
- **AND** no per-provider Zustand cache of those rows exists
- **AND** the panel does not fetch the full catalog on its own

### Requirement: Raw graph access in feature code is limited to a shrink-only baseline
The architecture gate SHALL reject new imports of `useGraphStore`, `graphStore` or a raw
graph write function in feature code (`frontend/src/features/**` and
`frontend/src/stores/**`). Existing uses SHALL be recorded in one baseline keyed by file
path and rule, not line number, that may only shrink. The files migrated by this
capability SHALL have no baseline entry.

#### Scenario: A feature file imports the raw graph store
- **WHEN** a feature file not in the baseline imports `useGraphStore` from `@/platform/entities`
- **THEN** the gate exits non-zero with rule `feature-raw-graph-access` and the import's source location

#### Scenario: A migrated file is edited
- **WHEN** any file under `features/models` or `features/providers` imports `useGraphStore` or calls a raw graph write
- **THEN** the gate exits non-zero because those files have no baseline entry

#### Scenario: A baseline entry outlives its violation
- **WHEN** a baselined file stops importing the raw graph store and its entry remains
- **THEN** the gate fails with a stale-entry result naming the entry

#### Scenario: The baseline is compared with the merge base
- **WHEN** the baseline contains an entry not present on the merge base
- **THEN** the gate fails with a growth result naming the entry

### Requirement: Server-mirrored thread records have one owner
Thread records the server confirms (title, `updated_at`, persisted flag) SHALL be owned
by the graph `Thread` entity and read through a platform domain hook. A client-first
registry MAY keep only state that is not a server-confirmed record. Mirroring graph
threads into a second store through a raw graph subscription SHALL NOT remain.

#### Scenario: The server confirms a thread
- **WHEN** the realtime `threads` topic delivers a new or renamed server thread
- **THEN** the thread list shows the new title without a registry-to-graph mirror hook

#### Scenario: An unsaved thread exists
- **WHEN** an operator opens a new conversation that the server has not yet persisted
- **THEN** it appears as an ephemeral thread owned by transient state
- **AND** a server row for the same id later takes over its title and `updated_at`

#### Scenario: The server removes a thread
- **WHEN** the realtime `threads` topic removes a thread
- **THEN** it disappears from the thread list and its local persisted row is removed

### Requirement: Large catalog lists render only visible rows
The Models page SHALL render only the catalog rows in or near the viewport when the
filtered list exceeds a bounded size, while keeping each row's compare control, stable
test key, keyboard focus order and the displayed model count.

#### Scenario: The unfiltered catalog is shown
- **WHEN** the page lists the full catalog of about 8,000 models
- **THEN** at most 200 catalog row elements are in the document at any time
- **AND** the displayed count still states the full number of models

#### Scenario: The list is scrolled or navigated by keyboard
- **WHEN** an operator scrolls to the end of the list or moves focus through rows with the keyboard
- **THEN** the last row becomes reachable and focus is not lost when rows leave the viewport

#### Scenario: A filter narrows the list
- **WHEN** an operator filters to a handful of models
- **THEN** every matching row is rendered with its compare control
