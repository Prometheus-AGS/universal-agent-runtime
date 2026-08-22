## ADDED Requirements

### Requirement: Release artifacts are traceable and verifiable
Every binary, archive and image SHALL have a checksum, SBOM, signature and provenance record bound to the released source commit.

#### Scenario: Artifact verification
- **WHEN** a customer downloads a release archive
- **THEN** documented commands verify its checksum, signature and provenance without trusting an unsigned side channel

#### Scenario: Local evidence production
- **WHEN** release evidence is generated
- **THEN** its manifest binds the immutable source SHA, local builder identity, and hashed local test/audit receipts without requiring a GitHub Actions run

#### Scenario: Independent local verification
- **WHEN** the evidence producer finishes
- **THEN** a separate local verification process re-downloads or reopens the exact indexed set and rejects any missing, added, or modified file

#### Scenario: Local security evidence
- **WHEN** release evidence is prepared
- **THEN** local Rust, JavaScript, OSV, image and Dependabot checks pass for the same source and digest-addressed candidate image, and their receipt is hashed into the manifest
