## ADDED Requirements

### Requirement: README estate follows documented ownership and navigation
Every tracked README SHALL have an authoritative source-manifest disposition. UAR-owned current README files SHALL identify their scope, link to the canonical portal or parent navigation, and avoid duplicating broad product claims that belong in the portal. Generated mirrors SHALL be updated from their source, historical README files SHALL carry dated supersession context, and vendored README files SHALL remain semantically unchanged.

#### Scenario: UAR-owned README is reconciled
- **WHEN** a reader opens a current UAR-owned README
- **THEN** it describes that directory or package accurately and links to the canonical public documentation for broader guidance

#### Scenario: Generated README changes
- **WHEN** a generated README requires a semantic correction
- **THEN** the correction is made in its declared source and the mirror is regenerated

#### Scenario: Vendored README is audited
- **WHEN** the documentation estate is reconciled
- **THEN** the vendored README remains unchanged and its third-party ownership and exclusion are recorded in the source manifest

### Requirement: Repository presentation exposes the working public portal
The root README SHALL present the UAR identity, purpose, support boundaries, and a public documentation link that resolves to the branded portal. The repository homepage field SHALL use the same URL after the deployed site has been observed working.

#### Scenario: Reader follows the documentation link
- **WHEN** a reader follows the root README documentation link
- **THEN** the branded Docusaurus portal root loads and its canonical deep routes are reachable

#### Scenario: Repository homepage is inspected
- **WHEN** a visitor opens the GitHub repository page after documentation deployment
- **THEN** the homepage field links to the same validated public portal URL

