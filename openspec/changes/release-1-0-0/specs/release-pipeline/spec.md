## ADDED Requirements

### Requirement: Releases are versioned, signed, and SBOM-bearing

A release SHALL publish a multi-arch container image to GHCR, signed with cosign
and accompanied by an SBOM and SLSA provenance, at a semantic version matching
the crate and package manifests.

#### Scenario: Promoting the certified candidate publishes the GA image

- **When** the operator promotes certified candidate `v1.0.0-rc.3` to `v1.0.0`
- **Then** it MUST publish the exact certified linux/amd64 and linux/arm64 OCI
  manifest digest as `ghcr.io/prometheus-ags/universal-agent-runtime:1.0.0`,
  retain its cosign signature, SBOM and SLSA provenance, and perform no rebuild

#### Scenario: Manifest versions agree

- **Given** the crate and package manifests
- **Then** `Cargo.toml` and each `package.json` version MUST equal the released
  tag version (1.0.0)
