# frontend-configuration-surfaces Specification

## Purpose

Define feature ownership, layering, behavior-preservation, semantic-token, shared-UI, and verification contracts for the React frontend's production configuration surfaces.
## Requirements
### Requirement: Configuration pages have explicit feature ownership
The agents, auth, compiler, cost, credentials, knowledge, memory, models, providers, runtime-console, settings, skills, and tools production pages SHALL reside in matching feature slices under `frontend/src/features/` and SHALL be imported by the admin composition root through each feature's public entry point.

#### Scenario: A production configuration section resolves
- **WHEN** the admin composition root renders any of the thirteen migrated sections
- **THEN** it resolves the same exported page component from its owning feature without importing `frontend/src/admin/pages/`

#### Scenario: Settings is migrated before decomposition
- **WHEN** C-14a completes
- **THEN** the settings page resides in its feature slice with its behavior intact and remains available for C-14b decomposition

### Requirement: Each migrated feature owns its UI-to-API path
Each configuration feature SHALL own its page UI, directly owned hook/view-model, Zustand store or entity projection, REST client, helper components, and focused tests under `ui/`, `model/`, and `api/` as applicable. Components MUST call hooks rather than stores or APIs; hooks MUST expose store state/actions without calling APIs; stores MUST own I/O orchestration; API modules MUST remain thin transport wrappers.

#### Scenario: A page performs a REST-backed action
- **WHEN** a migrated configuration page submits an existing action
- **THEN** intent flows through the feature hook and store to the feature API module with the existing request and response contract

#### Scenario: A component dependency is audited
- **WHEN** imports in a migrated page or feature UI helper are inspected
- **THEN** no component imports a service/API module or Zustand store directly

### Requirement: Cross-slice access uses public feature contracts
Migrated feature implementation files SHALL NOT be imported by another feature. An observed legacy or cross-domain consumer that still requires a moved API or model symbol MUST import that symbol through a deliberate public contract until the final C-14c boundary migration removes or formalizes the consumer. A public contract MAY be the feature root `index.ts`, `api/index.ts`, `model/index.ts`, or a named root entry when a narrower entry prevents lightweight consumers from pulling page composition into the initial bundle.

#### Scenario: A moved API has an external consumer
- **WHEN** a caller outside the owning feature still uses a moved API function
- **THEN** the caller imports a deliberately exported public symbol and does not reach into an implementation file beneath the feature's `api/`, `model/`, or `ui/` boundary

#### Scenario: A lightweight consumer uses a moved model symbol
- **WHEN** exporting the symbol from the feature root would pull an admin page into the initial application graph
- **THEN** the consumer imports the symbol through a narrow public model or named root entry and the initial bundle remains within its established budget

#### Scenario: Unused public surface is evaluated
- **WHEN** a symbol has no observed external consumer after its page cluster moves
- **THEN** it is not re-exported from the feature entry solely for hypothetical compatibility

### Requirement: Configuration behavior remains compatible through relocation
The migration SHALL preserve the existing section inventory, route and query-string behavior, visible controls, loading/empty/error states, CRUD and test actions, request paths and payloads, response decoding, error propagation, and reactive entity or runtime updates. It MUST NOT change provider/model selection, authentication semantics, AG-UI/A2UI behavior, PGlite persistence, or backend contracts.

#### Scenario: Existing page operation succeeds
- **WHEN** an existing configuration action receives the same successful backend response before and after relocation
- **THEN** the migrated page produces the same user-visible result and reconciled state

#### Scenario: Existing page operation fails
- **WHEN** an existing configuration action receives the same backend failure before and after relocation
- **THEN** the migrated page preserves its actionable error state and does not present false success

#### Scenario: Runtime-console state changes
- **WHEN** the entity graph, run trace, approval state, or protocol feed changes
- **THEN** the migrated runtime feature updates with the existing subscription and selection semantics

### Requirement: Legacy admin color expressions are retired within C-14a scope
The migrated models, memory, cost, skills, and compiler page implementations SHALL contain zero `hsl(var(--…))` expressions. Each replacement MUST use an existing semantic Tailwind 4 token, preserve light/dark meaning and contrast intent, and retain non-color status labels or icons.

#### Scenario: Migrated feature token scan runs
- **WHEN** the published matcher scans the five C-14a-owned migrated page implementations
- **THEN** it finds zero legacy `hsl(var())` expressions and no new arbitrary palette value

#### Scenario: Status color is rendered
- **WHEN** a migrated page displays success, warning, error, selected, or disabled state
- **THEN** the state retains its existing text, icon, label, or control semantics rather than relying on color alone

### Requirement: Shared configuration UI remains narrowly owned
Loading, empty, and error projections reused by multiple independent configuration features SHALL reside under a named shared configuration UI boundary. Domain-specific editors, dialogs, detail panels, and helpers SHALL move with their owning feature and MUST NOT be placed in a general shared dumping ground.

#### Scenario: Helper is used by multiple independent features
- **WHEN** a configuration-state helper has observed callers in more than one feature and contains no domain behavior
- **THEN** it is imported from the shared configuration UI boundary

#### Scenario: Helper contains domain behavior
- **WHEN** a helper edits agents, imports skills, or renders tool details
- **THEN** it resides inside the corresponding agents, skills, or tools feature

### Requirement: Each page migration is independently verifiable
The implementation SHALL retain one page-sized migration checkpoint at a time, update every observed import consumer, and run the affected focused tests plus compiler, lint, architecture-boundary, and Flat 2.0 gates before proceeding. The completed C-14a change SHALL also pass consolidated frontend and strict OpenSpec verification without modifying protected backend, submodule, or operator-staged paths.

#### Scenario: One page cluster has moved
- **WHEN** its destination and import rewrites are complete
- **THEN** its focused tests and cheap frontend gates pass before the next page cluster is migrated

#### Scenario: C-14a reaches closeout
- **WHEN** all thirteen page clusters and five token sets have migrated
- **THEN** consolidated verification passes, `frontend/src/admin/pages/` has no production page owner remaining, and `.gitmodules`, `crates/prometheus-skill-system`, `src/uar`, and operator-staged license deletions remain untouched by C-14a

### Requirement: Settings composition is decomposed by stable responsibility
The settings feature SHALL keep its route-level page focused on responsive navigation and panel composition. Shared controls, schema-driven rendering, the panel registry, and domain panel implementations MUST reside in named internal settings UI modules, and no resulting settings page or panel module SHALL exceed approximately 600 source lines.

#### Scenario: Settings source structure is inspected
- **WHEN** C-14b completes
- **THEN** the route-level settings page contains navigation and composition rather than every panel implementation
- **AND** each settings page or panel module remains at or below the established size ceiling

#### Scenario: A domain panel is maintained
- **WHEN** a provider, file-processing, resilience, governance, agent, memory, caching, or user-settings panel changes
- **THEN** its implementation resolves from a named domain module without widening the public settings feature root

### Requirement: Settings behavior remains compatible through decomposition
The decomposition SHALL preserve the existing navigation categories and order, namespace keys, default active panel, metadata-based availability, generic namespace fallback, responsive layout, visible controls and copy, loading/error/saved states, validation, save/reload actions, JWT gating, provider/model semantics, and realtime settings updates. It MUST NOT change REST payloads, persistence, authentication, AG-UI/A2UI, entity schemas, or backend contracts.

#### Scenario: Operator selects a settings namespace
- **WHEN** the operator selects any available custom or schema-driven settings item
- **THEN** the same panel, controls, values, and action semantics render as before decomposition

#### Scenario: Settings metadata is unavailable or incomplete
- **WHEN** type metadata is empty or an available namespace has no custom panel
- **THEN** the existing availability and generic-schema fallback behavior remains intact

#### Scenario: User settings authentication is evaluated
- **WHEN** the configured frontend key is or is not a JWT
- **THEN** the user-settings panel preserves its existing gated state and remote save/reload behavior

### Requirement: Settings decomposition has focused structural and composition evidence
C-14b SHALL include automated evidence for the internal module-size contract and focused React evidence for route composition, navigation, availability, and panel resolution. The completed change MUST pass TypeScript, lint, frontend architecture, Flat 2.0, token, full frontend, bundle-budget, strict OpenSpec, and scoped diff-integrity checks without modifying protected backend, submodule, or operator-staged paths.

#### Scenario: A decomposition regression is introduced
- **WHEN** a settings UI module exceeds the size ceiling or composition loses a required navigation/panel contract
- **THEN** a deterministic validation or focused test fails before archive

#### Scenario: C-14b reaches closeout
- **WHEN** all settings UI modules are wired and focused tests pass
- **THEN** consolidated frontend and bundle validation passes and protected paths remain unchanged by C-14b

