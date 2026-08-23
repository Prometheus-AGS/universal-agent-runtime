## ADDED Requirements

### Requirement: Canonical public portal metadata

The sole Pages deployment workflow SHALL publish and validate the complete
Docusaurus artifact. After the canonical URL is observed working, the repository
homepage and root README SHALL point to that URL.

#### Scenario: Reader enters from the repository

- **WHEN** a reader uses the repository homepage field or README documentation link
- **THEN** the link opens the observed branded portal and representative deep links remain reachable

#### Scenario: Actions workflow performs routine testing

- **WHEN** the Pages workflow contains unit, integration, conformance, lint, format, type, or other routine development checks
- **THEN** the local GitHub Actions policy gate fails before publication
