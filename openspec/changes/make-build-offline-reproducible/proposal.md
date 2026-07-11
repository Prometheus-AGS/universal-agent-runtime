## Why

Git dependencies and implicit build-script downloads prevent clean offline reproduction and complicate supply-chain verification.

## What Changes

- Publish/vendor Git dependencies.
- Version and digest catalog/model snapshots.
- Eliminate mandatory network access and enforce locked offline builds.

## Capabilities
### New Capabilities
- `offline-reproducible-build`

## Impact
Dependency sourcing, build.rs, artifacts, maintainer workflows and CI.
