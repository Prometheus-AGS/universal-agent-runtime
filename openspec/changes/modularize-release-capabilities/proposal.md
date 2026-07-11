## Why

Cargo features do not control major capability dependencies; empty backend labels and maintainer-only features make release matrices misleading.

## What Changes

- Optionalize heavyweight platform and backend dependencies.
- Add tested release bundles.
- Move model generation out of product features and isolate/remove unsupported Memory Palace integration.

## Capabilities
### New Capabilities
- `modular-release-capabilities`

## Impact
Cargo graph, Rust cfg/module boundaries, CI matrices and downstream consumers.
