# SLSA L3 self-attestation

## Purpose

Extend the existing signed/attested release pipeline with first-party
SBOM attestation, nightly cross-ecosystem vulnerability scanning, and
an RFC 9116 `security.txt` — closing the gaps found when auditing the
already-substantial supply-chain infrastructure in `supply-chain.yml`
and `ci.yml`.

## ADDED Requirements

### Requirement: SBOM attestation for canonical release artifacts
`supply-chain.yml` MUST attest at least the Linux x64 release tarball
and the container image against their own generated SBOMs via
`actions/attest-sbom`, in addition to the existing provenance
attestation of the payload checksums.

#### Scenario: A release is cut
- **WHEN** `supply-chain.yml`'s `artifacts-and-image` job runs for a
  release tag
- **THEN** the Linux x64 tarball and the container image each have
  their own SBOM attestation, verifiable via `gh attestation verify`

### Requirement: Nightly cross-ecosystem vulnerability scanning
A scheduled workflow MUST run `osv-scanner` against the repository's
dependency manifests and `grype` against the release container image
(or a locally-built equivalent), independent of the weekly
Rust-specific `cargo audit` in `security-audit.yml`. Both MUST fail the
job on HIGH-or-above severity findings.

#### Scenario: A HIGH-severity vulnerability is introduced
- **WHEN** a dependency update or base-image change introduces a
  HIGH-or-above severity vulnerability
- **THEN** the next nightly `vuln-scan.yml` run fails
- **AND** results are also uploaded to the GitHub code scanning
  dashboard (SARIF)

### Requirement: security.txt endpoint
`GET /.well-known/security.txt` MUST serve an RFC 9116-compliant
document whose `Contact` field points at the project's actual
vulnerability-reporting channel (not a placeholder), consistent with
`SECURITY.md`.

#### Scenario: A researcher looks up how to report a vulnerability
- **WHEN** a security researcher requests
  `GET /.well-known/security.txt`
- **THEN** they receive a `Contact` field pointing at GitHub private
  vulnerability reporting, an `Expires` date, and a `Policy`/`Canonical`
  link to `SECURITY.md`
- **AND** the document contains no fabricated contact information

### Requirement: Public SLSA L3 self-declaration
The README MUST self-declare SLSA L3 provenance and document the
verification commands a consumer can run themselves against a
downloaded release artifact or the published container image.

#### Scenario: A consumer wants to verify a downloaded release archive
- **WHEN** a consumer downloads a release tarball
- **THEN** the README shows the exact `cosign verify-blob` command,
  using the real certificate-identity regex the release workflow signs
  with
- **AND** running it against the corresponding `.sigstore.json` bundle
  succeeds
