# Supply Chain

UAR ships source-bound local SLSA provenance, CycloneDX/SPDX SBOMs, and keyless
Sigstore signatures. A separate local verification process reopens the exact
signed checksum set and rejects any added, removed, or changed evidence. Local
security-audit and test receipts are hashed into the release manifest.

GitHub Actions are reserved for deployment execution and deployment-specific
validation. Product tests, security scans, release builds, load/soak tests, and
release certification run locally before publication. UAR does not claim a
hosted-build SLSA level for this local builder.

## Topics

- Source-bound local SLSA provenance
- SBOM generation and signed checksum coverage
- Local vulnerability scanning
- Reproducible builds verification
- `/.well-known/security.txt`

## Related documents

- [Dependency management](https://github.com/Prometheus-AGS/universal-agent-runtime/blob/main/docs/DEPENDENCY_MANAGEMENT.md)
- [Build reproducibility](https://github.com/Prometheus-AGS/universal-agent-runtime/blob/main/docs/build-reproducibility.md)
- [Supply chain ADR](https://github.com/Prometheus-AGS/universal-agent-runtime/blob/main/docs/adr/0006-slsa-l3-supply-chain.md)
- [Security policy](https://github.com/Prometheus-AGS/universal-agent-runtime/blob/main/SECURITY.md)
