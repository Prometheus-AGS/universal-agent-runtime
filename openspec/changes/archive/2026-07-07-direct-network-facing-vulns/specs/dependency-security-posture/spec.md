## ADDED Requirements

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
