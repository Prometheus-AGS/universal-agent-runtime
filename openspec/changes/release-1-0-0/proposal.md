## Why

First stable release. Version was 0.1.0 while SECURITY.md advertises a 1.0.x
support line; the release pipeline published a single-arch Docker Hub image with
no SBOM, signing, or provenance — below 2026 supply-chain baselines.

## What Changes (PREPARED — operator authorizes promotion)

- Bump versions 0.1.0 -> 1.0.0 (Cargo.toml + root/frontend/sdks package.json).
- Add CHANGELOG.md (Keep-a-Changelog) with a 1.0.0 entry + a written stability
  statement (SemVer scope for the public HTTP/config/agent-descriptor surface).
- Build and certify signed archives and the GHCR amd64/arm64 image under the
  next unused candidate tag, `v1.0.0-rc.3`.
- Promote the certified source, archive bytes, evidence, and OCI manifest digest
  unchanged to `v1.0.0`; the GA tag does not trigger a second build.

## Not done here (operator action)

- Creating the candidate or GA tags and publishing GitHub/GHCR state. Those are
  operator-authorized effects. The candidate pipeline produces the bytes once;
  GA promotion reuses them without rerunning a build workflow.

## Capabilities

### New Capabilities
- `release-pipeline`: a versioned, signed, SBOM-bearing, multi-arch release
  published to GHCR with SLSA provenance.

## Impact

Version alignment, candidate release automation, and guarded no-rebuild GA
promotion. No runtime code.
