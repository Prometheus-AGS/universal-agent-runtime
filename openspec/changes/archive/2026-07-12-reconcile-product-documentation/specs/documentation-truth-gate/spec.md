## ADDED Requirements

### Requirement: Canonical documentation matches certified product behavior
Present-tense public documentation SHALL identify React as the primary UI and SHALL use the support matrix for protocol, provider, feature and platform claims.

#### Scenario: Stale HTMX claim
- **WHEN** an unmarked canonical document claims the primary UI avoids React or uses HTMX/Web Components
- **THEN** documentation validation fails

#### Scenario: Historical design
- **WHEN** superseded design material is retained
- **THEN** it contains a dated historical banner and link to current architecture
