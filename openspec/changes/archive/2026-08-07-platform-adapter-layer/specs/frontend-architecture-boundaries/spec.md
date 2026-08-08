## ADDED Requirements

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
