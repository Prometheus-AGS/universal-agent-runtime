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

### Requirement: Settings namespace reads use canonical backend slugs
The settings frontend SHALL translate internal namespace keys to the backend's canonical URL slug before every namespace read. The translation MUST preserve the existing response decoding and non-success error propagation, and MUST NOT change save routes, payloads, persistence, provider configuration, or realtime state.

#### Scenario: Provider namespace is read
- **WHEN** the frontend reads the internal `provider` namespace
- **THEN** it requests `/api/uar/settings/providers`
- **AND** it does not request `/api/uar/settings/provider`

#### Scenario: Underscored namespace is read
- **WHEN** the frontend reads the internal `context_management` namespace
- **THEN** it requests `/api/uar/settings/context-management`
- **AND** it does not issue an underscored settings namespace request

#### Scenario: Canonical namespace is unchanged
- **WHEN** the frontend reads the internal `server` namespace
- **THEN** it requests `/api/uar/settings/server`

#### Scenario: Canonical settings read fails
- **WHEN** a canonical settings namespace response is non-successful
- **THEN** the existing status-based error propagation is preserved
- **AND** the UI does not report a successful load

#### Scenario: Installed runtime settings are inspected
- **WHEN** the production static bundle is served by the installed runtime on port 1906 and an operator opens Provider Overrides and Context Management
- **THEN** the configured provider records render and Context Management loads through its hyphenated route
- **AND** the browser emits no singular, underscored, or other settings namespace 404

### Requirement: Provider default models use bounded selection
The Provider Overrides surface SHALL present each configured provider's default model as an accessible bounded selection control whose options are exactly that provider's enabled configured models. Inventories containing one through seven enabled models SHALL use the simple selection path, while inventories containing eight or more enabled models SHALL provide search over both display names and raw model identifiers without accepting free-form values.

#### Scenario: Provider model options are opened
- **WHEN** a provider has between one and seven enabled configured models and an operator opens its default-model control
- **THEN** every enabled model in that provider's configured model list is available through the simple selection path
- **AND** disabled models and models owned only by other providers are not available

#### Scenario: Large provider model inventory is opened
- **WHEN** a provider has eight or more enabled configured models and an operator opens its default-model control
- **THEN** the control provides a search input and every valid enabled model remains available
- **AND** the unfiltered option order matches the provider configuration order

#### Scenario: Provider models are searched
- **WHEN** an operator enters a search term in a large provider model inventory
- **THEN** matching is case-insensitive after trimming surrounding query whitespace
- **AND** a model remains visible when the literal term occurs in either its display name or raw model identifier
- **AND** a distinct no-match state appears when no valid model matches

#### Scenario: Provider default model is selected
- **WHEN** an operator selects one of the provider's available models with a pointer or keyboard
- **THEN** the provider settings draft records that model id as `default_model` exactly once
- **AND** the existing settings save and realtime reconciliation path remains in use
- **AND** arbitrary text cannot become the selected model

#### Scenario: Provider model labels are ambiguous
- **WHEN** two valid enabled models have the same display name
- **THEN** the selection results expose their raw model identifiers so the operator can distinguish them

#### Scenario: Stored provider model is unavailable
- **WHEN** the stored default model is not present in the provider's current enabled model list
- **THEN** the control reports the stale value as unavailable and offers valid replacements
- **AND** it does not automatically select or save a replacement

### Requirement: Provider settings controls expose complete accessible context
The Provider Overrides surface SHALL expose every provider card as a named group and SHALL give each visible field, enabled switch, and API-key reveal action a provider-specific programmatic name. Help text and invalid-state recovery text MUST be programmatically associated with the affected control.

#### Scenario: Provider controls are traversed with assistive technology
- **WHEN** an operator navigates provider settings without relying on visual layout
- **THEN** Base URL, Protocol, API Key, Default Model, Enabled, and API-key reveal controls are identifiable for the correct provider
- **AND** help or invalid-state text is included in the affected control's accessible description

#### Scenario: Provider default model is unavailable
- **WHEN** the stored default model is not currently selectable
- **THEN** the model control exposes an invalid state and the associated recovery guidance to assistive technology

### Requirement: Provider settings communicate asynchronous outcomes
The Provider Overrides surface SHALL expose loading and successful-save feedback as polite, atomic status updates and SHALL expose failures as alerts. A rejected save MUST NOT emit successful-save feedback and MUST preserve pending drafts.

#### Scenario: Provider settings load or save succeeds
- **WHEN** provider settings are loading or a save completes successfully
- **THEN** the corresponding visible message is available as a polite status update

#### Scenario: Provider settings operation fails
- **WHEN** a provider settings load or save operation fails
- **THEN** the visible error is announced as an alert
- **AND** a failed save retains the unsaved provider drafts

### Requirement: Provider settings protect unsaved drafts
The Provider Overrides surface SHALL derive its modified state from the authoritative provider settings draft. Save MUST be disabled while no provider draft exists; Refresh MUST be disabled while drafts exist or a provider settings operation is in flight; and browser unload MUST request confirmation while drafts exist.

#### Scenario: Provider settings are clean
- **WHEN** no provider settings draft exists and no operation is in flight
- **THEN** Save is disabled and Refresh is available

#### Scenario: Provider settings are modified
- **WHEN** one or more provider settings drafts exist
- **THEN** Save is enabled, Refresh is disabled, and visible text identifies each modified provider
- **AND** the operator is told to save changes before refreshing

#### Scenario: Browser unload is attempted with drafts
- **WHEN** browser navigation or window closure would unload provider settings while drafts exist
- **THEN** the browser unload event is cancelled so the browser can request confirmation

#### Scenario: Provider save succeeds
- **WHEN** all provider drafts are saved successfully
- **THEN** the dirty indicators clear, Save becomes disabled, and Refresh becomes available

#### Scenario: Provider save fails
- **WHEN** saving provider drafts fails
- **THEN** the dirty indicators remain and Refresh stays disabled

### Requirement: Provider settings remain usable at narrow widths
The Provider Overrides editor SHALL stack provider fields in one column at narrow widths and SHALL retain its two-column composition at desktop widths. Controls and long provider content MUST remain within the available viewport without clipping keyboard focus.

#### Scenario: Provider settings are viewed in a narrow viewport
- **WHEN** the available provider-panel width cannot support the desktop field composition
- **THEN** fields stack into one column without horizontal page scrolling
- **AND** controls remain fully keyboard accessible

#### Scenario: Provider settings are viewed at desktop width
- **WHEN** the available provider-panel width supports the incumbent desktop composition
- **THEN** provider fields render in two columns

### Requirement: Sensitive setting masks preserve secret length
Settings API responses SHALL obscure every character of a stored API key with one mask character and SHALL NOT return any plaintext character from the stored key.

#### Scenario: Stored provider API key is read
- **WHEN** a provider settings record contains an API key with N characters
- **THEN** the response contains an API-key mask with exactly N characters
- **AND** every returned character is a mask character

#### Scenario: Provider API key is absent
- **WHEN** a provider settings record has no API key or has an empty API key
- **THEN** the response does not fabricate a non-empty credential mask

### Requirement: Unchanged nested credential masks are non-destructive
The settings API SHALL preserve an existing nested API key when an update submits the unchanged response mask while modifying other fields in the same settings object.

#### Scenario: Unrelated provider field is saved
- **WHEN** an operator changes a non-sensitive provider field and the request includes the unchanged API-key mask returned by the settings API
- **THEN** the existing stored API key remains unchanged
- **AND** the response continues to return only its length-preserving mask

#### Scenario: Replacement provider API key is saved
- **WHEN** an operator submits a new API-key value that does not equal the current response mask
- **THEN** the new value replaces the stored API key
- **AND** subsequent reads return a mask matching the new value's character count
