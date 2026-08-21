## Why

Production use requires proof under shutdown, cancellation, outages, restarts, concurrency, long streams and persistence recovery—not just unit/build success.

## What Changes

- Add deterministic lifecycle, failure, load, soak and recovery certification.
- Test non-root container and backup/restore behavior.
- Publish thresholds and reports.
- Repair the installed-artifact build boundary so the embedded
  entity-management workspace installs and builds with its authenticated,
  pinned package manager.

## Capabilities
### New Capabilities
- `operational-resilience-certification`

## Impact
Integration/load tests, containers, CI artifacts, runbooks, and the pinned
entity-management tooling receipt. Product runtime behavior is unchanged.
