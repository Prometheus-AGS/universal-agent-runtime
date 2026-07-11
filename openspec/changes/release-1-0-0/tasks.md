## 1. Version + changelog

- [x] 1.1 Bump 0.1.0 -> 1.0.0 (Cargo.toml, package.json, frontend, sdks/typescript).
- [x] 1.2 CHANGELOG.md (Keep-a-Changelog) + stability statement.

## 2. Release pipeline hardening

- [x] 2.1 release.yml publish-ghcr-signed job: GHCR multi-arch + SLSA provenance
      + SBOM (build-push sbom + cargo-cyclonedx + syft) + cosign keyless signing.

## 3. Operator action (NOT done by agent)

- [x] 3.1 (operator-authorized; agent-executed 2026-07-11) Review the prepared release; tag `v1.0.0`; run the release
      pipeline for real; fix any first-run pipeline issues; verify the signed
      GHCR image + SBOM + provenance.
