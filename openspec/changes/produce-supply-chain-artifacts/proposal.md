## Why

Release artifacts lack a complete signed evidence set tying binaries and images to source, dependencies and tests.

## What Changes

- Generate SBOMs, checksums, keyless signatures and SLSA provenance.
- Publish signed multi-arch images.
- Create a machine-readable release manifest.

## Capabilities
### New Capabilities
- `signed-release-evidence`

## Impact
Release workflow, artifact publication, GHCR and release documentation.
