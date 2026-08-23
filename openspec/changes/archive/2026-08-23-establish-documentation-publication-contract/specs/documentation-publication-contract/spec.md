## Purpose

Define the complete, privacy-preserving contract that turns UAR's current, historical, generated, and third-party documentation sources into one traceable public documentation estate.

## ADDED Requirements

### Requirement: Every documentation source has an authoritative disposition
The project SHALL maintain a machine-readable source manifest that assigns every tracked README and documentation path exactly one disposition: `public`, `public-normalize`, `private-synthesis-only`, or `excluded`. Each entry SHALL identify its owner, current or historical status, canonical authority, public destination when applicable, and regeneration source when generated.

#### Scenario: An unclassified documentation path is added
- **WHEN** a tracked README or documentation path is absent from the source manifest
- **THEN** local documentation validation fails and identifies the unclassified path

#### Scenario: A generated mirror is classified
- **WHEN** a README is generated from another checked-in source
- **THEN** its manifest entry identifies that source and forbids independent semantic editing of the mirror

#### Scenario: A vendored document is classified
- **WHEN** a tracked document is owned by a third-party or git submodule
- **THEN** it is classified as `excluded` with its ownership recorded and is not rewritten as UAR-authored content

### Requirement: Private history is published only through reviewed synthesis
Raw `.prometheus` records, KBD event payloads, conversation or session logs, machine-local paths, credential-like material, and unreviewed wiki records MUST NOT enter a public documentation artifact. Public history derived from private-synthesis-only sources SHALL be reviewed prose with source provenance and SHALL preserve observed decisions, reversals, and evidence limits without inventing rationale.

#### Scenario: Raw history is copied into public content
- **WHEN** a public source or built artifact contains a raw private-history record, a machine-local user path, or secret-like material
- **THEN** the publication sanitizer fails and reports the source and matched rule

#### Scenario: A historical decision is synthesized
- **WHEN** public prose explains a decision derived from `.prometheus`, KBD, OpenSpec, or an ADR
- **THEN** the page identifies the relevant checked-in source records and distinguishes observed rationale from later interpretation

#### Scenario: The sanitizer negative control is executed
- **WHEN** a fixture containing a raw session record, a machine-local path, and secret-like content is passed to the sanitizer
- **THEN** the sanitizer rejects the fixture and returns a non-zero result

### Requirement: Public routes are complete and traceable
The project SHALL maintain an authoritative public-route manifest that maps the supported product inventory to canonical Docusaurus routes and identifies each route's governing source. Present-tense claims SHALL name their supported profile and current authority; retained historical documents SHALL carry a dated supersession banner and link to current authority.

#### Scenario: A supported product surface lacks a public route
- **WHEN** a product surface in the authoritative inventory has no canonical public route or explicit exclusion
- **THEN** local documentation validation fails and names the uncovered surface

#### Scenario: A historical document contradicts current architecture
- **WHEN** retained historical material describes a superseded architecture or method
- **THEN** it remains available with a dated historical banner and a link to the current governing document

#### Scenario: A current claim lacks a profile boundary
- **WHEN** public documentation makes a support or readiness claim that differs by `server-full`, `minimal`, or `embedded-mobile`
- **THEN** the claim identifies the applicable profile and does not imply transfer to other profiles

### Requirement: Routine documentation verification is local
Prose, truth, source-classification, privacy, completeness, broken-link, route, generated-reference, responsive, and accessibility checks SHALL run locally after documentation implementation is complete. GitHub Actions SHALL be limited to documentation deployment execution and validation of the deployed artifact.

#### Scenario: A routine test is added to a Pages workflow
- **WHEN** a GitHub Actions documentation workflow contains a unit, integration, lint, conformance, local accessibility, or other routine development test step
- **THEN** the GitHub Actions policy validator fails locally

#### Scenario: Completed documentation is verified
- **WHEN** the phase reaches documentation code and content completion
- **THEN** one local entrypoint runs the frozen site build and all publication-contract checks and records their observed outputs

### Requirement: One artifact owns GitHub Pages
Exactly one GitHub Actions workflow SHALL publish the UAR GitHub Pages environment, and that workflow SHALL publish the complete Docusaurus portal with any generated API references staged beneath it.

#### Scenario: A second Pages publisher exists
- **WHEN** more than one workflow uploads or deploys an artifact to the GitHub Pages environment
- **THEN** local workflow-policy validation fails and identifies every competing publisher

#### Scenario: The deployed portal is validated
- **WHEN** a documentation deployment completes from the accepted source SHA
- **THEN** the site root and representative deep routes serve the branded Docusaurus portal and the repository homepage field points to that observed working URL

### Requirement: Superseded portal contracts cannot remain active authority
The completed-but-unarchived `docs-hosted-rustdoc-typedoc-docusaurus-ia` change SHALL receive an explicit supersession disposition before this contract is considered implemented. Placeholder content, fail-open prose checks, and GitHub Actions routine-test requirements from that change MUST NOT govern the new portal.

#### Scenario: The earlier portal change is reviewed
- **WHEN** this publication contract is implemented
- **THEN** every unfinished or conflicting requirement from the earlier change is either absorbed into a current change, superseded with a reason, or retained as dated history

