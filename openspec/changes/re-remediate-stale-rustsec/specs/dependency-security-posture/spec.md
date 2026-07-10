## ADDED Requirements

### Requirement: Suppressed Advisories Are Genuinely Unfixable
Every advisory suppressed in CI audit steps SHALL be unfixable by this
project at suppression time (no upstream patch, or patch blocked by the
latest release of an upstream we do not control), with the rationale stating
the blocking condition and where it is tracked.

#### Scenario: Stale rationale is corrected when a patch ships
- **WHEN** an upstream patch becomes available for a suppressed advisory
- **THEN** the dependency is updated (directly, via fork, or via upstream
  PR) and the suppression removed, rather than the stale rationale persisting

#### Scenario: Broken optional features cannot hold vulnerable pins
- **WHEN** a vulnerable dependency is reachable only through an optional
  feature that does not compile
- **THEN** the feature is removed or repaired rather than the vulnerability
  suppressed indefinitely
