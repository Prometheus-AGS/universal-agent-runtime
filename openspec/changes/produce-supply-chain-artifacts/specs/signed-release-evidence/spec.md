## ADDED Requirements

### Requirement: Release artifacts are traceable and verifiable
Every binary, archive and image SHALL have a checksum, SBOM, signature and provenance record bound to the released source commit.

#### Scenario: Artifact verification
- **WHEN** a customer downloads a release archive
- **THEN** documented commands verify its checksum, signature and provenance without trusting an unsigned side channel
