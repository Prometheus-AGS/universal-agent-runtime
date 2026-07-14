# 6. Adopt SLSA Level 3 and OSV/Grype for supply-chain security

Date: 2026-07-13

## Status

Accepted

## Context

UAR consumes a large dependency graph and ships a container image. Operators and enterprise consumers need evidence of provenance, SBOMs, and vulnerability posture. The operator requested SLSA Level 3 self-attestation as the target.

## Decision

- Use `slsa-github-generator` in a reusable `.github/workflows/provenance.yml`.
- Sign provenance in an isolated job separate from the build job.
- Attest SBOMs with `actions/attest-sbom`.
- Run nightly vulnerability scans with `osv-scanner` and `grype`; block on `HIGH` and `CRITICAL` findings for published artifacts.
- Publish `/.well-known/security.txt` with a PGP key and a 90-day disclosure SLA.
- Add a best-effort reproducible-builds verification job.

## Consequences

- Consumers can verify the runtime artifact with `slsa-verifier` and `cosign`.
- Vulnerability findings are surfaced within 24 hours of database updates.
- Release artifacts include signed provenance and SBOM attestations.

## Alternatives considered

- Outsource to a third-party attestation service: rejected because it introduces vendor lock-in and ongoing cost.
- SLSA Level 2: rejected because the operator explicitly wanted Level 3.
