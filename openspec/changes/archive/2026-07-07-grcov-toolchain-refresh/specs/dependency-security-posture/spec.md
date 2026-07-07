## ADDED Requirements

### Requirement: Unused Dev-Dependency Elimination

When `cargo audit` flags a dependency chain attributable to a `[dev-dependencies]` entry, the system SHALL verify whether that entry has any actual library call sites in the repository before deciding on a fix, and MUST remove the entry entirely (rather than merely bumping its version) when it is confirmed unused.

#### Scenario: Flagged dev-dependency has zero call sites

- **Given** `cargo audit` reports unmaintained/unsound warnings for a
  transitive chain pulled in via a `[dev-dependencies]` entry
- **When** grepping the repository for `use <crate>`/`<crate>::` finds zero
  call sites, and the equivalent tooling is invoked as an independently
  installed CLI binary instead
- **Then** the `[dev-dependencies]` entry MUST be removed entirely from
  `Cargo.toml`, eliminating its exclusive transitive chain from
  `Cargo.lock`

#### Scenario: A prior plan mischaracterizes which crates a fix clears

- **Given** an earlier planning document lists specific crate names as
  cleared by a dependency-removal fix
- **When** `cargo tree -i <crate>` shows that crate's actual reverse
  dependency path does not go through the dependency being removed
- **Then** the proposal MUST correct the record and disclose which crates
  are actually unaffected, rather than silently claiming a broader fix
  than what the change actually delivers
