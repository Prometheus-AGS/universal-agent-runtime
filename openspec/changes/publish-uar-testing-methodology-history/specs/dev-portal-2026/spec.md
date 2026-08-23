## ADDED Requirements

### Requirement: Public testing methodology

The portal SHALL publish testing history, an evidence taxonomy, negative-control
practice, and local verification timing. It SHALL preserve the move from
coverage/synthetic emphasis to bounded genuine-model functional acceptance
without claiming earlier methods were useless or later evidence was broader than
its recorded server-full scope.

#### Scenario: Reader chooses a verification method

- **WHEN** a reader needs evidence for a code, inference, resilience, or deployment claim
- **THEN** the portal identifies the narrowest applicable evidence class, timing boundary, and non-claims

#### Scenario: Routine test is placed in Actions

- **WHEN** documentation proposes unit, integration, conformance, lint, format, type, or routine docs verification in GitHub Actions
- **THEN** local policy validation exits non-zero because Actions are deployment-only
