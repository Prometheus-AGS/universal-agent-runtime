## ADDED Requirements

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
