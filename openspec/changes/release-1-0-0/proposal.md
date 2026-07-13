## Why

First stable release. Version was 0.1.0 while SECURITY.md advertises a 1.0.x
support line; the release pipeline published a single-arch Docker Hub image with
no SBOM, signing, or provenance — below 2026 supply-chain baselines.

## What Changes (PREPARED — operator cuts the tag)

- Bump versions 0.1.0 -> 1.0.0 (Cargo.toml + root/frontend/sdks package.json).
- Add CHANGELOG.md (Keep-a-Changelog) with a 1.0.0 entry + a written stability
  statement (SemVer scope for the public HTTP/config/agent-descriptor surface).
- Extend release.yml with a `publish-ghcr-signed` job: GHCR multi-arch
  (amd64/arm64) build+push, SLSA provenance (`provenance: mode=max`), SBOM
  (build-push-action `sbom: true` + cargo-cyclonedx + syft image SBOM), and
  cosign keyless signing — all via GITHUB_TOKEN + OIDC, no new secrets.

## Not done here (operator action)

- Tagging `v1.0.0` and running the release pipeline for real. Per operator
  decision, this change PREPARES the release; the operator reviews and triggers
  it (first release ever — expect pipeline fixes on the first real run).

## Capabilities

### New Capabilities
- `release-pipeline`: a versioned, signed, SBOM-bearing, multi-arch release
  published to GHCR with SLSA provenance.

## Impact

Version bumps + CHANGELOG + one additive release.yml job. No runtime code.
KBD: change 9/9 (prepared; operator triggers the tag).
