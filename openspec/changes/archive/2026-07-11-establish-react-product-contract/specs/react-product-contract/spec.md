## ADDED Requirements

### Requirement: React is canonical
UAR SHALL identify React 19 + TypeScript as its primary first-party UI and SHALL define Component → Hook → Store → Service → API as the mandatory data-flow direction.

#### Scenario: Architecture discovery
- **WHEN** a contributor reads canonical frontend documentation
- **THEN** React ownership, layer responsibilities, and prohibited imports are unambiguous

### Requirement: UI surface inventory
Every live route and stable action SHALL map to an owner, endpoint, specification, maturity, and executable acceptance test.

#### Scenario: Unsupported surface
- **WHEN** a live surface lacks one of those mappings
- **THEN** it is classified experimental/internal or removed before GA
