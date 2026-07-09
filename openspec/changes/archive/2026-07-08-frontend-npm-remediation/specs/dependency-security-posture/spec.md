## ADDED Requirements

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
