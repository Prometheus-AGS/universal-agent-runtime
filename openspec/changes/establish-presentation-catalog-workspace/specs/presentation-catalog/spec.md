## ADDED Requirements

### Requirement: Durable reusable Presentation templates
The host SHALL persist Presentation identity, owner, revision, title, description, enabled state and declarative template content. Production clients SHALL manage these templates separately from the development-only A2UI tester.

#### Scenario: Owner edits a template
- **WHEN** an authenticated owner saves valid template content with the current expected revision
- **THEN** the host persists a new revision and a subsequent reload returns it

#### Scenario: Stale or foreign edit
- **WHEN** another owner addresses the record or an update supplies a stale revision
- **THEN** the host rejects the operation without mutating the record or exposing foreign content

### Requirement: Safe template validation and instantiation
The host SHALL accept only one declarative surface containing a complete rooted component tree in the first-party v0.9.1, uar.a2ui/1 profile and urn:uar:a2ui:catalog:1 catalog. Content SHALL satisfy the first-party renderer's identifier, variant, binding and size constraints. Each non-root component SHALL have exactly one parent so a small graph cannot expand into unbounded repeated rendering. Instantiation SHALL substitute only the host-selected surface identity and data model, never execute code or interpolate arbitrary strings. Initial data SHALL use escaped top-level JSON pointers compatible with the existing client reducer.

#### Scenario: Unsafe or broken template
- **WHEN** content contains an unapproved component/catalog, duplicate IDs, dangling children or a component cycle
- **THEN** validation rejects the save with an actionable error

#### Scenario: Exponentially expanding references
- **WHEN** multiple references would reuse the same component, even without a cycle
- **THEN** validation rejects the template rather than expanding repeated descendants at render time

#### Scenario: Authenticated empty catalog
- **WHEN** an authenticated principal loads a catalog with no records
- **THEN** the response includes the host-derived owner key and an empty presentations array so client hydration can be gated on the verified partition

#### Scenario: Preview without side effects
- **WHEN** the operator previews an unsaved template
- **THEN** the UI renders validated local content without dispatching declared actions or starting a run
