## Why

Release artifacts lack a complete signed evidence set tying binaries and images to source, dependencies and tests.

## What Changes

- Generate SBOMs, checksums, keyless signatures and SLSA provenance.
- Publish signed multi-arch images.
- Create a machine-readable release manifest whose builder and test/audit
  evidence are local source-bound receipts rather than GitHub Actions runs.
- Generate and independently verify the complete evidence set locally before
  any operator-authorized publication.

## Capabilities
### New Capabilities
- `signed-release-evidence`

## Impact
Local release tooling, artifact publication, GHCR and release documentation.
