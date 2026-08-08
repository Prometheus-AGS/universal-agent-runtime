## ADDED Requirements

### Requirement: Production primitives use the Base UI foundation
UAR's interactive production primitive wrappers SHALL use `@base-ui/react` as their
headless implementation foundation and SHALL expose those primitives through the local
`frontend/src/components/ui/` wrapper boundary.

#### Scenario: A product feature consumes an interactive primitive
- **WHEN** feature or page code needs an interactive primitive
- **THEN** it imports the local UI wrapper rather than importing a headless primitive
  package directly

#### Scenario: A production wrapper needs headless interaction behavior
- **WHEN** a wrapper under `frontend/src/components/ui/` implements headless interaction
  behavior
- **THEN** it uses the corresponding Base UI primitive and does not directly import a
  Radix UI primitive

### Requirement: Generator metadata preserves the selected primitive family
The frontend component generator configuration SHALL select the shadcn `base-vega` style
and the `neutral` base color so regeneration remains aligned with the Base UI foundation.

#### Scenario: UI wrappers are regenerated
- **WHEN** the shadcn generator reads `frontend/components.json`
- **THEN** it resolves the `base-vega` style and `neutral` base color

### Requirement: Primitive migration remains staged behind stable wrappers
UAR SHALL keep application-facing primitive imports stable while composition, icon, and
dependency-pruning follow-up changes are completed.

#### Scenario: A follow-up migration changes primitive internals
- **WHEN** composition, icon, or dependency cleanup changes the primitive implementation
- **THEN** existing feature and page imports continue resolving through the local UI
  wrapper boundary
