# dependency-security-posture Specification

## Purpose
TBD - created by archiving change kreuzberg-reachable-vulns. Update Purpose after archive.
## Requirements
### Requirement: Kreuzberg-Pinned Advisory Mitigation

When a `cargo audit` advisory is confirmed reachable through the pinned `kreuzberg` dependency and no upstream release fixes it, the system SHALL bound the blast radius via explicit resource limits rather than leaving the vulnerable code path unconstrained, and the disposition MUST be recorded in `docs/DEPENDENCY_MANAGEMENT.md`.

#### Scenario: Reachable advisory with no upstream fix

- **Given** `cargo audit` reports `RUSTSEC-2026-0187` (lopdf stack overflow)
  and `RUSTSEC-2026-0194` (quick-xml quadratic attribute DoS) as reachable
  via kreuzberg's document-processing path
- **When** no kreuzberg tag through the latest available release fixes both
  advisories
- **Then** `KreuzbergConfig` MUST expose `max_input_bytes` and
  `extraction_timeout_secs` limits wired into the document-processing entry
  points, and the disposition MUST be documented with a reachability trace

#### Scenario: Advisory confirmed not reachable

- **Given** `cargo audit` reports an advisory for a dependency pulled in by
  kreuzberg
- **When** source inspection finds no call site exercising the vulnerable
  API
- **Then** the advisory MUST be disclosed as not reachable in
  `docs/DEPENDENCY_MANAGEMENT.md` rather than silently left unaddressed

### Requirement: Surreal-Memory Transitive Advisory Disposition

When a `cargo audit` advisory affects a transitive dependency pulled in via `surreal-memory`, the system SHALL apply the narrowest safe fix available (a scoped `cargo update` rather than resyncing the git pin) when a compatible patched version exists, and MUST disclose accepted risk when no patched version exists.

#### Scenario: Compatible patched version available

- **Given** `cargo audit` reports an advisory for a transitive dependency
  reachable via `surreal-memory`
- **When** a semver-compatible patched version exists
- **Then** the fix SHALL be applied via a scoped `cargo update -p <crate>`
  rather than resyncing `surreal-memory`'s own git pin

#### Scenario: No patched version exists

- **Given** a `cargo audit` advisory has `patched = []` (no fixed version
  exists)
- **When** reachability is traced and the advisory's threat model is
  assessed against actual call sites
- **Then** the risk MUST be disclosed as accepted in
  `docs/DEPENDENCY_MANAGEMENT.md` with the reachability trace and threat
  model reasoning recorded

### Requirement: Network-Facing Dependency Reachability Verification

When a `cargo audit` advisory affects a network-facing or archival-format dependency, the system SHALL verify actual reachability via `cargo tree`/source-grep before deciding on a fix, and MUST eliminate confirmed-unused dependencies entirely rather than leaving them in place with a disclosure note.

#### Scenario: Advisory dependency confirmed unused

- **Given** `cargo audit` reports an advisory for a dependency with zero
  call sites anywhere in the repository
- **When** the dependency is confirmed to be unused (dev-only or otherwise)
- **Then** the dependency MUST be removed from `Cargo.toml` entirely so the
  advisory is eliminated rather than merely disclosed

#### Scenario: Advisory dependency gated behind an optional feature

- **Given** `cargo audit` reports an advisory for a dependency only pulled
  in behind an optional, off-by-default feature
- **When** grepping the feature's own crate family finds no actual call
  site exercising the vulnerable API
- **Then** the advisory MUST be disclosed as not reachable in
  `docs/DEPENDENCY_MANAGEMENT.md`, including the fix version constraint
  that would be required if the dependency were ever activated

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

