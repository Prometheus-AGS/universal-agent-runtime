# Supply Chain

UAR ships SLSA provenance and SBOM attestations via GitHub's native `actions/attest`/`actions/attest-sbom`, keyless Sigstore signatures with an independent in-workflow verification job, reproducible-source checks on every CI run, and nightly vulnerability scanning with `osv-scanner` and `grype`.

GitHub-native artifact attestations from an in-repo build workflow provide SLSA v1.0 **Build Level 2**; Build Level 3 additionally requires the build steps to run in a dedicated reusable workflow. UAR currently claims L2 and is on the L3 track (migrating the build/sign steps into a reusable workflow is planned).

## Topics

- SLSA provenance attestation (Build L2 attested, L3-track)
- SBOM generation and attestation
- Vulnerability scanning (`vuln-scan.yml`)
- Reproducible builds verification
- `/.well-known/security.txt`

## Related documents

- [Dependency management](https://github.com/Prometheus-AGS/universal-agent-runtime/blob/main/docs/DEPENDENCY_MANAGEMENT.md)
- [Build reproducibility](https://github.com/Prometheus-AGS/universal-agent-runtime/blob/main/docs/build-reproducibility.md)
- [Supply chain ADR](https://github.com/Prometheus-AGS/universal-agent-runtime/blob/main/docs/adr/0006-slsa-l3-supply-chain.md)
- [Security policy](https://github.com/Prometheus-AGS/universal-agent-runtime/blob/main/SECURITY.md)
