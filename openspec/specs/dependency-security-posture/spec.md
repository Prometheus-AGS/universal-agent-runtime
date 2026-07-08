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

### Requirement: npm Semver-Compatible Fix Application

When `npm audit` reports findings against the root `package-lock.json`, the system SHALL apply `npm audit fix` without `--force` when a live re-check confirms every finding resolves within already-declared semver ranges, and MUST evaluate any `--force`-only (potentially breaking) finding individually rather than blanket-applying it.

#### Scenario: All findings resolve within existing semver ranges

- **Given** `npm audit --json` reports a set of findings, each with
  `fixAvailable: true` (not an object naming a semver-major bump)
- **When** `npm audit fix` is run without `--force`
- **Then** the resulting `package-lock.json` diff MUST be verified against
  a fresh `npm audit` re-run showing the finding count drop to zero for
  those findings, with no `package.json` direct-dependency range edits
  required

#### Scenario: A finding requires a breaking change

- **Given** `npm audit fix --dry-run` shows a finding whose only fix
  requires `--force` and is flagged as a semver-major bump
- **When** deciding how to proceed
- **Then** that finding MUST be evaluated individually (checked for actual
  usage/breaking-change impact) rather than applied automatically as part
  of a blanket `--force` fix

### Requirement: pnpm Transitive Fix Within Declared Ranges

When `pnpm audit` reports a finding against a transitive dependency, the system SHALL first check whether the direct parent's own declared range already permits the patched version before adding a workspace override, and MUST pin any override to the exact patched version rather than an open-ended range when an override is unavoidable.

#### Scenario: Parent's declared range already permits the patch

- **Given** `pnpm audit` reports a finding for a transitive dependency
- **When** the parent package's own `package.json` range for that
  dependency already includes the patched version
- **Then** the fix MUST be applied via `pnpm update <dependency>` (or
  `pnpm -r update` for workspace-wide resolution), not a `pnpm-workspace.yaml`
  override

#### Scenario: No parent range permits the patch

- **Given** every reverse-dependency path pins the vulnerable package to a
  range that excludes the patched version, with no newer parent release
  available to relax it
- **When** an override is added to `pnpm-workspace.yaml`
- **Then** the override MUST pin the exact patched version (e.g.
  `"0.28.1"`), not an open-ended range (e.g. `">=0.28.1"`), to prevent an
  unintended major-version bump on the next install

#### Scenario: An unbounded override causes an unintended major bump

- **Given** an override or version range has no upper bound
- **When** `pnpm install` resolves a newer major version than intended as
  a result
- **Then** the mistake MUST be caught (via the dependency-diff output of
  `pnpm install`) and corrected with a properly bounded override before
  the change is considered complete

### Requirement: Scheduled Audit Trigger Independence

Security-audit CI checks SHALL run on their own dedicated trigger (schedule and/or manual dispatch) rather than being nested only inside an unrelated pipeline whose trigger condition may rarely or never fire, and a scheduled audit MUST ignore only advisories with a documented disposition so it fails on genuinely new findings.

#### Scenario: A security-relevant check is nested inside a rarely-firing trigger

- **Given** a repository's documentation claims a security audit step runs
  as part of another workflow (e.g. a release pipeline)
- **When** that other workflow's own trigger condition (e.g. a version-tag
  push) has never actually fired
- **Then** a dedicated workflow with its own schedule and/or
  `workflow_dispatch` trigger MUST be added, independent of the other
  workflow's trigger, and the documentation MUST be corrected to describe
  the actual trigger

#### Scenario: A scheduled audit ignores advisories

- **Given** a scheduled audit job ignores one or more advisory IDs to avoid
  permanent failure on already-triaged findings
- **When** the ignore list is defined
- **Then** each ignored ID MUST correspond to a disposition already
  documented (fixed-but-still-listed, mitigated, or accepted-risk) in the
  project's dependency-management documentation, so an advisory outside
  that list still fails the job

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

### Requirement: Architectural Decision Record Accuracy

An architectural decision record's factual claims about pinned dependency state SHALL be re-verified against live manifest state whenever a related dependency change lands, rather than assumed correct from a prior reading.

#### Scenario: A decision record's claim no longer matches the manifest

- **Given** an architectural decision document makes a specific claim
  about how a dependency is pinned (branch, tag, or commit)
- **When** live `Cargo.toml` (or equivalent manifest) state is checked
  directly and found to differ from the claim
- **Then** the decision record MUST be corrected to match live state, and
  the correction MUST be disclosed rather than silently edited without a
  trace of what was wrong

#### Scenario: A parallel record also drifted

- **Given** correcting one document's claim reveals that a parallel
  document tracking the same fact (e.g. a "current pinned versions"
  table) has also drifted from live manifest state
- **When** the correction is made
- **Then** all drifted entries MUST be corrected in the same change, not
  just the one that prompted the investigation

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

