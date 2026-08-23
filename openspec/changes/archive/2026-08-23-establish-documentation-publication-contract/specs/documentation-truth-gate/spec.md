## MODIFIED Requirements

### Requirement: Canonical documentation matches certified product behavior
Present-tense public documentation SHALL identify React as the primary UI, use the current support matrix and executable contracts for protocol, provider, feature, platform, and profile claims, and be covered by the source-classification and public-route manifests. Historical and private-synthesis-only sources SHALL NOT become current authority merely because they are checked in.

#### Scenario: Stale HTMX claim
- **WHEN** an unmarked canonical document claims the primary UI avoids React or uses HTMX/Web Components
- **THEN** documentation validation fails

#### Scenario: Historical design
- **WHEN** superseded design material is retained
- **THEN** it contains a dated historical banner and link to current architecture

#### Scenario: Unclassified public claim
- **WHEN** a present-tense public document is absent from the source manifest or lacks a governing authority
- **THEN** documentation validation fails and names the file and missing metadata

#### Scenario: Private source appears in public output
- **WHEN** a `private-synthesis-only` or `excluded` source is copied into a public source or built artifact
- **THEN** the publication sanitizer rejects the artifact

#### Scenario: Unsupported inference evidence is promoted
- **WHEN** public documentation describes mocked, recorded, replayed, or synthetic output as real-model inference or readiness evidence
- **THEN** documentation validation fails and identifies the unsupported claim

#### Scenario: Routine verification is assigned to GitHub Actions
- **WHEN** a public method document tells contributors to rely on GitHub Actions for routine documentation or product testing
- **THEN** documentation validation fails and points to the local verification authority

