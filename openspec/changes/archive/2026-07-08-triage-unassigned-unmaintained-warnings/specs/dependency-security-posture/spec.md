## ADDED Requirements

### Requirement: Abandoned Crate Disclosure

When an unmaintained-crate `cargo audit` warning has no patched version and no fix available within the project's own control, the disposition SHALL disclose the specific reason (upstream permanently abandoned, transitive chain too deep to control, or feature-gated non-default) rather than a generic "no fix available" note.

#### Scenario: The upstream crate is permanently abandoned

- **Given** a `cargo audit` warning states a crate has ceased development
  with no patched version ever planned
- **When** deciding a disposition
- **Then** the disclosure MUST name the reason development ceased (when
  known) and identify which first-party-uncontrolled parent dependency
  would need to migrate away from it, rather than asserting "no fix" with
  no supporting detail

#### Scenario: A dependency is many layers removed from any controlled repo

- **Given** an unmaintained crate is reachable only through a chain of
  transitive dependencies spanning multiple third-party repositories
- **When** deciding whether a fix is realistic
- **Then** the disclosure MUST show the full chain (each hop) so a future
  reader can judge for themselves whether upstream cooperation is
  plausible, rather than declaring it unfixable without evidence of the
  actual path
