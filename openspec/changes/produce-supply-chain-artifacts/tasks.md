## 1. Artifact evidence
- [ ] 1.1 Generate CycloneDX/SPDX SBOMs for binaries/source/image.
- [ ] 1.2 Generate SHA-256 checksums for every downloadable artifact.
- [x] 1.3 Add keyless signing and verification instructions.
- [ ] 1.4 Generate SLSA provenance tied to source SHA and workflow identity.
## 2. Images and manifest
- [ ] 2.1 Build/sign supported multi-arch GHCR images and verify non-root execution.
- [x] 2.2 Emit release manifest linking artifacts, SBOM, provenance, tests, audits and support matrix.
## 3. Verify
- [ ] 3.1 Verify signatures/checksums/provenance in a separate job.
- [ ] 3.2 Test artifact download/install and validate OpenSpec.
