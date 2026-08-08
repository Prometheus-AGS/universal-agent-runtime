## ADDED Requirements

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
