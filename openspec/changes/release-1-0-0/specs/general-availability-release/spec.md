## ADDED Requirements

### Requirement: Version and support surfaces agree
Cargo, npm, CLI, images, docs, changelog, tag and SECURITY policy SHALL report the same GA version.

#### Scenario: Publish 1.0.0
- **WHEN** `v1.0.0` is published
- **THEN** all public artifacts identify 1.0.0 and are derived from the certified candidate commit

### Requirement: GA evidence remains public
The release SHALL publish its checksums, SBOM, signatures, provenance, test/security reports and support matrix.

#### Scenario: Customer audit
- **WHEN** a customer audits the release
- **THEN** they can trace each artifact to source and verification evidence
