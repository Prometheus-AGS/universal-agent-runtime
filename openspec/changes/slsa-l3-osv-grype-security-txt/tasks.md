## 1. Audit existing supply-chain infrastructure
- [x] 1.1 Confirmed `supply-chain.yml` already: builds+pushes multi-arch image, generates SBOMs (syft), signs (cosign keyless), attests provenance (`actions/attest@v4`), and independently re-verifies in a separate `verify` job.
- [x] 1.2 Confirmed `ci.yml`'s "Offline Reproducible Source" job already satisfies the reproducible-builds done condition.
- [x] 1.3 Confirmed no `security.txt`, no `actions/attest-sbom`, and no osv-scanner/grype anywhere — these are the real gaps.

## 2. SBOM attestation
- [x] 2.1 `actions/attest-sbom@v4` step attesting the Linux x64 release tarball against its per-artifact SBOM.
- [x] 2.2 `actions/attest-sbom@v4` step attesting the container image against its SBOM.
- [ ] 2.3 Remaining per-platform artifacts (macOS, Windows, offline-source archive). Deferred — mechanical follow-up (copy step 2.1, change the `find` pattern).

## 3. Nightly vulnerability scanning
- [x] 3.1 New `.github/workflows/vuln-scan.yml`: `osv-scanner` reusable workflow (`google/osv-scanner-action/.github/workflows/osv-scanner-reusable.yml@v2.3.8`), recursive scan of `main`, `fail-on-vuln: true`, SARIF upload.
- [x] 3.2 `grype` job: builds `Dockerfile` locally (unpushed, no registry credentials needed), scans with `anchore/scan-action@v7`, `severity-cutoff: high`, `fail-build: true`, SARIF upload to code scanning.
- [ ] 3.3 Scanning the actual last-published registry image (vs. a locally-rebuilt copy of the Dockerfile). Deferred pending operator confirmation of the canonical registry (ghcr.io vs. ACR — see proposal.md).

## 4. security.txt
- [x] 4.1 `GET /.well-known/security.txt` handler in `src/server.rs`, RFC 9116 fields: `Contact` (GitHub private vulnerability reporting URL, not email/PGP), `Expires` (1 year out, needs annual operator rotation), `Preferred-Languages`, `Canonical`/`Policy` (link to `SECURITY.md`).
- [x] 4.2 Route registered alongside `/health`/`/.well-known/uar-config`.
- [x] 4.3 Unit test asserting the required fields are present and the reporting URL is the real one (not a placeholder).

## 5. README
- [x] 5.1 New "Supply-chain provenance (SLSA L3 self-declared)" section: what's attested, the independent `verify` job, `cosign verify-blob` and `gh attestation verify` proof commands using the real signing identity regex already used in `supply-chain.yml`'s own `verify` job.
- [x] 5.2 Links to `vuln-scan.yml` and the existing reproducible-source job.
- [x] 5.3 Security section links `SECURITY.md` and the new `/.well-known/security.txt`.

## 6. Verification
- [x] 6.1 `.github/workflows/supply-chain.yml` and `vuln-scan.yml` YAML syntax validated (`python3 -c "import yaml; ..."`).
- [x] 6.2 `cargo check --no-default-features --features server-full` clean (verified after a concurrent Change 7 (OpenCode) edit in the same working tree landed — see execution.md for the concurrency note).
- [x] 6.3 New `security_txt_handler` unit test passes.
- [ ] 6.4 **Deferred to consolidated validation pass**: actually running `vuln-scan.yml`/`supply-chain.yml` in GitHub Actions (both require live CI, not reproducible in this pass); full-workspace `cargo fmt`/`cargo clippy`.
