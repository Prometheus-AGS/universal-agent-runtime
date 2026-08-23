# documentation-publication-contract Specification

## Purpose
Define the complete, privacy-preserving contract that turns UAR's current, historical, generated, and third-party documentation sources into one traceable public documentation estate.

## Requirements

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

### Requirement: Reviewed historical synthesis

Public architecture history SHALL be synthesized from classified repository
records. Raw Prometheus logs, KBD event payloads, session transcripts,
machine-local paths, credentials, and unreviewed wiki copies MUST NOT be
published directly.

#### Scenario: Reader follows a decision to evidence

- **WHEN** a reader examines a selected architecture decision
- **THEN** the public record identifies its date, disposition, reviewed source class, and present authority without exposing raw private content

#### Scenario: Raw history enters public output

- **WHEN** a public history page copies a private record or cites an unreviewed wiki file as authority
- **THEN** local publication controls exit non-zero and identify the unsafe source

### Requirement: Supersession remains visible

The public history SHALL preserve material reversals and SHALL identify the
replacement for every selected superseded position. The record MUST include the
licensing, frontend, visual-authority, JWT-provider, verification-location,
inference-evidence, and placeholder-publication corrections.

#### Scenario: Old guidance conflicts with current authority

- **WHEN** a retained decision is no longer current
- **THEN** the history labels it superseded, names the replacement, and links the reader to current authority

#### Scenario: A correction is omitted

- **WHEN** one of the required correction records or its replacement is missing
- **THEN** local history validation exits non-zero

### Requirement: Evidence classes carry limits

Public testing documentation SHALL state what each evidence class proves and
what it does not prove. Results SHALL identify their source SHA and applicable
profile when making a behavior claim, and MUST NOT transfer to another profile
without separate evidence.

#### Scenario: Reader evaluates a passing test

- **WHEN** a test result is presented as evidence
- **THEN** the documentation identifies the exercised boundary, source, profile, and explicit non-claims

### Requirement: Inference evidence crosses a genuine model boundary

Only a request that traverses a supported packaged UAR boundary, reaches a real
loaded model through the configured provider path, performs inference, and
returns the result through UAR MAY support an inference integration claim.
Synthetic, mocked, stubbed, recorded, replayed, or hard-coded responses MUST be
described as non-certifying diagnostics.

#### Scenario: Recorded provider test passes

- **WHEN** a recorded or synthetic provider returns a successful response
- **THEN** the result may support protocol or orchestration diagnostics but does not certify model inference, soak, resilience, release, or production readiness

### Requirement: Fail-closed claims include observed negative controls

Every fail-closed requirement SHALL pair its passing assertion with an observed
failing negative control, a bounded mutation, exact restoration, and retained
command/output evidence.

#### Scenario: Guard always passes

- **WHEN** deliberate inversion does not make the assertion fail
- **THEN** the fail-closed claim remains unverified even if its positive path passes

### Requirement: Completed artifact certification

The portal SHALL be production-built only after all content lanes are complete.
The final local artifact SHALL contain the Docusaurus portal plus non-empty real
Rust and TypeScript references and SHALL pass source classification, privacy,
truth, link, and required-route validation. The Rust reference SHALL document
the public `universal-agent-runtime` library under `server-full`, excluding
workspace-only utility binaries from the represented public API.

#### Scenario: Generated reference is absent

- **WHEN** Rustdoc or TypeDoc output is missing or empty
- **THEN** staging exits non-zero and no placeholder reference is published

### Requirement: Rendered and live route evidence

The complete portal SHALL be inspected locally for responsive light/dark
rendering, brand identity, search, Mermaid, keyboard focus, accessibility,
console/network failures, and representative navigation. After deployment,
every required product route plus root, history, and generated-reference routes
SHALL return successful responses from the canonical Pages URL.

#### Scenario: Required deployed route is missing

- **WHEN** a required route returns a non-success response after bounded retries
- **THEN** deployment validation fails and the site/homepage claim remains unverified

### Requirement: Publication result remains documentation-scoped

A passing documentation artifact or Pages deployment MUST NOT be presented as
runtime, inference, release, security, or cross-profile readiness evidence.

#### Scenario: Portal is live

- **WHEN** the complete documentation site passes local and deployment checks
- **THEN** the result establishes documentation publication only and lists all product claims it did not exercise
