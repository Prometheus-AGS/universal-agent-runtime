# entity-explorer-fab-panel Specification

## Purpose

Dev-mode floating-action-button-launched entity explorer for `prometheus-entity-management` consumers, with five tabs (Tree / Inspector / Events / Stores / Duplicates), a multi-store registration API, an append-only event bus, production tree-shaking, and a documented pre-flight UI/UX research pass.

## Requirements

### Requirement: Pre-flight UI/UX Research Doc
Before any production component code is written, the implementation SHALL produce `docs/devtools-design-notes.md` in `prometheus-entity-management` summarising the best practices synthesised from the UI/UX routing discipline (memory recall, UI/UX Pro Max, Impeccable, Anthropic skills, Vercel skills, runtime-devtools web search, Chrome MV3 panel web search).

#### Scenario: File exists at HEAD
- **WHEN** the change is applied
- **THEN** `prometheus-entity-management/docs/devtools-design-notes.md` MUST exist with non-empty sections summarising at minimum: (a) the seven-step routing discipline application to this change, (b) consulted skills + outputs, (c) one-paragraph distilled best practices, (d) web-search references with URLs and fetch dates.

### Requirement: Dev-Mode Gating
The Entity Explorer SHALL render only in dev mode or when explicitly enabled via URL parameter, and SHALL be tree-shaken from production builds.

#### Scenario: Default dev render
- **WHEN** `process.env.NODE_ENV !== "production"` at module evaluation time
- **THEN** the `<EntityExplorerFab>` component renders the floating action button.

#### Scenario: URL override
- **WHEN** `window.location.search` contains `prometheus-devtools=1`
- **THEN** the FAB renders regardless of `NODE_ENV`.

#### Scenario: Production exclusion
- **WHEN** a production build is produced
- **THEN** the `src/devtools/` module MUST be tree-shaken; no panel code MUST ship in the production bundle.

### Requirement: Multi-Store Registration API
The package SHALL export `registerDevtoolsStore(config: DevtoolsStoreConfig)` letting application code register every Zustand store that holds entity data.

#### Scenario: Registration shape
- **WHEN** an application calls `registerDevtoolsStore({ id, label, store, describesEntityTypes })`
- **THEN** the registry MUST record the entry; subsequent calls with the same `id` MUST replace the prior entry.

#### Scenario: Enumeration
- **WHEN** the Stores tab is opened
- **THEN** it MUST render every entry in the registry with its id, label, declared entity types, and current entity-id count derived from the store snapshot.

#### Scenario: Production no-op
- **WHEN** `registerDevtoolsStore` is called in a production build
- **THEN** the call MUST be a no-op (tree-shaken or stubbed) and MUST NOT throw.

### Requirement: Floating Action Button + Panel Host
The package SHALL export `<EntityExplorerFab>` which renders a floating action button and a togglable panel host.

#### Scenario: FAB appearance
- **WHEN** the FAB is visible
- **THEN** it MUST be positioned fixed at viewport corner, sized to be discoverable but unobtrusive, and labeled for screen readers.

#### Scenario: Panel toggle
- **WHEN** the FAB is clicked or activated via keyboard
- **THEN** the panel MUST mount; subsequent activation MUST unmount it.

#### Scenario: Five tabs
- **WHEN** the panel is open
- **THEN** it MUST render exactly five tabs in this order: Tree, Inspector, Events, Stores, Duplicates.

### Requirement: Tree Tab
The Tree tab SHALL render entities grouped by `EntityType` with per-row metadata.

#### Scenario: Row content
- **WHEN** the Tree tab renders a row for entity `(T, id)`
- **THEN** the row MUST display type, id, badges for every registered store holding that id, last-updated timestamp, and status indicators for stale / fetching / patched states.

#### Scenario: Counts header
- **WHEN** the Tree tab is open
- **THEN** the header MUST show total entities, per-type counts, total patches, and stale count, sourced from `collectGraphDevStats`.

#### Scenario: Open Inspector
- **WHEN** a user clicks an entity id from the Tree
- **THEN** the panel MUST switch to the Inspector tab pre-loaded with that entity.

### Requirement: Inspector Tab
The Inspector SHALL render full state for a selected entity.

#### Scenario: Loaded state
- **WHEN** the Inspector receives `(T, id)`
- **THEN** it MUST display the normalised entity data, current patches, subscriber count, origin stores, and a timeline of events touching that entity filtered from the event bus.

#### Scenario: No selection
- **WHEN** the Inspector is opened without a selection
- **THEN** it MUST display an empty-state hint suggesting the Tree tab.

### Requirement: Events Tab
The Events tab SHALL render the contents of `devtools-event-bus` with filters.

#### Scenario: Live tail
- **WHEN** new events arrive on the bus
- **THEN** the Events tab MUST append them to the visible list without losing user scroll position when scrolled-to-bottom; when scrolled away, the tab MUST show a "N new events" jump-button.

#### Scenario: Event row
- **WHEN** a row is rendered
- **THEN** it MUST display timestamp, event type, source (engine / adapter / hook), and a payload preview with an "expand" affordance.

#### Scenario: Filters
- **WHEN** the user applies any combination of filters (event type, entity type, source, entity id)
- **THEN** the list MUST update to show only matching events; filters compose with AND semantics.

### Requirement: Stores Tab
The Stores tab SHALL enumerate every registered devtools store and report its current entity coverage.

#### Scenario: Store list
- **WHEN** the Stores tab is open
- **THEN** it MUST render one row per registered store with id, label, declared entity types, and current count of entity ids held.

#### Scenario: Diff against canonical
- **WHEN** the user expands a store row
- **THEN** it MUST display the set of entity ids in that store that diverge from the engine's canonical graph (missing in store, extra in store, value differs).

### Requirement: Duplicates Tab
The Duplicates tab SHALL surface `(type, id)` pairs held by more than one registered store.

#### Scenario: Duplicate detection
- **WHEN** the Duplicates tab is open
- **THEN** it MUST render every `(type, id)` present in ≥2 registered stores with a side-by-side data preview from each store.

#### Scenario: Promote to canonical
- **WHEN** the user clicks "Promote to canonical" on a duplicate row
- **THEN** the chosen variant MUST be written through the engine via the existing mutation path; other registered stores' copies become observers of the canonical graph.

### Requirement: Event Bus
The package SHALL ship `devtools-event-bus.ts` as an append-only ring buffer with a documented capacity and a `subscribe(cb)` API.

#### Scenario: Capacity
- **WHEN** the bus reaches its capacity (default 1000)
- **THEN** the oldest event MUST be evicted on the next push; bus order MUST remain chronological.

#### Scenario: Engine tap
- **WHEN** the engine performs any of `upsert`, `patch`, `delete`, `clearPatch`, list-mutation operations
- **THEN** the engine MUST emit an event onto the bus via `subscribeDevtoolsEvent`-style hook.

#### Scenario: Adapter tap
- **WHEN** any registered adapter emits a `ChangeSet`
- **THEN** an event of source `adapter` MUST be pushed to the bus carrying a summary (counts by type, list of affected list keys).

### Requirement: Companion Skill Update
The `entity-graph-optimize` skill SHALL gain a "Dev-mode entity explorer" subsection.

#### Scenario: Subsection present
- **WHEN** the skill set is read after this change
- **THEN** `entity-graph-optimize/SKILL.md` MUST contain a "Dev-mode entity explorer" section describing the FAB, the toggle conditions, the multi-store registration API, the five tabs, and pointing to `docs/devtools-design-notes.md`.

### Requirement: Test Coverage
The implementation SHALL ship at least the listed unit / integration tests.

#### Scenario: Registry tests
- **WHEN** the test suite runs
- **THEN** it MUST cover `registerDevtoolsStore` registration, replacement on same id, production no-op, and enumeration.

#### Scenario: Bus tests
- **WHEN** the test suite runs
- **THEN** it MUST cover bus capacity / eviction order and `subscribe()` callback delivery for engine and adapter events.

#### Scenario: Component smoke tests
- **WHEN** the test suite runs
- **THEN** it MUST render each of the five tabs against a fixture graph and assert the documented row content and counts.
