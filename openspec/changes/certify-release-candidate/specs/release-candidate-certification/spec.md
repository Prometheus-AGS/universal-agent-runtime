## ADDED Requirements

### Requirement: GA derives from an immutable certified candidate
The GA source commit SHALL have passed the complete supported capability, platform, security, offline-build and operational matrices as an immutable release candidate.

The candidate tag SHALL point to source already reporting the final `1.0.0`
product version so promotion to GA requires no source or artifact rebuild.

#### Scenario: Candidate source changes
- **WHEN** any source, dependency lock, embedded catalog/model or build workflow changes after certification
- **THEN** candidate certification is invalidated and rerun

#### Scenario: External installation
- **WHEN** an adopter follows published installation docs without a repository checkout
- **THEN** UAR starts, passes health checks and completes the stable smoke journey
