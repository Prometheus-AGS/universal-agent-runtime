## ADDED Requirements

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
