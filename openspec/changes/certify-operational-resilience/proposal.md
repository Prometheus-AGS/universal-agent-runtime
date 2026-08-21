## Why

Production use requires proof under shutdown, cancellation, outages, restarts, concurrency, long streams and persistence recovery—not just unit/build success.

## What Changes

- Add deterministic lifecycle, failure, load, soak and recovery certification.
- Test non-root container and backup/restore behavior.
- Run all product certification locally against one immutable candidate and
  retain replayable thresholds and reports.
- Remove GitHub Actions workflows that perform product, build-gate,
  release-certification, security, performance, or other non-deployment tests;
  retain Actions only for deployment execution and deployment-specific checks.
- Repair the installed-artifact build boundary so the embedded
  entity-management workspace installs and builds with its authenticated,
  pinned package manager.

## Capabilities
### New Capabilities
- `operational-resilience-certification`

## Impact
Local integration/load tests, containers, retained evidence, workflow policy,
runbooks, and the pinned entity-management tooling receipt. Product runtime
behavior is unchanged.
