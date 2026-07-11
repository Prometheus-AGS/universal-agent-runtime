## ADDED Requirements

### Requirement: Releases are versioned, signed, and SBOM-bearing

A release SHALL publish a multi-arch container image to GHCR, signed with cosign
and accompanied by an SBOM and SLSA provenance, at a semantic version matching
the crate and package manifests.

#### Scenario: Tagging a version publishes a signed image

- **When** a `v1.0.0` tag is pushed and the release pipeline runs
- **Then** it MUST publish `ghcr.io/prometheus-ags/universal-agent-runtime`
  for linux/amd64 and linux/arm64, sign it with cosign (keyless), and attach an
  SBOM and SLSA provenance

#### Scenario: Manifest versions agree

- **Given** the crate and package manifests
- **Then** `Cargo.toml` and each `package.json` version MUST equal the released
  tag version (1.0.0)
