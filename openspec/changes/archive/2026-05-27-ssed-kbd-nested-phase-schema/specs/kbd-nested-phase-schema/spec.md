## ADDED Requirements

### Requirement: Waypoint Nested-Phase Fields
The `current-waypoint.json` schema SHALL support three optional fields — `parentPhase`, `childPhases`, and `childPointer` — that together express a parent-and-child relationship between phases, and SHALL preserve backward compatibility with every existing reader.

#### Scenario: parentPhase field
- **WHEN** the waypoint file's top-level object is loaded
- **THEN** the loader MUST accept an optional string field `parentPhase`; when absent or `null`, the row represents a top-level (non-nested) phase.

#### Scenario: childPhases field
- **WHEN** the waypoint file's top-level object is loaded
- **THEN** the loader MUST accept an optional array field `childPhases` of unique non-empty strings; the order of the array MUST be the canonical iteration order used by `/kbd-next-child`; when absent, the loader MUST treat the field as the empty array `[]`.

#### Scenario: childPointer field
- **WHEN** the waypoint file's top-level object is loaded
- **THEN** the loader MUST accept an optional string field `childPointer`; the value, when non-null, MUST be a member of `childPhases`; when absent or `null`, no child is currently active.

#### Scenario: Pre-schema waypoint loads without modification
- **WHEN** a waypoint file written before this change (no `parentPhase` / `childPhases` / `childPointer` fields) is loaded by a reader that understands the new schema
- **THEN** the load MUST succeed without error and the three fields MUST take their documented defaults (`null`, `[]`, `null`).

#### Scenario: Tool that does not know about new fields
- **WHEN** a tool that predates this change reads a waypoint that includes the new fields
- **THEN** the unknown fields MUST be silently ignored; no parser is permitted to fail on their presence.

### Requirement: Waypoint Schema Template
The schema template shipped with the orchestrator skill SHALL document the new fields, their defaults, and a one-line meaning for each so downstream consumers can validate against it.

#### Scenario: Template lists every new field
- **WHEN** `references/schemas/current-waypoint.template.json` is loaded
- **THEN** it MUST contain entries for `parentPhase` (default `null`), `childPhases` (default `[]`), and `childPointer` (default `null`), each with a `description` string suitable for a one-line comment.

#### Scenario: Template version marker
- **WHEN** the template is updated as part of this change
- **THEN** it MUST carry a top-level `__schemaVersion` (or equivalent comment field) that allows consumers to detect the post-extension generation; the previous generation MUST remain readable.

### Requirement: Cross-Field Invariants
The waypoint schema SHALL enforce two structural invariants between the new fields whenever a writer populates them.

#### Scenario: childPointer references an unknown child
- **WHEN** a writer attempts to persist a waypoint with `childPointer` set to a value that is not a member of `childPhases`
- **THEN** the writer MUST reject the operation with a clear error naming both the offending pointer value and the current `childPhases` contents; no waypoint with that inconsistency is permitted to be written.

#### Scenario: childPhases contains duplicates
- **WHEN** a writer attempts to persist a waypoint whose `childPhases` array contains duplicate strings
- **THEN** the writer MUST reject the operation with an error naming the duplicated entry.

### Requirement: Project Schema worktreeRoot Field
The project schema template SHALL document the optional `worktreeRoot` field introduced by the worktree-persistence convention, with the same additive semantics as the waypoint fields.

#### Scenario: worktreeRoot present in template
- **WHEN** `references/schemas/project.template.json` is loaded
- **THEN** it MUST include `worktreeRoot` as an optional string field with default value `${HOME}/.claude/worktrees` and a `description` explaining that the value is treated literally and that environment-variable expansion is the consumer's responsibility.

#### Scenario: Project file without worktreeRoot
- **WHEN** a `project.json` file omits `worktreeRoot`
- **THEN** consumers MUST behave as if the field were the literal string `${HOME}/.claude/worktrees`, expanded against the current environment at the point of use.

#### Scenario: Project file with explicit worktreeRoot
- **WHEN** a `project.json` file sets `worktreeRoot` to any non-empty string
- **THEN** consumers MUST treat that value as authoritative and MUST NOT fall back to the default.

### Requirement: Migration Test Corpus
The orchestrator skill SHALL ship at least two regression test fixtures covering the schema transition.

#### Scenario: Pre-schema fixture loads
- **WHEN** the test corpus is run
- **THEN** at least one fixture MUST be a flat waypoint produced before this change; loading it MUST succeed and yield the documented defaults for the new fields.

#### Scenario: Fully-populated fixture loads
- **WHEN** the test corpus is run
- **THEN** at least one fixture MUST populate `parentPhase`, `childPhases` (≥1 entry), and `childPointer` consistently; loading it MUST succeed and the rendered chain MUST match a documented expectation string.

### Requirement: Skill Documentation
The `kbd-process-orchestrator` `SKILL.md` SHALL describe the nested-phase semantics so authors of `/kbd-new-child` and `/kbd-next-child` (separate changes) have a documented foundation.

#### Scenario: SKILL.md describes nesting
- **WHEN** a developer reads `kbd-process-orchestrator/SKILL.md` after this change lands
- **THEN** it MUST contain a "Nested phases" section that defines `parentPhase`, `childPhases`, `childPointer`, the canonical iteration order, and the cross-field invariants from the preceding requirement.
