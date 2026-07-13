## 1. Version alignment
- [x] 1.1 Set Cargo/npm/CLI/image/docs/schema versions to 1.0.0.
- [x] 1.2 Finalize changelog, stability/compatibility/migration and support policies.
- [x] 1.3 Verify SECURITY.md supported line matches published artifacts.
## 2. Publish
- [ ] 2.1 Confirm source equals certified candidate; otherwise rerun certification.
- [ ] 2.2 Create signed `v1.0.0` tag and run release workflow.
- [ ] 2.3 Publish GitHub release, signed images, SBOM/provenance/checksums and evidence manifest.
## 3. Post-release
- [ ] 3.1 Download and verify all artifacts from public endpoints.
- [ ] 3.2 Run production smoke/health and documentation link checks.
- [ ] 3.3 Archive OpenSpec changes and close KBD phase only if all goals are met.
