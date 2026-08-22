## 1. Artifact evidence
<!-- EVIDENCE: generation/signing integrations exist; check only real candidate outputs. -->
- [ ] 1.1 Generate CycloneDX/SPDX SBOMs for binaries/source/image.
- [ ] 1.2 Generate SHA-256 checksums for every downloadable artifact.
- [x] 1.3 Add keyless signing and verification instructions.
- [ ] 1.4 Generate SLSA provenance tied to source SHA and local builder identity.
## 2. Images and manifest
<!-- EVIDENCE: requires published digest-addressed candidate image. -->
- [ ] 2.1 Build/sign supported multi-arch GHCR images and verify non-root execution.
- [x] 2.2 Emit release manifest linking artifacts, SBOM, provenance, tests, audits and support matrix.
## 3. Verify
<!-- EVIDENCE: requires independent verification of published candidate artifacts. -->
- [ ] 3.1 Verify signatures/checksums/provenance in a separate local process.
- [ ] 3.2 Test artifact download/install and validate OpenSpec.
