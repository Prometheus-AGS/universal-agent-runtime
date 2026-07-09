## ADDED Requirements

### Requirement: Floating Git Dependency Resolution

A git-sourced dependency intended to provide reproducible builds SHALL be pinned to a fixed commit SHA or tag rather than a floating branch, unless the floating pin is an explicit, human-approved, and documented choice.

#### Scenario: An architectural decision claims reproducibility but a pin floats

- **Given** a project's architectural decision record states that
  git-sourced dependencies are pinned for reproducible builds
- **When** one of the listed dependencies is actually pinned to a
  floating branch rather than a fixed commit or tag
- **Then** the dependency MUST be moved to a fixed `rev` (the branch's
  current HEAD, re-verified immediately before the change is applied) or
  the floating pin MUST be explicitly re-affirmed with a documented
  reason, resolved via explicit human input rather than assumed

#### Scenario: Re-verifying the target commit before applying

- **Given** a specific commit SHA was resolved as a floating branch's
  HEAD at planning time
- **When** the change is actually applied
- **Then** the SHA MUST be re-verified via `git ls-remote` (or
  equivalent) immediately before use, and any drift between planning-time
  and execute-time HEAD MUST be disclosed rather than silently using a
  stale value
