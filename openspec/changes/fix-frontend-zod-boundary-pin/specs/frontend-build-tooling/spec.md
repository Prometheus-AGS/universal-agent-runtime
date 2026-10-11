## ADDED Requirements

### Requirement: Compatible first-party A2UI schema boundary
The first-party frontend SHALL construct its approved A2UI catalogs with compatible schema objects and the declared UAR protocol version. Production compilation MUST preserve concrete renderer implementation types without suppressing type errors or weakening validation.

#### Scenario: Production frontend compiles across the schema boundary
- **WHEN** the first-party frontend and UAR A2UI renderer are installed from the committed manifests and lockfiles
- **THEN** their schema objects MUST be compatible with the selected core catalog/binder API
- **AND** TypeScript and production bundling MUST complete without missing renderer implementations or protocol constructor arguments.
