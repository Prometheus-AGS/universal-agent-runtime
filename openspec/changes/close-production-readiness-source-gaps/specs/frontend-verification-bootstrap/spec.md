## ADDED Requirements

### Requirement: Execute prepared frontend checks without implicit dependency installation
Frontend verification SHALL retain dependency freshness checks and refuse missing or stale prepared dependencies before executing the requested runner, rather than implicitly install, update or repair them. Dependency provisioning SHALL be an independently authorized operation. Root and nested frontend workspace policies SHALL apply the same no-auto-install rule without weakening supply-chain, lockfile or build-script constraints.

#### Scenario: The prepared dependency tree is missing or stale
- **WHEN** the existing pnpm run or exec entry checks a missing or stale dependency tree
- **THEN** it rejects actionably without invoking install or the requested verification, and the outcome remains blocked rather than passed

#### Scenario: The prepared dependency tree is current
- **WHEN** the original five-file unit-project command is explicitly authorized against prepared dependencies
- **THEN** it uses the existing Vitest runner, project, setup, include/exclude and complete assertions without a replacement harness or implicit dependency repair

### Requirement: Preserve native configuration paths and actual story coverage
Vitest configuration SHALL derive the same module-relative paths without unsupported CommonJS globals. Storybook discovery SHALL retain all actual story files and configured accessibility/browser projects without absent-file warning patterns or warning-suppression settings.

#### Scenario: Native-compatible configuration loads
- **WHEN** the existing authorized runner reads the frontend configuration
- **THEN** its filesystem allowance, source alias, Storybook directory, test projects and coverage requirements retain their existing values while module paths resolve from import.meta.url

#### Scenario: Discovery matches the authored story corpus
- **WHEN** the configured Storybook roots contain the existing story modules and no MDX files
- **THEN** every authored story module remains included and absent MDX patterns do not emit misleading discovery warnings

### Requirement: Preserve fixture module and network boundaries before dispatch
Shared test setup SHALL retain graph-state isolation and React cleanup without importing application domains, stores, persistence or transports before test-file module mocks are registered. The public platform entity entry SHALL retain its existing runtime and domain API.

#### Scenario: Local hydration and mocked canonical backfill are collected
- **WHEN** the existing hydration and backfill files register their repository/session API mocks after shared setup
- **THEN** setup has loaded only the platform graph runtime entry and their real message-store logic binds the intended fixture collaborators
- **AND** original ownership, failure propagation, partial/terminal, canonical chronology/media/identity/persistence assertions remain required

#### Scenario: A local or API-mocked fixture accidentally reaches real HTTP
- **WHEN** either existing hydration or backfill fixture attempts fetch during a case
- **THEN** it rejects locally and fails a zero-fetch assertion without depending on a localhost server or credentials, and spies are restored even when that assertion fails


### Requirement: Preserve exact native continuation rejection and recoverable partials
Continuation recovery SHALL reject foreign native frames before persistence or reduction and retain the original interrupted run for exact-ID reattachment. Synthetic accepted native text SHALL use the actual producer message identity without relaxing journal validation.

#### Scenario: Foreign evidence is refused before exact-run recovery
- **WHEN** an accepted native partial is followed by a foreign terminal, content, state or memory event
- **THEN** the foreign evidence contributes no persistence, usage, phase timing or business reduction and the original partial remains interrupted
- **AND** reattachment of that same run with its native terminal recovers the same original bubble/content without another chat POST

#### Scenario: A conflicting response header has a bodyless or nonempty body
- **WHEN** an attached continuation receives a different x-uar-run-id header
- **THEN** the identity conflict is rejected before body readiness, identity adoption, run persistence or body consumption
- **AND** no assistant message or replacement chat dispatch is created


### Requirement: Preserve supported published artifact projections
Native published A2UI displays SHALL retain original artifact/run/event identity and complete publication payload in the existing a2ui-display projection, while input requests and ordinary artifacts retain their respective supported forms. Regression assertions SHALL preserve full identity, source, generation, metadata, content, persistence and renderer-policy evidence without requiring an obsolete generic fallback.

#### Scenario: Native display, input and plain report reach local history
- **WHEN** the actual stream persists a published A2UI display, an input request and an ordinary report
- **THEN** display uses a2ui-display with original toolCallId, profile/version/component and exact payload; input uses a2ui-input and report uses artifact
- **AND** original native payload, source/generation/publication metadata, renderer policy, local database evidence and no-repeat dispatch assertions remain required

### Requirement: Initialize the supported simulated standards document before rendering imports
Unit setup SHALL provide the actual HTML5 DocumentType before test-file rendering imports. If the simulator lacks compatMode, a fixture-scoped read-only getter SHALL derive its value from that supported real doctype and preserve any native implementation. No warning suppression or unconditional mode value SHALL replace document compatibility.

#### Scenario: KaTeX loads and renders in the supported unit document
- **WHEN** the existing selected regression imports and invokes real KaTeX DOM rendering
- **THEN** the actual document has html doctype with empty public/system identifiers and CSS1Compat, and rendering produces KaTeX/MathML with original source annotation
- **AND** graph reset, React cleanup, existing runtime collaborators and all original case assertions remain
