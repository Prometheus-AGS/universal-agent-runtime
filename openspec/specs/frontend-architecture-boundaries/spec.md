# frontend-architecture-boundaries Specification

## Purpose
TBD - created by archiving change platform-adapter-layer. Update Purpose after archive.
## Requirements
### Requirement: Infrastructure adapters have explicit platform ownership
UAR frontend infrastructure adapters SHALL reside under
`frontend/src/platform/`, and platform implementation files SHALL contain no
JSX or direct React imports.

#### Scenario: The platform adapter tree is audited
- **WHEN** the platform adapter gate scans `frontend/src/platform`
- **THEN** AG-UI, PGlite, and entity-management entry points exist there without `.tsx` files or direct React imports

### Requirement: AG-UI normalization is owned by the platform layer
UAR SHALL expose its AG-UI event schema and canonical event adapter from
`platform/agui/` and SHALL preserve lifecycle, ordering, replay, approval,
terminal, and state-patch behavior across the path migration.

#### Scenario: Official AG-UI events are reduced
- **WHEN** a valid `uar.agui/1` event is ingested through the platform adapter
- **THEN** it produces the same canonical chat/runtime event shape as before the move

#### Scenario: Replay and malformed events are handled
- **WHEN** the platform adapter receives a duplicate, late, malformed, or divergent state event
- **THEN** it preserves the existing rejection and recovery semantics

### Requirement: PGlite persistence is owned by the platform layer
UAR SHALL expose its PGlite singleton client from
`platform/pglite/client.ts` while its React provider remains outside the
platform layer.

#### Scenario: Existing persistence consumers initialize
- **WHEN** the React database provider and thread/message stores import the moved client
- **THEN** the same database name, migrations, singleton lifecycle, and typed thread/message APIs remain available

### Requirement: Entity-management integration has one package boundary
UAR application and test source SHALL import
`@prometheus-ags/prometheus-entity-management` only through one explicit
facade under `platform/entities/`.

#### Scenario: An entity consumer uses package functionality
- **WHEN** a store, hook, helper, page, or test needs an entity graph runtime value or type
- **THEN** it imports the explicitly re-exported symbol from `@/platform/entities`

#### Scenario: Adapter boundaries are checked in CI
- **WHEN** the platform adapter gate scans UAR application TypeScript and TSX under `frontend/src` and `frontend/e2e`
- **THEN** it rejects retired AG-UI and PGlite entry points and direct entity-management package imports outside the facade

### Requirement: The shared application shell is the sole route shell
UAR SHALL render retained configuration routes directly beneath the shared application shell and SHALL NOT retain a nested legacy admin shell or route-scoped terminal theme.

#### Scenario: A configuration route renders
- **WHEN** an operator navigates to a retained `/admin/*` route
- **THEN** the corresponding feature surface renders inside the shared navigation, breadcrumb, and responsive shell without a second navigation tree

#### Scenario: An unknown configuration route is requested
- **WHEN** a path does not name a retained configuration surface
- **THEN** the runtime cockpit default renders without activating a legacy terminal theme

### Requirement: Remaining configuration utilities have feature ownership
The development-only A2UI tester and MCP health surface SHALL be owned by their corresponding feature slices, and `frontend/src/admin/` plus their retired top-level hook, store, service, and entity-fetcher paths SHALL be absent.

#### Scenario: Development A2UI testing is requested
- **WHEN** a development build resolves the A2UI testing route
- **THEN** the feature-owned tester remains available while production route discovery continues to exclude it

#### Scenario: MCP health is requested
- **WHEN** an operator opens the MCP health route
- **THEN** the feature-owned health surface retains polling, entity projection, error, empty, and refresh behavior

### Requirement: Obsolete direct dependencies are retired safely
UAR SHALL contain no direct TanStack Query, highlight.js, or `@radix-ui/*` dependency declarations and SHALL retain any Radix packages still required transitively by supported dependencies.

#### Scenario: Dependency ownership is audited
- **WHEN** the frontend manifest and lockfile are inspected after a frozen installation
- **THEN** the retired packages are absent as direct dependencies and retained transitive Radix consumers remain resolvable

### Requirement: Section 6.3 and public-entry import boundaries are enforced
The frontend boundary gate SHALL enforce the binding §6.3 matrix: platform/shared code SHALL NOT import features/app, feature code SHALL NOT import app, and one feature SHALL NOT import another feature's implementation path instead of an explicit public root, `api`, or `model` entry.

#### Scenario: A valid feature dependency is scanned
- **WHEN** a feature imports another feature through its root, `api`, or `model` index
- **THEN** the boundary gate accepts the import

#### Scenario: A forbidden upward or implementation-path import is scanned
- **WHEN** platform/shared code imports features/app, feature code imports app, or one feature imports another feature's implementation file
- **THEN** the boundary gate fails with a deterministic rule identifier

