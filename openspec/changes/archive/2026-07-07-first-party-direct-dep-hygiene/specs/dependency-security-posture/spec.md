## ADDED Requirements

### Requirement: First-Party Direct Dependency Currency

When `cargo audit` flags a direct (first-party-controllable) dependency as unmaintained or unsound, the system SHALL replace it with an actively-maintained, API-compatible alternative rather than accepting the risk, since — unlike git-pinned dependencies covered by this project's D-D decision — a direct dependency can be swapped without waiting on upstream.

#### Scenario: Unmaintained direct dependency with a maintained alternative

- **Given** `cargo audit` flags a direct dependency in `Cargo.toml` as both
  unmaintained and unsound
- **When** an actively-maintained, API-compatible alternative exists
- **Then** the dependency MUST be replaced across all call sites, and any
  transitive dependency pulled in solely through the replaced crate MUST be
  confirmed absent from `Cargo.lock` afterward

#### Scenario: Flagged advisory no longer applies at the pinned version

- **Given** an assessment surfaces an unsoundness report for a dependency
  version that may differ from what is currently pinned
- **When** a fresh `cargo audit` run is checked against the currently
  pinned version
- **Then** the finding MUST be re-verified against the pinned version
  before any code change is made, and disclosed as not applicable if the
  advisory does not list that version
