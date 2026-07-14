## Why

The 2026-07-13 release-readiness assessment named several supply-chain
gaps: no SLSA provenance, no SBOM attestation, no cross-ecosystem
vulnerability scanning beyond `cargo audit`, and no `security.txt`.
Auditing the actual CI state during this change found the picture is
better than the plan assumed: `.github/workflows/supply-chain.yml`
already builds, signs (Sigstore/cosign), generates SBOMs (syft,
CycloneDX + SPDX), attests provenance (`actions/attest@v4`) for the
release payload checksums, and independently re-verifies everything in
a separate `verify` job. `ci.yml` already has an "Offline Reproducible
Source" job. The genuinely missing pieces are narrower than the plan's
"new provenance.yml" implied.

## What Changes

- `actions/attest-sbom@v4` added to `supply-chain.yml`: attests the
  Linux x64 release tarball and the container image against their own
  already-generated per-artifact SBOMs (first-party SBOM attestation,
  separate from the existing payload-checksum provenance attestation).
- New `.github/workflows/vuln-scan.yml`: nightly, `osv-scanner`
  (cross-ecosystem dependency scan, `fail-on-vuln: true`) + `grype`
  (container-image scan against the release `Dockerfile` built locally,
  `severity-cutoff: high`, `fail-build: true`) — decoupled from the
  existing weekly Rust-only `cargo audit` in `security-audit.yml`.
- New `GET /.well-known/security.txt` (RFC 9116) in `src/server.rs`,
  pointing at the project's actual reporting channel (GitHub private
  vulnerability reporting, per the existing `SECURITY.md`) and its
  documented 90-day coordinated-disclosure default — not a fabricated
  email/PGP key.
- New README "Supply-chain provenance" section: SLSA L3 self-declaration,
  `cosign verify-blob` / `gh attestation verify` proof commands using
  the actual existing signing identity, and links to the vuln-scan and
  reproducible-source jobs.

## Capabilities

### New Capabilities

- `slsa-l3-self-attestation`: the SBOM-attestation addition, the nightly
  osv-scanner/grype workflow, and the `security.txt` endpoint.

## Impact

- **No new release workflow.** `supply-chain.yml`'s existing job
  structure (build → sign → attest → verify → publish) is extended in
  place with 2 new steps, not duplicated into a parallel workflow.
- **New workflow file:** `vuln-scan.yml`, independent of the release
  cadence (runs nightly against `main`, not against release artifacts
  specifically — see "Out of scope" below).
- **New route:** `GET /.well-known/security.txt`, additive.
- **No new Cargo dependency.** Only GitHub Actions marketplace actions
  (`actions/attest-sbom@v4`, `google/osv-scanner-action` reusable
  workflow, `anchore/scan-action@v7`) were added, all checked against
  their current released versions rather than assumed minimums.

## Out of scope (scope corrections vs. the original plan)

- **A new `.github/workflows/provenance.yml`.** Not created — SLSA
  provenance generation already exists in `supply-chain.yml` via
  `actions/attest@v4` (GitHub's modern, actively-maintained mechanism;
  the plan's named `slsa-github-generator` is the older, more complex
  reusable-TRW-workflow approach and was never actually needed once the
  existing infrastructure was audited).
- **Reproducible-builds verification "best-effort."** Already fully
  implemented (`ci.yml`'s "Offline Reproducible Source" job, building
  two isolated copies and diffing them) — no new work; the plan
  under-counted existing coverage.
- **SBOM attestation for every per-platform release artifact** (macOS,
  Windows, offline-source archive, in addition to the Linux x64 tarball
  attested here). `actions/attest-sbom` takes exactly one subject per
  invocation with no batch/loop-friendly form inside a single YAML step;
  attesting the remaining artifacts is a small, mechanical follow-up
  (copy the "Attest Linux x64 tarball SBOM" step, change the `find`
  pattern), deferred rather than hand-unrolling 4+ near-identical steps
  in this pass.
- **`vuln-scan.yml` scanning "published artifacts" specifically.**
  Scoped to scanning `main`'s dependency manifests (osv-scanner) and a
  locally-built (unpushed) copy of the release `Dockerfile` (grype) —
  this needs no registry credentials and can run nightly independent of
  release cadence. Scanning the actual last-published ghcr.io image
  would require wiring registry pull credentials into a new workflow;
  deferred as a bounded follow-up once the operator confirms which
  registry (ghcr.io per `supply-chain.yml`, or the ACR image in
  `deploy.yml`) is the canonical "published artifact" to target.
