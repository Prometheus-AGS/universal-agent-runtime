## Why

The exact-source CI run required by candidate supply-chain certification fails because Clippy lints a vendored path dependency whose crate-level policy denies its own warnings. CI must lint UAR targets without converting dependency-owned warnings into UAR release failures.

## What Changes

- Restrict the primary CI Clippy invocation to UAR targets by using Clippy's dependency-exclusion flag.
- Preserve the project's `Cargo.toml` lint policy and the existing feature coverage.
- Add a static workflow assertion so dependency linting cannot silently re-enter the release gate.
- Keep deterministic release tests on their recorded fixture model instead of overriding them with a smoke-test model.
- Install `protoc` before resilience archive builds and provide an outer Docker shutdown margin beyond the runtime budget.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `ci-pipeline-health`: Clarify that dependency-owned Clippy warnings do not fail the UAR CI gate while UAR targets remain linted.

## Impact

- Affects CI, release, and operational-resilience workflows plus their static validators.
- No runtime UX, provider compatibility, API, dependency, or realtime-state behavior changes.
- KBD release state must supersede RC3 with a newly signed immutable candidate after the fix merges.
