## MODIFIED Requirements

### Requirement: Entity-management integration has one package boundary
UAR application and test source SHALL import
`@prometheus-ags/prometheus-entity-management` only through one explicit
facade under `platform/entities/`. The UAR product SHALL resolve that package
from the exact supported registry release and SHALL resolve one compatible
`@prometheus-ags/entity-graph-core` singleton; a checked-out workspace package
MUST NOT silently substitute for the supported release.

#### Scenario: An entity consumer uses package functionality
- **WHEN** a store, hook, helper, page, or test needs an entity graph runtime value or type
- **THEN** it imports the explicitly re-exported symbol from `@/platform/entities`

#### Scenario: Adapter boundaries are checked in CI
- **WHEN** the local platform adapter gate scans UAR application TypeScript and TSX under `frontend/src` and `frontend/e2e`
- **THEN** it rejects retired AG-UI and PGlite entry points and direct entity-management package imports outside the facade
- **AND** this product check runs locally and is not added to or run by GitHub Actions

#### Scenario: The supported release is installed
- **WHEN** dependencies are installed from either UAR workspace root
- **THEN** the UAR frontend resolves `@prometheus-ags/prometheus-entity-management` exactly to registry release `3.0.2`
- **AND** its compatible `@prometheus-ags/entity-graph-core` peer resolves to release `3.0.2` as one runtime singleton
- **AND** neither resolution points at `frontend/packages/prometheus-entity-management`

#### Scenario: Dependency resolution drifts
- **WHEN** a manifest or lockfile resolves the product dependency to a workspace prerelease, a version other than `3.0.2`, or more than one core runtime
- **THEN** the dependency verification fails before the following entity-flow change begins
